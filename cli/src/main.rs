use config::{
    SOCKET_PATH,
    error::VaultCliError,
    response::{Password, Response},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::cli::run::run;

mod cli;

mod helper;

#[tokio::main]
async fn main() -> Result<(), VaultCliError> {
    let req = run()?;

    let bytes = serde_json::to_vec(&req)?;

    let mut stream = UnixStream::connect(SOCKET_PATH).await?;
    stream.write_all(&bytes).await?;

    let mut buf = vec![0u8; 4096];

    let n = stream.read(&mut buf).await?;
    let res: Response<Vec<Password>> = serde_json::from_slice(&buf[..n])?;

    if res.success {
        println!("Success: {}", res.message);
        if let Some(v) = res.data {
            println!(
                "{:<20} {:<20} {:<30} {:<20}",
                "Username", "App", "Password", "Hint"
            );
            for pass in v {
                let password = match String::from_utf8(pass.password) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        continue;
                    }
                };
                println!(
                    "{:<20} {:<20} {:<30} {:<20}",
                    pass.username, pass.app, password, pass.hint
                );
            }
        }
    } else {
        eprintln!("Error: {}", res.message);
    }

    Ok(())
}
