use remote_unlock_lib::{
    config::Config,
    enrollment_code::EnrollmentCode,
    errors::RemoteUnlockError,
    net::{request::Request, response::Response},
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

pub fn run_socket(
    code_channel_sender: Sender<EnrollmentCode>,
) -> Result<JoinHandle<()>, RemoteUnlockError> {
    let handle = thread::spawn(move || {
        let config = Config::new();
        let sock: UnixListener = open_socket(config.socket_path()).unwrap();

        for stream in sock.incoming() {
            let mut stream = stream.unwrap();
            let sock_req = Request::from_stream(&mut stream).unwrap();
            stream.shutdown(std::net::Shutdown::Read).unwrap();

            if sock_req.path.unwrap().as_str() == "/begin_enroll"
                && sock_req.method.unwrap().as_str() == "POST"
            {
                let code: EnrollmentCode = EnrollmentCode::new();

                let mut resp = Response::new();
                serde_json::to_writer(&mut resp, &code).unwrap();
                resp.add_header("Content-Type", "application/json");
                resp.to_writer(&mut stream).unwrap();
                code_channel_sender.send(code).unwrap();
            }
            stream.shutdown(std::net::Shutdown::Write).unwrap();
        }
    });

    Ok(handle)
}
