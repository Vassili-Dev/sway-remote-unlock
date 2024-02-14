use remote_unlock_lib::{
    config::Config, enrollment_code::EnrollmentCode, errors::RemoteUnlockError,
    net::response::Response,
};
use std::{
    io::Read,
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
            let sock_req = httparse::Request::new(&mut sock_headers);
            // let body_pointer: httparse::Status<usize> = sock_req.parse(&sock_buf).unwrap();
            stream.shutdown(std::net::Shutdown::Read).unwrap();

            if sock_req.path.unwrap() == "/begin_enroll" && sock_req.method.unwrap() == "POST" {
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
