use crate::domain::goal::Goal;
use crate::repository::Repository;
use anyhow::Result;
use std::sync::Arc;

pub struct GoalService {
    repository: Arc<Repository>,
}

impl GoalService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn list_all(&self) -> Result<Vec<Goal>> {
        // Stub implementation
        Ok(Vec::new())
    }
}