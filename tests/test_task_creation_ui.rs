use plon::domain::task::{Task, TaskStatus, Priority};
use plon::repository::{Repository, database::init_database};
use tempfile::tempdir;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_create_task_from_ui() {
    // Test that we can create a new task from the UI
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Simulate creating a new task from the UI
    let task_title = "New Task from UI";
    let task_description = "This task was created from the user interface";
    
    let mut new_task = Task::new(task_title.to_string(), task_description.to_string());
    new_task.status = TaskStatus::Todo;
    new_task.priority = Priority::Medium;
    new_task.set_position(100.0, 200.0); // Position on the map
    
    // Save the task
    repo.tasks.create(&new_task).await.unwrap();
    
    // Verify it was saved correctly
    let saved_task = repo.tasks.get(new_task.id).await.unwrap().unwrap();
    assert_eq!(saved_task.title, task_title);
    assert_eq!(saved_task.description, task_description);
    assert_eq!(saved_task.status, TaskStatus::Todo);
    assert_eq!(saved_task.priority, Priority::Medium);
    assert_eq!(saved_task.position.x, 100.0);
    assert_eq!(saved_task.position.y, 200.0);
    
    println!("✅ Task created successfully from UI");
}

#[tokio::test]
async fn test_update_task_status_from_ui() {
    // Test dragging a task to change its status
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create initial task
    let mut task = Task::new("Task to Update".to_string(), "".to_string());
    task.status = TaskStatus::Todo;
    repo.tasks.create(&task).await.unwrap();
    
    // Simulate dragging to InProgress column
    task.status = TaskStatus::InProgress;
    repo.tasks.update(&task).await.unwrap();
    
    // Verify status change
    let updated = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(updated.status, TaskStatus::InProgress);
    
    // Simulate dragging to Done column
    task.status = TaskStatus::Done;
    task.completed_at = Some(Utc::now());
    repo.tasks.update(&task).await.unwrap();
    
    let completed = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(completed.status, TaskStatus::Done);
    assert!(completed.completed_at.is_some());
    
    println!("✅ Task status updated successfully via UI drag");
}

#[tokio::test]
async fn test_delete_task_from_ui() {
    // Test deleting a task from the UI
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create a task
    let task = Task::new("Task to Delete".to_string(), "".to_string());
    let task_id = task.id;
    repo.tasks.create(&task).await.unwrap();
    
    // Verify it exists
    let exists = repo.tasks.get(task_id).await.unwrap();
    assert!(exists.is_some());
    
    // Simulate delete action from UI
    repo.tasks.delete(task_id).await.unwrap();
    
    // Verify it's deleted
    let deleted = repo.tasks.get(task_id).await.unwrap();
    assert!(deleted.is_none());
    
    println!("✅ Task deleted successfully from UI");
}

#[tokio::test]
async fn test_task_position_update() {
    // Test updating task position on the map
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create a task at initial position
    let mut task = Task::new("Movable Task".to_string(), "".to_string());
    task.set_position(50.0, 50.0);
    repo.tasks.create(&task).await.unwrap();
    
    // Simulate dragging the task to a new position
    task.set_position(150.0, 250.0);
    repo.tasks.update(&task).await.unwrap();
    
    // Verify position was updated
    let moved = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(moved.position.x, 150.0);
    assert_eq!(moved.position.y, 250.0);
    
    println!("✅ Task position updated successfully");
}

#[tokio::test]
async fn test_bulk_task_operations() {
    // Test selecting and operating on multiple tasks
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create multiple tasks
    let mut task_ids = Vec::new();
    for i in 0..5 {
        let mut task = Task::new(format!("Bulk Task {}", i), "".to_string());
        task.status = TaskStatus::Todo;
        task.priority = Priority::Low;
        task.set_position((i as f64) * 50.0, 100.0);
        let id = task.id;
        repo.tasks.create(&task).await.unwrap();
        task_ids.push(id);
    }
    
    // Simulate bulk status update
    for id in &task_ids {
        let mut task = repo.tasks.get(*id).await.unwrap().unwrap();
        task.status = TaskStatus::InProgress;
        task.priority = Priority::High;
        repo.tasks.update(&task).await.unwrap();
    }
    
    // Verify all tasks were updated
    for id in &task_ids {
        let task = repo.tasks.get(*id).await.unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
        assert_eq!(task.priority, Priority::High);
    }
    
    println!("✅ Bulk task operations completed successfully");
}