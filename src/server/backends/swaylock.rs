use std::path::PathBuf;

use remote_unlock_lib::prelude::*;

pub struct SwaylockBackend {
    sway_socket_path: PathBuf,
}

impl SwaylockBackend {
    fn find_sway_socket(config: &Config) -> Result<PathBuf, Error> {
        let sway_socket_path = config.sway_socket_path()?;
        Ok(sway_socket_path)
    }
    pub fn try_new(config: &Config) -> Result<Self, Error> {
        let sway_socket_path = Self::find_sway_socket(config)?;
        Ok(Self { sway_socket_path })
    }

    pub fn unlock(&self) -> Result<(), Error> {
        self.unlock_swaylock()?;
        self.wake_screen()
    }

    fn unlock_swaylock(&self) -> Result<(), Error> {
        trace!("Unlocking swaylock");

        trace!("Sending USR1 signal to swaylock");
        let unlock_result = std::process::Command::new("pkill")
            .arg("-USR1")
            .arg("swaylock")
            .output()?;

        if unlock_result.status.success() {
            trace!("Swaylock unlocked");
            Ok(())
        } else {
            error!("Failed to unlock swaylock: {:?}", unlock_result);
            Err(Error::new(
                ErrorKind::SwaylockBackend,
                Some("Failed to unlock swaylock"),
            ))
        }
    }

    fn wake_screen(&self) -> Result<(), Error> {
        trace!("Waking screen");

        let socket_path = self
            .sway_socket_path
            .as_os_str()
            .to_str()
            .ok_or(Error::new(
                ErrorKind::SwaylockBackend,
                Some("Failed to convert sway socket path to string"),
            ))?;

        debug!("Using sway socket path: {:x?}", socket_path.as_bytes());

        trace!("Using sway socket path: {}", socket_path);
        let wake_result = std::process::Command::new("swaymsg")
            .arg("-s")
            .arg(socket_path)
            .arg("output * dpms on")
            .output()?;

        if wake_result.status.success() {
            trace!("Screen woken");
            Ok(())
        } else {
            error!("Failed to wake screen: {:?}", wake_result);
            Err(Error::new(
                ErrorKind::SwaylockBackend,
                Some("Failed to wake screen"),
            ))
        }
    }
}
