use crate::{errors::RemoteUnlockError, helper_types::ByteArray};
use der::{
    asn1::{OctetString, OctetStringRef},
    oid::ObjectIdentifier,
};

pub mod types;

#[derive(Debug, PartialEq, Eq, der::Sequence)]
pub struct AlgorithmIdentifier {
    oid: ObjectIdentifier,
}

impl AlgorithmIdentifier {
    pub fn new(oid: ObjectIdentifier) -> AlgorithmIdentifier {
        AlgorithmIdentifier { oid }
    }
}
#[derive(Debug, PartialEq, Eq, der::Sequence)]
pub struct DerKeyBorrowed<'a> {
    version: u8,              // 0
    oid: AlgorithmIdentifier, // 1.3.101.112
    key: types::NestedOctetStringRef<OctetStringRef<'a>>,
}

impl<'a> DerKeyBorrowed<'a> {
    pub fn as_bytes(&self) -> &[u8] {
        self.key.as_bytes()
    }

    pub fn from_key(key: &[u8]) -> Result<DerKeyBorrowed, RemoteUnlockError> {
        Ok(DerKeyBorrowed {
            version: 0,
            oid: AlgorithmIdentifier {
                oid: ObjectIdentifier::new("1.3.101.112").expect("Invalid OID"),
            },
            key: types::NestedOctetStringRef::new(OctetStringRef::new(key)?),
        })
    }

    pub fn oid(&self) -> &AlgorithmIdentifier {
        &self.oid
    }

    pub fn version(&self) -> u8 {
        self.version
    }
}

// Owned DerKey containing a ByteArray
#[derive(Debug, PartialEq, Eq, der::Sequence)]
pub struct DerKey<const N: usize = 1024> {
    version: u8,              // 0
    oid: AlgorithmIdentifier, // AlgoId
    key: types::NestedOctetStringRef<ByteArray<N>>,
}
