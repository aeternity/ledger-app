use alloc::borrow::ToOwned;
use alloc::string::ToString;
use ledger_device_sdk::ecc::{make_bip32_path, Ed25519};
use ledger_device_sdk::hash::{sha2::Sha2_256, HashInit};
use ledger_device_sdk::io::Comm;

use crate::app_ui::address::ui_display_address;
use crate::utils::{AePrefix, get_private_key, to_ae_string};
use crate::AppSW;

pub fn handler_get_address(comm: &mut Comm, confirm_needed: bool) -> Result<(), AppSW> {
    let data = comm.get_data().map_err(|_| AppSW::WrongApduLength)?;

    let account_number = u32::from_be_bytes(data.try_into().map_err(|_| AppSW::WrongApduLength)?);
    let pk = get_private_key(account_number)
        .public_key()
        .map_err(|_| AppSW::KeyDeriveFail)?;

    // the following is number 4 from the "Key Generation" section of RFC8032
    // the ledger library implements the first 3 steps and then return the
    // public key in an uncompressed format (0x04 byte followed by 32 bytes for x
    // and 32 bytes for y)
    let mut pk1 = pk.pubkey[33..].to_vec();
    // TODO: check that reversing here is needed and why?
    pk1.reverse();
    if (pk.pubkey[32] & 1) != 0 {
        pk1[31] |= 0x80;
    }

    let ae_address = to_ae_string(&pk1, AePrefix::AccountPubkey);

    if !confirm_needed || ui_display_address(ae_address.as_bytes())? {
        let address_len: u8 = ae_address
            .len()
            .try_into()
            .expect("AE addresses length must fit in a u8 int");

        comm.append(&[address_len]);
        comm.append(ae_address.as_bytes());
        
        Ok(())
    } else {
        Err(AppSW::Deny)
    }
}
