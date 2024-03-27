use crate::prelude::*;

use std::{io::Write, thread};

use super::{headers::Header, method::Method};

pub struct Request<const HV: usize = { 64 * 2 }> {
    pub path: Option<ByteArray<128>>,
    pub method: Option<Method>,
    pub headers: [Option<Header<32, HV>>; 16],
    pub body: [u8; 1024 * 2],

    pub body_len: usize,
    body_written: usize,

    num_headers: usize,
}

impl<const HV: usize> Write for Request<HV> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let remaining = self.body.len() - self.body_written;
        let write_amt = std::cmp::min(remaining, buf.len());

        self.body[self.body_written..self.body_written + write_amt]
            .copy_from_slice(&buf[..write_amt]);
        self.body_written += write_amt;
        self.body_len = self.body_written;

        Ok(write_amt)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<const HV: usize> Request<HV> {
    pub fn new() -> Self {
        Self {
            path: None,
            method: None,
            headers: [None; 16],
            body: [0; 1024 * 2],
            body_len: 0,
            body_written: 0,
            num_headers: 0,
        }
    }

    pub fn builder() -> RequestBuilder<HV> {
        RequestBuilder::<HV>::default()
    }

    pub fn add_header(&mut self, name: &str, value: &str) -> Result<(), Error> {
        for header in self.headers.iter_mut() {
            match header {
                Some(header) => {
                    if header.name.as_str()? == name {
                        header.value = ByteArray::try_from(value.as_bytes())?;
                        return Ok(());
                    }
                }
                None => {
                    let mut new_header = Header::new();
                    new_header.name = ByteArray::try_from(name.as_bytes())?;
                    new_header.value = ByteArray::try_from(value.as_bytes())?;
                    *header = Some(new_header);
                    self.num_headers += 1;
                    return Ok(());
                }
            };
        }
        Err(Error::new(ErrorKind::Server, Some("Too many headers")))
    }

    pub fn get_header(&self, name: &str) -> Option<&Header<32, HV>> {
        for header in self.headers.iter() {
            match header {
                Some(header) => {
                    if header.name.as_str().unwrap_or("") == name {
                        return Some(header);
                    }
                }
                None => break,
            }
        }
        None
    }

    pub fn to_writer(&self, writer: &mut impl Write) -> Result<(), Error> {
        trace!("Writing request to writer");
        let path = match self.path.as_ref() {
            Some(path) => path.as_str()?,
            None => "/",
        };
        let method = match self.method.as_ref() {
            Some(method) => method.as_str(),
            None => "GET",
        };

        trace!("Writing HTTP request line");
        writer.write_fmt(format_args!("{} {} HTTP/1.1\r\n", method, path))?;

        trace!("Writing request headers");
        for header in self.headers.iter().take(self.num_headers) {
            match header {
                Some(header) => {
                    writer.write_fmt(format_args!(
                        "{}: {}\r\n",
                        header.name.as_str()?,
                        header.value.as_str()?
                    ))?;
                }
                None => break,
            }
        }
        // Write content length

        trace!("Writing content length header");
        writer.write_fmt(format_args!("Content-Length: {}\r\n", self.body_written))?;
        writer.write_all(b"\r\n")?;

        trace!("Writing request body");
        writer.write_all(&self.body[..self.body_written])?;

        trace!("Finished writing request");
        Ok(())
    }

    pub fn from_stream(stream: &mut impl std::io::Read) -> Result<Self, Error> {
        trace!("Parsing request from stream");
        let mut builder = Self::builder();
        let mut buf = [0; Config::MAX_PACKET_SIZE];
        let mut buf_ptr = 0;

        // Read into buffer until stream is empty
        loop {
            if buf_ptr >= Config::MAX_PACKET_SIZE {
                return Err(ErrorKind::OversizePacket.into());
            }
            let (_, remaining) = buf.split_at_mut(buf_ptr);
            let read_amt = match stream.read(remaining) {
                Ok(amt) => amt,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        0
                    } else {
                        return Err(e.into());
                    }
                }
            };

            if (read_amt == 0) && (buf_ptr == 0) {
                trace!(
                    "No data received, trying again in {}ms",
                    Config::STREAM_RETRY_DELAY_MS
                );
                // Wait for request
                thread::sleep(std::time::Duration::from_millis(
                    Config::STREAM_RETRY_DELAY_MS,
                ));
                continue;
            }

            buf_ptr += read_amt;

            if read_amt == 0 {
                trace!("Finished reading data from stream");
                break;
            }
        }

        trace!("Parsing httparse request");
        // Process the buffer into a request
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let status = match req.parse(&buf) {
            Ok(httparse::Status::Complete(i)) => i,
            Ok(httparse::Status::Partial) => return Err(ErrorKind::IncompleteRequest.into()),
            Err(e) => return Err(e.into()),
        };

        trace!("Finished parsing httparse request");

        trace!("Building request from parsed data");
        let path = req
            .path
            .ok_or(Error::new(ErrorKind::Server, Some("Path missing")))?;

        trace!("Path: {}", path);

        let method = req
            .method
            .ok_or(Error::new(ErrorKind::Server, Some("Method missing")))?;
        trace!("Method: {}", method);

        builder = builder.path(path).method(method.into());

        trace!("Looking for content-length header");
        let content_length = match req
            .headers
            .iter()
            .find(|header| header.name == "Content-Length")
        {
            Some(header) => header.value,
            None => b"0",
        };

        let content_length = std::str::from_utf8(content_length)?
            .parse::<usize>()
            .unwrap_or(0);
        trace!("Content-Length: {}", content_length);

        trace!("Adding headers to request");
        for header in req.headers {
            if !header.name.is_empty() {
                let hv = std::str::from_utf8(header.value)?;

                trace!("Header: {}: {}", header.name, hv);
                builder = builder.add_header(header.name, hv)?;
            }
        }

        let body_start_ptr = status;
        let body_end_ptr = status + content_length;

        let body = &buf[body_start_ptr..body_end_ptr];
        trace!("Adding body to request: {} bytes", body.len());
        builder = builder.append_body(body)?;

        let ret = builder.build();

        if ret.body_len != content_length {
            error!(
                "Content-Length mismatch: {} vs {}",
                ret.body_len, content_length
            );
            let sign = if ret.body_len < content_length { -1 } else { 1 };
            let (larger, smaller) = if sign == 1 {
                (content_length, ret.body_len)
            } else {
                (ret.body_len, content_length)
            };

            let diff = larger - smaller;
            let diff = i32::try_from(diff).unwrap_or(i32::MAX);
            let diff = diff * sign;

            return Err(Error::new(
                ErrorKind::ContentLengthMismatch,
                Some(ByteArray::from(ByteArrayString::try_from(diff)?).as_str()?),
            ));
        }

        trace!("Finished parsing request");

        Ok(ret)
    }

    pub fn path(&self) -> Option<&str> {
        match self.path.as_ref() {
            Some(path) => match path.as_str() {
                Ok(path) => Some(path),
                Err(_) => None,
            },
            None => None,
        }
    }

    pub fn method(&self) -> Option<&Method> {
        self.method.as_ref()
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RequestBuilder<const HV: usize = { 64 * 2 }> {
    path: Option<ByteArray<128>>,
    method: Option<Method>,
    headers: [Option<Header<32, HV>>; 16],
    body: [u8; 1024 * 2],
    body_len: usize,
    body_written: usize,
    num_headers: usize,
}

impl<const HV: usize> Default for RequestBuilder<HV> {
    fn default() -> Self {
        Self {
            path: None,
            method: None,
            headers: [None; 16],
            body: [0; 1024 * 2],
            body_len: 0,
            body_written: 0,
            num_headers: 0,
        }
    }
}

impl<const HV: usize> RequestBuilder<HV> {
    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(ByteArray::try_from(path.as_bytes()).unwrap());
        self
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = Some(method);
        self
    }

    pub fn add_header(mut self, name: &str, value: &str) -> Result<Self, Error> {
        for header in self.headers.iter_mut() {
            match header {
                Some(header) => {
                    if header.name.as_str()? == name {
                        header.value = ByteArray::try_from(value.as_bytes())?;
                        return Ok(self);
                    }
                }
                None => {
                    let mut new_header = Header::new();
                    new_header.name = ByteArray::try_from(name.as_bytes())?;
                    new_header.value = ByteArray::try_from(value.as_bytes())?;
                    *header = Some(new_header);
                    self.num_headers += 1;
                    return Ok(self);
                }
            };
        }

        error!("Too many headers");
        Err(Error::new(ErrorKind::Server, Some("Too many headers")))
    }

    pub fn append_body(mut self, body: &[u8]) -> Result<Self, Error> {
        self.write_all(body)?;
        Ok(self)
    }

    pub fn body(mut self, body: &[u8]) -> Self {
        self.body_written = body.len();
        self.body[..self.body_written].copy_from_slice(body);
        self.body_len = self.body_written;
        self
    }

    pub fn build(self) -> Request<HV> {
        Request {
            path: self.path,
            method: self.method,
            headers: self.headers,
            body: self.body,
            body_len: self.body_len,
            body_written: self.body_written,
            num_headers: self.num_headers,
        }
    }
}

impl<const HV: usize> Write for RequestBuilder<HV> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        trace!("Writing to request builder");
        let remaining = self.body.len() - self.body_written;
        let write_amt = std::cmp::min(remaining, buf.len());

        self.body[self.body_written..self.body_written + write_amt]
            .copy_from_slice(&buf[..write_amt]);
        self.body_written += write_amt;
        self.body_len = self.body_written;

        trace!("Wrote {} bytes", write_amt);

        Ok(write_amt)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
