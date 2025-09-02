#[cfg(test)]
mod task_repository_tests {
    use plon::repository::{Repository, task_repository::TaskFilters};
    use plon::domain::task::{Task, TaskStatus, Priority};
    use sqlx::SqlitePool;
    use uuid::Uuid;
    use std::sync::Arc;
    use chrono::{Utc, Duration};

    async fn setup_test_repository() -> Arc<Repository> {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        Arc::new(Repository::new(pool))
    }

    #[tokio::test]
    async fn test_create_and_retrieve_task() {
        let repo = setup_test_repository().await;
        
        let task = Task::new(
            "Test Task".to_string(),
            "Test Description".to_string()
        );
        let task_id = task.id;
        
        // Create task
        repo.tasks.create(&task).await.unwrap();
        
        // Retrieve task
        let retrieved = repo.tasks.get(task_id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_task = retrieved.unwrap();
        assert_eq!(retrieved_task.title, "Test Task");
        assert_eq!(retrieved_task.description, "Test Description");
        assert_eq!(retrieved_task.status, TaskStatus::Todo);
    }

    #[tokio::test]
    async fn test_update_task() {
        let repo = setup_test_repository().await;
        
        let mut task = Task::new(
            "Original Title".to_string(),
            "Original Description".to_string()
        );
        let task_id = task.id;
        
        repo.tasks.create(&task).await.unwrap();
        
        // Update task
        task.title = "Updated Title".to_string();
        task.description = "Updated Description".to_string();
        task.status = TaskStatus::InProgress;
        task.priority = Priority::High;
        
        repo.tasks.update(&task).await.unwrap();
        
        // Verify update
        let updated = repo.tasks.get(task_id).await.unwrap().unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.description, "Updated Description");
        assert_eq!(updated.status, TaskStatus::InProgress);
        assert_eq!(updated.priority, Priority::High);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let repo = setup_test_repository().await;
        
        let task = Task::new(
            "Task to Delete".to_string(),
            "Will be deleted".to_string()
        );
        let task_id = task.id;
        
        repo.tasks.create(&task).await.unwrap();
        
        // Verify task exists
        assert!(repo.tasks.get(task_id).await.unwrap().is_some());
        
        // Delete task
        repo.tasks.delete(task_id).await.unwrap();
        
        // Verify task is deleted
        assert!(repo.tasks.get(task_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_tasks_with_filters() {
        let repo = setup_test_repository().await;
        
        // Create tasks with different statuses
        let todo_task = Task::new("Todo Task".to_string(), "".to_string());
        let mut in_progress_task = Task::new("In Progress Task".to_string(), "".to_string());
        in_progress_task.status = TaskStatus::InProgress;
        let mut done_task = Task::new("Done Task".to_string(), "".to_string());
        done_task.status = TaskStatus::Done;
        
        repo.tasks.create(&todo_task).await.unwrap();
        repo.tasks.create(&in_progress_task).await.unwrap();
        repo.tasks.create(&done_task).await.unwrap();
        
        // Test filter by status
        let filters = TaskFilters {
            status: Some(TaskStatus::InProgress),
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        };
        
        let filtered_tasks = repo.tasks.list(filters).await.unwrap();
        assert_eq!(filtered_tasks.len(), 1);
        assert_eq!(filtered_tasks[0].title, "In Progress Task");
    }

    #[tokio::test]
    async fn test_overdue_tasks_filter() {
        let repo = setup_test_repository().await;
        
        // Create overdue task
        let mut overdue_task = Task::new("Overdue Task".to_string(), "".to_string());
        overdue_task.due_date = Some(Utc::now() - Duration::days(1));
        
        // Create future task
        let mut future_task = Task::new("Future Task".to_string(), "".to_string());
        future_task.due_date = Some(Utc::now() + Duration::days(1));
        
        repo.tasks.create(&overdue_task).await.unwrap();
        repo.tasks.create(&future_task).await.unwrap();
        
        // Test overdue filter
        let filters = TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: true,
            limit: None,
        };
        
        let overdue_tasks = repo.tasks.list(filters).await.unwrap();
        assert_eq!(overdue_tasks.len(), 1);
        assert_eq!(overdue_tasks[0].title, "Overdue Task");
    }

    #[tokio::test]
    async fn test_task_with_subtasks() {
        let repo = setup_test_repository().await;
        
        let mut task = Task::new("Parent Task".to_string(), "".to_string());
        
        // Add subtasks - note: title is stored as description in DB
        task.subtasks = vec![
            plon::domain::task::SubTask {
                id: Uuid::new_v4(),
                title: "First subtask".to_string(),
                description: "First subtask".to_string(),
                completed: false,
                created_at: Utc::now(),
                completed_at: None,
            },
            plon::domain::task::SubTask {
                id: Uuid::new_v4(),
                title: "Second subtask".to_string(),
                description: "Second subtask".to_string(),
                completed: true,
                created_at: Utc::now(),
                completed_at: Some(Utc::now()),
            },
        ];
        
        let task_id = task.id;
        repo.tasks.create(&task).await.unwrap();
        
        // Retrieve and verify
        let retrieved = repo.tasks.get(task_id).await.unwrap().unwrap();
        assert_eq!(retrieved.subtasks.len(), 2);
        
        // Subtasks might be ordered differently, so check both exist
        let subtask_titles: Vec<String> = retrieved.subtasks.iter().map(|s| s.title.clone()).collect();
        assert!(subtask_titles.contains(&"First subtask".to_string()));
        assert!(subtask_titles.contains(&"Second subtask".to_string()));
        
        // Find specific subtasks to check completion status
        let subtask1 = retrieved.subtasks.iter().find(|s| s.title == "First subtask").unwrap();
        let subtask2 = retrieved.subtasks.iter().find(|s| s.title == "Second subtask").unwrap();
        assert!(!subtask1.completed);
        assert!(subtask2.completed);
    }

    #[tokio::test]
    async fn test_task_with_tags() {
        let repo = setup_test_repository().await;
        
        let mut task = Task::new("Tagged Task".to_string(), "".to_string());
        task.tags.insert("urgent".to_string());
        task.tags.insert("frontend".to_string());
        task.tags.insert("bug".to_string());
        
        let task_id = task.id;
        repo.tasks.create(&task).await.unwrap();
        
        // Retrieve and verify
        let retrieved = repo.tasks.get(task_id).await.unwrap().unwrap();
        assert_eq!(retrieved.tags.len(), 3);
        assert!(retrieved.tags.contains("urgent"));
        assert!(retrieved.tags.contains("frontend"));
        assert!(retrieved.tags.contains("bug"));
    }

    #[tokio::test]
    async fn test_task_position_persistence() {
        let repo = setup_test_repository().await;
        
        let mut task = Task::new("Positioned Task".to_string(), "".to_string());
        task.position.x = 123.45;
        task.position.y = 678.90;
        
        let task_id = task.id;
        repo.tasks.create(&task).await.unwrap();
        
        // Retrieve and verify position
        let retrieved = repo.tasks.get(task_id).await.unwrap().unwrap();
        assert_eq!(retrieved.position.x, 123.45);
        assert_eq!(retrieved.position.y, 678.90);
    }

    #[tokio::test]
    async fn test_task_archival() {
        let repo = setup_test_repository().await;
        
        let mut task = Task::new("Task to Archive".to_string(), "".to_string());
        task.is_archived = false;
        
        let task_id = task.id;
        repo.tasks.create(&task).await.unwrap();
        
        // Archive the task
        task.is_archived = true;
        repo.tasks.update(&task).await.unwrap();
        
        // Verify archival
        let archived = repo.tasks.get(task_id).await.unwrap().unwrap();
        assert!(archived.is_archived);
    }

    #[tokio::test]
    async fn test_bulk_task_operations() {
        let repo = setup_test_repository().await;
        
        // Create multiple tasks
        let tasks: Vec<Task> = (0..10)
            .map(|i| Task::new(
                format!("Task {}", i),
                format!("Description {}", i)
            ))
            .collect();
        
        for task in &tasks {
            repo.tasks.create(task).await.unwrap();
        }
        
        // List all tasks
        let all_tasks = repo.tasks.list(TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        }).await.unwrap();
        
        assert_eq!(all_tasks.len(), 10);
    }

    #[tokio::test]
    async fn test_task_with_parent_relationship() {
        let repo = setup_test_repository().await;
        
        // Create parent task
        let parent = Task::new("Parent Task".to_string(), "".to_string());
        let parent_id = parent.id;
        repo.tasks.create(&parent).await.unwrap();
        
        // Create child task
        let mut child = Task::new("Child Task".to_string(), "".to_string());
        child.parent_task_id = Some(parent_id);
        
        let child_id = child.id;
        repo.tasks.create(&child).await.unwrap();
        
        // Verify relationship
        let retrieved_child = repo.tasks.get(child_id).await.unwrap().unwrap();
        assert_eq!(retrieved_child.parent_task_id, Some(parent_id));
    }

    #[tokio::test]
    async fn test_task_estimated_vs_actual_hours() {
        let repo = setup_test_repository().await;
        
        let mut task = Task::new("Timed Task".to_string(), "".to_string());
        task.estimated_hours = Some(5.0);
        task.actual_hours = Some(7.5);
        
        let task_id = task.id;
        repo.tasks.create(&task).await.unwrap();
        
        // Retrieve and verify
        let retrieved = repo.tasks.get(task_id).await.unwrap().unwrap();
        assert_eq!(retrieved.estimated_hours, Some(5.0));
        assert_eq!(retrieved.actual_hours, Some(7.5));
    }

    #[tokio::test]
    async fn test_task_metadata_persistence() {
        let repo = setup_test_repository().await;
        
        let mut task = Task::new("Task with Metadata".to_string(), "".to_string());
        task.metadata.insert("custom_field".to_string(), "custom_value".to_string());
        task.metadata.insert("priority_score".to_string(), "95".to_string());
        
        let task_id = task.id;
        repo.tasks.create(&task).await.unwrap();
        
        // Retrieve and verify metadata
        let retrieved = repo.tasks.get(task_id).await.unwrap().unwrap();
        assert_eq!(retrieved.metadata.get("custom_field"), Some(&"custom_value".to_string()));
        assert_eq!(retrieved.metadata.get("priority_score"), Some(&"95".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_task_updates() {
        let repo = setup_test_repository().await;
        
        let task = Task::new("Concurrent Task".to_string(), "".to_string());
        let task_id = task.id;
        repo.tasks.create(&task).await.unwrap();
        
        // Simulate concurrent updates
        let repo1 = repo.clone();
        let repo2 = repo.clone();
        
        let handle1 = tokio::spawn(async move {
            let mut task = repo1.tasks.get(task_id).await.unwrap().unwrap();
            task.title = "Updated by Thread 1".to_string();
            repo1.tasks.update(&task).await.unwrap();
        });
        
        let handle2 = tokio::spawn(async move {
            let mut task = repo2.tasks.get(task_id).await.unwrap().unwrap();
            task.description = "Updated by Thread 2".to_string();
            repo2.tasks.update(&task).await.unwrap();
        });
        
        handle1.await.unwrap();
        handle2.await.unwrap();
        
        // Verify final state
        let final_task = repo.tasks.get(task_id).await.unwrap().unwrap();
        assert!(final_task.title.contains("Updated") || final_task.description.contains("Updated"));
    }
}