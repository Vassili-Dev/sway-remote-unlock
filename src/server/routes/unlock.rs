use remote_unlock_lib::crypto::key::PublicKey;
use remote_unlock_lib::net::method::Method;
use remote_unlock_lib::net::status::Status;
use remote_unlock_lib::{
    net::{request::Request, response::Response},
    prelude::*,
    unlock_request::UnlockRequestBody,
};
use std::io::Write;
use std::net::TcpStream;

use base64::prelude::*;

use crate::context::ServerContext;

use super::route::Route;

pub struct UnlockRoute<'a, 'c: 'a, T: Write = TcpStream> {
    context: &'a mut ServerContext<'c, T>,
    id: Option<uuid::Uuid>,
}

impl<'a, 'c: 'a, T: Write> Route<'a, 'c, T> for UnlockRoute<'a, 'c, T> {
    const PATH: &'static str = "/unlock";
    const METHOD: Method = Method::POST;

    fn new(context: &'a mut ServerContext<'c, T>) -> Self {
        Self { context, id: None }
    }

    fn context(&mut self) -> &mut ServerContext<'c, T> {
        self.context
    }

    fn post_run(&mut self, response: &Response) -> Result<(), Error> {
        let id = self.id.ok_or(Error::new(
            ErrorKind::Server,
            Some("Missing id in unlock route"),
        ))?;
        if response.status() == Status::Ok {
            self.context.state().commit_nonce_update(id);
            if let Some(backend) = self.context.backend() {
                backend.unlock()?;
            }
        } else {
            self.context.state().rollback_nonce_update(id);
        }

        Ok(())
    }
    fn run(&mut self, req: &Request) -> Result<Response, Error> {
        trace!("Parsing unlock request");
        // Parse the body of the request
        let body_str = std::str::from_utf8(&req.body[..req.body_len])?;
        let unlock_req = serde_json::from_str::<UnlockRequestBody>(body_str);
        debug!("Unlock request: {:?}", &unlock_req);
        let builder = Response::builder();

        match unlock_req {
            Ok(unlock_req) => {
                // Get the signature header
                let signature_header = req.get_header("X-RemoteUnlock-Signature");

                let signature_header = match signature_header {
                    Some(s) => s,
                    None => {
                        warn!("Unsigned unlock request received");
                        let resp = builder.status(Status::BadRequest).build();
                        return Ok(resp);
                    }
                };

                let mut pubkey_path = self
                    .context
                    .config()
                    .keys_dir()
                    .join(std::str::from_utf8(unlock_req.id())?);

                pubkey_path.set_extension("pub");

                debug!("Opening public key file: {:?}", &pubkey_path);
                if !pubkey_path.exists() {
                    warn!("Public key not found for user: {:?}", &unlock_req.id());
                    let resp = builder.status(Status::NotFound).build();
                    return Ok(resp);
                }

                // Try to retrieve the public key from storage
                let pubkey = match PublicKey::read_pem_file(pubkey_path.as_path()) {
                    Ok(pubkey) => pubkey,
                    Err(_) => {
                        error!("Error parsing public key file");
                        return Ok(builder.status(Status::InternalServerError).build());
                    }
                };

                debug!("Public key loaded: {:?}", &pubkey.inner());

                trace!("Decoding signature from Base64 Header");
                let mut signature_bytes = [0u8; 1024];
                let signature_length = BASE64_STANDARD
                    .decode_slice(signature_header.value.as_bytes(), &mut signature_bytes)?;
                debug!(
                    "Signature received: {:?}",
                    &signature_bytes[..signature_length]
                );

                let signature_valid =
                    unlock_req.verify(&signature_bytes[..signature_length], &pubkey)?;

                let id = uuid::Uuid::try_parse_ascii(unlock_req.id())?;
                self.id = Some(id);
                let valid_request =
                    signature_valid && self.context.state().validate_nonce(&id, unlock_req.nonce());

                if valid_request {
                    Ok(builder.status(Status::Ok).build())
                } else {
                    warn!("Unlock request authorization failed");
                    Ok(builder.status(Status::Forbidden).build())
                }
            }
            Err(e) => {
                error!("Error parsing enrollment request: {}", e);
                Ok(builder.status(Status::BadRequest).build())
            }
        }
    }
}
