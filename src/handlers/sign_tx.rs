use alloc::{borrow::ToOwned, string::String, vec::Vec};

use ledger_device_sdk::io::Comm;
use ledger_device_sdk::hash::{HashInit, blake2::Blake2b_256};

use aerlp::{FromRlpItem, RlpItem};
use num_bigint::{BigInt, BigUint};
use num_rational::BigRational;

use crate::app_ui::sign::ui_display_tx;
use crate::utils::{self, to_ae_string};
use crate::AppSW;

const SPEND_TRANSACTION_PREFIX: u8 = 0x0c;

#[derive(Default)]
pub struct TxFirstChunk {
    // TODO: does spend_tx even matter?
    spend_tx: bool,
    pub recipient: String,
    pub amount: BigRational,
    pub fee: BigRational,
    pub payload: String,
}

#[derive(Default)]
pub struct TxContext {
    /// Header data
    account_number: u32,
    remain_tx_bytes: u32,
    network_id: Vec<u8>,

    /// Transaction data in the first chunk
    tx: TxFirstChunk,

    /// Hash of all transaction's chunks
    blake2b: Blake2b_256,
}

impl TxContext {
    pub fn new() -> Self {
        Default::default()
    }

    fn parse_header_data<'a>(&mut self, mut data: &'a [u8]) -> Result<&'a [u8], AppSW> {
        let account_number = u32::from_be_bytes(data[..4].try_into().unwrap());
        data = &data[4..];

        // TODO: perform a check that tx_len is equal to the transaction length
        //       there should be an error just like when calling get_data with wrong
        //       data length
        let tx_len = u32::from_be_bytes(data[..4].try_into().unwrap());
        data = &data[4..];

        let network_id_len = data[0] as usize;
        data = &data[1..];
        // TODO: make sure network_id_len is less than NETWORK_ID_MAX_LENGTH

        let network_id = data[..network_id_len].to_vec();
        data = &data[network_id_len..];

        Ok(data)
    }

    fn parse_tx_first_chunk(&mut self, data: &[u8]) -> Result<(), AppSW> {
        // TODO: use a better status word for the error
        let (rlp_item, remain) = RlpItem::try_deserialize(data).map_err(|_| AppSW::WrongApduLength)?;
        // TODO: the rlp item has a length, assert that it's ok
        // TODO: is it fine if something remains? or should I check if reamin.empty() == true

        // TODO: generate a better error
        let list = rlp_item.list().map_err(|_| AppSW::WrongApduLength)?;

        // TODO: use a better status word for the error
        let spend_tx = if SPEND_TRANSACTION_PREFIX
            == u8::from_rlp_item(&list[0]).map_err(|_| AppSW::WrongApduLength)?
        {
            true
        } else {
            false
        };
        let _ = convert_address(&list[2].byte_array().map_err(|_| AppSW::WrongApduLength)?)?;
        let recipient = convert_address(&list[3].byte_array().map_err(|_| AppSW::WrongApduLength)?)?;
        let amountx =
            BigUint::from_bytes_be(&list[4].byte_array().map_err(|_| AppSW::WrongApduLength)?);
        let amount = BigRational::new(BigInt::from(amountx), BigInt::from(10u64.pow(18)));
        let feex = BigUint::from_bytes_be(&list[5].byte_array().map_err(|_| AppSW::WrongApduLength)?);
        let fee = BigRational::new(BigInt::from(feex), BigInt::from(10u64.pow(18)));
        let payload = core::str::from_utf8(&list[8].byte_array().map_err(|_| AppSW::WrongApduLength)?)
            .unwrap()
            .to_owned();

        // TODO: extract the rlp items from list in a cleaner way (don't use map_err that many times)

        self.tx = TxFirstChunk {
            spend_tx,
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
        let tx_bytes = ctx.parse_header_data(data)?;
        ctx.parse_tx_first_chunk(tx_bytes)?;
        ctx.blake2b.update(tx_bytes);
    } else {
        ctx.blake2b.update(data);
        return Ok(());
    }

    if ui_display_tx(&ctx.tx)? {
        let mut hash: [u8; 32] = [0; 32];
        ctx.blake2b.finalize(&mut hash);
        let data_to_sign = [&ctx.network_id[..], &hash].concat();
        let privkey = utils::get_private_key(ctx.account_number);
        // TODO: find a better error
        let (sig, sig_len) = privkey.sign(&data_to_sign).map_err(|_| AppSW::WrongApduLength)?;
        // assert that sig_len is 64
        comm.append(&sig);
        Ok(())
    } else {
        Err(AppSW::Deny)
    }
}

fn convert_address(address: &[u8]) -> Result<String, AppSW> {
    assert_eq!(address.len(), 33);

    const ACCOUNT_ADDRESS_PREFIX: u8 = 1;
    const ACCOUNT_NAMEHASH_PREFIX: u8 = 2;

    let prefix = match address[0] {
        ACCOUNT_ADDRESS_PREFIX => "ak_",
        ACCOUNT_NAMEHASH_PREFIX => "nm_",
        // TODO: better error code
        _ => Err(AppSW::WrongApduLength)?,
    };

    Ok(to_ae_string(address[1..].try_into().unwrap(), prefix))
}
