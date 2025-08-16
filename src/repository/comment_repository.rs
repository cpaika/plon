use anyhow::Result;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;
use crate::domain::comment::Comment;

#[derive(Clone)]
pub struct CommentRepository {
    pool: Arc<SqlitePool>,
}

impl CommentRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, comment: &Comment) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    pub async fn update(&self, comment: &Comment) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Comment>> {
        // TODO: Implement
        Ok(None)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        // TODO: Implement
        Ok(false)
    }

    pub async fn list_for_entity(&self, entity_id: Uuid) -> Result<Vec<Comment>> {
        // TODO: Implement
        Ok(Vec::new())
    }
}