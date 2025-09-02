use plon::domain::task::{Task, TaskStatus, Priority};
use plon::domain::goal::Goal;
use plon::repository::{Repository, database::init_database};
use plon::repository::task_repository::TaskFilters;
use tempfile::tempdir;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_filter_by_status() {
    // Test filtering tasks by status
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create tasks with different statuses
    let mut todo_task = Task::new("Todo Task".to_string(), "".to_string());
    todo_task.status = TaskStatus::Todo;
    
    let mut in_progress = Task::new("In Progress".to_string(), "".to_string());
    in_progress.status = TaskStatus::InProgress;
    
    let mut done_task = Task::new("Done Task".to_string(), "".to_string());
    done_task.status = TaskStatus::Done;
    done_task.completed_at = Some(Utc::now());
    
    let mut blocked_task = Task::new("Blocked Task".to_string(), "".to_string());
    blocked_task.status = TaskStatus::Blocked;
    
    repo.tasks.create(&todo_task).await.unwrap();
    repo.tasks.create(&in_progress).await.unwrap();
    repo.tasks.create(&done_task).await.unwrap();
    repo.tasks.create(&blocked_task).await.unwrap();
    
    // Filter by InProgress status
    let mut filters = TaskFilters::default();
    filters.status = Some(TaskStatus::InProgress);
    let results = repo.tasks.list(filters).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, TaskStatus::InProgress);
    assert_eq!(results[0].title, "In Progress");
    
    // Filter by Done status
    let mut filters = TaskFilters::default();
    filters.status = Some(TaskStatus::Done);
    let done_results = repo.tasks.list(filters).await.unwrap();
    
    assert_eq!(done_results.len(), 1);
    assert_eq!(done_results[0].status, TaskStatus::Done);
    assert!(done_results[0].completed_at.is_some());
    
    // Filter by Blocked status
    let mut filters = TaskFilters::default();
    filters.status = Some(TaskStatus::Blocked);
    let blocked_results = repo.tasks.list(filters).await.unwrap();
    
    assert_eq!(blocked_results.len(), 1);
    assert_eq!(blocked_results[0].status, TaskStatus::Blocked);
    
    println!("✅ Tasks filtered by status successfully");
}

#[tokio::test]
async fn test_filter_overdue_tasks() {
    // Test filtering overdue tasks
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create overdue task
    let mut overdue_task = Task::new("Overdue Task".to_string(), "Should have been done yesterday".to_string());
    overdue_task.due_date = Some(Utc::now() - chrono::Duration::days(2));
    overdue_task.status = TaskStatus::InProgress;
    
    // Create future task
    let mut future_task = Task::new("Future Task".to_string(), "Due next week".to_string());
    future_task.due_date = Some(Utc::now() + chrono::Duration::days(7));
    future_task.status = TaskStatus::Todo;
    
    // Create task with no due date
    let no_due = Task::new("No Due Date".to_string(), "".to_string());
    
    repo.tasks.create(&overdue_task).await.unwrap();
    repo.tasks.create(&future_task).await.unwrap();
    repo.tasks.create(&no_due).await.unwrap();
    
    // Filter overdue tasks
    let mut filters = TaskFilters::default();
    filters.overdue = true;
    let overdue_results = repo.tasks.list(filters).await.unwrap();
    
    // Should find the overdue task
    assert!(overdue_results.iter().any(|t| t.title == "Overdue Task"));
    
    // Filter non-overdue tasks
    let mut filters = TaskFilters::default();
    filters.overdue = false;
    let not_overdue = repo.tasks.list(filters).await.unwrap();
    
    // Should include future and no-due-date tasks
    assert!(not_overdue.iter().any(|t| t.title == "Future Task"));
    assert!(not_overdue.iter().any(|t| t.title == "No Due Date"));
    
    println!("✅ Overdue tasks filtered successfully");
}

#[tokio::test]
async fn test_filter_with_limit() {
    // Test limiting the number of results
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create 10 tasks
    for i in 0..10 {
        let task = Task::new(format!("Task {}", i), format!("Description {}", i));
        repo.tasks.create(&task).await.unwrap();
    }
    
    // Get first 5 tasks
    let mut filters = TaskFilters::default();
    filters.limit = Some(5);
    let limited = repo.tasks.list(filters).await.unwrap();
    
    assert_eq!(limited.len(), 5);
    
    // Get all tasks (no limit)
    let all = repo.tasks.list(TaskFilters::default()).await.unwrap();
    assert_eq!(all.len(), 10);
    
    println!("✅ Task limit filter works correctly");
}

