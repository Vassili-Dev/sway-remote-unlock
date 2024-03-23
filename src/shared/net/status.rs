use crate::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

    pub fn from_u16(code: u16) -> Result<Status, Error> {
        match code {
            200 => Ok(Status::Ok),
            400 => Ok(Status::BadRequest),
            403 => Ok(Status::Forbidden),
            404 => Ok(Status::NotFound),
            500 => Ok(Status::InternalServerError),
            _ => {
                let code_str = ByteArray::<5>::from(ByteArrayString::try_from(code)?);
                Err(Error::new(
                    ErrorKind::UnkownStatus,
                    Some(code_str.as_str()?),
                ))
            }
        }
    }
}
