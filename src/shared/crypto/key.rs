use der::{pem::PemLabel, Decode, DecodePem, Encode, Length};

use crate::errors::RemoteUnlockError;

use super::der::DerKey;

use der::pem::LineEnding;

#[derive(Debug)]
pub enum Key {
    PrivateKey(PrivateKeyOwned),
    PublicKey(PublicKeyOwned),
}

#[derive(Debug)]
pub struct PrivateKeyOwned(DerKey);
#[derive(Debug)]
pub struct PublicKeyOwned(DerKey);

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

impl<'a> Decode<'a> for PublicKeyOwned {
    fn decode<R: der::Reader<'a>>(reader: &mut R) -> der::Result<Self> {
        let key = DerKey::decode(reader)?;
        Ok(PublicKeyOwned(key))
    }
}

impl Encode for PublicKeyOwned {
    fn encoded_len(&self) -> der::Result<Length> {
        self.0.encoded_len()
    }

    fn encode(&self, writer: &mut impl der::Writer) -> der::Result<()> {
        self.0.encode(writer)
    }
}

impl PublicKeyOwned {
    pub fn new_from_key(key: &[u8]) -> Self {
        Self(DerKey::new_from_key_bits(key))
    }
}

impl PrivateKeyOwned {
    pub fn new_from_key(key: &[u8]) -> Self {
        Self(DerKey::new_from_key_bytes(key))
    }
}

impl Key {
    pub fn public_from_pem(file: &[u8]) -> Result<Key, RemoteUnlockError> {
        let der = PublicKeyOwned::from_pem(&file)?;
        Ok(Key::PublicKey(der))
    }

    pub fn private_from_pem(file: &[u8]) -> Result<Key, RemoteUnlockError> {
        let der = PrivateKeyOwned::from_pem(&file)?;
        Ok(Key::PrivateKey(der))
    }

    pub fn public_from_der(file: &[u8]) -> Result<Key, RemoteUnlockError> {
        let key = PublicKeyOwned::from_der(file)?;
        Ok(Key::PublicKey(key))
    }

    pub fn private_from_der(file: &[u8]) -> Result<Key, RemoteUnlockError> {
        let key = PrivateKeyOwned::from_der(file)?;
        Ok(Key::PrivateKey(key))
    }

    pub fn to_der(&self, writer: &mut impl der::Writer) -> Result<(), RemoteUnlockError> {
        match self {
            Self::PrivateKey(key) => Ok(key.encode(writer)?),
            Self::PublicKey(key) => Ok(key.encode(writer)?),
        }
    }

    pub fn key(&self) -> &[u8] {
        match self {
            Self::PrivateKey(key) => key.0.key(),
            Self::PublicKey(key) => key.0.key(),
        }
    }

    pub fn to_pem(&self, writer: &mut [u8]) -> Result<usize, RemoteUnlockError> {
        let label = match self {
            Self::PublicKey(_) => "PUBLIC KEY",
            Self::PrivateKey(_) => "PRIVATE KEY",
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
    use crate::crypto::der::{AlgorithmIdentifier, DerKey};
    use der::asn1::ObjectIdentifier;
    use der::Decode;

    use super::Key;

    const DER_KEY_FILE: &'static [u8] = include_bytes!("../../../test_data/der_test");
    const DER_EXPECTED_KEY: &'static [u8] = include_bytes!("../../../test_data/der_expected_key");
    #[test]
    fn test_parse_der() {
        let key = DerKey::from_der(DER_KEY_FILE).unwrap();
        assert_eq!(key.version(), 0);
        assert_eq!(
            *key.oid(),
            AlgorithmIdentifier::new(ObjectIdentifier::new("1.3.101.112").expect("Invalid OID"))
        );
        assert_eq!(key.key(), DER_EXPECTED_KEY);
    }

    const PEM_PRIVATE: &'static str = include_str!("../../../test_data/pem_test");
    const PEM_EXPECTED_PRIVATE_KEY: &'static [u8] =
        include_bytes!("../../../test_data/pem_expected_private_key");

    #[test]
    fn test_parse_private_pem() {
        let key = Key::private_from_pem(PEM_PRIVATE.as_bytes()).unwrap();
        assert_eq!(key.key(), PEM_EXPECTED_PRIVATE_KEY);
    }

    #[test]
    fn test_reject_private_key_as_public() {
        let key = Key::public_from_pem(PEM_PRIVATE.as_bytes());
        assert!(key.is_err());
    }

    const PEM_PUBLIC: &'static str = include_str!("../../../test_data/pem_test.pub");
    const PEM_EXPECTED_PUBLIC_KEY: &'static [u8] =
        include_bytes!("../../../test_data/pem_expected_public_key");
    #[test]
    fn test_parse_public_pem() {
        let key = Key::public_from_pem(PEM_PUBLIC.as_bytes()).unwrap();
        assert_eq!(key.key(), PEM_EXPECTED_PUBLIC_KEY);
    }
}
