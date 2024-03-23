use std::io::Write;

use remote_unlock_lib::{
    enroll_request::EnrollmentRequest, enroll_response, net::request::Request,
    net::response::Response, prelude::*,
};

use crate::code_buffer::CodeBuffer;

pub struct EnrollRoute<'a, T: Write> {
    config: &'a Config,
    stream: &'a mut T,
    code_buffer: &'a mut CodeBuffer,
}

impl<'a, T: Write> EnrollRoute<'a, T> {
    pub fn new(
        config: &'a Config,
        stream: &'a mut T,
        code_buffer: &'a mut CodeBuffer,
    ) -> EnrollRoute<'a, T> {
        EnrollRoute {
            config,
            stream,
            code_buffer,
        }
    }
    pub fn post(&mut self, req: Request) -> Result<(), Error> {
        // Parse the body of the request
        let body_str = std::str::from_utf8(&req.body[..req.body_len]).unwrap();
        let enroll_req = serde_json::from_str::<EnrollmentRequest>(body_str);
        let mut resp = Response::new();

        match enroll_req {
            Ok(enroll_req) => {
                let code = enroll_req.code();
                let enroll_response = enroll_response::EnrollmentResponse::new();

                let mut id_buf: [u8; 32] = [0; 32];
                let id = enroll_response.id().as_simple().encode_lower(&mut id_buf);

                let pem = enroll_req.pubkey_pem();
                let pubkey = remote_unlock_lib::crypto::key::PublicKey::from_pem(pem.as_bytes())?;
                let path = std::path::Path::new(self.config.storage_dir()).join(id);
                pubkey.save_to_pem_file(path.as_path())?;

                if self.code_buffer.verify(code) {
                    resp.status = remote_unlock_lib::net::status::Status::Ok;
                    resp.add_header("Content-Type", "application/json")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use remote_unlock_lib::enrollment_code::EnrollmentCode;
    const PUBKEY_PEM: &'static str = include_str!("../../../test_data/pem_test.pub");

    #[test]
    fn test_post() {
        let config = Config::new();
        let mut mock_server = ByteArray::<{ Config::MAX_PACKET_SIZE * 2 }>::new();
        let mut code_buffer = CodeBuffer::new();
        let enrollment_code = EnrollmentCode::new();

        let code_num = enrollment_code.code();
        code_buffer.insert(enrollment_code).unwrap();
        let mut enroll_route = EnrollRoute::new(&config, &mut mock_server, &mut code_buffer);

        let pubkey = ByteArray::try_from(PUBKEY_PEM.as_bytes()).unwrap();

        let enroll_req = EnrollmentRequest::new(code_num, pubkey);

        let mut req = Request::new();
        serde_json::to_writer(&mut req, &enroll_req).unwrap();
        req.flush().unwrap();

        enroll_route.post(req).unwrap();

        let resp = Response::from_stream(&mut mock_server).unwrap();

        assert!(resp.status == remote_unlock_lib::net::status::Status::Ok);
    }
}
