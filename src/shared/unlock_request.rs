use crate::prelude::*;

use p256::ecdsa::{self, signature::Verifier, VerifyingKey};
use spki::DecodePublicKey;

// Serial format: {"id":"...","nonce":...}
const SERIAL_LEN: usize = 1024;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UnlockRequestBody<'a> {
    id: &'a str,
    nonce: u128,
}

impl<'a> UnlockRequestBody<'a> {
    pub fn verify(
        &self,
        signature: &[u8],
        pubkey: &crate::crypto::key::PublicKey,
    ) -> Result<bool, Error> {
        trace!("Start request signature verification");
        let mut serial: ByteArray<SERIAL_LEN> = ByteArray::new();

        trace!("Serializing request to sign");
        serde_json::to_writer(&mut serial, self)?;

        debug!(
            "Serialized request for signature verification: {}",
            &serial.as_str()?
        );

        trace!("Decoding public key from der");
        let verifying_key: VerifyingKey =
            p256::PublicKey::from_public_key_der(pubkey.der()?.as_bytes())?.into();

        trace!("Decoded public key");

        trace!("Decoding signature from der");
        let signature = ecdsa::Signature::from_der(signature)?;
        trace!("Decoded signature");

        debug!("Verifying signature");
        match verifying_key.verify(serial.as_bytes(), &signature) {
            Ok(_) => {
                debug!("Signature verified");
                Ok(true)
            }
            Err(e) => {
                warn!("Signature verification failed: {}", e);
                Ok(false)
            }
        }
    }

    pub fn id(&self) -> &[u8] {
        self.id.as_bytes()
    }

    pub fn nonce(&self) -> u128 {
        self.nonce
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::key::PublicKey;

    use super::*;

    use p256::ecdsa::SigningKey;
    use rand::rngs::OsRng;
    use spki::EncodePublicKey;

    #[test]
    fn test_verify() {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let unlock_request = UnlockRequestBody {
            id: "test",
            nonce: 0,
        };

        let mut serial: ByteArray<SERIAL_LEN> = ByteArray::new();

        serde_json::to_writer(&mut serial, &unlock_request).unwrap();

        let (signature, _) = signing_key.sign_recoverable(serial.as_bytes()).unwrap();

        let pubkey =
            PublicKey::from_der(verifying_key.to_public_key_der().unwrap().as_bytes()).unwrap();

        let valid = unlock_request
            .verify(signature.to_der().as_bytes(), &pubkey)
            .unwrap();
        assert!(valid);
    }
}
