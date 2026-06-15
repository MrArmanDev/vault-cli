use crate::{
    cli::cli::{Cli, Commands, UserCommands},
    helper::helper::unlock,
};
use clap::Parser;
use config::{
    error::VaultCliError,
    request::{Request, UserRequest},
};

pub fn run() -> Result<Request, VaultCliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::User { user } => match user {
            UserCommands::Add { name } => Ok(Request::User(UserRequest::Add { name })),
            UserCommands::Remove { name } => Ok(Request::User(UserRequest::Remove { name })),
            UserCommands::Rename { old_name, new_name } => {
                Ok(Request::User(UserRequest::Rename { old_name, new_name }))
            }
        },

        Commands::Unlock => match unlock() {
            Ok(v) => Ok(Request::Unlock(v)),
            Err(e) => Err(e),
        },
        Commands::Lock => Ok(Request::Lock),

        Commands::Default { name } => Ok(Request::Default(name))

    }
}
