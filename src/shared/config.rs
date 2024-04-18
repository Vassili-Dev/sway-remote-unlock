use crate::types::{Error, ErrorKind};
use evdev::Device;
use log::warn;
use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    str::FromStr,
};

const DEFAULT_SOCKET_PATH: &str = "/tmp/remote_unlock.sock";
const DEFAULT_STORAGE_DIR: &str = "./var/lib/remote_unlock";

#[cfg(debug_assertions)]
const DEFAULT_GENERATED_KEYS_DIR: &str = "/tmp/remote_unlock_keys";

const ENV_SOCKET_PATH: &str = "REMOTE_UNLOCK_SOCKET_PATH";
const ENV_STORAGE_DIR: &str = "REMOTE_UNLOCK_STORAGE_DIR";
const ENV_SERVER_IP: &str = "REMOTE_UNLOCK_SERVER_IP";
const ENV_SERVER_PORT: &str = "REMOTE_UNLOCK_SERVER_PORT";
const ENV_LOG_LEVEL: &str = "REMOTE_UNLOCK_LOG_LEVEL";
const ENV_MDNS_SERVICE_TYPE: &str = "REMOTE_UNLOCK_MDNS_SERVICE_TYPE";

// Backend Specific Config
const ENV_SWAY_SOCKET_PATH: &str = "SWAYSOCK";

#[cfg(debug_assertions)]
const ENV_GENERATED_KEYS_DIR: &str = "REMOTE_UNLOCK_GENERATED_KEYS_DIR";

pub struct Config {
    socket_path: Option<String>,
    storage_dir: Option<String>,
    server_ip: Option<String>,
    server_port: Option<u16>,
    wake_device_path: Option<PathBuf>,
    log_level: Option<log::LevelFilter>,
    sway_socket_path: Option<String>,
    service_type: Option<String>,

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

        let server_ip = std::env::var(ENV_SERVER_IP).ok();
        let server_port = std::env::var(ENV_SERVER_PORT)
            .ok()
            .map(|port| port.parse::<u16>().unwrap());

        let log_level = std::env::var(ENV_LOG_LEVEL).ok().map(|level| {
            log::LevelFilter::from_str(level.as_str()).unwrap_or(log::LevelFilter::Info)
        });

        let wake_device_path = Self::try_detect_wake_device_path();

        let service_type = std::env::var(ENV_MDNS_SERVICE_TYPE).ok();

        let sway_socket_path = std::env::var(ENV_SWAY_SOCKET_PATH).ok();

        #[cfg(debug_assertions)]
        let generated_keys_dir = std::env::var(ENV_GENERATED_KEYS_DIR).ok();

        Config {
            socket_path,
            storage_dir,
            server_ip,
            server_port,
            log_level,
            sway_socket_path,
            wake_device_path,
            service_type,
            #[cfg(debug_assertions)]
            generated_keys_dir,
        }
    }

    fn try_detect_wake_device_path() -> Option<PathBuf> {
        let mut devices = evdev::enumerate();
        let lid_device = devices.find(|(_, device)| {
            device
                .supported_keys()
                .map_or(false, |keys| keys.contains(evdev::Key::KEY_WAKEUP))
        });

        match lid_device {
            Some((pb, _)) => Some(pb),
            None => {
                warn!("Failed to detect lid device");
                None
            }
        }
    }
    fn try_detect_sway_socket_path() -> Option<PathBuf> {
        let uid = match std::fs::metadata("/proc/self").map(|m| m.uid()) {
            Ok(uid) => uid,
            Err(_) => return None,
        };
        std::fs::read_dir("/run/user")
            .map(|mut entries| {
                let first_sock = entries.find(|entry| {
                    entry
                        .as_ref()
                        .map(|entry| {
                            entry
                                .file_name()
                                .to_str()
                                .map(|name| name.starts_with(format!("sway-ipc.{}.", uid).as_str()))
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
                });

                match first_sock {
                    Some(entry) => entry.ok().map(|entry| entry.path()),
                    None => None,
                }
            })
            .unwrap_or(None)
    }

    pub fn sway_socket_path(&self) -> Result<PathBuf, Error> {
        match self.sway_socket_path {
            Some(ref path) => Ok(path.into()),
            None => {
                warn!("SWAYSOCK environment variable not set, attempting to construct path");
                match Self::try_detect_sway_socket_path() {
                    Some(path) => Ok(path),
                    None => {
                        warn!("Failed to detect sway socket path");
                        Err(Error::new(
                            ErrorKind::SwaylockBackend,
                            Some("Failed to detect sway socket path"),
                        ))
                    }
                }
            }
        }
    }

    pub fn wake_device(&self) -> Option<Device> {
        match &self.wake_device_path {
            Some(path) => Device::open(path).ok(),
            None => None,
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

    pub fn service_type(&self) -> &str {
        match &self.service_type {
            Some(service_type) => service_type,
            None => "_remote-unlock._tcp.local.",
        }
    }

    pub fn server_hostname(&self) -> &str {
        include_str!("/etc/hostname").trim()
    }

    #[cfg(debug_assertions)]
    pub fn generated_keys_dir(&self) -> &str {
        match &self.generated_keys_dir {
            Some(path) => path,
            None => DEFAULT_GENERATED_KEYS_DIR,
        }
    }

    pub fn server_ip(&self) -> &str {
        match &self.server_ip {
            Some(server_ip) => server_ip,
            None => "0.0.0.0",
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
