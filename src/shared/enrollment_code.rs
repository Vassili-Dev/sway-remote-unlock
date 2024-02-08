use chrono::{Duration, Utc};
use rand::prelude::*;

// 30 minutes
const CODE_LIFETIME: Duration = Duration::milliseconds(30 * 1000);

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct EnrollmentCode {
    code: u32,
    expires: i64,
}

impl EnrollmentCode {
    pub fn new() -> EnrollmentCode {
        let mut rng = rand::thread_rng();
        let code = rng.gen_range(100_000..1_000_000);
        let expires = (Utc::now() + CODE_LIFETIME).timestamp();

        EnrollmentCode { code, expires }
    }

    pub fn expired(&self) -> bool {
        Utc::now().timestamp() > self.expires
    }

    pub fn verify(&self, code: u32) -> bool {
        !self.expired() && self.code == code
    }
}
