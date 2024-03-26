use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::status::Status;
use remote_unlock_lib::prelude::*;
use std::net::TcpListener;
use std::sync::mpsc;

mod code_buffer;
mod context;
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

    let mut context = context::ServerContext::builder()
        .config(&config)
        .code_receiver(server_recv)
        .state(state::State::new())
        .build()?;

    context.init()?;

    debug!("Starting server");
    let listener: TcpListener =
        TcpListener::bind((config.server_hostname(), config.server_port()))?;
    info!(
        "Server started on {}:{}",
        config.server_hostname(),
        config.server_port()
    );

    for stream in listener.incoming() {
        let stream = stream?;
        stream.set_nonblocking(true)?;
        context.replace_stream(stream);
        context.process_codes()?;

        let req = match Request::from_stream(context.stream()?) {
            Ok(req) => req,
            Err(_e) => {
                let error_resp = Response::new(Status::BadRequest);
                error_resp.to_writer(context.stream()?)?;

                context.remove_stream();
                continue;
            }
        };

        let router = router::Router::new();

        router.route(&mut context, &req)?;

        context.remove_stream();
    }

    sock_handle.join().unwrap();

    Ok(())
}
