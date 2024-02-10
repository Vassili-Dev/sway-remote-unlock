use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
#[command(version, about, long_about = None)]
pub enum Command {
    BeginEnroll(BeginEnrollCommand),

    Terminate(TerminateCommand),
}

#[derive(Args, Debug)]
pub struct BeginEnrollCommand {}

#[derive(Args, Debug)]
pub struct TerminateCommand {}
