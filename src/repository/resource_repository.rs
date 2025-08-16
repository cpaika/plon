use anyhow::Result;
use sqlx::SqlitePool;
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
        // TODO: Implement
        Ok(())
    }

    pub async fn update(&self, resource: &Resource) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Resource>> {
        // TODO: Implement
        Ok(None)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        // TODO: Implement
        Ok(false)
    }

    pub async fn list(&self) -> Result<Vec<Resource>> {
        // TODO: Implement
        Ok(Vec::new())
    }
}