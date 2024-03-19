use der::{
    asn1::OctetStringRef, DecodeOwned, DecodeValue, Encode, EncodeValue, FixedTag, Header, Length,
    Reader, Writer,
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

impl<'a, T: DecodeOwned + Encode> DecodeValue<'a> for NestedOctetString<T> {
    fn decode_value<R: Reader<'a>>(reader: &mut R, header: Header) -> der::Result<Self> {
        let inner = OctetStringRef::decode_value(reader, header)?.decode_into::<T>()?;
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

impl<const N: usize> NestedOctetString<ByteArray<N>> {
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}
