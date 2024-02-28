use code_buffer::CodeBuffer;
use remote_unlock_lib::config::Config;
use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::errors::RemoteUnlockError;
use remote_unlock_lib::net::request::Request;
use std::net::TcpListener;
use std::sync::mpsc;

mod code_buffer;
mod routes;
mod socket;
mod state;

fn main() -> Result<(), RemoteUnlockError> {
    let config = Config::new();
    // TODO: Convert to crossbeam MPMC bounded channel
    let (sock_sender, server_recv) = mpsc::channel::<EnrollmentCode>();

    let sock_handle = socket::run_socket(sock_sender)?;

    let listener: TcpListener = TcpListener::bind("127.0.0.1:8142")?;
    let mut code_buffer = code_buffer::CodeBuffer::new();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        stream.set_nonblocking(true).unwrap();

        process_codes(&mut code_buffer, &server_recv);

        let req = Request::from_stream(&mut stream)?;

        if req.path.unwrap().as_str() == "/enroll" && req.method.unwrap().as_str() == "POST" {
            routes::enroll::EnrollRoute::new(&config, &mut stream, &mut code_buffer).post(req)?;
        } else if req.path.unwrap().as_str() == "/unlock" && req.method.unwrap().as_str() == "POST"
        {
            routes::unlock::UnlockRoute::new(&config, &mut stream).post(req)?;
        }
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
