use std::io::{Read, Write};
use std::path::Path;

use remote_unlock_lib::pubkey::Pubkey;
use remote_unlock_lib::{
    config::Config,
    errors::RemoteUnlockError,
    net::{request::Request, response::Response},
    pubkey,
    unlock_request::UnlockRequest,
};

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

                let pubkey_path = Path::new(self.config.storage_dir())
                    .join(std::str::from_utf8(unlock_req.id()).unwrap());

                if !pubkey_path.exists() {
                    resp.status = remote_unlock_lib::net::status::Status::NotFound;
                    resp.to_writer(self.stream).unwrap();
                    self.stream.flush().unwrap();
                    return Ok(());
                }

                // Try to retrieve the public key from storage
                let pubkey = match std::fs::File::open(pubkey_path) {
                    Ok(mut file) => {
                        let mut pubkey_buf = [0; 2048];
                        let mut pubkey = Pubkey::new();
                        file.read(&mut pubkey_buf).unwrap();
                        pubkey.read_from_bytes(&pubkey_buf);
                        pubkey
                    }
                    Err(e) => {
                        println!("Error reading pubkey: {:?}", e);
                        resp.status = remote_unlock_lib::net::status::Status::InternalServerError;
                        resp.to_writer(self.stream).unwrap();
                        self.stream.flush().unwrap();
                        return Ok(());
                    }
                };

                let valid_request = unlock_req.verify(signature_header.value.as_bytes(), &pubkey);

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
