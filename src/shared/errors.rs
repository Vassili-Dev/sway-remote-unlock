use std::{error::Error, fmt::Display, io};

#[derive(Debug)]
pub struct SocketError {
    msg: String,
}

impl std::fmt::Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RemoteUnlock -- SocketError: {}", self.msg)
    }
}

impl Error for SocketError {}

impl From<std::io::Error> for SocketError {
    fn from(err: std::io::Error) -> SocketError {
        SocketError {
            msg: err.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CodeBufferFullError;

impl std::fmt::Display for CodeBufferFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "RemoteUnlock -- CodeBufferFullError: Code buffer is full"
        )
    }
}

impl Error for CodeBufferFullError {}

#[derive(Debug)]
pub struct OversizePacketError;

impl std::fmt::Display for OversizePacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "RemoteUnlock -- OversizePacketError: Packet is too large"
        )
    }
}

#[derive(Debug)]
pub struct ServerError {
    msg: String,
}

impl ServerError {
    pub fn new(msg: String) -> ServerError {
        ServerError { msg }
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RemoteUnlock -- ServerError: {}", self.msg)
    }
}
// #[derive(Debug)]
// pub struct ServerError {
//     msg: String,
// }

// impl std::fmt::Display for ServerError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "RemoteUnlock -- ServerError: {}", self.msg)
//     }
// }

// impl Error for ServerError {}

// impl From<std::io::Error> for ServerError {
//     fn from(err: std::io::Error) -> ServerError {
//         ServerError {
//             msg: err.to_string(),
//         }
//     }
// }

#[derive(Debug)]
pub enum RemoteUnlockError {
    SocketError(SocketError),
    CodeBufferFullError(CodeBufferFullError),
    OversizePacketError(OversizePacketError),
    ServerError(ServerError),
}

impl Display for RemoteUnlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RemoteUnlockError::SocketError(e) => write!(f, "RemoteUnlock -- {}", e),
            RemoteUnlockError::CodeBufferFullError(e) => write!(f, "RemoteUnlock -- {}", e),
            RemoteUnlockError::OversizePacketError(e) => write!(f, "RemoteUnlock -- {}", e),
            RemoteUnlockError::ServerError(e) => write!(f, "RemoteUnlock -- {}", e),
        }
    }
}

impl From<SocketError> for RemoteUnlockError {
    fn from(err: SocketError) -> RemoteUnlockError {
        RemoteUnlockError::SocketError(err)
    }
}

impl From<CodeBufferFullError> for RemoteUnlockError {
    fn from(err: CodeBufferFullError) -> RemoteUnlockError {
        RemoteUnlockError::CodeBufferFullError(err)
    }
}

impl From<OversizePacketError> for RemoteUnlockError {
    fn from(err: OversizePacketError) -> RemoteUnlockError {
        RemoteUnlockError::OversizePacketError(err)
    }
}

impl From<ServerError> for RemoteUnlockError {
    fn from(err: ServerError) -> RemoteUnlockError {
        RemoteUnlockError::ServerError(err)
    }
}

impl From<io::Error> for RemoteUnlockError {
    fn from(err: io::Error) -> RemoteUnlockError {
        RemoteUnlockError::SocketError(SocketError {
            msg: err.to_string(),
        })
    }
}

impl Error for RemoteUnlockError {}
