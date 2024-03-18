use der::{
    pem::PemLabel, Decode, DecodeOwned, DecodePem, DecodeValue, Encode, EncodeValue, Length,
};

use crate::errors::RemoteUnlockError;

use super::{
    der::{DerKey, DerKeyBorrowed},
    pem::{self, PemDataBorrowed},
};

use der::pem::LineEnding;

#[derive(Debug)]
pub enum KeyBorrowed<'a> {
    SecretKey(DerKeyBorrowed<'a>),
    PublicKey(DerKeyBorrowed<'a>),
}

impl<'a> KeyBorrowed<'a> {
    pub fn as_array(&'a self) -> &'a [u8] {
        match self {
            Self::SecretKey(key) => key.as_bytes(),
            Self::PublicKey(key) => key.as_bytes(),
        }
    }

    pub fn from_der(file: &'a [u8]) -> Result<KeyBorrowed<'a>, RemoteUnlockError> {
        let der_key = DerKeyBorrowed::from_der(file)?;
        Ok(Self::PublicKey(der_key))
    }

    pub fn to_der(&self, writer: &mut impl der::Writer) -> Result<(), RemoteUnlockError> {
        match self {
            Self::SecretKey(key) => Ok(key.encode(writer)?),
            Self::PublicKey(key) => Ok(key.encode(writer)?),
        }
    }

    pub fn to_pem(&self, writer: &mut [u8]) -> Result<usize, RemoteUnlockError> {
        let label = match self {
            Self::PublicKey(_) => "PUBLIC KEY",
            Self::SecretKey(_) => "PRIVATE KEY",
        };

        let mut pem = der::PemWriter::new(label, LineEnding::LF, writer)?;

        self.to_der(&mut pem)?;

        match pem.finish() {
            Ok(0) => Err(RemoteUnlockError::KeyParseError(der::Error::new(
                der::ErrorKind::Pem(der::pem::Error::Length),
                Length::ZERO,
            ))),
            Err(e) => Err(e.into()),
            Ok(written) => Ok(written),
        }
    }
}

#[derive(Debug)]
pub enum KeyOwned {
    SecretKey(PrivateKeyOwned),
    PublicKey(PublicKeyOwned),
}

#[derive(Debug)]
struct PrivateKeyOwned(DerKey);
#[derive(Debug)]
struct PublicKeyOwned(DerKey);

impl PemLabel for PrivateKeyOwned {
    const PEM_LABEL: &'static str = "PRIVATE KEY";
}

impl PemLabel for PublicKeyOwned {
    const PEM_LABEL: &'static str = "PUBLIC KEY";
}

impl<'a> Decode<'a> for PrivateKeyOwned {
    fn decode<R: der::Reader<'a>>(reader: &mut R) -> der::Result<Self> {
        let key = DerKey::decode(reader)?;
        Ok(PrivateKeyOwned(key))
    }
}

impl Encode for PrivateKeyOwned {
    fn encoded_len(&self) -> der::Result<Length> {
        self.0.encoded_len()
    }

    fn encode(&self, writer: &mut impl der::Writer) -> der::Result<()> {
        self.0.encode(writer)
    }
}

impl KeyOwned {
    pub fn from_pem(file: &[u8]) -> Result<KeyOwned, RemoteUnlockError> {
        let der = PrivateKeyOwned::from_pem(&file)?;
        Ok(KeyOwned::SecretKey(der))
    }

    pub fn from_der(file: &[u8]) -> Result<KeyOwned, RemoteUnlockError> {
        let key = DerKey::from_der(file)?;
        Ok(KeyOwned::PublicKey(key))
    }

    pub fn to_der(&self, writer: &mut impl der::Writer) -> Result<(), RemoteUnlockError> {
        match self {
            Self::SecretKey(key) => Ok(key.encode(writer)?),
            Self::PublicKey(key) => Ok(key.encode(writer)?),
        }
    }

    pub fn to_pem(&self, writer: &mut [u8]) -> Result<usize, RemoteUnlockError> {
        let label = match self {
            Self::PublicKey(_) => "PUBLIC KEY",
            Self::SecretKey(_) => "PRIVATE KEY",
        };

        let mut pem = der::PemWriter::new(label, LineEnding::LF, writer)?;

        self.to_der(&mut pem)?;

        match pem.finish() {
            Ok(0) => Err(RemoteUnlockError::KeyParseError(der::Error::new(
                der::ErrorKind::Pem(der::pem::Error::Length),
                Length::ZERO,
            ))),
            Err(e) => Err(e.into()),
            Ok(written) => Ok(written),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::der::{AlgorithmIdentifier, DerKeyBorrowed};
    use crate::crypto::key::KeyBorrowed;
    use der::asn1::ObjectIdentifier;
    use der::Decode;

    use super::KeyOwned;

    // 302e020100300506032b657004220420e6d402bca22a67721c8ce8b1ff7ac6b4a556462f558fac148da972992b6f32df
    const KEY: [u8; 48] = [
        0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
        0x20, 0xe6, 0xd4, 0x02, 0xbc, 0xa2, 0x2a, 0x67, 0x72, 0x1c, 0x8c, 0xe8, 0xb1, 0xff, 0x7a,
        0xc6, 0xb4, 0xa5, 0x56, 0x46, 0x2f, 0x55, 0x8f, 0xac, 0x14, 0x8d, 0xa9, 0x72, 0x99, 0x2b,
        0x6f, 0x32, 0xdf,
    ];

    #[test]
    fn test_parse_der() {
        let key = DerKeyBorrowed::from_der(&KEY).unwrap();
        assert_eq!(key.version(), 0);
        assert_eq!(
            *key.oid(),
            AlgorithmIdentifier::new(ObjectIdentifier::new("1.3.101.112").expect("Invalid OID"))
        );
        assert_eq!(key.as_bytes(), &KEY[16..]);
    }

    const KEY_PEM: &'static str = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIKoqSsjwM1ZKfRLiCl2uvlshQnkX3nOgn1bNQOKUPHY2
-----END PRIVATE KEY-----";

    // aa2a4ac8f033564a7d12e20a5daebe5b21427917de73a09f56cd40e2943c7636
    const EXCPECTED_KEY: [u8; 32] = [
        0xaa, 0x2a, 0x4a, 0xc8, 0xf0, 0x33, 0x56, 0x4a, 0x7d, 0x12, 0xe2, 0x0a, 0x5d, 0xae, 0xbe,
        0x5b, 0x21, 0x42, 0x79, 0x17, 0xde, 0x73, 0xa0, 0x9f, 0x56, 0xcd, 0x40, 0xe2, 0x94, 0x3c,
        0x76, 0x36,
    ];
    #[test]
    fn test_parse_pem() {}
}
