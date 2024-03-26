use std::io::Write;
use std::sync::mpsc::Receiver;

use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::prelude::*;

use crate::state::State;

pub struct ServerContext<'a, T: Write> {
    state: State,
    code_receiver: Receiver<EnrollmentCode>,
    config: &'a Config,
    stream: Option<T>,
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

    pub fn code_receiver(&self) -> &Receiver<EnrollmentCode> {
        &self.code_receiver
    }

    pub fn config(&self) -> &Config {
        &self.config
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
            stream: self.stream,
        })
    }
}
