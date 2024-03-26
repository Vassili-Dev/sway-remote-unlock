use code_buffer::CodeBuffer;
use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::status::Status;
use remote_unlock_lib::prelude::*;
use std::net::TcpListener;
use std::sync::mpsc;

mod code_buffer;
mod context;
mod router;
mod routes;
mod socket;
mod state;

fn main() -> Result<(), Error> {
    let config = Config::new();
    // TODO: Convert to crossbeam MPMC bounded channel
    let (sock_sender, server_recv) = mpsc::channel::<EnrollmentCode>();

    let sock_handle = socket::run_socket(sock_sender)?;

    let listener: TcpListener =
        TcpListener::bind((config.server_hostname(), config.server_port()))?;
    let mut code_buffer = code_buffer::CodeBuffer::new();

    let mut context = context::ServerContext::builder()
        .config(&config)
        .code_receiver(server_recv)
        .state(state::State::new())
        .build()?;

    for stream in listener.incoming() {
        let stream = stream?;
        stream.set_nonblocking(true)?;
        context.replace_stream(stream);

        process_codes(&mut code_buffer, context.code_receiver());

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

fn process_codes(buffer: &mut CodeBuffer, recv: &mpsc::Receiver<EnrollmentCode>) {
    // Clear expired codes from the buffer and shift the rest down
    buffer.clear_expired();

    // Drain the code channel into the buffer
    'buffer_drain: while let Ok(code) = recv.try_recv() {
        match buffer.insert(code) {
            Ok(_) => {}
            Err(_) => {
                println!("Code buffer full, ignoring code {:?}", code);
                break 'buffer_drain;
            }
        }
    }
}
