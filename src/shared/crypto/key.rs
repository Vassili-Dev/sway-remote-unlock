use std::io::{self, Read, Write};

use crate::{config::Config, errors::RemoteUnlockError, helper_types::ByteArray};

use super::der::SubjectPublicKeyInfoOwned;
use der::{Decode, DecodePem, SecretDocument};
use pkcs8::DecodePrivateKey;
use serde_json::from_reader;

pub struct PublicKey(SubjectPublicKeyInfoOwned);
pub struct PrivateKey(SecretDocument);

impl PublicKey {
    pub fn from_pem(bytes: &[u8]) -> Result<Self, RemoteUnlockError> {
        let spki = SubjectPublicKeyInfoOwned::from_pem(bytes)?;
        Ok(Self(spki))
    }

    pub fn read_pem_file(path: &str) -> Result<Self, RemoteUnlockError> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        io::copy(&mut file, &mut bytes)?;

        SubjectPublicKeyInfoOwned::from_pem(bytes.as_bytes())
            .map(|k| Self(k))
            .map_err(|err| err.into())
    }

    pub fn from_der(bytes: &[u8]) -> Result<Self, RemoteUnlockError> {
        let spki = SubjectPublicKeyInfoOwned::from_der(bytes)?;
        Ok(Self(spki))
    }

    pub fn read_der_file(path: &str) -> Result<Self, RemoteUnlockError> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        io::copy(&mut file, &mut bytes)?;

        SubjectPublicKeyInfoOwned::from_der(bytes.as_bytes())
            .map(|k| Self(k))
            .map_err(|err| err.into())
    }

    pub fn save_to_pem_file(&self, path: &str) -> Result<(), RemoteUnlockError> {
        let mut file = std::fs::File::create(path)?;
        file.write(self.0.to_pem()?.as_bytes())?;
        Ok(())
    }
}
