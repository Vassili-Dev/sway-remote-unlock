use std::io::Write;

use remote_unlock_lib::net::{method::Method, request::Request};
use remote_unlock_lib::prelude::*;

use crate::context::ServerContext;

pub trait Route<'a, 'c: 'a, T: Write> {
    const PATH: &'static str;
    const METHOD: Method;

    fn run(&mut self, request: &Request) -> Result<(), Error>;

    fn new(context: &'a mut ServerContext<'c, T>) -> Self;
}
