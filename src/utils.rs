use alloc::string::{String, ToString};

use ledger_device_sdk::ecc::{ECPrivateKey, Ed25519, make_bip32_path};
use ledger_device_sdk::hash::{HashInit, sha2::Sha2_256};

pub enum AePrefix {
    AccountPubkey,
    NameId,
}

impl ToString for AePrefix {
    fn to_string(&self) -> String {
        use AePrefix::*;

        let s = match self {
            AccountPubkey => "ak",
            NameId => "nm",
        };
        s.to_string()
    }
}


pub fn get_private_key(account_number: u32) -> ECPrivateKey<32, 'E'> {
    const ALLOWED_PATH_LEN: usize = 5;
    const BIP32_PATH: [u32; ALLOWED_PATH_LEN] = make_bip32_path(b"m/44'/457'/0'/0'/0'");

    let mut path = BIP32_PATH.clone();
    path[2] |= account_number;
    Ed25519::derive_from_path_slip10(&path)
}

pub fn sign(account_number: u32, data: &[u8]) -> Option<[u8; 64]> {
    get_private_key(account_number).sign(data).map(|(sig, _)| sig).ok()
}

pub fn to_ae_string(pubkey: &[u8], prefix: AePrefix) -> String {
    let pk = [pubkey, &make_check(pubkey)].concat();

    let mut output = prefix.to_string();
    output.push('_');
    // The output buffer is resizeable, so it's fine to use unwrap here
    let _ = bs58::encode(pk).onto(&mut output).unwrap();

    output
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
