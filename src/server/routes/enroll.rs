use std::io::Write;

use remote_unlock_lib::{
    config::Config, enroll_request::EnrollmentRequest, enroll_response, errors::RemoteUnlockError,
    net::request::Request, net::response::Response,
};

use crate::code_buffer::CodeBuffer;

pub struct EnrollRoute<'a> {
    config: &'a Config,
    stream: &'a mut std::net::TcpStream,
    code_buffer: &'a mut CodeBuffer,
}

impl<'a> EnrollRoute<'a> {
    pub fn new(
        config: &'a Config,
        stream: &'a mut std::net::TcpStream,
        code_buffer: &'a mut CodeBuffer,
    ) -> EnrollRoute<'a> {
        EnrollRoute {
            config,
            stream,
            code_buffer,
        }
    }
    pub fn post(&mut self, req: Request) -> Result<(), RemoteUnlockError> {
        // Parse the body of the request
        let body_str = std::str::from_utf8(&req.body[..req.body_len]).unwrap();
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
                    .save(self.config.storage_dir(), std::str::from_utf8(&id).unwrap());

                if self.code_buffer.verify(code) {
                    resp.status = remote_unlock_lib::net::status::Status::Ok;
                    resp.add_header("Content-Type", "application/json");
                    serde_json::to_writer(&mut resp, &enroll_response).unwrap();
                    resp.to_writer(self.stream).unwrap();
                    self.stream.flush().unwrap();
                } else {
                    resp.status = remote_unlock_lib::net::status::Status::Forbidden;
                    resp.to_writer(self.stream).unwrap();
                    self.stream.flush().unwrap();
                }
            }
            Err(e) => {
                resp.status = remote_unlock_lib::net::status::Status::BadRequest;
                resp.to_writer(self.stream).unwrap();
                println!("Error parsing enrollment request: {:?}", e);
                self.stream.flush().unwrap();
            }
        }

        Ok(())
    }
}
