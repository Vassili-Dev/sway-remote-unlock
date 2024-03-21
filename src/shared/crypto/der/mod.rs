use crate::helper_types::ByteArray;
use der::asn1::BitStringRef;
use der::oid::ObjectIdentifier;
use der::{Choice, Decode, DecodeValue, Encode, EncodeValue, Header, Length, Reader, Tagged};

use self::types::NestedOctetString;

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
    key: BytesChoice<N>,
}

impl DerKey {
    pub fn new_from_key_bytes(key: &[u8]) -> Self {
        Self {
            version: 0,
            oid: AlgorithmIdentifier::new(
                ObjectIdentifier::new("1.3.101.112").expect("Invalid OID"),
            ),
            key: BytesChoice::NestedOctetString(NestedOctetString::new(ByteArray::new_from_slice(
                key,
            ))),
        }
    }

    pub fn new_from_key_bits(key: &[u8]) -> Self {
        Self {
            version: 0,
            oid: AlgorithmIdentifier::new(
                ObjectIdentifier::new("1.3.101.112").expect("Invalid OID"),
            ),
            key: BytesChoice::BitString(ByteArray::new_from_slice(key)),
        }
    }

    pub fn key(&self) -> &[u8] {
        match self.key {
            BytesChoice::BitString(ref bytes) => bytes.as_bytes(),
            BytesChoice::NestedOctetString(ref nested) => nested.as_bytes(),
        }
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn oid(&self) -> &AlgorithmIdentifier {
        &self.oid
    }
}

#[derive(Debug, PartialEq, Eq)]
enum BytesChoice<const N: usize> {
    BitString(ByteArray<N, 3>),
    NestedOctetString(NestedOctetString<ByteArray<N, 4>>),
}

impl<'a, const N: usize> Choice<'a> for BytesChoice<N> {
    fn can_decode(tag: der::Tag) -> bool {
        tag == der::Tag::BitString || tag == der::Tag::OctetString
    }
}

impl<const N: usize> Tagged for BytesChoice<N> {
    fn tag(&self) -> der::Tag {
        match self {
            Self::BitString(_) => der::Tag::BitString,
            Self::NestedOctetString(_) => der::Tag::OctetString,
        }
    }
}

impl<const N: usize> Encode for BytesChoice<N> {
    fn encoded_len(&self) -> der::Result<Length> {
        match self {
            Self::BitString(bytes) => bytes.encoded_len(),
            Self::NestedOctetString(nested) => nested.encoded_len(),
        }
    }
    fn encode(&self, writer: &mut impl der::Writer) -> der::Result<()> {
        match self {
            Self::BitString(bytes) => bytes.encode(writer),
            Self::NestedOctetString(nested) => nested.encode_value(writer),
        }
    }
}
impl<'a, const N: usize> DecodeValue<'a> for BytesChoice<N> {
    fn decode_value<R: Reader<'a>>(reader: &mut R, header: Header) -> der::Result<Self> {
        match header.tag {
            der::Tag::BitString => {
                let bit_string = BitStringRef::decode_value(reader, header)?;
                Ok(Self::BitString(ByteArray::new_from_slice(
                    bit_string.as_bytes().unwrap(),
                )))
            }
            der::Tag::OctetString => {
                let nested_octet_string = NestedOctetString::decode_value(reader, header)?;
                Ok(Self::NestedOctetString(nested_octet_string))
            }
            _ => Err(der::Error::new(
                der::ErrorKind::Noncanonical { tag: header.tag },
                reader.position(),
            )),
        }
    }
}

impl<'a, const N: usize> Decode<'a> for BytesChoice<N> {
    fn decode<R: Reader<'a>>(reader: &mut R) -> der::Result<Self> {
        let header = Header::decode(reader)?;
        Self::decode_value(reader, header)
    }
}
