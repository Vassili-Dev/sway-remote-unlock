use std::str::Utf8Error;

#[derive(Debug)]
pub enum RemoteUnlockError {
    SocketError(std::io::Error),
    PubkeyNotFoundError,
    CodeBufferFullError,
    OversizePacketError,
    IncompleteRequestError,
    ServerError(String),
    HTTParseError(httparse::Error),
    KeyExists(String),
    KeyParseError(der::Error),
}

impl std::fmt::Display for RemoteUnlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RemoteUnlockError::SocketError(e) => write!(f, "RemoteUnlock -- SocketError: {}", e),
            RemoteUnlockError::CodeBufferFullError => write!(
                f,
                "RemoteUnlock -- CodeBufferFullError: Code buffer is full"
            ),
            RemoteUnlockError::OversizePacketError => write!(
                f,
                "RemoteUnlock -- OversizePacketError: Packet is too large"
            ),
            RemoteUnlockError::IncompleteRequestError => write!(
                f,
                "RemoteUnlock -- IncompleteRequestError: Attempted to parse incomplete request"
            ),
            RemoteUnlockError::ServerError(e) => write!(f, "RemoteUnlock -- ServerError: {}", e),
            RemoteUnlockError::HTTParseError(e) => {
                write!(f, "RemoteUnlock -- HTTParseError: {}", e)
            }
            RemoteUnlockError::PubkeyNotFoundError => {
                write!(f, "RemoteUnlock -- PubketNotFoundError")
            }
            RemoteUnlockError::KeyExists(msg) => {
                write!(f, "RemoteUnlock -- KeyExists: {}", msg)
            }
            RemoteUnlockError::KeyParseError(e) => {
                write!(f, "RemoteUnlock -- KeyParseError: {}", e)
            }
        }
    }
}

impl std::error::Error for RemoteUnlockError {}

impl From<std::io::Error> for RemoteUnlockError {
    fn from(err: std::io::Error) -> RemoteUnlockError {
        RemoteUnlockError::SocketError(err)
    }
}

impl From<httparse::Error> for RemoteUnlockError {
    fn from(err: httparse::Error) -> RemoteUnlockError {
        RemoteUnlockError::HTTParseError(err)
    }
}

impl From<der::Error> for RemoteUnlockError {
    fn from(err: der::Error) -> RemoteUnlockError {
        RemoteUnlockError::KeyParseError(err)
    }
}

impl From<Utf8Error> for RemoteUnlockError {
    fn from(_: Utf8Error) -> RemoteUnlockError {
        RemoteUnlockError::ServerError("Invalid UTF-8".to_string())
    }
}
