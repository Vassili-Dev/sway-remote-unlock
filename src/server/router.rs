use std::net::TcpStream;

use remote_unlock_lib::net::method::Method;

use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::prelude::*;

use crate::context::ServerContext;
use crate::routes::enroll::EnrollRoute;
use crate::routes::not_found::NotFound;
use crate::routes::route::Route;
use crate::routes::unlock::UnlockRoute;
use crate::routes::Routes;

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
        let mut route = match (request.method(), request.path()) {
            (Some(Method::GET), Some(EnrollRoute::<TcpStream>::PATH)) => {
                Routes::Enroll(EnrollRoute::new(context))
            }
            (Some(Method::POST), Some(UnlockRoute::<TcpStream>::PATH)) => {
                Routes::Unlock(UnlockRoute::new(context))
            }
            _ => Routes::NotFound(NotFound::new(context)),
        };

        route.run(request)?;

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
