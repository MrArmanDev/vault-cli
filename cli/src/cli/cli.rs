use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vaultcli", version, about = "A simple password manager CLI tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    User {
        #[command(subcommand)]
        user: UserCommands,
    },

    Unlock,

    Lock,

    Default {
        #[arg(short, long)]
        name : String
    }
    
}

#[derive(Subcommand)]
pub enum UserCommands {
    Add {
        #[arg(short, long)]
        name: String,
    },

    Remove {
        #[arg(short, long)]
        name: String,
    },

    Rename {
        #[arg(short, long)]
        old_name: String,

        #[arg(short, long)]
        new_name: String,
    },

}
