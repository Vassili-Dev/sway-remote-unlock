use code_buffer::CodeBuffer;
use remote_unlock_lib::config::{self, Config};
use remote_unlock_lib::enroll_request::EnrollmentRequest;
use remote_unlock_lib::enroll_response;
use remote_unlock_lib::enrollment_code::EnrollmentCode;
// use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use remote_unlock_lib::errors::RemoteUnlockError;
use remote_unlock_lib::net::response::Response;
// use remote_unlock_lib::messages::Message;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::thread;
use std::{io::Read, net::TcpListener};

pub mod code_buffer;

// Opens a Unix socket and returns its listener.
fn open_socket(sock_path: &str) -> std::io::Result<UnixListener> {
    let path = std::path::Path::new(sock_path);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    UnixListener::bind(sock_path)
}

fn main() -> Result<(), RemoteUnlockError> {
    let config = Config::new();
    // TODO: Convert to crossbeam MPMC bounded channel
    let (sock_sender, server_recv) = mpsc::channel::<EnrollmentCode>();

    thread::spawn(move || {
        let config = Config::new();
        let sock: UnixListener = open_socket(config.socket_path()).unwrap();
        let mut sock_buf = [0; Config::MAX_PACKET_SIZE];

        for stream in sock.incoming() {
            let mut stream = stream.unwrap();
            sock_buf.fill(0);
            let read_amt = stream.read(&mut sock_buf).unwrap();

            if read_amt > Config::MAX_PACKET_SIZE - 1 {
                println!(
                    "Received more than {} bytes from socket, ignoring",
                    Config::MAX_PACKET_SIZE
                );
                continue;
            }

            let mut sock_headers = [httparse::EMPTY_HEADER; 16];
            sock_headers.fill(httparse::EMPTY_HEADER);
            let mut sock_req = httparse::Request::new(&mut sock_headers);
            let body_pointer: httparse::Status<usize> = sock_req.parse(&sock_buf).unwrap();
            stream.shutdown(std::net::Shutdown::Read).unwrap();

            if sock_req.path.unwrap() == "/begin_enroll" && sock_req.method.unwrap() == "POST" {
                let code: EnrollmentCode = EnrollmentCode::new();

                let mut resp = Response::new();
                serde_json::to_writer(&mut resp, &code).unwrap();
                resp.add_header("Content-Type", "application/json");
                resp.to_writer(&mut stream).unwrap();
                sock_sender.send(code).unwrap();
            }
            stream.shutdown(std::net::Shutdown::Write).unwrap();
        }
    });

    let mut buf = [0; Config::MAX_PACKET_SIZE];
    let listener: TcpListener = TcpListener::bind("127.0.0.1:8142")?;
    let mut code_buffer = code_buffer::CodeBuffer::new();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        stream.read(&mut buf).unwrap();

        process_codes(&mut code_buffer, &server_recv);

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let body_pointer: httparse::Status<usize> = req.parse(&buf).unwrap();

        if req.path.unwrap() == "/enroll" && req.method.unwrap() == "POST" {
            let content_length = req
                .headers
                .iter()
                .find(|h| h.name == "Content-Length")
                .unwrap()
                .value;

            let content_length = std::str::from_utf8(content_length)
                .unwrap()
                .parse::<usize>()
                .unwrap();

            // Parse the body of the request
            let body = &buf[body_pointer.unwrap()..body_pointer.unwrap() + content_length];
            let body_str = std::str::from_utf8(body).unwrap();
            let enroll_req = serde_json::from_str::<EnrollmentRequest>(body_str);
            let mut resp = Response::new();

            match enroll_req {
                Ok(enroll_req) => {
                    let code = enroll_req.code();
                    let enroll_response = enroll_response::EnrollmentResponse::new();

                    let mut id: [u8; 32] = [0; 32];
                    enroll_response.id().as_simple().encode_lower(&mut id);

                    enroll_req
                        .pubkey()
                        .save(&config.storage_dir(), std::str::from_utf8(&id).unwrap());

                    if code_buffer.verify(code) {
                        resp.status = remote_unlock_lib::net::status::Status::Ok;
                        resp.add_header("Content-Type", "application/json");
                        serde_json::to_writer(&mut resp, &enroll_response).unwrap();
                        resp.to_writer(&mut stream).unwrap();
                        stream.flush().unwrap();
                    } else {
                        resp.status = remote_unlock_lib::net::status::Status::Forbidden;
                        resp.to_writer(&mut stream).unwrap();
                        stream.flush().unwrap();
                    }
                }
                Err(e) => {
                    resp.status = remote_unlock_lib::net::status::Status::BadRequest;
                    resp.to_writer(&mut stream).unwrap();
                    println!("Error parsing enrollment request: {:?}", e);
                    stream.flush().unwrap();
                }
            }
        }
    }

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
