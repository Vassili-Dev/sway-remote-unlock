use std::io::Write;
use std::sync::mpsc::Receiver;

use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::prelude::*;

use crate::backends::swaylock::SwaylockBackend;
use crate::logging;
use crate::state::State;

pub struct ServerContext<'a, T: Write> {
    state: State,
    code_receiver: Receiver<EnrollmentCode>,
    config: &'a Config,
    stream: Option<T>,
    backend: Option<SwaylockBackend>,
}

impl<'a, T: Write> ServerContext<'a, T> {
    pub fn builder() -> ServerContextBuilder<'a, T> {
        ServerContextBuilder {
            state: None,
            code_receiver: None,
            config: None,
            stream: None,
        }
    }

    pub fn state(&mut self) -> &mut State {
        &mut self.state
    }

    #[allow(dead_code)]
    pub fn code_receiver(&self) -> &Receiver<EnrollmentCode> {
        &self.code_receiver
    }

    pub fn config(&self) -> &Config {
        self.config
    }

    pub fn stream(&mut self) -> Result<&mut T, Error> {
        self.stream
            .as_mut()
            .ok_or(Error::new(ErrorKind::Server, Some("Unset stream")))
    }

    pub fn replace_stream(&mut self, stream: T) {
        self.stream.replace(stream);
    }

    pub fn remove_stream(&mut self) {
        self.stream = None;
    }

    pub fn create_storage_dirs(&mut self) -> Result<(), Error> {
        let keys_dir = self.config.keys_dir();
        debug!(
            "Creating keys directory: {}",
            keys_dir.to_str().unwrap_or("Malformed path")
        );

        std::fs::create_dir_all(keys_dir)?;

        let nonce_dir = self.config.nonce_dir();
        debug!(
            "Creating nonce directory: {}",
            nonce_dir.to_str().unwrap_or("Malformed path")
        );
        std::fs::create_dir_all(nonce_dir)?;

        Ok(())
    }

    fn register_backend(&mut self) -> Result<(), Error> {
        let swaylock_backend = SwaylockBackend::try_new()?;
        self.backend.replace(swaylock_backend);
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<(), Error> {
        self.backend
            .as_mut()
            .ok_or(Error::new(
                ErrorKind::Server,
                Some("Backend not initialized"),
            ))?
            .unlock()?;

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), Error> {
        logging::Logger::init(self.config)?;
        self.create_storage_dirs()?;
        self.register_backend()?;
        Ok(())
    }

    pub fn process_codes(&mut self) -> Result<(), Error> {
        trace!("Processing codes from buffer");

        let code_buffer = self.state.code_buffer();
        let code_receiver = &self.code_receiver;

        // Clear expired codes from the buffer and shift the rest down
        code_buffer.clear_expired();

        // Drain the code channel into the buffer
        'buffer_drain: while let Ok(code) = code_receiver.try_recv() {
            match code_buffer.insert(code) {
                Ok(_) => {
                    debug!("Inserted code into buffer: {}", code);
                }
                Err(_) => {
                    warn!("Code buffer full, ignoring code {:?}", code);
                    break 'buffer_drain;
                }
            }
        }

        Ok(())
    }
}

pub struct ServerContextBuilder<'a, T: Write> {
    state: Option<State>,
    code_receiver: Option<Receiver<EnrollmentCode>>,
    config: Option<&'a Config>,
    stream: Option<T>,
}

impl<'a, T: Write> ServerContextBuilder<'a, T> {
    pub fn state(mut self, state: State) -> Self {
        self.state = Some(state);
        self
    }

    pub fn code_receiver(mut self, code_receiver: Receiver<EnrollmentCode>) -> Self {
        self.code_receiver = Some(code_receiver);
        self
    }

    pub fn config(mut self, config: &'a Config) -> Self {
        self.config = Some(config);
        self
    }

    #[allow(dead_code)]
    pub fn stream(mut self, stream: T) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn build(self) -> Result<ServerContext<'a, T>, Error> {
        Ok(ServerContext {
            state: self
                .state
                .ok_or(Error::new(ErrorKind::Server, Some("State not set")))?,
            code_receiver: self
                .code_receiver
                .ok_or(Error::new(ErrorKind::Server, Some("Receiver not set")))?,
            config: self
                .config
                .ok_or(Error::new(ErrorKind::Server, Some("Config not set")))?,
            backend: None,
            stream: self.stream,
        })
    }
}
