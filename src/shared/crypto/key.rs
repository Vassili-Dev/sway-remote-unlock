use std::{io, path::Path};

use crate::{config::Config, errors::RemoteUnlockError, helper_types::ByteArray};

use super::der::SubjectPublicKeyInfoOwned;
use der::{pem::PemLabel, Decode, DecodePem, Encode, PemWriter, SecretDocument};
use pkcs8::{DecodePrivateKey, PrivateKeyInfo};

use spki::EncodePublicKey;
pub struct PublicKey(SubjectPublicKeyInfoOwned);
pub struct PrivateKey(SecretDocument);

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, RemoteUnlockError> {
        let bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new_from_slice(bytes);
        let spki = bytes
            .to_public_key_der()?
            .decode_msg::<SubjectPublicKeyInfoOwned>()?;
        // let spki = SubjectPublicKeyInfoOwned::from_der(bytes)
        Ok(Self(spki))
    }

    pub fn from_pem(bytes: &[u8]) -> Result<Self, RemoteUnlockError> {
        let spki = SubjectPublicKeyInfoOwned::from_pem(bytes)?;
        Ok(Self(spki))
    }

    pub fn read_pem_file(path: &Path) -> Result<Self, RemoteUnlockError> {
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

    pub fn read_der_file(path: &Path) -> Result<Self, RemoteUnlockError> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        io::copy(&mut file, &mut bytes)?;

        SubjectPublicKeyInfoOwned::from_der(bytes.as_bytes())
            .map(|k| Self(k))
            .map_err(|err| err.into())
    }

    pub fn save_to_pem_file(&self, path: &Path) -> Result<(), RemoteUnlockError> {
        let mut pem = self.pem()?;

        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        let mut file = options.open(path)?;

        std::io::copy(&mut pem, &mut file)?;

        Ok(())
    }

    pub fn save_to_der_file(&self, path: &Path) -> Result<(), RemoteUnlockError> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        let mut file = options.open(path)?;

        let mut der = self.der()?;

        std::io::copy(&mut der, &mut file)?;

        Ok(())
    }

    pub fn der(&self) -> Result<ByteArray<{ Config::BUFFER_SIZE }>, RemoteUnlockError> {
        let mut buf = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        self.0.encode(&mut buf)?;
        Ok(buf)
    }

    pub fn pem(&self) -> Result<ByteArray<{ Config::BUFFER_SIZE }>, RemoteUnlockError> {
        let mut buf = [0; { Config::BUFFER_SIZE }];
        let mut writer = PemWriter::new(
            SubjectPublicKeyInfoOwned::PEM_LABEL,
            pkcs8::LineEnding::LF,
            &mut buf,
        )?;

        self.0.encode(&mut writer).unwrap();
        let written = writer.finish()?;

        let buf = ByteArray::new_from_slice(&buf[..written]);

        Ok(buf)
    }
}

impl PrivateKey {
    pub fn from_pem(bytes: &[u8]) -> Result<Self, RemoteUnlockError> {
        let pem_str = std::str::from_utf8(bytes)?;
        let (_, secret) = SecretDocument::from_pem(pem_str)?;
        Ok(Self(secret))
    }

    pub fn read_pem_file(path: &Path) -> Result<Self, RemoteUnlockError> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        io::copy(&mut file, &mut bytes)?;

        let pem_str = std::str::from_utf8(bytes.as_bytes())?;

        SecretDocument::from_pkcs8_pem(pem_str)
            .map(|k| Self(k))
            .map_err(|err| err.into())
    }

    pub fn from_der(bytes: &[u8]) -> Result<Self, RemoteUnlockError> {
        let secret = SecretDocument::from_pkcs8_der(bytes)?;
        Ok(Self(secret))
    }

    pub fn read_der_file(path: &Path) -> Result<Self, RemoteUnlockError> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        io::copy(&mut file, &mut bytes)?;

        SecretDocument::from_pkcs8_der(bytes.as_bytes())
            .map(|k| Self(k))
            .map_err(|err| err.into())
    }

    pub fn der(&self) -> Result<ByteArray<{ Config::BUFFER_SIZE }>, RemoteUnlockError> {
        let buf = ByteArray::new_from_slice(self.0.as_bytes());
        Ok(buf)
    }

    pub fn pem(&self) -> Result<ByteArray<{ Config::BUFFER_SIZE }>, RemoteUnlockError> {
        let mut buf = [0; { Config::BUFFER_SIZE }];
        let mut writer =
            PemWriter::new(PrivateKeyInfo::PEM_LABEL, pkcs8::LineEnding::LF, &mut buf)?;

        let der = self.0.as_bytes();
        let pkcs8 = PrivateKeyInfo::from_der(der)?;

        pkcs8.encode(&mut writer)?;
        let written = writer.finish()?;

        let buf = ByteArray::new_from_slice(&buf[..written]);

        Ok(buf)
    }

    pub fn save_to_pem_file(&self, path: &Path) -> Result<(), RemoteUnlockError> {
        let mut pem = self.pem()?;

        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        let mut file = options.open(path)?;

        std::io::copy(&mut pem, &mut file)?;

        Ok(())
    }

    pub fn save_to_der_file(&self, path: &Path) -> Result<(), RemoteUnlockError> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        let mut file = options.open(path)?;

        std::io::copy(&mut self.0.as_bytes(), &mut file)?;

        Ok(())
    }
}
