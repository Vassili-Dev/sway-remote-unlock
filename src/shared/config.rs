use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

const DEFAULT_SOCKET_PATH: &str = "/tmp/remote_unlock.sock";
const DEFAULT_STORAGE_DIR: &str = "./var/lib/remote_unlock";

#[cfg(debug_assertions)]
const DEFAULT_GENERATED_KEYS_DIR: &str = "/tmp/remote_unlock_keys";

const ENV_SOCKET_PATH: &str = "REMOTE_UNLOCK_SOCKET_PATH";
const ENV_STORAGE_DIR: &str = "REMOTE_UNLOCK_STORAGE_DIR";
const ENV_SERVER_HOSTNAME: &str = "REMOTE_UNLOCK_SERVER_HOSTNAME";
const ENV_SERVER_PORT: &str = "REMOTE_UNLOCK_SERVER_PORT";
const ENV_LOG_LEVEL: &str = "REMOTE_UNLOCK_LOG_LEVEL";

#[cfg(debug_assertions)]
const ENV_GENERATED_KEYS_DIR: &str = "REMOTE_UNLOCK_GENERATED_KEYS_DIR";

pub struct Config {
    socket_path: Option<String>,
    storage_dir: Option<String>,
    server_hostname: Option<String>,
    server_port: Option<u16>,
    log_level: Option<log::LevelFilter>,
    #[cfg(debug_assertions)]
    generated_keys_dir: Option<String>,
}

impl Config {
    pub const MAX_PACKET_SIZE: usize = 1024 * 4;
    pub const BUFFER_SIZE: usize = 1024;
    pub const ERROR_STRING_SIZE: usize = 64;
    pub const STREAM_RETRY_DELAY_MS: u64 = 100;

    pub fn new() -> Config {
        let socket_path = std::env::var(ENV_SOCKET_PATH).ok();

        let storage_dir = std::env::var(ENV_STORAGE_DIR).ok();

        let server_hostname = std::env::var(ENV_SERVER_HOSTNAME).ok();
        let server_port = std::env::var(ENV_SERVER_PORT)
            .ok()
            .map(|port| port.parse::<u16>().unwrap());

        let log_level = std::env::var(ENV_LOG_LEVEL).ok().map(|level| {
            log::LevelFilter::from_str(level.as_str()).unwrap_or(log::LevelFilter::Info)
        });

        #[cfg(debug_assertions)]
        let generated_keys_dir = std::env::var(ENV_GENERATED_KEYS_DIR).ok();

        Config {
            socket_path,
            storage_dir,
            server_hostname,
            server_port,
            log_level,
            #[cfg(debug_assertions)]
            generated_keys_dir,
        }
    }

    pub fn socket_path(&self) -> &str {
        match &self.socket_path {
            Some(path) => path,
            None => DEFAULT_SOCKET_PATH,
        }
    }

    fn storage_dir(&self) -> &str {
        match &self.storage_dir {
            Some(path) => path,
            None => DEFAULT_STORAGE_DIR,
        }
    }

    pub fn keys_dir(&self) -> PathBuf {
        Path::new(self.storage_dir()).join("keys")
    }

    pub fn nonce_dir(&self) -> PathBuf {
        Path::new(self.storage_dir()).join("nonces")
    }

    #[cfg(debug_assertions)]
    pub fn generated_keys_dir(&self) -> &str {
        match &self.generated_keys_dir {
            Some(path) => path,
            None => DEFAULT_GENERATED_KEYS_DIR,
        }
    }

    pub fn server_hostname(&self) -> &str {
        match &self.server_hostname {
            Some(hostname) => hostname,
            None => "127.0.0.1",
        }
    }

    pub fn server_port(&self) -> u16 {
        match &self.server_port {
            Some(port) => *port,
            None => 8142,
        }
    }

    pub fn log_level(&self) -> log::LevelFilter {
        match &self.log_level {
            Some(level) => *level,
            None => log::LevelFilter::Info,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
