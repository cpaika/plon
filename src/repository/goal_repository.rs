use anyhow::Result;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;
use crate::domain::goal::Goal;

#[derive(Clone)]
pub struct GoalRepository {
    pool: Arc<SqlitePool>,
}

impl GoalRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, goal: &Goal) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    pub async fn update(&self, goal: &Goal) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Goal>> {
        // TODO: Implement
        Ok(None)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        // TODO: Implement
        Ok(false)
    }

    pub async fn list(&self) -> Result<Vec<Goal>> {
        // TODO: Implement
        Ok(Vec::new())
    }
}