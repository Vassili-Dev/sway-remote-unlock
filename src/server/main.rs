use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::status::Status;
use remote_unlock_lib::prelude::*;
use std::net::TcpListener;
use std::sync::mpsc;

mod backends;
mod code_buffer;
mod context;
mod discovery;
mod logging;
mod router;
mod routes;
mod socket;
mod state;

fn main() -> Result<(), Error> {
    let config = Config::new();

    // TODO: Convert to crossbeam MPMC bounded channel
    let (sock_sender, server_recv) = mpsc::channel::<EnrollmentCode>();

    let sock_handle = socket::run_socket(sock_sender)?;
    let discovery = discovery::start_discovery_daemon(&config)?;

    let mut context = context::ServerContext::builder()
        .config(&config)
        .code_receiver(server_recv)
        .state(state::State::new())
        .build()?;

    context.init()?;

    debug!("Starting server");
    let listener: TcpListener = TcpListener::bind((config.server_ip(), config.server_port()))?;
    info!(
        "Server started on {}:{}",
        config.server_ip(),
        config.server_port()
    );

    for stream in listener.incoming() {
        let stream = stream?;
        trace!("New connection from: {}", stream.peer_addr()?);
        stream.set_nonblocking(true)?;
        context.replace_stream(stream);

        context.process_codes()?;

        let req = match Request::from_stream(context.stream()?) {
            Ok(req) => req,
            Err(_e) => {
                let error_resp = Response::<{ 64 * 2 }>::new(Status::BadRequest);
                error_resp.to_writer(context.stream()?)?;

                context.remove_stream();
                continue;
            }
        };

        let router = router::Router::new();

        match router.route(&mut context, &req) {
            Ok(_) => (),
            Err(e) => {
                error!("Error routing request: {}", e);
                let error_resp = Response::<{ 64 * 2 }>::new(Status::InternalServerError);
                error_resp.to_writer(context.stream()?)?;
            }
        };

        context.remove_stream();
    }

    info!("Shutting down server");
    sock_handle.join().unwrap();
    discovery.shutdown().unwrap();

    Ok(())
}
