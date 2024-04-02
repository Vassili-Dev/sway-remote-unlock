use std::net::TcpStream;

use crate::context::ServerContext;
use crate::routes::enroll::EnrollRoute;
use crate::routes::not_found::NotFound;
use crate::routes::route::Route;
use crate::routes::unlock::UnlockRoute;
use crate::routes::Routes;
use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::status::Status;
use remote_unlock_lib::prelude::*;

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
                trace!("Routing to Enroll handler");
                Routes::Enroll(EnrollRoute::new(context))
            }
            request if UnlockRoute::<TcpStream>::match_route(request)? => {
                trace!("Routing to Unlock handler");
                Routes::Unlock(UnlockRoute::new(context))
            }
            _ => {
                trace!("Unknown route");
                Routes::NotFound(NotFound::new(context))
            }
        };

        let resp = route
            .run(request)
            .unwrap_or(Response::new(Status::InternalServerError));

        trace!("Writing response to stream");
        route.write_response(&resp)?;

        trace!("Response sent");
        match route.post_run(&resp) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to run post route: {}", e);
            }
        }

        Ok(())
    }
}
