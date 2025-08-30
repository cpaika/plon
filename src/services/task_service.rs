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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Priority, TaskStatus};
    use crate::repository::database::init_test_database;

    async fn setup() -> TaskService {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        TaskService::new(repository)
    }

    #[tokio::test]
    async fn test_create_task() {
        let service = setup().await;
        let task = Task::new("Test Task".to_string(), "Description".to_string());

        let created = service.create(task.clone()).await.unwrap();
        assert_eq!(created.title, task.title);
        assert_eq!(created.description, task.description);
    }

    #[tokio::test]
    async fn test_get_task() {
        let service = setup().await;
        let task = Task::new("Test Task".to_string(), "Description".to_string());
        let created = service.create(task.clone()).await.unwrap();

        let retrieved = service.get(created.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_update_task() {
        let service = setup().await;
        let mut task = Task::new("Original".to_string(), "Original desc".to_string());
        let created = service.create(task.clone()).await.unwrap();

        task = created;
        task.title = "Updated".to_string();
        task.status = TaskStatus::InProgress;
        task.priority = Priority::High;

        let updated = service.update(task.clone()).await.unwrap();
        assert_eq!(updated.title, "Updated");

        let retrieved = service.get(updated.id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated");
        assert_eq!(retrieved.status, TaskStatus::InProgress);
        assert_eq!(retrieved.priority, Priority::High);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let service = setup().await;
        let task = Task::new("To Delete".to_string(), "".to_string());
        let created = service.create(task).await.unwrap();

        let deleted = service.delete(created.id).await.unwrap();
        assert!(deleted);

        let retrieved = service.get(created.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_all_tasks() {
        let service = setup().await;

        // Create multiple tasks
        for i in 1..=3 {
            let task = Task::new(format!("Task {}", i), "".to_string());
            service.create(task).await.unwrap();
        }

        let tasks = service.list_all().await.unwrap();
        assert_eq!(tasks.len(), 3);
    }
}
