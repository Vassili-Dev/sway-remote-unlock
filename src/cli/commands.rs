use remote_unlock_lib::config::Config;
use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::errors::{OversizePacketError, RemoteUnlockError, ServerError};
use remote_unlock_lib::net::request::Request;
use std::io::Read;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::thread;

pub fn begin_enroll(config: &Config) -> Result<(), RemoteUnlockError> {
    let mut stream = UnixStream::connect(config.socket_path()).unwrap();
    let mut req = Request::new();
    req.method = Some("POST");
    req.path = Some("/begin_enroll");
    let mut buf: [u8; Config::MAX_PACKET_SIZE] = [0; Config::MAX_PACKET_SIZE];

    req.to_writer(&mut stream).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    thread::sleep(std::time::Duration::from_millis(50));

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut response = httparse::Response::new(&mut headers);

    let read_amt = stream.read(&mut buf).unwrap();
    let status = match response.parse(&buf) {
        Ok(httparse::Status::Complete(i)) => Some(i),
        Ok(httparse::Status::Partial) => None,
        Err(e) => {
            println!("Error parsing response: {}", e);
            println!("Response: {:?}", std::str::from_utf8(&buf).unwrap());
            None
        }
    };

    if status.is_none() {
        return Err(OversizePacketError.into());
    }

    if response.code.unwrap() != 200 {
        let err = ServerError::new(format!(
            "Server returned status code {}",
            response.code.unwrap()
        ));
        return Err(err.into());
    }

    let content_length = response
        .headers
        .iter()
        .find(|header| header.name == "Content-Length")
        .unwrap()
        .value;

    let content_length = std::str::from_utf8(content_length)
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let body = &buf[status.unwrap()..status.unwrap() + content_length];
    let code = serde_json::from_slice::<EnrollmentCode>(body).unwrap();

    println!("{}", code);

    Ok(())
}
