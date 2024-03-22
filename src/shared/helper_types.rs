use core::fmt::{self, Debug};
use der::{Decode, DecodeValue, EncodeValue, FixedTag};
use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
use serde::ser::{Serialize, Serializer};
use spki::EncodePublicKey;
use std::io::Read;
use std::io::Write;
use zeroize::Zeroize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteArray<const N: usize, const T: u8 = 4> {
    data: [u8; N],
    length: usize,
}

impl<const N: usize, const T: u8> ByteArray<N, T> {
    pub fn new() -> Self {
        ByteArray {
            data: [0; N],
            length: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.length]
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data[..self.length]
    }

    pub fn new_from_slice(slice: &[u8]) -> Self {
        let mut data = [0; N];
        data[..slice.len()].copy_from_slice(slice);
        ByteArray {
            data,
            length: slice.len(),
        }
    }
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.length]).unwrap()
    }

    pub fn to_stdout_raw(&self) {
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(&self.data[..self.length]).unwrap();
        stdout.write_all(b"\n").unwrap();
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        self.data[..slice.len()].copy_from_slice(slice);
        self.length = slice.len();
    }

    pub fn append(&mut self, other: &Self) {
        self.data[self.length..self.length + other.length]
            .copy_from_slice(&other.data[..other.length]);
        self.length += other.length;
    }

    pub fn append_slice(&mut self, slice: &[u8]) {
        self.data[self.length..self.length + slice.len()].copy_from_slice(slice);
        self.length += slice.len();
    }
}

impl<const N: usize, const T: u8> Write for ByteArray<N, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = core::cmp::min(buf.len(), N - self.length);
        self.data[self.length..self.length + len].copy_from_slice(&buf[..len]);
        self.length += len;
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<const N: usize, const T: u8> Read for ByteArray<N, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = core::cmp::min(buf.len(), self.length);
        buf[..len].copy_from_slice(&self.data[..len]);
        self.data.copy_within(len..self.length, 0);
        self.length -= len;
        Ok(len)
    }
}

impl<const N: usize> FixedTag for ByteArray<N, 4> {
    const TAG: der::Tag = der::Tag::OctetString;
}

impl<const N: usize> FixedTag for ByteArray<N, 3> {
    const TAG: der::Tag = der::Tag::BitString;
}

impl<'a, const N: usize, const T: u8> DecodeValue<'a> for ByteArray<N, T> {
    fn decode_value<R: der::Reader<'a>>(reader: &mut R, header: der::Header) -> der::Result<Self> {
        let mut data = [0; N];
        let length = u32::from(header.length) as usize;
        assert!(length <= N, "byte array too large");

        reader.read_into(&mut data[..length])?;

        Ok(ByteArray { data, length })
    }
}

impl<const N: usize, const T: u8> EncodeValue for ByteArray<N, T> {
    fn value_len(&self) -> der::Result<der::Length> {
        Ok(der::Length::new(self.length as u16))
    }

    fn encode_value(&self, writer: &mut impl der::Writer) -> der::Result<()> {
        writer.write(&self.data[..self.length])
    }
}

struct ByteArrayVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for ByteArrayVisitor<N> {
    type Value = ByteArray<N>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a byte array of length <= {}", N)
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<ByteArray<N>, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut bytes = [0; N];
        let mut length = N;

        for (idx, byte) in bytes.iter_mut().enumerate() {
            match seq.next_element::<u8>()? {
                Some(b) => *byte = b,
                None => {
                    // Null terminate the string
                    *byte = b'\0';
                    length = idx;
                    break;
                }
            }
        }
        Ok(ByteArray {
            data: bytes,
            length,
        })
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<ByteArray<N>, E>
    where
        E: Error,
    {
        let mut bytes = [0; N];
        bytes[..v.len()].copy_from_slice(v);
        let length = v.len();

        Ok(ByteArray {
            data: bytes,
            length,
        })
    }

    fn visit_str<E>(self, v: &str) -> Result<ByteArray<N>, E>
    where
        E: Error,
    {
        self.visit_bytes(v.as_bytes())
    }
}

impl<'de, const N: usize> Deserialize<'de> for ByteArray<N> {
    fn deserialize<D>(deserializer: D) -> Result<ByteArray<N>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ByteArrayVisitor::<N>)
    }
}

impl<const N: usize> Serialize for ByteArray<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.data[..self.length])
    }
}

impl<const N: usize> dryoc::types::Bytes for ByteArray<N> {
    fn as_slice(&self) -> &[u8] {
        &self.data[..self.length]
    }

    fn is_empty(&self) -> bool {
        self.length == 0
    }

    fn len(&self) -> usize {
        self.length
    }
}

impl<const N: usize> dryoc::types::ByteArray<N> for ByteArray<N> {
    fn as_array(&self) -> &[u8; N] {
        &self.data
    }
}

impl<const N: usize> Zeroize for ByteArray<N> {
    fn zeroize(&mut self) {
        self.data.zeroize();
        self.length.zeroize();
    }
}

impl<const N: usize> Default for ByteArray<N> {
    fn default() -> Self {
        ByteArray::new()
    }
}

impl<const N: usize, const T: u8> EncodePublicKey for ByteArray<N, T> {
    fn to_public_key_der(&self) -> spki::Result<der::Document> {
        let doc = der::Document::from_der(self.as_bytes())?;
        Ok(doc)
    }
}
