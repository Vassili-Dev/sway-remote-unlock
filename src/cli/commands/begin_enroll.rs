use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::net::method::Method;
use remote_unlock_lib::net::request::Request;
use remote_unlock_lib::net::response::Response;
use remote_unlock_lib::net::status::Status;
use remote_unlock_lib::prelude::*;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;

pub fn begin_enroll(config: &Config) -> Result<(), Error> {
    let mut stream = UnixStream::connect(config.socket_path())?;
    let req = Request::<{ 64 * 2 }>::builder()
        .method(Method::POST)
        .path("/begin_enroll")
        .build();

    req.to_writer(&mut stream)?;
    stream.shutdown(Shutdown::Write)?;
    let response = Response::from_stream(&mut stream)?;

    if response.status != Status::Ok {
        let err = Error::new(ErrorKind::Server, Some(response.status.to_string()));
        return Err(err);
    }

    let code = match serde_json::from_slice::<EnrollmentCode>(&response.body[..response.body_len]) {
        Ok(c) => c,
        Err(e) => {
            println!("Response: {:?}", response);
            println!("Headers: {:?}", response.headers);
            println!("Body: {:?}", std::str::from_utf8(&response.body)?);
            panic!("Error parsing response: {}", e);
        }
    };

    println!("{}", code);

    Ok(())
}
