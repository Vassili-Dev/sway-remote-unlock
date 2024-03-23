const DEFAULT_SOCKET_PATH: &str = "/tmp/remote_unlock.sock";
const DEFAULT_STORAGE_DIR: &str = "./var/lib/remote_unlock";

#[cfg(debug_assertions)]
const DEFAULT_GENERATED_KEYS_DIR: &str = "/tmp/remote_unlock_keys";

const ENV_SOCKET_PATH: &str = "REMOTE_UNLOCK_SOCKET_PATH";
const ENV_STORAGE_DIR: &str = "REMOTE_UNLOCK_STORAGE_DIR";
const ENV_SERVER_HOSTNAME: &str = "REMOTE_UNLOCK_SERVER_HOSTNAME";
const ENV_SERVER_PORT: &str = "REMOTE_UNLOCK_SERVER_PORT";

#[cfg(debug_assertions)]
const ENV_GENERATED_KEYS_DIR: &str = "REMOTE_UNLOCK_GENERATED_KEYS_DIR";

pub struct Config {
    socket_path: Option<String>,
    storage_dir: Option<String>,
    server_hostname: Option<String>,
    server_port: Option<u16>,
    #[cfg(debug_assertions)]
    generated_keys_dir: Option<String>,
}

impl Config {
    pub const MAX_PACKET_SIZE: usize = 1024 * 4;
    pub const BUFFER_SIZE: usize = 1024;
    pub const ERROR_STRING_SIZE: usize = 64 * 2;

    pub fn new() -> Config {
        let socket_path = std::env::var(ENV_SOCKET_PATH).ok();

        let storage_dir = std::env::var(ENV_STORAGE_DIR).ok();

        let server_hostname = std::env::var(ENV_SERVER_HOSTNAME).ok();
        let server_port = std::env::var(ENV_SERVER_PORT)
            .ok()
            .map(|port| port.parse::<u16>().unwrap());

        #[cfg(debug_assertions)]
        let generated_keys_dir = std::env::var(ENV_GENERATED_KEYS_DIR).ok();

        Config {
            socket_path,
            storage_dir,
            server_hostname,
            server_port,
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

    pub fn storage_dir(&self) -> &str {
        match &self.storage_dir {
            Some(path) => path,
            None => DEFAULT_STORAGE_DIR,
        }
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
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
