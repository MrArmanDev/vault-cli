use config::error::VaultCliError;
use keyring::Entry;
use sqlx::PgPool;

use crate::{helper::is_valid};

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
                master VARCHAR(255) NOT NULL UNIQUE
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

    Ok("Database initialized successfully".to_string())
}

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

// pub async fn remove_user(name: String, pool: PgPool) -> Result<String, VaultCliError> {todo!()}