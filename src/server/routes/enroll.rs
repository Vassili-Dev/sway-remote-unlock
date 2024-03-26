use std::{io::Write, net::TcpStream};

use remote_unlock_lib::{
    enroll_request::EnrollmentRequest,
    enroll_response,
    net::{request::Request, response::Response, status::Status},
    prelude::*,
};

use crate::context::ServerContext;

use super::route::Route;

pub struct EnrollRoute<'a, 'c: 'a, T: Write = TcpStream> {
    context: &'a mut ServerContext<'c, T>,
}

impl<'a, 'c: 'a, T: Write> Route<'a, 'c, T> for EnrollRoute<'a, 'c, T> {
    const PATH: &'static str = "/enroll";
    const METHOD: remote_unlock_lib::net::method::Method =
        remote_unlock_lib::net::method::Method::POST;

    fn new(context: &'a mut ServerContext<'c, T>) -> EnrollRoute<'a, 'c, T> {
        Self { context }
    }
    fn run(&mut self, req: &Request) -> Result<(), Error> {
        // Parse the body of the request
        let body_str = std::str::from_utf8(&req.body[..req.body_len]).unwrap();
        let enroll_req = serde_json::from_str::<EnrollmentRequest>(body_str);
        let builder = Response::builder();

        match enroll_req {
            Ok(enroll_req) => {
                let code = enroll_req.code();
                let enroll_response = enroll_response::EnrollmentResponse::new();

                let mut id_buf: [u8; 32] = [0; 32];
                let id = enroll_response.id().as_simple().encode_lower(&mut id_buf);

                let pem = enroll_req.pubkey_pem();
                let pubkey = remote_unlock_lib::crypto::key::PublicKey::from_pem(pem.as_bytes())?;
                let path = std::path::Path::new(self.context.config().storage_dir()).join(id);
                pubkey.save_to_pem_file(path.as_path())?;

                if self.context.state().code_buffer().verify(code) {
                    let mut resp = builder
                        .status(Status::Ok)
                        .add_header("Content-Type", "application/json")?
                        .build();
                    serde_json::to_writer(&mut resp, &enroll_response).unwrap();
                    resp.to_writer(self.context.stream()?)?;
                    self.context.stream()?.flush()?;
                } else {
                    let resp = builder.status(Status::Forbidden).build();
                    resp.to_writer(self.context.stream()?)?;
                    self.context.stream()?.flush()?;
                }
            }
            Err(e) => {
                let resp = builder.status(Status::BadRequest).build();
                resp.to_writer(self.context.stream()?)?;
                println!("Error parsing enrollment request: {:?}", e);
                self.context.stream()?.flush()?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::{context, state::State};

    use super::*;
    use remote_unlock_lib::enrollment_code::EnrollmentCode;
    const PUBKEY_PEM: &'static str = include_str!("../../../test_data/pem_test.pub");

    #[test]
    fn test_post() {
        let config = Config::new();
        let mock_server = ByteArray::<{ Config::MAX_PACKET_SIZE * 2 }>::new();
        let mut context: ServerContext<ByteArray<{ Config::MAX_PACKET_SIZE * 2 }>> =
            context::ServerContext::builder()
                .config(&config)
                .code_receiver(mpsc::channel::<EnrollmentCode>().1)
                .state(State::new())
                .stream(mock_server)
                .build()
                .unwrap();
        let enrollment_code = EnrollmentCode::new();

        let code_num = enrollment_code.code();
        context
            .state()
            .code_buffer()
            .insert(enrollment_code)
            .unwrap();
        let mut enroll_route = EnrollRoute::new(&mut context);

        let pubkey = ByteArray::try_from(PUBKEY_PEM.as_bytes()).unwrap();

        let enroll_req = EnrollmentRequest::new(code_num, pubkey);

        let mut req = Request::new();
        serde_json::to_writer(&mut req, &enroll_req).unwrap();
        req.flush().unwrap();

        enroll_route.run(&req).unwrap();

        let resp = Response::from_stream(&mut context.stream().unwrap()).unwrap();

        assert!(resp.status == remote_unlock_lib::net::status::Status::Ok);
    }
}
