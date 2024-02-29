use clap::{Args, Parser, Subcommand, ValueEnum};

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

    #[cfg(debug_assertions)]
    GenerateKeys(GenerateKeysCommand),
}

#[derive(Args, Debug)]
pub struct BeginEnrollCommand {}

#[derive(Args, Debug)]
pub struct TerminateCommand {}

#[derive(Args, Debug)]
pub struct GenerateKeysCommand {
    #[arg(short, long, help = "The name of the keypair")]
    pub output: Option<String>,

    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Overwrite the keypair if it exists"
    )]
    pub force: bool,

    #[arg(long, default_value = "pem", help = "Format of the keypair")]
    pub format: KeyFormat,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum KeyFormat {
    PEM,
    DIR,
    BIN,
}
