use remote_unlock_lib::enrollment_code::EnrollmentCode;
use remote_unlock_lib::prelude::*;

pub struct CodeBuffer {
    codes: [Option<EnrollmentCode>; 16],
}

impl CodeBuffer {
    pub fn new() -> CodeBuffer {
        CodeBuffer { codes: [None; 16] }
    }

    pub fn insert(&mut self, code: EnrollmentCode) -> Result<(), Error> {
        if self.codes.iter().all(|c| c.is_some()) {
            return Err(ErrorKind::CodeBufferFull.into());
        }

        for i in 0..self.codes.len() {
            if self.codes[i].is_none() {
                self.codes[i] = Some(code);
                break;
            }
        }

        Ok(())
    }

    pub fn clear_expired(&mut self) {
        for code_opt in self.codes.iter_mut() {
            match code_opt {
                Some(c) => {
                    if c.expired() {
                        *code_opt = None;
                    }
                }
                None => {}
            };
        }
    }

    // Verifies and removes the code from the buffer if it is valid
    pub fn verify(&mut self, code: &u32) -> bool {
        let found = self.codes.iter_mut().find(|code_opt| match code_opt {
            Some(c) => c.verify(code),
            None => false,
        });

        match found {
            Some(c) => {
                *c = None;
                true
            }
            None => false,
        }
    }
}
