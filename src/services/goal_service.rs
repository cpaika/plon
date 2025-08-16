use crate::domain::goal::Goal;
use crate::repository::Repository;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

pub struct GoalService {
    repository: Arc<Repository>,
}

impl GoalService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, goal: Goal) -> Result<Goal> {
        self.repository.goals.create(&goal).await?;
        Ok(goal)
    }

    pub async fn update(&self, goal: Goal) -> Result<Goal> {
        self.repository.goals.update(&goal).await?;
        Ok(goal)
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Goal>> {
        self.repository.goals.get(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        self.repository.goals.delete(id).await
    }

    pub async fn list_all(&self) -> Result<Vec<Goal>> {
        self.repository.goals.list_all().await
    }
}