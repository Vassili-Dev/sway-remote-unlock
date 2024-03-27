use std::{io, path::Path};

use crate::prelude::*;

use super::der::SubjectPublicKeyInfoOwned;
use der::{pem::PemLabel, Decode, DecodePem, Encode, PemWriter, SecretDocument};
use pkcs8::{DecodePrivateKey, PrivateKeyInfo};

use spki::EncodePublicKey;

pub struct PublicKey(SubjectPublicKeyInfoOwned);
pub struct PrivateKey(SecretDocument);

impl PublicKey {
    pub fn inner(&self) -> &SubjectPublicKeyInfoOwned {
        &self.0
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let bytes = ByteArray::<{ Config::BUFFER_SIZE }>::try_from(bytes)?;
        let spki = bytes
            .to_public_key_der()?
            .decode_msg::<SubjectPublicKeyInfoOwned>()?;
        Ok(Self(spki))
    }

    pub fn from_pem(bytes: &[u8]) -> Result<Self, Error> {
        let spki = SubjectPublicKeyInfoOwned::from_pem(bytes)?;
        Ok(Self(spki))
    }

    pub fn read_pem_file(path: &Path) -> Result<Self, Error> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        io::copy(&mut file, &mut bytes)?;

        SubjectPublicKeyInfoOwned::from_pem(bytes.as_bytes())
            .map(Self)
            .map_err(|err| err.into())
    }

    pub fn from_der(bytes: &[u8]) -> Result<Self, Error> {
        let spki = SubjectPublicKeyInfoOwned::from_der(bytes)?;
        Ok(Self(spki))
    }

    pub fn read_der_file(path: &Path) -> Result<Self, Error> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        io::copy(&mut file, &mut bytes)?;

        SubjectPublicKeyInfoOwned::from_der(bytes.as_bytes())
            .map(Self)
            .map_err(|err| err.into())
    }

    pub fn save_to_pem_file(&self, path: &Path) -> Result<(), Error> {
        debug!("Saving public key to file: {:?}", path);
        let mut pem = self.pem()?;

        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);

        trace!("Opening file: {:?}", path);
        let mut file = options.open(path)?;

        trace!("Copying public key to file");
        std::io::copy(&mut pem, &mut file)?;

        Ok(())
    }

    pub fn save_to_der_file(&self, path: &Path) -> Result<(), Error> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        let mut file = options.open(path)?;

        let mut der = self.der()?;

        std::io::copy(&mut der, &mut file)?;

        Ok(())
    }

    pub fn der(&self) -> Result<ByteArray<{ Config::BUFFER_SIZE }>, Error> {
        let mut buf = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        self.0.encode(&mut buf)?;
        Ok(buf)
    }

    pub fn pem(&self) -> Result<ByteArray<{ Config::BUFFER_SIZE }>, Error> {
        let mut buf = [0; { Config::BUFFER_SIZE }];
        let mut writer = PemWriter::new(
            SubjectPublicKeyInfoOwned::PEM_LABEL,
            pkcs8::LineEnding::LF,
            &mut buf,
        )?;

        self.0.encode(&mut writer).unwrap();
        let written = writer.finish()?;

        let buf = ByteArray::try_from(&buf[..written])?;

        Ok(buf)
    }
}

impl PrivateKey {
    pub fn inner(&self) -> &SecretDocument {
        &self.0
    }
    pub fn from_pem(bytes: &[u8]) -> Result<Self, Error> {
        let pem_str = std::str::from_utf8(bytes)?;
        let (_, secret) = SecretDocument::from_pem(pem_str)?;
        Ok(Self(secret))
    }

    pub fn read_pem_file(path: &Path) -> Result<Self, Error> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        io::copy(&mut file, &mut bytes)?;

        let pem_str = std::str::from_utf8(bytes.as_bytes())?;

        SecretDocument::from_pkcs8_pem(pem_str)
            .map(Self)
            .map_err(|err| err.into())
    }

    pub fn from_der(bytes: &[u8]) -> Result<Self, Error> {
        let secret = SecretDocument::from_pkcs8_der(bytes)?;
        Ok(Self(secret))
    }

    pub fn read_der_file(path: &Path) -> Result<Self, Error> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();
        io::copy(&mut file, &mut bytes)?;

        SecretDocument::from_pkcs8_der(bytes.as_bytes())
            .map(Self)
            .map_err(|err| err.into())
    }

    pub fn der(&self) -> Result<ByteArray<{ Config::BUFFER_SIZE }>, Error> {
        let buf = ByteArray::try_from(self.0.as_bytes())?;
        Ok(buf)
    }

    pub fn pem(&self) -> Result<ByteArray<{ Config::BUFFER_SIZE }>, Error> {
        let mut buf = [0; { Config::BUFFER_SIZE }];
        let mut writer =
            PemWriter::new(PrivateKeyInfo::PEM_LABEL, pkcs8::LineEnding::LF, &mut buf)?;

        let der = self.0.as_bytes();
        let pkcs8 = PrivateKeyInfo::from_der(der)?;

        pkcs8.encode(&mut writer)?;
        let written = writer.finish()?;

        let buf = ByteArray::try_from(&buf[..written])?;

        Ok(buf)
    }

    pub fn save_to_pem_file(&self, path: &Path) -> Result<(), Error> {
        let mut pem = self.pem()?;

        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        let mut file = options.open(path)?;

        std::io::copy(&mut pem, &mut file)?;

        Ok(())
    }

    pub fn save_to_der_file(&self, path: &Path) -> Result<(), Error> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        let mut file = options.open(path)?;

        std::io::copy(&mut self.0.as_bytes(), &mut file)?;

        Ok(())
    }
}
