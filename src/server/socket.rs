use remote_unlock_lib::{
    enrollment_code::EnrollmentCode,
    net::{request::Request, response::Response, status::Status},
    prelude::*,
};
use std::{
    os::unix::net::UnixListener,
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

// Opens a Unix socket and returns its listener.
fn open_socket(sock_path: &str) -> std::io::Result<UnixListener> {
    let path = std::path::Path::new(sock_path);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    UnixListener::bind(sock_path)
}

pub fn run_socket(code_channel_sender: Sender<EnrollmentCode>) -> Result<JoinHandle<()>, Error> {
    let handle = thread::spawn(move || {
        let config = Config::new();
        let sock: UnixListener = open_socket(config.socket_path()).unwrap();

        for stream in sock.incoming() {
            let mut stream = stream.unwrap();
            let sock_req = Request::from_stream(&mut stream).unwrap();
            stream.shutdown(std::net::Shutdown::Read).unwrap();

            let path_array = sock_req.path.unwrap();
            let path_str = match path_array.as_str() {
                Ok(s) => s,
                Err(_) => {
                    stream.shutdown(std::net::Shutdown::Write).unwrap();
                    continue;
                }
            };

            let method_array = sock_req.method;
            let method_str = match method_array {
                Some(s) => s.as_str(),
                None => {
                    stream.shutdown(std::net::Shutdown::Write).unwrap();
                    continue;
                }
            };

            if path_str == "/begin_enroll" && method_str == "POST" {
                let code: EnrollmentCode = EnrollmentCode::new();

                let mut resp = Response::new(Status::Ok);
                serde_json::to_writer(&mut resp, &code).unwrap();
                match resp.add_header("Content-Type", "application/json") {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error adding header, {}", e);
                        stream.shutdown(std::net::Shutdown::Write).unwrap();
                        continue;
                    }
                };

                resp.to_writer(&mut stream).unwrap();
                code_channel_sender.send(code).unwrap();
            }
            stream.shutdown(std::net::Shutdown::Write).unwrap();
        }
    });

    Ok(handle)
}
