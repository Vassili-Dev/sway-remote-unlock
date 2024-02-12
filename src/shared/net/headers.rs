use crate::{
    errors::{RemoteUnlockError, ServerError},
    helper_types::ByteArray,
};

#[derive(Debug, Clone, Copy)]
pub struct Header<const N: usize = 32, const V: usize = 64> {
    pub name: ByteArray<N>,
    pub value: ByteArray<V>,
}

impl<const N: usize, const V: usize> Header<N, V> {
    pub fn new() -> Header<N, V> {
        Header {
            name: ByteArray::new(),
            value: ByteArray::new(),
        }
    }

    pub fn from_stream(stream: &mut impl std::io::Read) -> std::io::Result<Header<N, V>> {
        let mut header = Header::new();
        let mut buf = [0; 1];
        let mut name = true;
        let mut name_len = 0;
        let mut value_len = 0;

        loop {
            if (name && name_len == N) || (value_len == V) {
                break;
            }
            stream.read_exact(&mut buf)?;

            if buf[0] == b':' {
                name = false;
                continue;
            }

            if buf[0] == b'\r' {
                stream.read_exact(&mut buf)?;
                if buf[0] == b'\n' {
                    break;
                }
            }

            if name {
                header.name.as_bytes_mut()[name_len] = buf[0];
                name_len += 1;
            } else {
                header.value.as_bytes_mut()[value_len] = buf[0];
                value_len += 1;
            }
        }

        Ok(header)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Header<N, V>, RemoteUnlockError> {
        let mut header = Header::new();
        let mut name = true;
        let mut name_len = 0;
        let mut value_len = 0;

        for byte in bytes {
            if *byte == b':' {
                name = false;
                continue;
            }

            if *byte == b'\r' {
                continue;
            }

            if *byte == b'\n' {
                break;
            }

            if (name && name_len == N) || (value_len == V) {
                return Err(ServerError::new("Header name or value too long".to_string()).into());
            }

            if name {
                header.name.as_bytes_mut()[name_len] = *byte;
                name_len += 1;
            } else {
                header.value.as_bytes_mut()[value_len] = *byte;
                value_len += 1;
            }
        }

        Ok(header)
    }
}
