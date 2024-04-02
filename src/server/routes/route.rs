use std::io::Write;

use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::{method::Method, request::Request};
use remote_unlock_lib::prelude::*;

use crate::context::ServerContext;

pub trait Route<'a, 'c: 'a, T: Write> {
    const PATH: &'static str;
    const METHOD: Method;

    fn run(&mut self, request: &Request) -> Result<Response, Error>;
    fn context(&mut self) -> &mut ServerContext<'c, T>;

    fn post_run(&mut self, response: &Response) -> Result<(), Error>;

    fn new(context: &'a mut ServerContext<'c, T>) -> Self;

    fn write_response(&mut self, response: &Response) -> Result<(), Error> {
        response.to_writer(self.context().stream()?)?;
        Ok(())
    }

    fn match_route(request: &Request) -> Result<bool, Error> {
        let path = request
            .path
            .as_ref()
            .map(|path_buf| path_buf.as_str())
            .unwrap_or(Ok(""))?;

        if (path == Self::PATH) && (request.method == Some(Self::METHOD)) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
