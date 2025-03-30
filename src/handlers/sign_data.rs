use ledger_device_sdk::io::Comm;

use crate::app_ui::sign_data::ui_display_data;
use crate::{utils, AppSW};

pub fn handler_sign_data(comm: &mut Comm) -> Result<(), AppSW> {
    let data = comm.get_data().map_err(|_| AppSW::WrongApduLength)?;

    let (account_number_bytes, rest) =
        data.split_first_chunk::<4>().ok_or(AppSW::MsgWrongLength)?;
    let account_number = u32::from_be_bytes(*account_number_bytes);

    let (data_length_bytes, actual_data) =
        rest.split_first_chunk::<4>().ok_or(AppSW::MsgWrongLength)?;
    let data_length = usize::from_be_bytes(*data_length_bytes);

    if data_length != actual_data.len() {
        return Err(AppSW::DataWrongLength);
    }

    if ui_display_data(actual_data)? {
        let sig = utils::sign(account_number, actual_data).ok_or(AppSW::DataSignFail)?;
        comm.append(&sig);
        Ok(())
    } else {
        Err(AppSW::Deny)
    }
}
