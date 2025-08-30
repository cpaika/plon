use plon::domain::task::{Priority, Task, TaskStatus};
use plon::repository::{Repository, database};
use plon::services::TaskService;
use std::sync::Arc;
use tempfile::NamedTempFile;
use uuid::Uuid;

#[tokio::test]
async fn test_task_persistence() {
    // Create a temporary database file
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    // Test task data
    let task_id = Uuid::new_v4();
    let task_title = "Test Task".to_string();
    let task_description = "This task should persist".to_string();

    // Scope 1: Create and save a task
    {
        let pool = database::init_database(db_path).await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository.clone());

        let mut task = Task::new(task_title.clone(), task_description.clone());
        task.id = task_id; // Use a fixed ID so we can verify it later
        task.priority = Priority::High;
        task.status = TaskStatus::InProgress;

        // Save the task
        service.create(task.clone()).await.unwrap();

        // Verify it was saved
        let loaded_task = service.get(task_id).await.unwrap();
        assert!(
            loaded_task.is_some(),
            "Task should be found immediately after creation"
        );
        assert_eq!(loaded_task.unwrap().title, task_title);
    } // Pool is dropped here, simulating application shutdown

    // Scope 2: Reopen database and verify task persists
    {
        let pool = database::init_database(db_path).await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository.clone());

        // Load all tasks
        let all_tasks = service.list_all().await.unwrap();
        assert!(!all_tasks.is_empty(), "Should have at least one task");

        // Find our specific task
        let loaded_task = service.get(task_id).await.unwrap();
        assert!(
            loaded_task.is_some(),
            "Task should persist after database reconnection"
        );

        let task = loaded_task.unwrap();
        assert_eq!(task.title, task_title);
        assert_eq!(task.description, task_description);
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.status, TaskStatus::InProgress);
    }
}

#[tokio::test]
async fn test_multiple_tasks_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let task_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

    // Create multiple tasks
    {
        let pool = database::init_database(db_path).await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository.clone());

        for (i, &id) in task_ids.iter().enumerate() {
            let mut task = Task::new(format!("Task {}", i), format!("Description for task {}", i));
            task.id = id;
            service.create(task).await.unwrap();
        }
    }

    // Verify all tasks persist
    {
        let pool = database::init_database(db_path).await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository.clone());

        let all_tasks = service.list_all().await.unwrap();
        assert_eq!(all_tasks.len(), 5, "Should have exactly 5 tasks");

        // Verify each task exists
        for id in task_ids {
            let task = service.get(id).await.unwrap();
            assert!(task.is_some(), "Task with id {} should exist", id);
        }
    }
}

#[tokio::test]
async fn test_task_update_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let task_id = Uuid::new_v4();

    // Create and update a task
    {
        let pool = database::init_database(db_path).await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository.clone());

        let mut task = Task::new(
            "Original Title".to_string(),
            "Original Description".to_string(),
        );
        task.id = task_id;
        service.create(task.clone()).await.unwrap();

        // Update the task
        task.title = "Updated Title".to_string();
        task.status = TaskStatus::Done;
        service.update(task).await.unwrap();
    }

    // Verify updates persist
    {
        let pool = database::init_database(db_path).await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository.clone());

        let task = service.get(task_id).await.unwrap().unwrap();
        assert_eq!(task.title, "Updated Title");
        assert_eq!(task.status, TaskStatus::Done);
    }
}

#[tokio::test]
async fn test_task_deletion_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let task_id = Uuid::new_v4();

    // Create and delete a task
    {
        let pool = database::init_database(db_path).await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository.clone());

        let mut task = Task::new("To Delete".to_string(), "Will be deleted".to_string());
        task.id = task_id;
        service.create(task).await.unwrap();

        // Delete the task
        let deleted = service.delete(task_id).await.unwrap();
        assert!(deleted, "Task should be deleted");
    }

    // Verify deletion persists
    {
        let pool = database::init_database(db_path).await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository.clone());

        let task = service.get(task_id).await.unwrap();
        assert!(task.is_none(), "Deleted task should not exist");
    }
}
