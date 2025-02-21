use alloc::string::{String, ToString};
use alloc::vec::Vec;

use ledger_device_sdk::ecc::{make_bip32_path, ECPrivateKey, Ed25519};
use ledger_device_sdk::hash::{sha2::Sha2_256, HashInit};

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

pub fn varuint_encode(n: usize) -> Vec<u8> {
    let mut output = Vec::new();

    if n <= 0xFC {
        output.push(n as u8);
    } else if n <= 0xFFFF {
        output.push(0xFD);
        output.extend((n as u16).to_le_bytes());
    } else if n <= 0xFFFFFFFF {
        output.push(0xFE);
        output.extend((n as u32).to_le_bytes());
    } else {
        output.push(0xFF);
        output.extend((n as u64).to_le_bytes());
    }

    output
}

pub fn get_private_key(account_number: u32) -> ECPrivateKey<32, 'E'> {
    const ALLOWED_PATH_LEN: usize = 5;
    const BIP32_PATH: [u32; ALLOWED_PATH_LEN] = make_bip32_path(b"m/44'/457'/0'/0'/0'");

    let mut path = BIP32_PATH.clone();
    path[2] |= account_number;
    Ed25519::derive_from_path_slip10(&path)
}

pub fn sign(account_number: u32, data: &[u8]) -> Option<[u8; 64]> {
    get_private_key(account_number)
        .sign(data)
        .map(|(sig, _)| sig)
        .ok()
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
    let digest = sha256(&sha256(input));
    *digest
        .first_chunk::<4>()
        .expect("SHA-256 digest must be 32 bytes")
}

fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha2_256::new();
    let mut output: [u8; 32] = [0; 32];
    let _ = hasher.hash(input, &mut output);
    output
}
