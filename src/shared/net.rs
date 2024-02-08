use std::io::Write;

pub struct Response {
    pub status: u16,
    pub headers: [httparse::Header<'static>; 16],
    pub body: [u8; 1024],

    pub body_len: usize,
    body_written: usize,

    num_headers: usize,
}

impl Write for Response {
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

impl Response {
    fn new() -> Response {
        Response {
            status: 200,
            headers: [httparse::EMPTY_HEADER; 16],
            body: [0; 1024],
            body_len: 0,
            body_written: 0,
            num_headers: 0,
        }
    }

    pub fn add_header(&mut self, name: &'static str, value: &'static str) {
        for header in self.headers.iter_mut() {
            if header.name.is_empty() {
                header.name = name;
                header.value = value.as_bytes();
                self.num_headers += 1;
                return;
            }
        }
    }
}
