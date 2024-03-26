use remote_unlock_lib::prelude::*;
use systemd_journal_logger::{connected_to_journal, JournalLog};
pub struct Logger {}

impl Logger {
    pub fn init(config: &Config) -> Result<(), Error> {
        let error_mapper = || Error::new(ErrorKind::Server, Some("Failed to initialize logger"));
        if connected_to_journal() {
            JournalLog::new()
                .map_err(|_| error_mapper())?
                .install()
                .map_err(|_| error_mapper())?;
        } else {
            simple_logger::SimpleLogger::new()
                .env()
                .init()
                .map_err(|_| error_mapper())?;
        }

        log::set_max_level(config.log_level());

        debug!("Logger initialized");

        Ok(())
    }
}
