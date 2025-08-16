use anyhow::Result;
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use uuid::Uuid;
use crate::domain::resource::Resource;

#[derive(Clone)]
pub struct ResourceRepository {
    pool: Arc<SqlitePool>,
}

impl ResourceRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, resource: &Resource) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO resources (
                id, name, email, role, skills, metadata_filters,
                weekly_hours, current_load, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(resource.id.to_string())
        .bind(&resource.name)
        .bind(&resource.email)
        .bind(&resource.role)
        .bind(serde_json::to_string(&resource.skills)?)
        .bind(serde_json::to_string(&resource.metadata_filters)?)
        .bind(resource.weekly_hours)
        .bind(resource.current_load)
        .bind(resource.created_at.to_rfc3339())
        .bind(resource.updated_at.to_rfc3339())
        .execute(self.pool.as_ref())
        .await?;
        
        Ok(())
    }

    pub async fn update(&self, _resource: &Resource) -> Result<()> {
        // TODO: Implement update
        Ok(())
    }

    pub async fn get(&self, _id: Uuid) -> Result<Option<Resource>> {
        // TODO: Implement get
        Ok(None)
    }

    pub async fn delete(&self, _id: Uuid) -> Result<bool> {
        // TODO: Implement delete
        Ok(false)
    }

    pub async fn list_all(&self) -> Result<Vec<Resource>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, email, role, skills, metadata_filters,
                   weekly_hours, current_load, created_at, updated_at
            FROM resources
            ORDER BY name
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        
        let mut resources = Vec::new();
        
        for row in rows {
            use chrono::DateTime;
            
            let resource = Resource {
                id: Uuid::parse_str(row.get("id"))?,
                name: row.get("name"),
                email: row.get("email"),
                role: row.get("role"),
                skills: serde_json::from_str(row.get("skills"))?,
                metadata_filters: serde_json::from_str(row.get("metadata_filters"))?,
                weekly_hours: row.get("weekly_hours"),
                current_load: row.get("current_load"),
                availability: Vec::new(), // TODO: Load from separate table if needed
                created_at: DateTime::parse_from_rfc3339(row.get("created_at"))?.with_timezone(&chrono::Utc),
                updated_at: DateTime::parse_from_rfc3339(row.get("updated_at"))?.with_timezone(&chrono::Utc),
            };
            
            resources.push(resource);
        }
        
        Ok(resources)
    }
}