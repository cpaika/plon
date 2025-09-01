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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::goal::GoalStatus;
    use crate::domain::task::Task;
    use crate::repository::database::init_test_database;

    async fn setup() -> GoalService {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        GoalService::new(repository)
    }

    #[tokio::test]
    async fn test_create_goal() {
        let service = setup().await;
        let goal = Goal::new("Test Goal".to_string(), "Goal description".to_string());

        let created = service.create(goal.clone()).await.unwrap();
        assert_eq!(created.title, goal.title);
        assert_eq!(created.description, goal.description);
        assert_eq!(created.status, GoalStatus::NotStarted);
    }

    #[tokio::test]
    async fn test_get_goal() {
        let service = setup().await;
        let goal = Goal::new("Test Goal".to_string(), "Description".to_string());
        let created = service.create(goal).await.unwrap();

        let retrieved = service.get(created.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_update_goal() {
        let service = setup().await;
        let mut goal = Goal::new("Original Goal".to_string(), "Original desc".to_string());
        let created = service.create(goal.clone()).await.unwrap();

        goal = created;
        goal.title = "Updated Goal".to_string();
        goal.status = GoalStatus::InProgress;
        goal.progress = 50.0;

        let updated = service.update(goal.clone()).await.unwrap();
        assert_eq!(updated.title, "Updated Goal");

        let retrieved = service.get(updated.id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Goal");
        assert_eq!(retrieved.status, GoalStatus::InProgress);
        assert_eq!(retrieved.progress, 50.0);
    }

    #[tokio::test]
    async fn test_delete_goal() {
        let service = setup().await;
        let goal = Goal::new("To Delete".to_string(), "".to_string());
        let created = service.create(goal).await.unwrap();

        let deleted = service.delete(created.id).await.unwrap();
        assert!(deleted);

        let retrieved = service.get(created.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_all_goals() {
        let service = setup().await;

        // Create multiple goals
        for i in 1..=3 {
            let goal = Goal::new(format!("Goal {}", i), "".to_string());
            service.create(goal).await.unwrap();
        }

        let goals = service.list_all().await.unwrap();
        assert_eq!(goals.len(), 3);
    }

    #[tokio::test]
    async fn test_goal_with_tasks() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = GoalService::new(repository.clone());

        // Create actual tasks first
        let task1 = Task::new("Task 1".to_string(), "Description 1".to_string());
        let task2 = Task::new("Task 2".to_string(), "Description 2".to_string());

        repository.tasks.create(&task1).await.unwrap();
        repository.tasks.create(&task2).await.unwrap();

        // Now create a goal with those task IDs
        let mut goal = Goal::new("Goal with tasks".to_string(), "".to_string());
        goal.add_task(task1.id);
        goal.add_task(task2.id);

        let created = service.create(goal).await.unwrap();
        assert_eq!(created.task_ids.len(), 2);
        assert!(created.task_ids.contains(&task1.id));
        assert!(created.task_ids.contains(&task2.id));
    }
}
