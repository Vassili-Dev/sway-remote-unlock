use der::{
    DecodeOwned, DecodeValue, Encode, EncodeValue, FixedTag, Header, Length, Reader, Tagged, Writer,
};

use crate::helper_types::ByteArray;
#[derive(Debug, PartialEq, Eq)]
pub struct NestedOctetString<T>
where
    T: DecodeOwned + Encode,
{
    inner: T,
}

impl<T: DecodeOwned + Encode> EncodeValue for NestedOctetString<T> {
    fn value_len(&self) -> der::Result<Length> {
        self.inner.encoded_len()
    }

    fn encode_value(&self, writer: &mut impl Writer) -> der::Result<()> {
        self.inner.encode(writer)
    }
}

impl<'a, T: DecodeOwned + Encode + Tagged> DecodeValue<'a> for NestedOctetString<T> {
    fn decode_value<R: Reader<'a>>(reader: &mut R, header: Header) -> der::Result<Self> {
        let inner = reader.read_nested(header.length, T::decode)?;
        Ok(Self { inner })
    }
}

impl<T: DecodeOwned + Encode> FixedTag for NestedOctetString<T> {
    const TAG: der::Tag = der::Tag::OctetString;
}

impl<T: DecodeOwned + Encode> NestedOctetString<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<const N: usize> NestedOctetString<ByteArray<N, 4>> {
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

impl<const N: usize> NestedOctetString<ByteArray<N, 3>> {
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}
