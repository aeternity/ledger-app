use alloc::borrow::ToOwned;
use alloc::string::ToString;
use ledger_device_sdk::io::Comm;
use ledger_device_sdk::hash::{HashInit, sha2::Sha2_256};
use ledger_device_sdk::ecc::{Ed25519, make_bip32_path};

use crate::app_ui::address::ui_display_address;
use crate::AppSW;

// TODO: check why the first one does not work, while the second one works
//       what is the meaning of the tick ' in the byte sequence?
//       note that the first one I got from the docs of make_bip32_path
//const BIP32_PATH: [u32; 5] = make_bip32_path(b"m/44'/457'/0'/0/0");
const BIP32_PATH: [u32; 5] = make_bip32_path(b"m/44'/457'/0'/0'/0'");
//const BIP32_PATH: [u32; 5] = [44 | 0x80000000, 457 | 0x80000000,  0x80000000,  0x80000000,  0x80000000];

pub fn handler_get_address(comm: &mut Comm, confirm_needed: bool) -> Result<(), AppSW> {
    let data = comm.get_data().map_err(|_| AppSW::WrongApduLength)?;
    // TODO: probably a different error than WrongApduLength is needed here
    let bytes: [u8; 4] = data.try_into().map_err(|_| AppSW::WrongApduLength)?;
    let account_number = u32::from_be_bytes(bytes);

    let mut path = BIP32_PATH.clone();
    path[2] |= account_number;
    // TODO: probably a different error than WrongApduLength is needed here
    let pk = Ed25519::derive_from_path_slip10(&path).public_key().map_err(|_| AppSW::WrongApduLength)?;
    assert_eq!(pk.pubkey.len(), 65);

    // the following is number 4 from the "Key Generation" section of RFC8032
    // the ledger library implements the first 3 steps and then return the
    // public key in an uncompressed format (0x04 byte followed by 32 bytes for x
    // and 32 bytes for y)
    let mut pk1 = pk.pubkey[33..].to_vec();
    assert_eq!(pk1.len(), 32);
    // TODO: check that reversing here is needed and why?
    pk1.reverse();
    if (pk.pubkey[32] & 1) != 0 {
        pk1[31] |= 0x80;
    }

    pk1.extend_from_slice(&make_check(&pk1));
    assert_eq!(pk1.len(), 36);

    let mut output = "ak_".to_owned();

    // TODO: do not use unwrap
    let _ = bs58::encode(pk1).onto(&mut output).unwrap();

    if !confirm_needed || ui_display_address(output.as_bytes())? {
        // TODO: convert to u8 in a safer way
        comm.append(&[output.len() as u8]);
        comm.append(output.as_bytes());
        Ok(())
    } else {
        Err(AppSW::Deny)
    }
}

fn make_check(input: &[u8]) -> [u8; 4] {
    let mut sha2 = Sha2_256::new();

    let mut hash1: [u8; 32] = [0u8; 32];
    let mut hash2: [u8; 32] = [0u8; 32];

    sha2.update(input).unwrap();
    sha2.finalize(&mut hash1).unwrap();

    sha2.update(&hash1).unwrap();
    sha2.finalize(&mut hash2).unwrap();

    let mut check: [u8; 4] = [0u8; 4];
    check.copy_from_slice(&hash2[0..4]);
    check
}
