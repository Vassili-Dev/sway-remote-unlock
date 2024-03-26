use std::net::TcpStream;

use remote_unlock_lib::net::method::Method;
use remote_unlock_lib::net::{request::Request, response::Response};
use remote_unlock_lib::prelude::*;

use crate::context::ServerContext;
use crate::routes::enroll::EnrollRoute;

pub struct Router {}

impl Router {
    pub fn new() -> Self {
        Self {}
    }

    pub fn route(
        &self,
        context: &mut ServerContext<TcpStream>,
        request: &Request,
    ) -> Result<(), Error> {
        match (request.method(), request.path()) {
            (Some(Method::GET), Some(EnrollRoute::PATH)) => EnrollRoute::new(self.config),
            _ => {
                response.status = Status::NotFound;
                response.add_header("Content-Type", "text/plain")?;
                write!(response, "Not found")?;
            }
        }

        // match request.path.as_ref() {
        //     Some(path) => match path.as_str()? {
        //         "/hello" => {
        //             response.status = Status::OK;
        //             response.add_header("Content-Type", "text/plain")?;
        //             write!(response, "Hello, world!")?;
        //         }
        //         _ => {
        //             response.status = Status::NotFound;
        //             response.add_header("Content-Type", "text/plain")?;
        //             write!(response, "Not found")?;
        //         }
        //     },
        //     None => {
        //         response.status = Status::BadRequest;
        //         response.add_header("Content-Type", "text/plain")?;
        //         write!(response, "Bad request")?;
        //     }
        // }

        Ok(())
    }
}
