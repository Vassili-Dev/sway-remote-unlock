use std::marker::PhantomData;

use der::{
    asn1::OctetStringRef, Decode, DecodeOwned, DecodeValue, Encode, EncodeValue, FixedTag, Header,
    Length, Reader, Writer,
};
#[derive(Debug, PartialEq, Eq)]
pub struct NestedOctetStringRef<T: for<'a> Decode<'a> + Encode> {
    inner: T,
}

impl<'a, T: Decode<'a> + Encode> EncodeValue for NestedOctetStringRef<T> {
    fn value_len(&self) -> der::Result<Length> {
        self.inner.encoded_len()
    }

    fn encode_value(&self, writer: &mut impl Writer) -> der::Result<()> {
        self.inner.encode(writer)
    }
}

impl<'a, T: Decode<'a> + Encode> DecodeValue<'a> for NestedOctetStringRef<T> {
    fn decode_value<R: Reader<'a>>(reader: &mut R, header: Header) -> der::Result<Self> {
        let inner = OctetStringRef::decode_value(reader, header)?.decode_into::<T>()?;
        Ok(Self { inner })
    }
}

impl<'a> NestedOctetStringRef<OctetStringRef<'a>> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }

    pub fn new(inner: OctetStringRef<'a>) -> Self {
        Self { inner }
    }
}

impl<'a, T: Decode<'a> + Encode> FixedTag for NestedOctetStringRef<T> {
    const TAG: der::Tag = der::Tag::OctetString;
}
