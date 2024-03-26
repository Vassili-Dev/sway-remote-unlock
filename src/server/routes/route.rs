use std::io::Write;

use remote_unlock_lib::net::{method::Method, request::Request};
use remote_unlock_lib::prelude::*;
use uuid::timestamp::context;

use crate::context::ServerContext;

pub trait Route {
    const PATH: &'static str;
    const METHOD: Method;

    fn run(&self, request: &Request) -> Result<(), Error>;

    fn new<T: Write>(context: &mut ServerContext<T>) -> Self;
}
