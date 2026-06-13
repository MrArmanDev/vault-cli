use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vaultcli-deamon", version, about = "A simple password manager CLI tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init {
        #[arg(long, short)]
        url: String,
    },

    Start,
}
