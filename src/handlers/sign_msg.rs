use alloc::vec::Vec;

use ledger_device_sdk::hash::{blake2::Blake2b_256, HashInit};
use ledger_device_sdk::io::Comm;

use crate::app_ui::sign_msg::ui_display_msg;
use crate::{utils, AppSW};

const SIGN_MAGIC: &str = "Aeternity Signed Message:\n";
// Conversion using `as` is safe to do here because the string is constant and
// the length is known at compile time.
const SIGN_MAGIC_LEN: u8 = SIGN_MAGIC.len() as u8;

pub fn handler_sign_message(comm: &mut Comm) -> Result<(), AppSW> {
    let data = comm.get_data().map_err(|_| AppSW::WrongApduLength)?;

    let (account_number_bytes, rest) =
        data.split_first_chunk::<4>().ok_or(AppSW::MsgWrongLength)?;
    let account_number = u32::from_be_bytes(*account_number_bytes);

    let (message_length_bytes, message) =
        rest.split_first_chunk::<4>().ok_or(AppSW::MsgWrongLength)?;
    let message_length = usize::from_be_bytes(*message_length_bytes);

    if message_length != message.len() {
        return Err(AppSW::MsgWrongLength);
    }

    if ui_display_msg(message)? {
        let sig = sign_message(account_number, message)?;
        comm.append(&sig);
        Ok(())
    } else {
        Err(AppSW::Deny)
    }
}

fn sign_message(account_number: u32, message: &[u8]) -> Result<[u8; 64], AppSW> {
    let mut data_to_sign = Vec::new();

    data_to_sign.push(SIGN_MAGIC_LEN);
    data_to_sign.extend(SIGN_MAGIC.bytes());
    data_to_sign.extend(utils::varuint_encode(message.len()));
    data_to_sign.extend(message);

    let hash: [u8; 32] = {
        let mut blake2b = Blake2b_256::new();
        let mut output = [0; 32];
        blake2b
            .update(&data_to_sign)
            .map_err(|_| AppSW::MsgHashFail)?;
        blake2b
            .finalize(&mut output)
            .map_err(|_| AppSW::MsgHashFail)?;
        output
    };

    utils::sign(account_number, &hash).ok_or(AppSW::MsgSignFail)
}
