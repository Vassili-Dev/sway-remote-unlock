use core::fmt::{self, Debug};
use der::{Decode, DecodeValue, EncodeValue, FixedTag};
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, Serializer};
use spki::EncodePublicKey;
use std::fmt::Display;
use std::io::Read;
use std::io::Write;
use zeroize::Zeroize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteArray<const N: usize> {
    data: [u8; N],
    length: usize,
}

impl<const N: usize> ByteArray<N> {
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

    fn new_from_slice(slice: &[u8]) -> Result<Self, error::Error> {
        if slice.len() > N {
            return Err(error::ErrorKind::Bounds.into());
        }
        let mut data = [0; N];

        data[..slice.len()].copy_from_slice(slice);

        Ok(ByteArray {
            data,
            length: slice.len(),
        })
    }

    pub fn as_str(&self) -> Result<&str, error::Error> {
        Ok(core::str::from_utf8(&self.data[..self.length])?)
    }

    pub fn to_stdout_raw(&self) -> Result<(), error::Error> {
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(&self.data[..self.length])?;
        stdout.write_all(b"\n")?;

        Ok(())
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) -> Result<(), error::Error> {
        if slice.len() > N {
            return Err(error::ErrorKind::Bounds.into());
        }
        self.data[..slice.len()].copy_from_slice(slice);
        self.length = slice.len();

        Ok(())
    }

    pub fn append(&mut self, other: &Self) -> Result<(), error::Error> {
        if self.length + other.length > N {
            return Err(error::ErrorKind::Bounds.into());
        }
        self.data[self.length..self.length + other.length]
            .copy_from_slice(&other.data[..other.length]);
        self.length += other.length;

        Ok(())
    }

    pub fn append_slice(&mut self, slice: &[u8]) -> Result<(), error::Error> {
        if self.length + slice.len() > N {
            return Err(error::ErrorKind::Bounds.into());
        }
        self.data[self.length..self.length + slice.len()].copy_from_slice(slice);
        self.length += slice.len();

        Ok(())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.length]
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn len(&self) -> usize {
        self.length
    }
    pub fn as_array(&self) -> &[u8; N] {
        &self.data
    }
}

impl<const N: usize> TryFrom<&[u8]> for ByteArray<N> {
    type Error = error::Error;
    fn try_from(slice: &[u8]) -> Result<Self, error::Error> {
        ByteArray::new_from_slice(slice)
    }
}

impl<const N: usize> From<([u8; N], usize)> for ByteArray<N> {
    fn from((data, length): ([u8; N], usize)) -> Self {
        ByteArray { data, length }
    }
}

impl<const N: usize> Write for ByteArray<N> {
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

impl<const N: usize> Read for ByteArray<N> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = core::cmp::min(buf.len(), self.length);
        buf[..len].copy_from_slice(&self.data[..len]);
        self.data.copy_within(len..self.length, 0);
        self.length -= len;
        Ok(len)
    }
}

impl<const N: usize> FixedTag for ByteArray<N> {
    const TAG: der::Tag = der::Tag::BitString;
}

impl<'a, const N: usize> DecodeValue<'a> for ByteArray<N> {
    fn decode_value<R: der::Reader<'a>>(reader: &mut R, header: der::Header) -> der::Result<Self> {
        let mut data = [0; N];
        let length = u32::from(header.length) as usize;
        assert!(length <= N, "byte array too large");

        reader.read_into(&mut data[..length])?;

        Ok(ByteArray { data, length })
    }
}

