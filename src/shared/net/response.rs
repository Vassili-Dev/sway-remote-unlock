use crate::prelude::*;

use super::{headers::Header, status::Status};
use std::{io::Write, thread};

#[derive(Debug)]
pub struct Response<const HV: usize = { 64 * 2 }> {
    pub status: Status,
    pub headers: [Option<Header<32, HV>>; 16],
    pub body: [u8; 1024 * 2],

    pub body_len: usize,
    body_written: usize,

    num_headers: usize,
}

impl<const HV: usize> Write for Response<HV> {
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

impl<const HV: usize> Response<HV> {
    pub fn new(status: Status) -> Self {
        let headers = [None; 16];

        Self {
            status,
            headers,
            body: [0; 1024 * 2],
            body_len: 0,
            body_written: 0,
            num_headers: 0,
        }
    }

    pub fn builder() -> ResponseBuilder<HV> {
        ResponseBuilder::new()
    }

    pub fn add_header(&mut self, name: &'static str, value: &'static str) -> Result<(), Error> {
        for header in self.headers.iter_mut() {
            match header {
                Some(header) => {
                    if header.name.as_str()? == name {
                        header.value.copy_from_slice(value.as_bytes())?;
                        return Ok(());
                    }
                }
                None => {
                    let mut new_header = Header::new();
                    new_header.name.copy_from_slice(name.as_bytes())?;
                    new_header.value.copy_from_slice(value.as_bytes())?;
                    *header = Some(new_header);
                    self.num_headers += 1;
                    return Ok(());
                }
            };
        }

        error!("Too many headers");
        Err(Error::new(ErrorKind::Server, Some("Too many headers")))
    }

    pub fn to_writer(&self, writer: &mut impl Write) -> Result<(), Error> {
        trace!("Writing response to writer");

        trace!("Writing status line");
        writer.write_fmt(format_args!(
            "HTTP/1.1 {} {}\r\n",
            self.status.to_u16(),
            self.status.to_string()
        ))?;

        trace!("Writing headers");
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

        trace!("Writing content length header");
        writer.write_fmt(format_args!("Content-Length: {}\r\n", self.body_written))?;

        writer.write_all(b"\r\n")?;

        trace!("Writing response body");
        writer.write_all(&self.body[..self.body_written])?;

        trace!("Finished writing response to writer");
        Ok(())
    }

    pub fn from_stream(stream: &mut impl std::io::Read) -> Result<Self, Error> {
        trace!("Parsing response from stream");
        let mut builder = Self::builder();
        let mut buf = [0; Config::MAX_PACKET_SIZE];
        let mut buf_ptr = 0;

        // Read response into buffer
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
                // Wait for response
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

        trace!("Parsing httparse response");
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut response = httparse::Response::new(&mut headers);
        let status = match response.parse(&buf) {
            Ok(httparse::Status::Complete(i)) => i,
            Ok(httparse::Status::Partial) => return Err(ErrorKind::IncompleteRequest.into()),
            Err(e) => return Err(e.into()),
        };
        trace!("Finished parsing httparse request");

        trace!("Building response from parsed data");
        let code = (&response).code.ok_or(Error::new(
            ErrorKind::Server,
            Some("No status code in response"),
        ))?;

        if code != 200 {
            let code_str: ByteArray<5> = ByteArrayString::try_from(code)?.into();
            let message: ByteArray<{ Config::ERROR_STRING_SIZE }> =
                ByteArrayString::<{ Config::ERROR_STRING_SIZE }>::try_from(
                    "Server returned status code",
                )?
                .into();

            warn!("Received non-200 status code: {}", code_str.as_str()?);
            let mut message = ByteArray::from(message);
            message.append_slice(code_str.as_bytes())?;

            return Err(Error::new(ErrorKind::Server, Some(message.as_str()?)));
        }

        builder = builder.status(Status::Ok);

        trace!("Looking for content-length header");
        let content_length = match response
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
        for header in response.headers.iter() {
            if !header.name.is_empty() {
                let hv = std::str::from_utf8(header.value)?;

                trace!("Header: {}: {}", &header.name, hv);
                builder = builder.add_header(header.name, hv)?;
            }
        }

        let body_start_ptr = status;
        let body_end_ptr = status + content_length;

        let body = &buf[body_start_ptr..body_end_ptr];

        builder = builder.append_body(body)?;

        let ret = builder.build();

        trace!("Finished building response from stream");
        Ok(ret)
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new(Status::Ok)
    }
}

pub struct ResponseBuilder<const HV: usize = { 64 * 2 }> {
    status: Option<Status>,
    headers: [Option<Header<32, HV>>; 16],
    body: [u8; 1024 * 2],
    body_len: usize,
    body_written: usize,
    num_headers: usize,
}

impl<const HV: usize> ResponseBuilder<HV> {
    fn new() -> Self {
        Self {
            status: None,
            headers: [None; 16],
            body: [0; 1024 * 2],
            body_len: 0,
            body_written: 0,
            num_headers: 0,
        }
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    pub fn add_header(mut self, name: &str, value: &str) -> Result<Self, Error> {
        for header in self.headers.iter_mut() {
            match header {
                Some(header) => {
                    if header.name.as_str()? == name {
                        header.value.copy_from_slice(value.as_bytes())?;
                        return Ok(self);
                    }
                }
                None => {
                    let mut new_header = Header::new();
                    new_header.name.copy_from_slice(name.as_bytes())?;
                    new_header.value.copy_from_slice(value.as_bytes())?;
                    *header = Some(new_header);
                    self.num_headers += 1;
                    return Ok(self);
                }
            };
        }

        Err(Error::new(ErrorKind::Server, Some("Too many headers")))
    }

    pub fn body(mut self, body: &[u8]) -> Self {
        let remaining = self.body.len() - self.body_written;
        let write_amt = std::cmp::min(remaining, body.len());

        self.body[self.body_written..self.body_written + write_amt]
            .copy_from_slice(&body[..write_amt]);
        self.body_written += write_amt;

        self
    }

    pub fn append_body(mut self, body: &[u8]) -> Result<Self, Error> {
        self.write_all(body)?;
        Ok(self)
    }

    pub fn build(self) -> Response<HV> {
        Response::<HV> {
            status: self.status.unwrap_or(Status::Ok),
            headers: self.headers,
            body: self.body,
            body_len: self.body_len,
            body_written: self.body_written,
            num_headers: self.num_headers,
        }
    }
}
impl<const HV: usize> Write for ResponseBuilder<HV> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        trace!("Writing to response builder");
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
