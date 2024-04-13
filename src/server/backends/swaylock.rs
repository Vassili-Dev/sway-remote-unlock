use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet,
};
use remote_unlock_lib::prelude::*;

pub struct SwaylockBackend {
    waker: VirtualDevice,
}

impl SwaylockBackend {
    pub fn try_new() -> Result<Self, Error> {
        let mut keys = AttributeSet::new();
        keys.insert(evdev::Key::KEY_WAKEUP);

        let waker = VirtualDeviceBuilder::new()?
            .name("RemoteUnlock--Waker")
            .with_keys(&keys)?
            .build()?;

        Ok(Self { waker })
    }

    pub fn unlock(&mut self) -> Result<(), Error> {
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

    fn wake_screen(&mut self) -> Result<(), Error> {
        trace!("Waking screen");

        trace!("Sending lid open key");

        let wake_down = evdev::InputEvent::new(evdev::EventType::KEY, evdev::Key::KEY_WAKEUP.0, 1);
        let wake_up = evdev::InputEvent::new(evdev::EventType::KEY, evdev::Key::KEY_WAKEUP.0, 0);

        match self.waker.emit(&[wake_down, wake_up]) {
            Ok(_) => (),
            Err(e) => {
                debug!("Wakeup result: {}", e);
                return Ok(());
            }
        };

        Ok(())
    }
}
