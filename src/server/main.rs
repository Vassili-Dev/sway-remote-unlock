use code_buffer::CodeBuffer;
use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::status::Status;
use remote_unlock_lib::prelude::*;
use std::net::TcpListener;
use std::sync::mpsc::{self, Receiver};

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
        let mut stream = stream?;
        stream.set_nonblocking(true)?;
        context.replace_stream(stream);

        process_codes(&mut code_buffer, &server_recv);

        let req = match Request::from_stream(context.stream().ok_or(Error::new(
            ErrorKind::Server,
            Some("No stream for parsing request"),
        ))?) {
            Ok(req) => req,
            Err(_e) => {
                let error_resp = Response::new(Status::BadRequest);
                error_resp.to_writer(&mut stream);

                context.remove_stream();
                continue;
            }
        };

        if req.path.unwrap().as_str()? == "/enroll" && req.method.unwrap().as_str()? == "POST" {
            routes::enroll::EnrollRoute::new(&config, &mut stream, &mut code_buffer).post(req)?;
        } else if req.path.unwrap().as_str()? == "/unlock"
            && req.method.unwrap().as_str()? == "POST"
        {
            routes::unlock::UnlockRoute::new(&config, &mut stream).post(req)?;
        }

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
