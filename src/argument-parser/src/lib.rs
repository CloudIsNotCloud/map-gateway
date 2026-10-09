use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Server {
        #[command(subcommand)]
        action: Option<ServerActions>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ServerActions {
    Start {
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    Stop,
    Restart,
}
