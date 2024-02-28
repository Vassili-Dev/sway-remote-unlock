use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EnrollmentResponse {
    pub id: uuid::Uuid,
}

impl EnrollmentResponse {
    pub fn new() -> EnrollmentResponse {
        EnrollmentResponse {
            id: uuid::Uuid::new_v4(),
        }
    }

    pub fn id(&self) -> &uuid::Uuid {
        &self.id
    }
}

impl Default for EnrollmentResponse {
    fn default() -> Self {
        Self::new()
    }
}
