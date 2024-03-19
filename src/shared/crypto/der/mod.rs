use crate::helper_types::ByteArray;
use der::oid::ObjectIdentifier;

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

// Owned DerKey containing a ByteArray
#[derive(Debug, PartialEq, Eq, der::Sequence)]
pub struct DerKey<const N: usize = 1024> {
    version: u8,              // 0
    oid: AlgorithmIdentifier, // AlgoId
    key: types::NestedOctetString<ByteArray<N>>,
}

impl DerKey {
    pub fn new_from_key(key: &[u8]) -> DerKey {
        DerKey {
            version: 0,
            oid: AlgorithmIdentifier::new(
                ObjectIdentifier::new("1.3.101.112").expect("Invalid OID"),
            ),
            key: types::NestedOctetString::new(ByteArray::new_from_slice(key)),
        }
    }
    pub fn key(&self) -> &[u8] {
        self.key.as_bytes()
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn oid(&self) -> &AlgorithmIdentifier {
        &self.oid
    }
}
