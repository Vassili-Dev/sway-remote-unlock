use remote_unlock_lib::config::Config;
use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::errors::{RemoteUnlockError, ServerError};
use remote_unlock_lib::helper_types::ByteArray;
use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::status::Status;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
pub fn begin_enroll(config: &Config) -> Result<(), RemoteUnlockError> {
    let mut stream = UnixStream::connect(config.socket_path()).unwrap();
    let mut req = Request::new();
    req.method = Some(ByteArray::new_from_slice("POST".as_bytes()));
    req.path = Some(ByteArray::new_from_slice("/begin_enroll".as_bytes()));

    req.to_writer(&mut stream).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let response = Response::from_stream(&mut stream).unwrap();

    if response.status != Status::Ok {
        let err = ServerError::new(format!(
            "Server returned status code {}",
            response.status.to_u16()
        ));
        return Err(err.into());
    }

    let code = match serde_json::from_slice::<EnrollmentCode>(&response.body[..response.body_len]) {
        Ok(c) => c,
        Err(e) => {
            println!("Response: {:?}", response);
            println!("Headers: {:?}", response.headers);
            println!("Body: {:?}", std::str::from_utf8(&response.body).unwrap());
            panic!("Error parsing response: {}", e);
        }
    };

    println!("{}", code);

    Ok(())
}
