use argon2::password_hash::{SaltString, rand_core::OsRng};
use config::error::VaultCliError;
use keyring::Entry;
use sqlx::PgPool;

pub async fn initialize(url: String) -> Result<String, VaultCliError> {
    let pool = PgPool::connect(&url).await?;

    let entry = Entry::new("vaultcli", "db-url")?;
    entry.set_password(&url)?;

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_name = 'users'
        )",
    )
    .fetch_one(&pool)
    .await?;

    if !exists {
        sqlx::query(
            "CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                master VARCHAR(255) NOT NULL UNIQUE,
                is_default BOOLEAN NOT NULL DEFAULT FALSE
            )",
        )
        .execute(&pool)
        .await?;

        println!("users table created");
    }

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_name = 'data'
        )",
    )
    .fetch_one(&pool)
    .await?;

    if !exists {
        sqlx::query(
            "CREATE TABLE data (
                id SERIAL PRIMARY KEY,
                master VARCHAR(255) NOT NULL REFERENCES users(master) ON DELETE CASCADE ON UPDATE CASCADE,   
                username VARCHAR(255) NOT NULL,
                password VARCHAR(255) NOT NULL,
                app VARCHAR(255) NOT NULL,
                hint VARCHAR(255) NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&pool)
        .await?;

        println!("data table created");
    }

    let salt = SaltString::generate(&mut OsRng);
    let salt_str = salt.as_str().to_string();

    let entry = Entry::new("vaultcli", "salt")?;
    entry.set_password(&salt_str)?;

    println!("Salt: {}", salt_str);

    Ok("Database initialized successfully".to_string())
}

pub mod user {

    use crate::helper::is_valid;
    use config::error::VaultCliError;
    use sqlx::PgPool;

    pub async fn add_user(name: String, pool: PgPool) -> Result<String, VaultCliError> {
        if !is_valid(&name) {
            return Err(VaultCliError::AppError(
                "Invalid username. Only alphanumeric characters and underscores are allowed."
                    .to_string(),
            ));
        }

        let result = sqlx::query("INSERT INTO users (master) VALUES ($1) ON CONFLICT DO NOTHING")
            .bind(&name)
            .execute(&pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(VaultCliError::AppError(format!(
                "User '{}' already exists. Please choose a different name.",
                name
            )));
        }

        Ok("User added successfully".to_string())
    }

    pub async fn remove_user(name: String, pool: PgPool) -> Result<String, VaultCliError> {
        if !is_valid(&name) {
            return Err(VaultCliError::AppError(
                "Invalid username. Only alphanumeric characters and underscores are allowed."
                    .to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM users WHERE master = $1")
            .bind(&name)
            .execute(&pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(VaultCliError::AppError(format!(
                "User '{}' does not exist.",
                &name
            )));
        }

        Ok(format!("User '{}' removed successfully", &name))
    }

    pub async fn rename_user(
        old_name: String,
        new_name: String,
        pool: PgPool,
    ) -> Result<String, VaultCliError> {
        if !is_valid(&old_name) || !is_valid(&new_name) {
            return Err(VaultCliError::AppError(
                "Invalid username. Only alphanumeric characters and underscores are allowed."
                    .to_string(),
            ));
        }

        sqlx::query("UPDATE users SET master = $1 WHERE master = $2")
            .bind(&new_name)
            .bind(&old_name)
            .execute(&pool)
            .await?;

        Ok(format!(
            "User '{}' renamed to '{}' successfully",
            &old_name, &new_name
        ))
    }
}

pub mod secure {
    use argon2::Argon2;
    use config::error::VaultCliError;
    use keyring::Entry;
    use sqlx::PgPool;
    use zeroize::Zeroize;

    use crate::{confiig::Key, helper::is_valid};

    pub async fn unlock(mut pass: String, master_key: Key) -> Result<String, VaultCliError> {
        let salt = Entry::new("vaultcli", "salt")?.get_password()?;

        let mut key = [0u8; 32];

        match Argon2::default().hash_password_into(pass.as_bytes(), salt.as_bytes(), &mut key) {
            Ok(_) => {
                pass.zeroize();
            }
            Err(e) => {
                pass.zeroize();
                return Err(VaultCliError::AppError(format!(
                    "Salt Err: {}",
                    e.to_string()
                )));
            }
        };

        let mut master = master_key.lock().await;

        if master.is_none() {
            *master = Some(key);
        }

        Ok("Vault unlocked.".to_string())
    }

    pub async fn lock(master_key: Key) -> String {
        let mut master = master_key.lock().await;

        if master.is_some() {
            master.zeroize();
        }

        *master = None;

        "Vault locked.".to_string()
    }

    pub async fn default(name: String, pool: PgPool) -> Result<String, VaultCliError> {
        if !is_valid(&name) {
            return Err(VaultCliError::AppError(
                "Invalid username. Only alphanumeric characters and underscores are allowed."
                    .to_string(),
            ));
        }

        sqlx::query(
            "UPDATE users SET is_default = FALSE WHERE is_default = TRUE ON CONFLICT DO NOTHING",
        )
        .execute(&pool)
        .await?;

        sqlx::query("UPDATE users SET is_default = TRUE WHERE master = $1")
            .bind(&name)
            .execute(&pool)
            .await?;



        Ok(format!("Default user is now: {}", name))
    }
}
