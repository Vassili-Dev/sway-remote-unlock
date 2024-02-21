use std::path::Path;

use super::helper_types::ByteArray;
use dryoc::sign::PublicKey;
use serde::{Deserialize, Serialize};

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
}

impl Into<PublicKey> for &Pubkey {
    fn into(self) -> PublicKey {
        let mut ret = PublicKey::new();
        ret.copy_from_slice(&self.raw.as_bytes());

        ret
    }
}
