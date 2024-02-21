use std::{borrow::BorrowMut, io::Write};

use crate::{
    config::Config,
    errors::{OversizePacketError, RemoteUnlockError},
    helper_types::ByteArray,
};

use super::headers::Header;

pub struct Request<const HV: usize = 64> {
    pub path: Option<ByteArray<128>>,
    pub method: Option<ByteArray<16>>,
    pub headers: [Option<Header<32, HV>>; 16],
    pub body: [u8; 1024 * 2],

    pub body_len: usize,
    body_written: usize,

    num_headers: usize,
}

impl Write for Request {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let remaining = self.body.len() - self.body_written;
        let write_amt = std::cmp::min(remaining, buf.len());

        self.body[self.body_written..self.body_written + write_amt]
            .copy_from_slice(&buf[..write_amt]);
        self.body_written += write_amt;

        Ok(write_amt)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Request {
    pub fn new() -> Request {
        Request {
            path: None,
            method: None,
            headers: [None; 16],
            body: [0; 1024 * 2],
            body_len: 0,
            body_written: 0,
            num_headers: 0,
        }
    }

    pub fn add_header(&mut self, name: &'static str, value: &'static str) {
        for header in self.headers.iter_mut() {
            match header {
                Some(header) => {
                    if header.name.as_str() == name {
                        header.value.copy_from_slice(value.as_bytes());
                        return;
                    }
                }
                None => {
                    let mut new_header = Header::new();
                    new_header.name.copy_from_slice(name.as_bytes());
                    new_header.value.copy_from_slice(value.as_bytes());
                    *header = Some(new_header);
                    self.num_headers += 1;
                    return;
                }
            }
        }
    }

    pub fn to_writer(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let path = match self.path.as_ref() {
            Some(path) => path.as_str(),
            None => "/",
        };
        let method = match self.method.as_ref() {
            Some(method) => method.as_str(),
            None => "GET",
        };

        writer.write_fmt(format_args!("{} {} HTTP/1.1\r\n", method, path))?;

        for header in self.headers.iter().take(self.num_headers) {
            match header {
                Some(header) => {
                    writer.write_fmt(format_args!(
                        "{}: {}\r\n",
                        header.name.as_str(),
                        header.value.as_str()
                    ))?;
                }
                None => break,
            }
        }
        // Write content length
        writer.write_fmt(format_args!("Content-Length: {}\r\n", self.body_written))?;
        writer.write(b"\r\n")?;
        writer.write(&self.body[..self.body_written])?;
        Ok(())
    }

    pub fn from_stream(stream: &mut impl std::io::Read) -> Result<Request, RemoteUnlockError> {
        let mut ret = Request::new();
        let mut buf = [0; Config::MAX_PACKET_SIZE];
        let mut buf_ptr = 0;
        let mut retries = 0;

        loop {
            if buf_ptr == Config::MAX_PACKET_SIZE {
                return Err(OversizePacketError.into());
            }
            let (_, remaining) = buf.split_at_mut(buf_ptr);
            let read_amt = stream.read(remaining)?;
            *(&mut buf_ptr) += read_amt;

            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = httparse::Request::new(&mut headers);
            let status = match req.parse(&buf) {
                Ok(httparse::Status::Complete(0)) => None,
                Ok(httparse::Status::Complete(i)) => Some(i),
                Ok(httparse::Status::Partial) => None,
                Err(_) => None,
            };

            if status.is_none() {
                if retries > 5 {
                    return Err(OversizePacketError.into());
                }
                retries += 1;
                continue;
            }

            *(retries.borrow_mut()) = 0;

            ret.path = Some(ByteArray::new_from_slice(req.path.unwrap().as_bytes()));
            ret.method = Some(ByteArray::new_from_slice(req.method.unwrap().as_bytes()));

            let content_length = req
                .headers
                .iter()
                .find(|header| header.name == "Content-Length")
                .unwrap()
                .value;

            let content_length = std::str::from_utf8(content_length)
                .unwrap()
                .parse::<usize>()
                .unwrap();

            let body = &buf[status.unwrap()..status.unwrap() + content_length];

            ret.write(body).unwrap();
            ret.body_len = content_length;

            break;
        }

        Ok(ret)
    }
}
