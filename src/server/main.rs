use code_buffer::CodeBuffer;
use remote_unlock_lib::enrollment_code::EnrollmentCode;
// use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use remote_unlock_lib::errors::RemoteUnlockError;
use serde::Serialize;
// use remote_unlock_lib::messages::Message;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::thread;
use std::{io::Read, net::TcpListener};

const SOCKET_PATH: &str = "/tmp/remote_unlock.sock";

pub mod code_buffer;

// Opens a Unix socket and returns its listener.
fn open_socket() -> std::io::Result<UnixListener> {
    let path = std::path::Path::new(SOCKET_PATH);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    UnixListener::bind(SOCKET_PATH)
}

fn main() -> Result<(), RemoteUnlockError> {
    // TODO: Convert to crossbeam MPMC bounded channel
    let (sock_sender, server_recv) = mpsc::channel::<EnrollmentCode>();

    thread::spawn(move || {
        let sock: UnixListener = open_socket().unwrap();
        let mut sock_buf = [0; 1024];

        for stream in sock.incoming() {
            sock_buf.fill(0);
            let mut stream = stream.unwrap();
            let read_amt = stream.read(&mut sock_buf).unwrap();

            if read_amt > 1023 {
                println!("Received more than 1023 bytes from socket, ignoring");
                continue;
            }

            let mut sock_headers = [httparse::EMPTY_HEADER; 16];
            let mut sock_req = httparse::Request::new(&mut sock_headers);
            let body_pointer: httparse::Status<usize> = sock_req.parse(&sock_buf).unwrap();

            if sock_req.path.unwrap() == "/begin_enroll" && sock_req.method.unwrap() == "POST" {
                let code: EnrollmentCode = EnrollmentCode::new();
                stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                let res = serde_json::to_writer(&mut stream, &code);

                stream.flush().unwrap();
                sock_sender.send(code).unwrap();

                // sock_sender.send(Message::new(path, code)).unwrap();
            }
        }
    });

    let mut buf = [0; 1024];
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
            // Parse the body of the request
            let body = &buf[body_pointer.unwrap()..];
            let body_str = std::str::from_utf8(body).unwrap();
            let code = body_str[0..6].parse();

            match code {
                Ok(code) => {
                    if code_buffer.verify(code) {
                        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                        stream.flush().unwrap();
                    } else {
                        stream.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n").unwrap();
                        stream.flush().unwrap();
                    }
                }
                Err(_) => {
                    stream
                        .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                        .unwrap();
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
