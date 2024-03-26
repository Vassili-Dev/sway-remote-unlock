use core::fmt;
use log::error;
use std::fmt::{Debug, Display};

use crate::config::Config;

use super::ByteArrayString;

pub trait ErrorKindMarker: Display + Debug + Sized {}

pub struct OwnError<K: ErrorKindMarker> {
    pub kind: K,
    pub message: Option<ByteArrayString<{ Config::ERROR_STRING_SIZE }>>,
}

impl<K: ErrorKindMarker> OwnError<K> {
    pub fn new(kind: K, message: Option<&str>) -> Self {
        let message = match message {
            Some(msg) => match ByteArrayString::try_from(msg) {
                Ok(m) => Some(m),
                Err(e) => {
                    error!(
                        "Warning: Failed to convert error message to ByteArrayString: {}",
                        e
                    );
                    None
                }
            },
            None => None,
        };

        OwnError { kind, message }
    }
}

impl<K: ErrorKindMarker> From<K> for OwnError<K> {
    fn from(kind: K) -> Self {
        OwnError::new(kind, None)
    }
}

impl<K: ErrorKindMarker> Display for OwnError<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.message {
            Some(msg) => write!(f, "{}: {}", self.kind, msg),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl<K: ErrorKindMarker> Debug for OwnError<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.message {
            Some(msg) => write!(f, "{}: {}", self.kind, msg),
            None => write!(f, "{}", self.kind),
        }
    }
}
