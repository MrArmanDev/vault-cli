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
                password BYTEA NOT NULL,
                nonce BYTEA NOT NULL,
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
    if entry.get_password().is_err() {
        entry.set_password(&salt_str)?;
    }

    println!("Salt: {}", salt_str);

    Ok("Database initialized successfully".to_string())
}

pub mod user {

    use crate::helper::is_valid;
    use config::error::VaultCliError;
    use sqlx::PgPool;

    pub async fn add_user(name: String, pool: &PgPool) -> Result<String, VaultCliError> {
        if !is_valid(&name) {
            return Err(VaultCliError::AppError(
                "Invalid username. Only alphanumeric characters and underscores are allowed."
                    .to_string(),
            ));
        }

        let result = sqlx::query("INSERT INTO users (master) VALUES ($1) ON CONFLICT DO NOTHING")
            .bind(&name)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(VaultCliError::AppError(format!(
                "User '{}' already exists. Please choose a different name.",
                name
            )));
        }

        Ok("User added successfully".to_string())
    }

    pub async fn remove_user(name: String, pool: &PgPool) -> Result<String, VaultCliError> {
        if !is_valid(&name) {
            return Err(VaultCliError::AppError(
                "Invalid username. Only alphanumeric characters and underscores are allowed."
                    .to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM users WHERE master = $1")
            .bind(&name)
            .execute(pool)
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
        pool: &PgPool,
    ) -> Result<String, VaultCliError> {
        if !is_valid(&old_name) || !is_valid(&new_name) {
            return Err(VaultCliError::AppError(
                "Invalid username. Only alphanumeric characters and underscores are allowed."
                    .to_string(),
            ));
        }

        let result = sqlx::query("UPDATE users SET master = $1 WHERE master = $2")
            .bind(&new_name)
            .bind(&old_name)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(VaultCliError::AppError(
                format!("User name change failed.",),
            ));
        }

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

    pub async fn unlock(mut pass: String, master_key: &Key) -> Result<String, VaultCliError> {
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

    pub async fn lock(master_key: &Key) -> String {
        let mut master = master_key.lock().await;

        if master.is_some() {
            master.zeroize();
        }

        *master = None;

        "Vault locked.".to_string()
    }

    pub async fn default(name: String, pool: &PgPool) -> Result<String, VaultCliError> {
        if !is_valid(&name) {
            return Err(VaultCliError::AppError(
                "Invalid username. Only alphanumeric characters and underscores are allowed."
                    .to_string(),
            ));
        }

        sqlx::query("UPDATE users SET is_default = FALSE WHERE is_default = TRUE;")
            .execute(pool)
            .await?;

        let result = sqlx::query("UPDATE users SET is_default = TRUE WHERE master = $1")
            .bind(&name)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(VaultCliError::AppError(format!(
                "Default user change failed. Please check user exits or not"
            )));
        }

        Ok(format!("Default user is now: {}", name))
    }
}

pub mod vault {
    use crate::{confiig::AppStates, helper::get_master};
    use chacha20poly1305::{
        ChaCha20Poly1305, Key, KeyInit, Nonce,
        aead::{Aead, Payload},
    };
    use config::{
        error::VaultCliError,
        request::{VaultAdd, VaultGet},
        response::Password,
    };
    use sqlx::Postgres;

    pub async fn add_pass(data: VaultAdd, state: &AppStates) -> Result<String, VaultCliError> {
        let Some(master) = get_master(data.master, &state.pool).await? else {
            return Err(VaultCliError::AppError(
                "Default user is not set please provide user or set default user".to_string(),
            ));
        };

        let key = {
            let master = state.key.lock().await;

            match master.as_ref() {
                Some(v) => *v,
                None => return Err(VaultCliError::AppError("Vault is locked".to_string())),
            }
        };

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));

        let mut nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_mut_slice(&mut nonce_bytes);

        let text = match cipher.encrypt(nonce, Payload::from(data.pass.as_bytes())) {
            Ok(v) => v,
            Err(v) => {
                return Err(VaultCliError::AppError(format!(
                    "Enctyption error: {:#?}",
                    v
                )));
            }
        };

        let result = sqlx::query(
            r#"
        INSERT INTO data (master, username, password, nonce, app, hint) VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        )
        .bind(&master)
        .bind(&data.username)
        .bind(text)
        .bind(nonce_bytes)
        .bind(&data.app)
        .bind(&data.hint)
        .execute(&state.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(VaultCliError::AppError(
                "Failed to add password".to_string(),
            ));
        }

        Ok("Password added.".to_string())
    }

    pub async fn get_pass(
        data: VaultGet,
        state: &AppStates,
    ) -> Result<(String, Vec<Password>), VaultCliError> {
        let Some(master) = get_master(None, &state.pool).await? else {
            return Err(VaultCliError::AppError(
                "Default user is not set please provide user or set default user".to_string(),
            ));
        };

        let key = {
            let master = state.key.lock().await;

            match master.as_ref() {
                Some(v) => *v,
                None => return Err(VaultCliError::AppError("Vault is locked".to_string())),
            }
        };

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));

        let mut db = sqlx::QueryBuilder::<Postgres>::new(
            "SELECT id, username, app, hint, master, password, nonce, created_at FROM data WHERE 1=1",
        );

        db.push(" AND master = ");
        db.push_bind(master);

        if let Some(v) = data.username {
            db.push(" AND username = ");
            db.push_bind(v);
        }

        if let Some(v) = data.app {
            db.push(" AND app = ");
            db.push_bind(v);
        }

        let query = db.build_query_as::<Password>();
        let mut data = query.fetch_all(&state.pool).await?;

        if data.is_empty() {
            return Err(VaultCliError::AppError("No password found".to_string()));
        }

        for pass in data.iter_mut() {
            let nonce = Nonce::from_slice(&pass.nonce);
            let payload = Payload {
                msg: &pass.password,
                aad: &[],
            };

            let text = cipher.decrypt(nonce, payload);

            match text {
                Ok(v) => pass.password = v,
                Err(_) => pass.password = pass.password.clone(),
            }
        }

        Ok((format!("yes"), data))
    }
}
