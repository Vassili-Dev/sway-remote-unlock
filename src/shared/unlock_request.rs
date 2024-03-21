use crate::helper_types::ByteArray;
use dryoc::{classic::crypto_sign_ed25519::Signature, sign::SignedMessage, types::NewByteArray};

// Serial format: {"id":"...","nonce":...}
const SERIAL_LEN: usize = 1024;

#[derive(Debug, serde::Deserialize)]
pub struct UnlockRequest {
    id: ByteArray<16>,
    nonce: u128,
}

impl UnlockRequest {
    pub fn verify(&self, signature: &[u8], pubkey: &crate::pubkey::Pubkey) -> bool {
        let mut serial: ByteArray<SERIAL_LEN> = ByteArray::new();
        serial.append_slice("{\"id\":\"".as_bytes());
        serial.append_slice(self.id.as_bytes());
        serial.append_slice(&self.nonce.to_be_bytes());
        serial.append_slice("\"}".as_bytes());

        let public_key: dryoc::sign::PublicKey = pubkey.into();
        let mut dryoc_signature = Signature::new_byte_array();
        dryoc_signature.copy_from_slice(signature);

        let message = SignedMessage::from_parts(dryoc_signature, serial);

        message.verify(&public_key).map_or(false, |_v| true)
    }

    pub fn id(&self) -> &[u8] {
        self.id.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pubkey::Pubkey;
    use dryoc::{
        classic::crypto_sign::PublicKey,
        sign::{SecretKey, Signature, SigningKeyPair},
        types::ByteArray,
    };

    #[test]
    fn test_verify() {
        let key_pair: SigningKeyPair<[u8; 32], SecretKey> = SigningKeyPair::gen();
        let public_key: PublicKey = key_pair.public_key;
        let mut pubkey = Pubkey::new();
        pubkey.read_from_bytes(&public_key);

        let unlock_request = UnlockRequest {
            id: crate::helper_types::ByteArray::new_from_slice("test".as_bytes()),
            nonce: 0,
        };

        let mut serial: crate::helper_types::ByteArray<SERIAL_LEN> =
            crate::helper_types::ByteArray::new();
        serial.append_slice("{\"id\":\"".as_bytes());
        serial.append_slice(unlock_request.id.as_bytes());
        serial.append_slice(&unlock_request.nonce.to_be_bytes());
        serial.append_slice("\"}".as_bytes());

        let signed: SignedMessage<Signature, crate::helper_types::ByteArray<SERIAL_LEN>> =
            key_pair.sign(serial).unwrap();
        let (signature, _message) = signed.into_parts();
        let signature: &[u8] = signature.as_array();

        let valid = unlock_request.verify(signature, &pubkey);
        assert!(valid);
    }
}
