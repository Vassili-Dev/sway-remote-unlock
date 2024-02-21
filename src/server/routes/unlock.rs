use std::io::Write;

use remote_unlock_lib::{
    config::Config, errors::RemoteUnlockError, net::request::Request, net::response::Response,
    unlock_request::UnlockRequest,
};

use crate::code_buffer::CodeBuffer;

pub struct UnlockRoute<'a> {
    config: &'a Config,
    stream: &'a mut std::net::TcpStream,
}

impl<'a> UnlockRoute<'a> {
    pub fn new(config: &'a Config, stream: &'a mut std::net::TcpStream) -> UnlockRoute<'a> {
        UnlockRoute { config, stream }
    }
    pub fn post(&mut self, req: Request) -> Result<(), RemoteUnlockError> {
        // Parse the body of the request
        let body_str = std::str::from_utf8(&req.body[..req.body_len]).unwrap();
        let unlock_req = serde_json::from_str::<UnlockRequest>(body_str);
        let mut resp = Response::new();

        match unlock_req {
            Ok(unlock_req) => {
                // Get the signature header
                let signature_header = req
                    .headers
                    .iter()
                    .find(|h| h.is_some() && h.as_ref().unwrap().name.as_str() == "Signature");

                if signature_header.is_none() {
                    resp.status = remote_unlock_lib::net::status::Status::BadRequest;
                    resp.to_writer(self.stream).unwrap();
                    self.stream.flush().unwrap();
                    return Ok(());
                }

                let signature_header = signature_header.unwrap().as_ref().unwrap();

                // Try to retrieve the public key from storage
                let pubkey = self
                    .config
                    .pubkey()
                    .load(self.config.storage_dir(), unlock_req.id.as_str());
                // let code = enroll_req.code();
                // let enroll_response = enroll_response::EnrollmentResponse::new();

                // let mut id: [u8; 32] = [0; 32];
                // enroll_response.id().as_simple().encode_lower(&mut id);

                // enroll_req
                //     .pubkey()
                //     .save(self.config.storage_dir(), std::str::from_utf8(&id).unwrap());

                // if self.code_buffer.verify(code) {
                //     resp.status = remote_unlock_lib::net::status::Status::Ok;
                //     resp.add_header("Content-Type", "application/json");
                //     serde_json::to_writer(&mut resp, &enroll_response).unwrap();
                //     resp.to_writer(self.stream).unwrap();
                //     self.stream.flush().unwrap();
                // } else {
                //     resp.status = remote_unlock_lib::net::status::Status::Forbidden;
                //     resp.to_writer(self.stream).unwrap();
                //     self.stream.flush().unwrap();
                // }
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
