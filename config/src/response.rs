use serde::{Deserialize, Serialize};

use crate::error::VaultCliError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> Response<T> {
    pub fn ok(message: String) -> Self {
        Self {
            success: true,
            message,
            data: None,
        }
    }

    pub fn error(err: VaultCliError) -> Self {
        Self {
            success: false,
            message:format!("{:#?}", err),
            data: None,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, )]
pub struct Password {
    pub id: i32,
    pub username: String,
    pub app: String,
    pub hint: String,
    pub master: String,
    pub password: Vec<u8>,
    pub nonce: Vec<u8>,
    pub date: String,
}