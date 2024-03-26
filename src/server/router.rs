use std::net::TcpStream;

use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::status::Status;
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
        let mut route = match request {
            request if EnrollRoute::<TcpStream>::match_route(request)? => {
                Routes::Enroll(EnrollRoute::new(context))
            }
            request if UnlockRoute::<TcpStream>::match_route(request)? => {
                Routes::Unlock(UnlockRoute::new(context))
            }
            _ => Routes::NotFound(NotFound::new(context)),
        };

        let resp = route
            .run(request)
            .unwrap_or(Response::new(Status::InternalServerError));

        resp.to_writer(context.stream()?)?;
        Ok(())
    }
}
