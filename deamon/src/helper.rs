use config::error::VaultCliError;
use sqlx::PgPool;

use crate::confiig::UserRepo;

pub fn is_valid(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub async fn get_default_user(pool: &PgPool) -> Result<Option<UserRepo>, VaultCliError> {
    let user = sqlx::query_as::<_, UserRepo>("SELECT * FROM users WHERE is_default = TRUE")
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

pub async fn get_master(
    master: Option<String>,
    pool: &PgPool,
) -> Result<Option<String>, VaultCliError> {
    if let Some(v) = master {
        if !is_valid(&v) {
            return Err(VaultCliError::AppError(format!(
                "Invalid master name. Only alphanumeric characters and underscores are allowed."
            )));
        }

        let result: bool = sqlx::query_scalar("SELECT * FROM users WHERE master = $1")
            .bind(&v)
            .fetch_one(pool)
            .await?;

        if !result {
            return Err(VaultCliError::AppError(format!(
                "Invalid master this master is not exits."
            )));
        }

        return Ok(Some(v));
    }

    let Some(user) = get_default_user(pool).await? else {
        return Ok(None);
    };

    Ok(Some(user.master))
}
