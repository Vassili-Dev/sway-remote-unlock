use std::path::Path;

use super::helper_types::ByteArray;
use serde::{Deserialize, Serialize};

pub enum Algorithm {
    RSA,
    ECDSA,
    ED25519,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pubkey {
    raw: ByteArray<2048>,
}

impl Pubkey {
    pub fn new() -> Pubkey {
        Pubkey {
            raw: ByteArray::new(),
        }
    }

    pub fn read_from_bytes(&mut self, bytes: &[u8]) {
        self.raw.copy_from_slice(bytes);
    }

    pub fn save(&self, dir: &str, id: &str) {
        let storage_dir = Path::new(dir);
        if !storage_dir.exists() {
            std::fs::create_dir_all(storage_dir).unwrap();
        }

        let file_path = storage_dir.join(id);

        std::fs::write(file_path, self.raw.as_bytes()).unwrap();
    }

    pub fn algorithm(&self) -> Algorithm {
        let algo_bytes = self
            .raw
            .as_bytes()
            .split(|byte| *byte == b' ')
            .next()
            .unwrap();

        if algo_bytes.starts_with(b"ssh-rsa") {
            Algorithm::RSA
        } else if algo_bytes.starts_with(b"ecdsa-sha2") {
            Algorithm::ECDSA
        } else if algo_bytes.starts_with(b"ssh-ed25519") {
            Algorithm::ED25519
        } else {
            panic!("Unknown algorithm");
        }
    }
}