#[tokio::test]
async fn test_filter_by_goal() {
    // Test filtering tasks by goal
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create goals first
    let goal = Goal::new("Main Goal".to_string(), "Test goal".to_string());
    let goal_id = goal.id;
    repo.goals.create(&goal).await.unwrap();
    
    let other_goal = Goal::new("Other Goal".to_string(), "Another goal".to_string());
    let other_goal_id = other_goal.id;
    repo.goals.create(&other_goal).await.unwrap();
    
    // Create tasks with different goals
    let mut goal_task1 = Task::new("Goal Task 1".to_string(), "".to_string());
    goal_task1.goal_id = Some(goal_id);
    
    let mut goal_task2 = Task::new("Goal Task 2".to_string(), "".to_string());
    goal_task2.goal_id = Some(goal_id);
    
    let mut other_goal_task = Task::new("Other Goal Task".to_string(), "".to_string());
    other_goal_task.goal_id = Some(other_goal_id);
    
    let no_goal_task = Task::new("No Goal Task".to_string(), "".to_string());
    
    repo.tasks.create(&goal_task1).await.unwrap();
    repo.tasks.create(&goal_task2).await.unwrap();
    repo.tasks.create(&other_goal_task).await.unwrap();
    repo.tasks.create(&no_goal_task).await.unwrap();
    
    // Filter by specific goal
    let mut filters = TaskFilters::default();
    filters.goal_id = Some(goal_id);
    let goal_results = repo.tasks.list(filters).await.unwrap();
    
    assert_eq!(goal_results.len(), 2);
    assert!(goal_results.iter().all(|t| t.goal_id == Some(goal_id)));
    
    println!("✅ Tasks filtered by goal successfully");
}

#[tokio::test]
async fn test_combined_filters() {
    // Test combining multiple filters
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create goal first
    let goal = Goal::new("Test Goal".to_string(), "For combined filter test".to_string());
    let goal_id = goal.id;
    repo.goals.create(&goal).await.unwrap();
    
    // Create various tasks
    let mut in_progress_goal = Task::new("In Progress Goal Task".to_string(), "".to_string());
    in_progress_goal.status = TaskStatus::InProgress;
    in_progress_goal.goal_id = Some(goal_id);
    
    let mut todo_goal = Task::new("Todo Goal Task".to_string(), "".to_string());
    todo_goal.status = TaskStatus::Todo;
    todo_goal.goal_id = Some(goal_id);
    
    let mut in_progress_no_goal = Task::new("In Progress No Goal".to_string(), "".to_string());
    in_progress_no_goal.status = TaskStatus::InProgress;
    
    repo.tasks.create(&in_progress_goal).await.unwrap();
    repo.tasks.create(&todo_goal).await.unwrap();
    repo.tasks.create(&in_progress_no_goal).await.unwrap();
    
    // Filter by InProgress status AND specific goal
    let mut filters = TaskFilters::default();
    filters.status = Some(TaskStatus::InProgress);
    filters.goal_id = Some(goal_id);
    let results = repo.tasks.list(filters).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "In Progress Goal Task");
    assert_eq!(results[0].status, TaskStatus::InProgress);
    assert_eq!(results[0].goal_id, Some(goal_id));
    
    println!("✅ Combined filters work correctly");
}

#[tokio::test]
async fn test_overdue_with_status_filter() {
    // Test combining overdue and status filters
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create overdue InProgress task
    let mut overdue_in_progress = Task::new("Overdue InProgress".to_string(), "".to_string());
    overdue_in_progress.status = TaskStatus::InProgress;
    overdue_in_progress.due_date = Some(Utc::now() - chrono::Duration::days(1));
    
    // Create overdue Todo task
    let mut overdue_todo = Task::new("Overdue Todo".to_string(), "".to_string());
    overdue_todo.status = TaskStatus::Todo;
    overdue_todo.due_date = Some(Utc::now() - chrono::Duration::days(2));
    
    // Create non-overdue InProgress task
    let mut future_in_progress = Task::new("Future InProgress".to_string(), "".to_string());
    future_in_progress.status = TaskStatus::InProgress;
    future_in_progress.due_date = Some(Utc::now() + chrono::Duration::days(3));
    
    repo.tasks.create(&overdue_in_progress).await.unwrap();
    repo.tasks.create(&overdue_todo).await.unwrap();
    repo.tasks.create(&future_in_progress).await.unwrap();
    
    // Filter overdue InProgress tasks
    let mut filters = TaskFilters::default();
    filters.overdue = true;
    filters.status = Some(TaskStatus::InProgress);
    let results = repo.tasks.list(filters).await.unwrap();
    
    // Should only find overdue InProgress task
    assert!(results.iter().any(|t| t.title == "Overdue InProgress"));
    assert!(!results.iter().any(|t| t.title == "Overdue Todo"));
    assert!(!results.iter().any(|t| t.title == "Future InProgress"));
    
    println!("✅ Overdue with status filter works correctly");
}