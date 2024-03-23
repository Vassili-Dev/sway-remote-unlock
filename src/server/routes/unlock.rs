use std::io::Write;
use std::path::Path;

use remote_unlock_lib::crypto::key::PublicKey;
use remote_unlock_lib::{
    net::{request::Request, response::Response},
    prelude::*,
    unlock_request::UnlockRequest,
};

use base64::prelude::*;

pub struct UnlockRoute<'a> {
    config: &'a Config,
    stream: &'a mut std::net::TcpStream,
}

impl<'a> UnlockRoute<'a> {
    pub fn new(config: &'a Config, stream: &'a mut std::net::TcpStream) -> UnlockRoute<'a> {
        UnlockRoute { config, stream }
    }
    pub fn post(&mut self, req: Request) -> Result<(), Error> {
        // Parse the body of the request
        let body_str = std::str::from_utf8(&req.body[..req.body_len]).unwrap();
        let unlock_req = serde_json::from_str::<UnlockRequest>(body_str);
        let mut resp = Response::new();

        match unlock_req {
            Ok(unlock_req) => {
                // Get the signature header
                let signature_header = req.headers.iter().find(|h| {
                    h.is_some()
                        && h.as_ref().unwrap().name.as_str().unwrap_or("")
                            == "X-RemoteUnlock-Signature"
                });

                if signature_header.is_none() {
                    println!("{:?}", req.headers);
                    resp.status = remote_unlock_lib::net::status::Status::BadRequest;
                    resp.to_writer(self.stream).unwrap();
                    self.stream.flush().unwrap();
                    return Ok(());
                }

                let signature_header = signature_header.unwrap().as_ref().unwrap();

                let pubkey_path = Path::new(self.config.storage_dir())
                    .join(std::str::from_utf8(unlock_req.id()).unwrap());

                if !pubkey_path.exists() {
                    resp.status = remote_unlock_lib::net::status::Status::NotFound;
                    resp.to_writer(self.stream).unwrap();
                    self.stream.flush().unwrap();
                    return Ok(());
                }

                // Try to retrieve the public key from storage
                let pubkey = match PublicKey::read_pem_file(pubkey_path.as_path()) {
                    Ok(pubkey) => pubkey,
                    Err(_) => {
                        resp.status = remote_unlock_lib::net::status::Status::InternalServerError;
                        resp.to_writer(self.stream).unwrap();
                        self.stream.flush().unwrap();
                        return Ok(());
                    }
                };

                let mut signature_bytes = [0u8; 1024];
                let signature_length = BASE64_STANDARD
                    .decode_slice(signature_header.value.as_bytes(), &mut signature_bytes)?;

                let valid_request =
                    unlock_req.verify(&signature_bytes[..signature_length], &pubkey)?;

                if valid_request {
                    resp.status = remote_unlock_lib::net::status::Status::Ok;
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
