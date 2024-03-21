use der::{
    Choice, Decode, DecodeOwned, DecodeValue, Encode, EncodeValue, FixedTag, Header, Length,
    Reader, Tagged, Writer,
};
use dryoc::types::Bytes;

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

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct AnyOwned<const N: usize = 64> {
    tag: der::Tag,
    bytes: ByteArray<N>,
}

impl<const N: usize> AnyOwned<N> {
    pub fn new(tag: der::Tag, bytes: ByteArray<N>) -> Self {
        Self { tag, bytes }
    }
}

impl<const N: usize> Tagged for AnyOwned<N> {
    fn tag(&self) -> der::Tag {
        self.tag
    }
}

impl<const N: usize> Choice<'_> for AnyOwned<N> {
    fn can_decode(_: der::Tag) -> bool {
        true
    }
}

impl<'a, const N: usize> DecodeValue<'a> for AnyOwned<N> {
    fn decode_value<R: Reader<'a>>(reader: &mut R, header: Header) -> der::Result<Self> {
        let bytes = reader.read_slice(header.length)?;
        Ok(Self {
            tag: header.tag,
            bytes: ByteArray::new_from_slice(bytes),
        })
    }
}

impl<'a, const N: usize> Decode<'a> for AnyOwned<N> {
    fn decode<R: Reader<'a>>(reader: &mut R) -> der::Result<Self> {
        let header = Header::decode(reader)?;

        Self::decode_value(reader, header)
    }
}

impl<const N: usize> EncodeValue for AnyOwned<N> {
    fn value_len(&self) -> der::Result<Length> {
        let len = self.bytes.len();
        Ok(Length::new(len as u16))
    }

    fn encode_value(&self, writer: &mut impl Writer) -> der::Result<()> {
        writer.write(self.bytes.as_bytes())
    }
}
