use crate::domain::task_config::TaskConfiguration;
use anyhow::Result;
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct TaskConfigRepository {
    pool: Arc<SqlitePool>,
}

impl TaskConfigRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create_tables(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task_configurations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                metadata_schema TEXT NOT NULL,
                state_machine TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        )
        .execute(self.pool.as_ref())
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_task_configurations_name 
             ON task_configurations(name)"
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    pub async fn create(&self, config: &TaskConfiguration) -> Result<()> {
        sqlx::query(
            "INSERT INTO task_configurations (
                id, name, description, metadata_schema, state_machine, 
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )
        .bind(config.id.to_string())
        .bind(&config.name)
        .bind(&config.description)
        .bind(serde_json::to_string(&config.metadata_schema)?)
        .bind(serde_json::to_string(&config.state_machine)?)
        .bind(config.created_at.to_rfc3339())
        .bind(config.updated_at.to_rfc3339())
        .execute(self.pool.as_ref())
        .await?;
        
        Ok(())
    }

    pub async fn update(&self, config: &TaskConfiguration) -> Result<()> {
        sqlx::query(
            "UPDATE task_configurations SET 
                name = ?2,
                description = ?3,
                metadata_schema = ?4,
                state_machine = ?5,
                updated_at = ?6
             WHERE id = ?1"
        )
        .bind(config.id.to_string())
        .bind(&config.name)
        .bind(&config.description)
        .bind(serde_json::to_string(&config.metadata_schema)?)
        .bind(serde_json::to_string(&config.state_machine)?)
        .bind(config.updated_at.to_rfc3339())
        .execute(self.pool.as_ref())
        .await?;
        
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<TaskConfiguration>> {
        let result = sqlx::query(
            "SELECT id, name, description, metadata_schema, state_machine, 
                    created_at, updated_at
             FROM task_configurations 
             WHERE id = ?1"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool.as_ref())
        .await?;
        
        Ok(result.map(|row| self.row_to_config(row)))
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<TaskConfiguration>> {
        let result = sqlx::query(
            "SELECT id, name, description, metadata_schema, state_machine, 
                    created_at, updated_at
             FROM task_configurations 
             WHERE name = ?1"
        )
        .bind(name)
        .fetch_optional(self.pool.as_ref())
        .await?;
        
        Ok(result.map(|row| self.row_to_config(row)))
    }

    pub async fn list_all(&self) -> Result<Vec<TaskConfiguration>> {
        let rows = sqlx::query(
            "SELECT id, name, description, metadata_schema, state_machine, 
                    created_at, updated_at
             FROM task_configurations 
             ORDER BY name"
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        
        Ok(rows.into_iter().map(|row| self.row_to_config(row)).collect())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM task_configurations WHERE id = ?1")
            .bind(id.to_string())
            .execute(self.pool.as_ref())
            .await?;
        
        Ok(())
    }

    fn row_to_config(&self, row: sqlx::sqlite::SqliteRow) -> TaskConfiguration {
        let id: String = row.get(0);
        let name: String = row.get(1);
        let description: String = row.get(2);
        let metadata_schema_json: String = row.get(3);
        let state_machine_json: String = row.get(4);
        let created_at: String = row.get(5);
        let updated_at: String = row.get(6);
        
        TaskConfiguration {
            id: Uuid::parse_str(&id).unwrap(),
            name,
            description,
            metadata_schema: serde_json::from_str(&metadata_schema_json).unwrap(),
            state_machine: serde_json::from_str(&state_machine_json).unwrap(),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task_config::{
        MetadataFieldConfig, FieldType, StateDefinition, StateTransition
    };

    async fn setup_test_db() -> Arc<SqlitePool> {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        Arc::new(pool)
    }

    #[tokio::test]
    async fn test_create_and_get_config() {
        let pool = setup_test_db().await;
        let repo = TaskConfigRepository::new(pool);
        repo.create_tables().await.unwrap();

        let mut config = TaskConfiguration::new("Test Config".to_string());
        config.description = "Test description".to_string();
        
        repo.create(&config).await.unwrap();
        
        let retrieved = repo.get(config.id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Test Config");
        assert_eq!(retrieved.description, "Test description");
    }

    #[tokio::test]
    async fn test_update_config() {
        let pool = setup_test_db().await;
        let repo = TaskConfigRepository::new(pool);
        repo.create_tables().await.unwrap();

        let mut config = TaskConfiguration::new("Original".to_string());
        repo.create(&config).await.unwrap();
        
        config.name = "Updated".to_string();
        config.updated_at = chrono::Utc::now();
        repo.update(&config).await.unwrap();
        
        let retrieved = repo.get(config.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated");
    }

    #[tokio::test]
    async fn test_list_all_configs() {
        let pool = setup_test_db().await;
        let repo = TaskConfigRepository::new(pool);
        repo.create_tables().await.unwrap();

        let config1 = TaskConfiguration::new("Config A".to_string());
        let config2 = TaskConfiguration::new("Config B".to_string());
        
        repo.create(&config1).await.unwrap();
        repo.create(&config2).await.unwrap();
        
        let configs = repo.list_all().await.unwrap();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].name, "Config A");
        assert_eq!(configs[1].name, "Config B");
    }

    #[tokio::test]
    async fn test_delete_config() {
        let pool = setup_test_db().await;
        let repo = TaskConfigRepository::new(pool);
        repo.create_tables().await.unwrap();

        let config = TaskConfiguration::new("To Delete".to_string());
        repo.create(&config).await.unwrap();
        
        repo.delete(config.id).await.unwrap();
        
        let retrieved = repo.get(config.id).await.unwrap();
        assert!(retrieved.is_none());
    }
}