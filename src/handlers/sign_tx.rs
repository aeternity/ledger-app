use alloc::{borrow::ToOwned, string::String, vec::Vec};

use ledger_device_sdk::hash::{blake2::Blake2b_256, HashInit};
use ledger_device_sdk::io::Comm;

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::NbglHomeAndSettings;

use primitive_types::U256;

use aerlp::{FromRlpItem, RlpItem};

use crate::app_ui::sign_tx::ui_display_tx;
use crate::utils::{self, AePrefix};
use crate::AppSW;

const SPEND_TRANSACTION_PREFIX: u8 = 0x0c;
const NETWORK_ID_MAX_LENGTH: usize = 32;

#[derive(Default)]
pub struct TxFirstChunk {
    pub recipient: String,
    pub amount: U256,
    pub fee: U256,
    pub payload: String,
}

#[derive(Default)]
pub struct TxContext {
    /// Header data
    account_number: u32,
    remain_tx_len: u32,
    network_id: Vec<u8>,

    /// Transaction data in the first chunk
    tx: TxFirstChunk,

    /// Hash of all transaction's chunks
    blake2b: Blake2b_256,

    #[cfg(any(target_os = "stax", target_os = "flex"))]
    pub home: NbglHomeAndSettings,
}

impl TxContext {
    pub fn new() -> Self {
        Default::default()
    }

    // TODO: Fix later when long rlp items are supported
    pub fn is_finished(&self) -> bool {
        true
    }

    pub fn reset(&mut self) {
        self.account_number = 0;
        self.remain_tx_len = 0;
        self.network_id = Vec::new();
        self.tx = Default::default();
        self.blake2b.reset();
    }

    fn parse_header_data<'a>(&mut self, data: &'a [u8]) -> Result<&'a [u8], AppSW> {
        let (account_number_bytes, rest) = data.split_first_chunk::<4>().ok_or(AppSW::TxParsingFail)?;
        let (tx_len_bytes, rest) = rest.split_first_chunk::<4>().ok_or(AppSW::TxParsingFail)?;
        let (network_id_len_byte, rest) = rest.split_first().ok_or(AppSW::TxParsingFail)?;

        let network_id_len: usize = (*network_id_len_byte).into();
        if network_id_len > NETWORK_ID_MAX_LENGTH {
            return Err(AppSW::TxParsingFail);
        }

        let (network_id, rest) = rest
            .split_at_checked(network_id_len)
            .ok_or(AppSW::TxParsingFail)?;

        self.account_number = u32::from_be_bytes(*account_number_bytes);
        self.remain_tx_len = u32::from_be_bytes(*tx_len_bytes);
        self.network_id = network_id.to_vec();

        Ok(rest)
    }

    fn parse_tx_first_chunk(&mut self, data: &[u8]) -> Result<(), AppSW> {
        let (rlp_item, _remain) =
            RlpItem::try_deserialize(data).map_err(|_| AppSW::TxParsingFail)?;
        // TODO: the rlp item has a length, assert that it's ok
        // TODO: is it fine if something remains? or should I check if reamin.empty() == true

        let list = rlp_item.list().map_err(|_| AppSW::TxParsingFail)?;

        if u8::from_rlp_item(&list[0]).map_err(|_| AppSW::TxParsingFail)?
            != SPEND_TRANSACTION_PREFIX
        {
            // TODO: this should be changed later. non-spend txns should be signed
            //       but they should not be treated like spend txns
            // TODO: use a better status word for the error
            return Err(AppSW::Deny);
        }
        let _ = parse_address(&list[2].byte_array().map_err(|_| AppSW::TxParsingFail)?)?;
        let recipient = parse_address(&list[3].byte_array().map_err(|_| AppSW::TxParsingFail)?)?;
        let amount = U256::from_big_endian(&list[4].byte_array().map_err(|_| AppSW::TxParsingFail)?);
        let fee = U256::from_big_endian(&list[5].byte_array().map_err(|_| AppSW::TxParsingFail)?);
        let payload =
            core::str::from_utf8(&list[8].byte_array().map_err(|_| AppSW::TxParsingFail)?)
                .unwrap()
                .to_owned();

        // TODO: extract the rlp items from list in a cleaner way (don't use map_err that many times)

        self.tx = TxFirstChunk {
            recipient,
            amount,
            fee,
            payload,
        };

        Ok(())
    }
}

pub fn handler_sign_tx(
    comm: &mut Comm,
    first_chunk: bool,
    ctx: &mut TxContext,
) -> Result<(), AppSW> {
    let data = comm.get_data().map_err(|_| AppSW::WrongApduLength)?;

    if first_chunk {
        ctx.reset();
        let tx_bytes = ctx.parse_header_data(data)?;
        ctx.parse_tx_first_chunk(tx_bytes)?;
        ctx.blake2b.update(tx_bytes).map_err(|_| AppSW::TxHashFail)?;
    } else {
        ctx.blake2b.update(data).map_err(|_| AppSW::TxHashFail)?;
        return Ok(());
    }

    if ui_display_tx(&ctx.tx)? {
        let mut hash: [u8; 32] = [0; 32];
        ctx.blake2b.finalize(&mut hash).map_err(|_| AppSW::TxHashFail)?;
        let data_to_sign = [&ctx.network_id[..], &hash].concat();
        let sig = utils::sign(ctx.account_number, &data_to_sign).ok_or(AppSW::TxSignFail)?;
        comm.append(&sig);
        Ok(())
    } else {
        Err(AppSW::Deny)
    }
}

fn parse_address(address: &[u8]) -> Result<String, AppSW> {
    const ACCOUNT_ADDRESS_PREFIX: u8 = 1;
    const ACCOUNT_NAMEID_PREFIX: u8 = 2;

    let (prefix_byte, rest) = address.split_first().ok_or(AppSW::TxParsingFail)?;

    let prefix = match *prefix_byte {
        ACCOUNT_ADDRESS_PREFIX => AePrefix::AccountPubkey,
        ACCOUNT_NAMEID_PREFIX => AePrefix::NameId,
        _ => Err(AppSW::TxParsingFail)?,
    };

    let address_bytes: [u8; 32] = rest.try_into().map_err(|_| AppSW::TxParsingFail)?;

    Ok(utils::to_ae_string(&address_bytes, prefix))
}
