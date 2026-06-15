use std::{sync::Arc, time::Duration};

use config::error::VaultCliError;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::sync::Mutex;

pub type Key = Arc<Mutex<Option<[u8; 32]>>>;

pub struct AppStates {
    pub pool: PgPool,
    pub key: Key,
}

impl AppStates {
    pub async fn new(url: &str) -> Result<Self, VaultCliError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&url)
            .await?;
        Ok(AppStates {
            pool,
            key: Arc::new(Mutex::new(None)),
        })
    }
}
