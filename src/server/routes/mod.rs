use crate::routes::route::Route;
use remote_unlock_lib::net::request::Request;
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
    pub fn run(&mut self, request: &Request) -> Result<(), Error> {
        match self {
            Routes::Enroll(route) => route.run(request),
            Routes::NotFound(route) => route.run(request),
            Routes::Unlock(route) => route.run(request),
        }
    }
}
