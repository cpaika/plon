use plon::domain::task::{Task, TaskStatus, Priority};
use plon::repository::{Repository, database::init_database};
use tempfile::tempdir;
use chrono::Utc;

#[tokio::test]
async fn test_edit_task_title_and_description() {
    // Test editing task title and description
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create initial task
    let mut task = Task::new("Original Title".to_string(), "Original Description".to_string());
    repo.tasks.create(&task).await.unwrap();
    
    // Edit the task
    task.title = "Updated Title".to_string();
    task.description = "This is the new description with more details".to_string();
    task.updated_at = Utc::now();
    repo.tasks.update(&task).await.unwrap();
    
    // Verify changes were saved
    let updated = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.description, "This is the new description with more details");
    assert!(updated.updated_at > updated.created_at);
    
    println!("✅ Task title and description edited successfully");
}

#[tokio::test]
async fn test_change_task_priority() {
    // Test changing task priority
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create low priority task
    let mut task = Task::new("Priority Task".to_string(), "".to_string());
    task.priority = Priority::Low;
    repo.tasks.create(&task).await.unwrap();
    
    // Change to high priority
    task.priority = Priority::High;
    repo.tasks.update(&task).await.unwrap();
    
    let updated = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(updated.priority, Priority::High);
    
    // Change to critical priority
    task.priority = Priority::Critical;
    repo.tasks.update(&task).await.unwrap();
    
    let critical = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(critical.priority, Priority::Critical);
    
    println!("✅ Task priority changed successfully");
}

#[tokio::test]
async fn test_set_due_date() {
    // Test setting and updating due dates
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create task without due date
    let mut task = Task::new("Task with Deadline".to_string(), "".to_string());
    assert!(task.due_date.is_none());
    repo.tasks.create(&task).await.unwrap();
    
    // Set due date
    let due_date = Utc::now() + chrono::Duration::days(7);
    task.due_date = Some(due_date);
    repo.tasks.update(&task).await.unwrap();
    
    let with_due = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert!(with_due.due_date.is_some());
    
    // Clear due date
    task.due_date = None;
    repo.tasks.update(&task).await.unwrap();
    
    let no_due = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert!(no_due.due_date.is_none());
    
    println!("✅ Task due date set and cleared successfully");
}

#[tokio::test]
async fn test_add_remove_tags() {
    // Test adding and removing tags
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create task with no tags
    let mut task = Task::new("Tagged Task".to_string(), "".to_string());
    assert!(task.tags.is_empty());
    repo.tasks.create(&task).await.unwrap();
    
    // Add tags
    task.tags.insert("urgent".to_string());
    task.tags.insert("backend".to_string());
    task.tags.insert("bug".to_string());
    repo.tasks.update(&task).await.unwrap();
    
    let with_tags = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(with_tags.tags.len(), 3);
    assert!(with_tags.tags.contains("urgent"));
    assert!(with_tags.tags.contains("backend"));
    assert!(with_tags.tags.contains("bug"));
    
    // Remove a tag
    task.tags.remove("urgent");
    repo.tasks.update(&task).await.unwrap();
    
    let updated_tags = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(updated_tags.tags.len(), 2);
    assert!(!updated_tags.tags.contains("urgent"));
    
    println!("✅ Tags added and removed successfully");
}

#[tokio::test]
async fn test_estimate_hours() {
    // Test setting estimated and actual hours
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create task
    let mut task = Task::new("Estimated Task".to_string(), "".to_string());
    repo.tasks.create(&task).await.unwrap();
    
    // Set estimated hours
    task.estimated_hours = Some(8.0);
    repo.tasks.update(&task).await.unwrap();
    
    let with_estimate = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(with_estimate.estimated_hours, Some(8.0));
    
    // Update with actual hours - transition through InProgress first
    task.status = TaskStatus::InProgress;
    repo.tasks.update(&task).await.unwrap();
    
    task.actual_hours = Some(10.5);
    task.status = TaskStatus::Done;
    task.completed_at = Some(Utc::now());
    repo.tasks.update(&task).await.unwrap();
    
    let completed = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(completed.estimated_hours, Some(8.0));
    assert_eq!(completed.actual_hours, Some(10.5));
    
    println!("✅ Task hours estimated and tracked successfully");
}

#[tokio::test]
async fn test_task_archiving() {
    // Test archiving and unarchiving tasks
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create active task
    let mut task = Task::new("Task to Archive".to_string(), "".to_string());
    assert!(!task.is_archived);
    repo.tasks.create(&task).await.unwrap();
    
    // Archive the task
    task.is_archived = true;
    repo.tasks.update(&task).await.unwrap();
    
    let archived = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert!(archived.is_archived);
    
    // Unarchive the task
    task.is_archived = false;
    repo.tasks.update(&task).await.unwrap();
    
    let active = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert!(!active.is_archived);
    
    println!("✅ Task archived and unarchived successfully");
}

#[tokio::test]
async fn test_assignee_management() {
    // Test assigning and unassigning tasks
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create unassigned task
    let mut task = Task::new("Unassigned Task".to_string(), "".to_string());
    assert!(task.assignee.is_none());
    repo.tasks.create(&task).await.unwrap();
    
    // Assign to someone
    task.assignee = Some("alice@example.com".to_string());
    repo.tasks.update(&task).await.unwrap();
    
    let assigned = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(assigned.assignee, Some("alice@example.com".to_string()));
    
    // Reassign to someone else
    task.assignee = Some("bob@example.com".to_string());
    repo.tasks.update(&task).await.unwrap();
    
    let reassigned = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(reassigned.assignee, Some("bob@example.com".to_string()));
    
    // Unassign
    task.assignee = None;
    repo.tasks.update(&task).await.unwrap();
    
    let unassigned = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert!(unassigned.assignee.is_none());
    
    println!("✅ Task assignee managed successfully");
}