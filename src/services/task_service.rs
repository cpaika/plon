use crate::domain::task::Task;
use crate::repository::Repository;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct TaskService {
    pub repository: Arc<Repository>,
}

impl TaskService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, task: Task) -> Result<Task> {
        self.repository.tasks.create(&task).await?;
        Ok(task)
    }

    pub async fn update(&self, task: Task) -> Result<Task> {
        self.repository.tasks.update(&task).await?;
        Ok(task)
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Task>> {
        self.repository.tasks.get(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        self.repository.tasks.delete(id).await
    }

    pub async fn list_all(&self) -> Result<Vec<Task>> {
        self.repository.tasks.list(Default::default()).await
    }
}