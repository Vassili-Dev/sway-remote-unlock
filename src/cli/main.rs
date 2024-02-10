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
        Command::Terminate(_) => {}
    }
}
