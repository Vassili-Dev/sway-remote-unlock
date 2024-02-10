use crate::pubkey::Pubkey;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnrollmentRequest {
    code: u32,
    pubkey: Pubkey,
}

impl EnrollmentRequest {
    pub fn new(code: u32, pubkey: Pubkey) -> EnrollmentRequest {
        EnrollmentRequest { code, pubkey }
    }

    pub fn code(&self) -> &u32 {
        &self.code
    }

    pub fn pubkey(&self) -> &Pubkey {
        &self.pubkey
    }
}
