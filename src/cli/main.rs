mod args;
mod commands;
use args::{Cli, Command};
use clap::Parser;
use remote_unlock_lib::config::Config;

fn main() {
    let config = Config::new();
    let args = Cli::parse();

    match args.command {
        Command::BeginEnroll(_) => {
            commands::begin_enroll(&config).unwrap();
        }
        #[cfg(debug_assertions)]
        Command::GenerateKeys(generate_keys) => {
            commands::generate_keys(&config, generate_keys).unwrap();
        }
        Command::Terminate(_) => {}
    }
}
