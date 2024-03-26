use std::collections::HashMap;

use crate::code_buffer::CodeBuffer;

pub struct State {
    // Map of strictly increasing nonces for each client
    nonces: HashMap<uuid::Uuid, u128>,
    code_buffer: CodeBuffer,
}

impl State {
    pub fn new() -> State {
        State {
            nonces: HashMap::new(),
            code_buffer: CodeBuffer::new(),
        }
    }

    pub fn get_nonce(&self, id: &uuid::Uuid) -> Option<&u128> {
        self.nonces.get(id)
    }

    pub fn increment_nonce(&mut self, id: uuid::Uuid) {
        let nonce = self.nonces.entry(id).or_insert(0);
        *nonce += 1;
    }

    pub fn validate_and_update_nonce(&mut self, id: &uuid::Uuid, nonce: u128) -> bool {
        let result = match self.nonces.get(id) {
            Some(last_nonce) => nonce > *last_nonce,
            None => true,
        };

        if result {
            self.nonces.insert(*id, nonce);
        }

        result
    }

    pub fn code_buffer(&mut self) -> &mut CodeBuffer {
        &mut self.code_buffer
    }
}
