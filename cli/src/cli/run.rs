use crate::{
    cli::cli::{Cli, Commands, UserCommands},
};
use clap::Parser;
use config::{error::VaultCliError, request::{Request, UserRequest}};


pub fn run() -> Result<Request, VaultCliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::User { user  } => match user {
            UserCommands::Add { name } => Ok(Request::User(UserRequest::Add { name })),
            UserCommands::Remove { name } => Ok(Request::User(UserRequest::Remove { name })),
            UserCommands::Rename { old_name, new_name } => Ok(Request::User(UserRequest::Rename { old_name, new_name })),
        }
    }
}
