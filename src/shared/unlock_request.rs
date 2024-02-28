use std::mem::size_of;

use crate::helper_types::ByteArray;
use dryoc::{classic::crypto_sign_ed25519::Signature, sign::SignedMessage, types::NewByteArray};

// Serial format: {"id":"...","nonce":...}
const SERIAL_LEN: usize = size_of::<u8>() * 21 + size_of::<u128>();

#[derive(Debug, serde::Deserialize)]
pub struct UnlockRequest {
    id: ByteArray<32>,
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
