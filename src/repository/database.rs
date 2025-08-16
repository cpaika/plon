use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;
use tokio::fs;

pub async fn init_database(db_path: &str) -> Result<SqlitePool> {
    // Ensure the directory exists
    if let Some(parent) = Path::new(db_path).parent() {
        fs::create_dir_all(parent).await?;
    }

    // Create the database file if it doesn't exist
    if !Path::new(db_path).exists() {
        fs::File::create(db_path).await?;
    }

    // Create connection pool with proper options
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite://{}?mode=rwc", db_path))
        .await?;

    // Run migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

pub async fn init_test_database() -> Result<SqlitePool> {
    // Use in-memory database for tests
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await?;

    // Check if we've already run migrations by checking for a key table
    let table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tasks'"
    )
    .fetch_one(pool)
    .await?;

    if table_exists.0 > 0 {
        // Database already initialized, skip migrations
        return Ok(());
    }

    // Read and execute migration files
    let migration_sql = include_str!("../../migrations/001_initial_schema.sql");
    
    // Split by semicolon and execute each statement
    for statement in migration_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            // Ignore errors for CREATE INDEX statements (in case they already exist)
            if trimmed.starts_with("CREATE INDEX") {
                let _ = sqlx::query(trimmed).execute(pool).await;
            } else {
                sqlx::query(trimmed)
                    .execute(pool)
                    .await?;
            }
        }
    }

    Ok(())
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