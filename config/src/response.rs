use serde::{Deserialize, Serialize};

use crate::error::VaultCliError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub message: String,
    pub data: Option<String>,
}

impl Response {
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
