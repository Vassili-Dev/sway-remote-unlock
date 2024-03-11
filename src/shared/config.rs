const DEFAULT_SOCKET_PATH: &str = "/tmp/remote_unlock.sock";
const DEFAULT_STORAGE_DIR: &str = "./var/lib/remote_unlock";

#[cfg(debug_assertions)]
const DEFAULT_GENERATED_KEYS_DIR: &str = "/tmp/remote_unlock_keys";

const ENV_SOCKET_PATH: &str = "REMOTE_UNLOCK_SOCKET_PATH";
const ENV_STORAGE_DIR: &str = "REMOTE_UNLOCK_STORAGE_DIR";

#[cfg(debug_assertions)]
const ENV_GENERATED_KEYS_DIR: &str = "REMOTE_UNLOCK_GENERATED_KEYS_DIR";

pub struct Config {
    socket_path: Option<String>,
    storage_dir: Option<String>,
    #[cfg(debug_assertions)]
    generated_keys_dir: Option<String>,
}

impl Config {
    pub const MAX_PACKET_SIZE: usize = 1024 * 4;
    pub const BUFFER_SIZE: usize = 1024;

    pub fn new() -> Config {
        let socket_path = std::env::var(ENV_SOCKET_PATH).ok();

        let storage_dir = std::env::var(ENV_STORAGE_DIR).ok();

        #[cfg(debug_assertions)]
        let generated_keys_dir = std::env::var(ENV_GENERATED_KEYS_DIR).ok();

        Config {
            socket_path,
            storage_dir,
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
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
