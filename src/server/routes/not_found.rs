use std::io::Write;
use std::net::TcpStream;

use remote_unlock_lib::net::method::Method;
use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::status::Status;
use remote_unlock_lib::prelude::*;

use crate::context::ServerContext;

use super::route::Route;

pub struct NotFound<'a, 'c: 'a, T: Write = TcpStream> {
    context: &'a mut ServerContext<'c, T>,
}

impl<'a, 'c: 'a, T: Write> Route<'a, 'c, T> for NotFound<'a, 'c, T> {
    const PATH: &'static str = "/404";
    const METHOD: Method = Method::GET;

    fn new(context: &'a mut ServerContext<'c, T>) -> Self {
        Self { context }
    }

    fn run(&mut self, _req: &Request) -> Result<(), Error> {
        let mut response = Response::new(Status::NotFound);
        response.add_header("Content-Type", "text/plain")?;
        response.write("Not found".as_bytes())?;

        response.to_writer(self.context.stream()?)?;
        Ok(())
    }
}
