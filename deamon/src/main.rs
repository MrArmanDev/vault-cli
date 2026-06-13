use std::time::Duration;

use clap::Parser;
use config::{SOCKET_PATH, error::VaultCliError, request::Request, response::Response};
use keyring::Entry;
use sqlx::postgres::PgPoolOptions;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixListener,
};

use crate::{
    cli::cli::{Cli, Commands},
    handler::handler::handle,
    worker::worker::initialize,
};

mod cli;
mod error;
mod handler;
mod helper;
mod worker;

#[tokio::main]
async fn main() -> Result<(), VaultCliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { url } => {
            let result = initialize(url).await;

            match result {
                Ok(v) => println!("{}", v),
                Err(e) => {
                    println!("{:#?}", e)
                }
            }
        }
        Commands::Start => {
            let listener = UnixListener::bind(SOCKET_PATH)?;

            println!("Vault Cli Server Start.");

            let enry = Entry::new("vaultcli", "db-url")?;
            let url = enry.get_password()?;

            let pool = PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(Duration::from_secs(3))
                .connect(&url)
                .await?;

            loop {
                let (mut stream, _) = listener.accept().await?;

                let cpool = pool.clone();

                tokio::spawn(async move {
                    let mut buf = vec![0u8; 4096];
                    let n = match stream.read(&mut buf).await {
                        Ok(n) => n,
                        Err(err) => {
                            let error_response = Response {
                                success: false,
                                message: format!("Failed to read from stream: {}", err),
                                data: None,
                            };

                            if let Ok(err) = serde_json::to_vec(&error_response) {
                                let _ = stream.write_all(&err).await;
                            }

                            return;
                        }
                    };

                    let command: Request = match serde_json::from_slice(&buf[..n]) {
                        Ok(cmd) => cmd,
                        Err(err) => {
                            let error_response = Response {
                                success: false,
                                message: format!("Failed to deserialize command: {}", err),
                                data: None,
                            };

                            if let Ok(err) = serde_json::to_vec(&error_response) {
                                let _ = stream.write_all(&err).await;
                            }

                            return;
                        }
                    };

                    let response = handle(command, cpool).await;
                    let response_bytes = match serde_json::to_vec(&response) {
                        Ok(bytes) => bytes,
                        Err(err) => {
                            let error_response = Response {
                                success: false,
                                message: format!("Failed to serialize response: {}", err),
                                data: None,
                            };

                            if let Ok(err) = serde_json::to_vec(&error_response) {
                                let _ = stream.write_all(&err).await;
                            }

                            return;
                        }
                    };

                    let _ = stream.write_all(&response_bytes).await;
                });
            }
        }
    }

    Ok(())
}
