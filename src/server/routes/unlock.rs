use std::io::Write;
use std::net::TcpStream;
use std::path::Path;

use remote_unlock_lib::crypto::key::PublicKey;
use remote_unlock_lib::net::method::Method;
use remote_unlock_lib::net::status::Status;
use remote_unlock_lib::{
    net::{request::Request, response::Response},
    prelude::*,
    unlock_request::UnlockRequestBody,
};

use base64::prelude::*;

use crate::context::ServerContext;

use super::route::Route;

pub struct UnlockRoute<'a, 'c: 'a, T: Write = TcpStream> {
    context: &'a mut ServerContext<'c, T>,
}

impl<'a, 'c: 'a, T: Write> Route<'a, 'c, T> for UnlockRoute<'a, 'c, T> {
    const PATH: &'static str = "/unlock";
    const METHOD: Method = Method::POST;

    fn new(context: &'a mut ServerContext<'c, T>) -> Self {
        Self { context }
    }
    fn run(&mut self, req: &Request) -> Result<Response, Error> {
        // Parse the body of the request
        let body_str = std::str::from_utf8(&req.body[..req.body_len])?;
        let unlock_req = serde_json::from_str::<UnlockRequestBody>(body_str);
        let builder = Response::builder();

        match unlock_req {
            Ok(unlock_req) => {
                // Get the signature header
                let signature_header = req.get_header("X-RemoteUnlock-Signature");

                let signature_header = match signature_header {
                    Some(s) => s,
                    None => {
                        let resp = builder.status(Status::BadRequest).build();
                        return Ok(resp);
                    }
                };

                let mut pubkey_path = Path::new(self.context.config().storage_dir())
                    .join(std::str::from_utf8(unlock_req.id())?);

                pubkey_path.set_extension("pub");

                if !pubkey_path.exists() {
                    let resp = builder.status(Status::NotFound).build();
                    return Ok(resp);
                }

                // Try to retrieve the public key from storage
                let pubkey = match PublicKey::read_pem_file(pubkey_path.as_path()) {
                    Ok(pubkey) => pubkey,
                    Err(_) => {
                        return Ok(builder.status(Status::InternalServerError).build());
                    }
                };

                let mut signature_bytes = [0u8; 1024];
                let signature_length = BASE64_STANDARD
                    .decode_slice(signature_header.value.as_bytes(), &mut signature_bytes)?;

                let signature_valid =
                    unlock_req.verify(&signature_bytes[..signature_length], &pubkey)?;

                let id = uuid::Uuid::try_parse_ascii(unlock_req.id())?;
                let valid_request = signature_valid
                    && self
                        .context
                        .state()
                        .validate_and_update_nonce(&id, unlock_req.nonce());

                if valid_request {
                    Ok(builder.status(Status::Ok).build())
                } else {
                    Ok(builder.status(Status::Forbidden).build())
                }
            }
            Err(e) => {
                println!("Error parsing enrollment request: {:?}", e);
                Ok(builder.status(Status::BadRequest).build())
            }
        }
    }
}
