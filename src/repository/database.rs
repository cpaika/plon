use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::path::Path;
use tokio::fs;

pub async fn init_database(db_path: &str) -> Result<SqlitePool> {
    // Ensure the directory exists
    if let Some(parent) = Path::new(db_path).parent() {
        fs::create_dir_all(parent).await?;
    }

    let db_url = format!("sqlite://{}", db_path);
    
    // Create database if it doesn't exist
    if !Sqlite::database_exists(&db_url).await? {
        println!("Creating new database at: {}", db_path);
        Sqlite::create_database(&db_url).await?;
    }
    
    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run SQLx migrations
    println!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    println!("Migrations completed successfully");

    Ok(pool)
}

pub async fn init_test_database() -> Result<SqlitePool> {
    // Use in-memory database for tests
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    // Run SQLx migrations for tests
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_test_database() {
        let pool = init_test_database().await.unwrap();
        
        // Verify tables exist
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(&pool)
            .await
            .unwrap();
        
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_foreign_keys_enabled() {
        let pool = init_test_database().await.unwrap();
        
        let result: (i32,) = sqlx::query_as("PRAGMA foreign_keys")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        assert_eq!(result.0, 1);
    }
}