use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultCliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("AppError error: {0}")]
    AppError(String),

    #[error("Entry error: {0}")]
    Entry(#[from] keyring::Error),

    #[error("Postgres error: {0}")]
    Postgres(#[from] sqlx::Error),
}