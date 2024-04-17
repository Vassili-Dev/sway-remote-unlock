use crate::types::{ByteArrayError, OwnError};
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    SocketError(std::io::Error),
    HTTParseError(httparse::Error),
    KeyParseError(der::Error),
    PKCS8Error(pkcs8::Error),
    SPKIError(spki::Error),
    SerdeJSONError(serde_json::Error),
    P256KeyError(p256::elliptic_curve::Error),
    P256SignatureParseError(p256::ecdsa::Error),
    SignatureDecodeError(base64::DecodeSliceError),
    UuidError(uuid::Error),
    ByteArrayError(ByteArrayError),
    MDNSDaemon(mdns_sd::Error),
    OwnError(OwnError<ErrorKind>),
    Utf8Error(std::str::Utf8Error),
}

#[derive(Debug)]
pub enum ErrorKind {
    PubkeyNotFound,
    IncompleteRequest,
    Server,
    UnkownStatus,
    OversizePacket,
    CodeBufferFull,
    KeyExists,
    ContentLengthMismatch,
    NonceQueueFull,
    SwaylockBackend,
}

impl Error {
    pub fn new(kind: ErrorKind, message: Option<&str>) -> Self {
        Self::OwnError(OwnError::new(kind, message))
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorKind::PubkeyNotFound => write!(f, "Public key not found"),
            ErrorKind::IncompleteRequest => write!(f, "Incomplete request"),
            ErrorKind::Server => write!(f, "Server error"),
            ErrorKind::OversizePacket => write!(f, "Oversize packet"),
            ErrorKind::CodeBufferFull => write!(f, "Code buffer full"),
            ErrorKind::KeyExists => write!(f, "Key exists"),
            ErrorKind::UnkownStatus => write!(f, "Unknown status"),
            ErrorKind::ContentLengthMismatch => write!(f, "Content length mismatch"),
            ErrorKind::NonceQueueFull => write!(f, "Nonce queue full"),
            ErrorKind::SwaylockBackend => write!(f, "Swaylock backend error"),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::OwnError(kind.into())
    }
}

impl crate::types::ErrorKindMarker for ErrorKind {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SocketError(e) => write!(f, "SocketError: {}", e),

            Self::HTTParseError(e) => {
                write!(f, "HTTParseError: {}", e)
            }
            Self::KeyParseError(e) => {
                write!(f, "KeyParseError: {}", e)
            }
            Self::PKCS8Error(e) => {
                write!(f, "PKCS8Error: {}", e)
            }
            Self::SPKIError(e) => {
                write!(f, "SPKIError: {}", e)
            }
            Self::P256KeyError(e) => {
                write!(f, "P256Error: {}", e)
            }
            Self::P256SignatureParseError(e) => {
                write!(f, "P256SignatureParseError: {}", e)
            }
            Self::SignatureDecodeError(e) => {
                write!(f, "SignatureDecodeError: {}", e)
            }
            Self::SerdeJSONError(e) => {
                write!(f, "SerdeJSONError: {}", e)
            }
            Self::ByteArrayError(e) => {
                write!(f, "ByteArrayError: {}", e)
            }
            Self::Utf8Error(e) => {
                write!(f, "Utf8Error: {}", e)
            }
            Self::MDNSDaemon(e) => {
                write!(f, "MDNSDaemon: {}", e)
            }
            Self::UuidError(e) => {
                write!(f, "UuidError: {}", e)
            }
            Self::OwnError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::SocketError(err)
    }
}

impl From<httparse::Error> for Error {
    fn from(err: httparse::Error) -> Self {
        Self::HTTParseError(err)
    }
}

impl From<der::Error> for Error {
    fn from(err: der::Error) -> Self {
        Self::KeyParseError(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8Error(e)
    }
}

impl From<der::pem::Error> for Error {
    fn from(err: der::pem::Error) -> Self {
        Self::KeyParseError(err.into())
    }
}

impl From<pkcs8::Error> for Error {
    fn from(err: pkcs8::Error) -> Self {
        Self::PKCS8Error(err)
    }
}

impl From<spki::Error> for Error {
    fn from(err: spki::Error) -> Self {
        Self::SPKIError(err)
    }
}

impl From<p256::elliptic_curve::Error> for Error {
    fn from(err: p256::elliptic_curve::Error) -> Self {
        Self::P256KeyError(err)
    }
}

impl From<p256::ecdsa::Error> for Error {
    fn from(err: p256::ecdsa::Error) -> Self {
        Self::P256SignatureParseError(err)
    }
}

impl From<base64::DecodeSliceError> for Error {
    fn from(err: base64::DecodeSliceError) -> Self {
        Self::SignatureDecodeError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJSONError(err)
    }
}

impl From<ByteArrayError> for Error {
    fn from(err: ByteArrayError) -> Self {
        Self::ByteArrayError(err)
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Self::UuidError(err)
    }
}

impl From<mdns_sd::Error> for Error {
    fn from(err: mdns_sd::Error) -> Self {
        Self::MDNSDaemon(err)
    }
}
