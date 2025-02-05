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

    // From RFC 8032 ("Key Generation" section):
    // Link: https://datatracker.ietf.org/doc/html/rfc8032#section-5.1.5
    //
    // 4.  The public key A is the encoding of the point [s]B.  First,
    //     encode the y-coordinate (in the range 0 <= y < p) as a little-
    //     endian string of 32 octets.  The most significant bit of the
    //     final octet is always zero.  To form the encoding of the point
    //     [s]B, copy the least significant bit of the x coordinate to the
    //     most significant bit of the final octet.  The result is the
    //     public key.
    //
    // The ledger library implements the first 3 steps and then return the
    // public key in an uncompressed format (0x04 byte followed by 32 bytes
    // for x and 32 bytes for y).
    let mut pk1 = pk.pubkey[33..].to_vec();
    // Reverse to make it little-endian
    pk1.reverse();
    // Copy the least significant bit
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
