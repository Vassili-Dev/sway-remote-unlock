use der::{PemReader, Reader};

use crate::errors::RemoteUnlockError;

use super::der::DerKeyBorrowed;

pub struct PemDataBorrowed<'a> {
    reader: PemReader<'a>,
}

impl<'a> PemDataBorrowed<'a> {
    pub fn new(raw: &'a [u8]) -> Result<PemDataBorrowed<'a>, RemoteUnlockError> {
        let reader = PemReader::new(raw)?;

        Ok(PemDataBorrowed { reader })
    }

    pub fn label(&mut self) -> &'a str {
        self.reader.type_label()
    }

    pub fn key(&mut self) -> Result<DerKeyBorrowed<'a>, RemoteUnlockError> {
        let key = self.reader.decode::<DerKeyBorrowed<'a>>()?;

        Ok(key)
    }
}
