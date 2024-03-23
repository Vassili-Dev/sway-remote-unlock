use crate::prelude::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnrollmentRequest {
    code: u32,
    pubkey_pem: ByteArray<{ Config::BUFFER_SIZE }>,
}

impl EnrollmentRequest {
    pub fn new(code: u32, pubkey_pem: ByteArray<{ Config::BUFFER_SIZE }>) -> EnrollmentRequest {
        EnrollmentRequest { code, pubkey_pem }
    }

    pub fn code(&self) -> &u32 {
        &self.code
    }

    pub fn pubkey_pem(&self) -> &ByteArray<{ Config::BUFFER_SIZE }> {
        &self.pubkey_pem
    }
}
