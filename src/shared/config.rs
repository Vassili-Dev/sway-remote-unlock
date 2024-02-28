const DEFAULT_SOCKET_PATH: &str = "/tmp/remote_unlock.sock";
const DEFAULT_STORAGE_DIR: &str = "./var/lib/remote_unlock";
const ENV_SOCKET_PATH: &str = "REMOTE_UNLOCK_SOCKET_PATH";
const ENV_STORAGE_DIR: &str = "REMOTE_UNLOCK_STORAGE_DIR";
pub struct Config {
    socket_path: Option<String>,
    storage_dir: Option<String>,
}

impl Config {
    pub const MAX_PACKET_SIZE: usize = 1024 * 4;

    pub fn new() -> Config {
        let socket_path = std::env::var(ENV_SOCKET_PATH).ok();

        let storage_dir = std::env::var(ENV_STORAGE_DIR).ok();

        Config {
            socket_path,
            storage_dir,
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
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