impl<const N: usize> EncodeValue for ByteArray<N> {
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
        E: serde::de::Error,
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
        E: serde::de::Error,
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

impl<const N: usize> EncodePublicKey for ByteArray<N> {
    fn to_public_key_der(&self) -> spki::Result<der::Document> {
        let doc = der::Document::from_der(self.as_bytes())?;
        Ok(doc)
    }
}

#[derive(Debug)]
pub struct ByteArrayString<const N: usize = 64>(ByteArray<N>);
impl<const N: usize> From<ByteArray<N>> for ByteArrayString<N> {
    fn from(ba: ByteArray<N>) -> Self {
        ByteArrayString(ba)
    }
}

impl<const N: usize> From<ByteArrayString<N>> for ByteArray<N> {
    fn from(bas: ByteArrayString<N>) -> Self {
        bas.0
    }
}

impl<const N: usize> TryFrom<&str> for ByteArrayString<N> {
    type Error = error::Error;
    fn try_from(s: &str) -> Result<Self, error::Error> {
        Ok(ByteArrayString(ByteArray::try_from(s.as_bytes())?))
    }
}

impl TryFrom<u16> for ByteArrayString<5> {
    type Error = error::Error;
    fn try_from(n: u16) -> Result<Self, error::Error> {
        let mut data = [0; 5];
        let digits = n.checked_ilog10().unwrap_or(0);

        if digits == 0 {
            return Ok(ByteArrayString::try_from("0")?);
        }

        for digit in 0..digits {
            let divisor = 10u16.pow(digits - digit - 1);
            let digit = (n / divisor) % 10;
            data[digit as usize] = b'0' + digit as u8;
        }
        Ok(ByteArrayString(ByteArray {
            data,
            length: digits as usize,
        }))
    }
}

impl TryFrom<u8> for ByteArrayString<5> {
    type Error = error::Error;
    fn try_from(n: u8) -> Result<Self, error::Error> {
        let mut data = [0; 5];
        let digits = n.checked_ilog10().unwrap_or(0);

        if digits == 0 {
            return Ok(ByteArrayString::try_from("0")?);
        }

        for digit in 0..digits {
            let divisor = 10u8.pow(digits - digit - 1);
            let digit = (n / divisor) % 10;
            data[digit as usize] = b'0' + digit as u8;
        }
        Ok(ByteArrayString(ByteArray {
            data,
            length: digits as usize,
        }))
    }
}

impl<const N: usize> Display for ByteArrayString<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_str().unwrap_or("Malformed String"))
    }
}

impl TryFrom<i8> for ByteArrayString<5> {
    type Error = error::Error;
    fn try_from(n: i8) -> Result<Self, error::Error> {
        let mut data = [0; 5];
        let digits = n.checked_ilog10().unwrap_or(0);

        if digits == 0 {
            return Ok(ByteArrayString::try_from("0")?);
        }

        let start_idx = if n < 0 {
            data[0] = b'-';
            1
        } else {
            0
        };

        let n = n.abs() as u8;
        for digit in start_idx..digits {
            let divisor = 10u8.pow(digits - digit - 1);
            let digit = (n / divisor) % 10;
            data[digit as usize] = b'0' + digit as u8;
        }
        Ok(ByteArrayString(ByteArray {
            data,
            length: digits as usize,
        }))
    }
}

impl TryFrom<i32> for ByteArrayString<11> {
    type Error = error::Error;
    fn try_from(n: i32) -> Result<Self, error::Error> {
        let mut data = [0; 11];
        let digits = n.checked_ilog10().unwrap_or(0);

        if digits == 0 {
            return Ok(ByteArrayString::try_from("0")?);
        }

        let start_idx = if n < 0 {
            data[0] = b'-';
            1
        } else {
            0
        };

        let n = n.abs() as u16;
        for digit in start_idx..digits {
            let divisor = 10u16.pow(digits - digit - 1);
            let digit = (n / divisor) % 10;
            data[digit as usize] = b'0' + digit as u8;
        }
        Ok(ByteArrayString(ByteArray {
            data,
            length: digits as usize,
        }))
    }
}
mod error {
    use crate::types::OwnError;
    use std::fmt::Display;

    #[derive(Debug)]
    pub enum Error {
        OwnError(OwnError<ErrorKind>),
        Io(std::io::Error),
        Utf8(std::str::Utf8Error),
    }

    impl From<ErrorKind> for Error {
        fn from(kind: ErrorKind) -> Self {
            Self::OwnError(kind.into())
        }
    }

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                Error::OwnError(e) => write!(f, "{}", e),
                Error::Io(e) => write!(f, "{}", e),
                Error::Utf8(e) => write!(f, "{}", e),
            }
        }
    }

    #[derive(Debug)]
    pub enum ErrorKind {
        Bounds,
        Utf8,
    }

    impl crate::types::ErrorKindMarker for ErrorKind {}

    impl Display for ErrorKind {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                ErrorKind::Bounds => write!(f, "Buffer bounds error"),
                ErrorKind::Utf8 => write!(f, "UTF-8 error"),
            }
        }
    }

    impl From<std::io::Error> for Error {
        fn from(e: std::io::Error) -> Self {
            Error::Io(e)
        }
    }

    impl From<std::str::Utf8Error> for Error {
        fn from(e: std::str::Utf8Error) -> Self {
            Error::Utf8(e)
        }
    }

    impl std::error::Error for Error {}
}

pub use error::Error;
