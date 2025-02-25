use std::{collections::HashMap, thread};

use crate::code_buffer::CodeBuffer;
use remote_unlock_lib::prelude::*;

pub struct State {
    // Map of strictly increasing nonces for each client
    nonces: HashMap<uuid::Uuid, u128>,
    code_buffer: CodeBuffer,
    pending_nonce_updates: HashMap<uuid::Uuid, u128>,
}

impl State {
    pub fn new() -> State {
        State {
            nonces: HashMap::new(),
            code_buffer: CodeBuffer::new(),
            pending_nonce_updates: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get_nonce(&self, id: &uuid::Uuid) -> Option<&u128> {
        self.nonces.get(id)
    }

    #[allow(dead_code)]
    pub fn increment_nonce(&mut self, id: uuid::Uuid) {
        let nonce = self.nonces.entry(id).or_insert(0);
        *nonce += 1;
    }

    fn save_nonce_to_file(id: uuid::Uuid, nonce: u128) -> Result<(), Error> {
        let mut id_buf: [u8; 32] = [0; 32];
        let path = Config::new()
            .nonce_dir()
            .join(id.as_simple().encode_lower(&mut id_buf));
        debug!("Writing nonce to file: {:?}", &path);

        let mut file = match std::fs::File::create(&path) {
            Ok(file) => file,
            Err(e) => {
                error!("Error creating nonce file: {}", e);
                return Err(e.into());
            }
        };

        let nonce_str = nonce.to_string();
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::try_from(nonce_str.as_bytes())?;

        std::io::copy(&mut bytes, &mut file)?;

        Ok(())
    }

    fn save_nonce_to_file_async(id: uuid::Uuid, nonce: u128) {
        thread::spawn(move || Self::save_nonce_to_file(id, nonce));
    }

    pub fn update_nonce(&mut self, id: uuid::Uuid, nonce: u128) {
        Self::save_nonce_to_file_async(id, nonce);

        self.nonces.insert(id, nonce);
    }

    pub fn try_load_nonce_from_file(
        &mut self,
        config: &Config,
        id: &uuid::Uuid,
    ) -> Result<u128, Error> {
        let mut id_buf: [u8; 32] = [0; 32];
        let path = config
            .nonce_dir()
            .join(id.as_simple().encode_lower(&mut id_buf));
        debug!("Loading nonce from file: {:?}", &path);

        let mut file = std::fs::File::open(&path);
        let mut bytes = ByteArray::<{ Config::BUFFER_SIZE }>::new();

        match file {
            Ok(ref mut file) => {
                std::io::copy(file, &mut bytes)?;
                let nonce_str = bytes.as_str().unwrap_or("0");
                debug!("Loaded nonce: {}", nonce_str);

                let nonce = nonce_str.parse::<u128>().unwrap_or(0);
                Ok(nonce)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                debug!("Nonce file not found: {:?}", &path);
                Err(e.into())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn validate_nonce(&mut self, id: &uuid::Uuid, nonce: u128) -> bool {
        trace!("Checking nonce for id: {}", &id);
        let current_nonce = match self.nonces.get(id) {
            Some(last_nonce) => last_nonce.to_owned(),
            None => {
                debug!("No nonce found for id: {}, fetching from file", &id);
                let loaded_nonce = self.try_load_nonce_from_file(&Config::new(), id);

                loaded_nonce.unwrap_or(0)
            }
        };

        let result = nonce >= current_nonce;

        if !result {
            warn!("Invalid nonce for id: {}", &id);
        }

        trace!("Synchronizing nonce for id: {}", &id);

        if result {
            return self
                .queue_nonce_update(*id, nonce + 1)
                .map(|_| true)
                .unwrap_or(false);
        }

        result
    }

    pub fn commit_nonce_update(&mut self, id: uuid::Uuid) {
        trace!("Committing nonce update for id: {}", &id);
        if let Some(nonce) = self.pending_nonce_updates.remove(&id) {
            self.update_nonce(id, nonce);
        }
    }

    pub fn rollback_nonce_update(&mut self, id: uuid::Uuid) {
        trace!("Rolling back nonce update for id: {}", &id);
        self.pending_nonce_updates.remove(&id);
    }

    fn queue_nonce_update(&mut self, id: uuid::Uuid, nonce: u128) -> Result<(), Error> {
        trace!("Queueing nonce update for id: {}", &id);
        self.pending_nonce_updates.insert(id, nonce);

        Ok(())
    }

    pub fn code_buffer(&mut self) -> &mut CodeBuffer {
        &mut self.code_buffer
    }
}
