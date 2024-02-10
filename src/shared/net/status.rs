#[derive(Debug, Copy, Clone)]
pub enum Status {
    Ok = 200,
    BadRequest = 400,
    Forbidden = 403,
    NotFound = 404,
    InternalServerError = 500,
}

impl Status {
    pub fn to_string(&self) -> &str {
        match self {
            Status::Ok => "OK",
            Status::BadRequest => "Bad Request",
            Status::Forbidden => "Forbidden",
            Status::NotFound => "Not Found",
            Status::InternalServerError => "Internal Server Error",
        }
    }

    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}
