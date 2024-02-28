use std::io::Read;
use std::path::Path;

use crate::errors::RemoteUnlockError;

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

    pub fn from_file(dir: &str, id: &str) -> Result<Pubkey, RemoteUnlockError> {
        let file_path = Path::new(dir).join(id);

        if !file_path.exists() {
            return Err(RemoteUnlockError::PubkeyNotFoundError);
        }

        let mut f = std::fs::File::open(file_path)?;

        // Try to retrieve the public key from storage

        let mut pubkey_buf = [0; 2048];
        let mut buf_ptr = 0;
        let mut pubkey = Pubkey::new();

        // Read the file from storage until buffer full or file end
        loop {
            if buf_ptr >= 2048 {
                break;
            }
            let (_, remaining) = pubkey_buf.split_at_mut(buf_ptr);
            let read_amt = match f.read(remaining) {
                Ok(amt) => amt,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        0
                    } else {
                        return Err(e.into());
                    }
                }
            };

            if read_amt == 0 {
                break;
            }

            buf_ptr += read_amt;
        }
        // file.read(&mut pubkey_buf).unwrap();
        pubkey.read_from_bytes(&pubkey_buf);

        Ok(pubkey)
    }
}

impl Default for Pubkey {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&Pubkey> for PublicKey {
    fn from(pubkey: &Pubkey) -> PublicKey {
        let mut ret = PublicKey::new();
        ret.copy_from_slice(pubkey.raw.as_bytes());

        ret
    }
}
