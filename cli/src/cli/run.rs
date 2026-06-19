use crate::{
    cli::cli::{Cli, Commands, UserCommands, Vault},
    helper::helper::{add_pass, unlock},
};
use clap::Parser;
use config::{
    error::VaultCliError,
    request::{Request, UserRequest, Vault as ReqVault, VaultGet},
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

        Commands::Default { name } => Ok(Request::Default(name)),

        Commands::Vault { vault } => match vault {
            Vault::Add {
                username,
                app,
                hint,
                master
            } => match add_pass(username, app, hint, master) {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            },

            Vault::Get {
                username,
                app
            } => Ok(Request::Vault(ReqVault::Get(VaultGet {username, app})))
        },
    }
}
