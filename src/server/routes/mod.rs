use crate::routes::route::Route;
use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::prelude::*;

pub mod enroll;
pub mod not_found;
pub mod route;
pub mod unlock;

pub enum Routes<'a, 'c: 'a> {
    Enroll(enroll::EnrollRoute<'a, 'c>),
    NotFound(not_found::NotFound<'a, 'c>),
    Unlock(unlock::UnlockRoute<'a, 'c>),
}

impl<'a, 'c: 'a> Routes<'a, 'c> {
    pub fn run(&mut self, request: &Request) -> Result<Response, Error> {
        match self {
            Routes::Enroll(route) => route.run(request),
            Routes::NotFound(route) => route.run(request),
            Routes::Unlock(route) => route.run(request),
        }
    }

    pub fn write_response(&mut self, response: &Response) -> Result<(), Error> {
        match self {
            Routes::Enroll(route) => route.write_response(response),
            Routes::NotFound(route) => route.write_response(response),
            Routes::Unlock(route) => route.write_response(response),
        }
    }

    pub fn post_run(&mut self, response: &Response) -> Result<(), Error> {
        match self {
            Routes::Enroll(route) => route.post_run(response),
            Routes::NotFound(route) => route.post_run(response),
            Routes::Unlock(route) => route.post_run(response),
        }
    }
}
