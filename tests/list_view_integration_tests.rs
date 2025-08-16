use plon::domain::task::{Task, TaskStatus, Priority};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::repository::task_repository::TaskFilters;
use plon::services::TaskService;
use std::sync::Arc;
use chrono::Utc;
use futures::future;

#[tokio::test]
async fn test_list_view_task_filtering() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository.clone());
    
    // Create tasks with different titles
    let tasks = vec![
        ("Important Meeting", "meeting"),
        ("Code Review", "review"),
        ("Important Bug Fix", "bug"),
        ("Documentation Update", "docs"),
        ("Important Feature", "feature"),
    ];
    
    for (title, _) in &tasks {
        let task = Task::new(title.to_string(), "".to_string());
        service.create(task).await.unwrap();
    }
    
    // Test filtering by text
    let all_tasks = service.list_all().await.unwrap();
    
    // Filter for "Important"
    let important_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.title.to_lowercase().contains("important"))
        .collect();
    assert_eq!(important_tasks.len(), 3);
    
    // Filter for "Review"
    let review_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.title.to_lowercase().contains("review"))
        .collect();
    assert_eq!(review_tasks.len(), 1);
    assert_eq!(review_tasks[0].title, "Code Review");
}

#[tokio::test]
async fn test_list_view_status_filtering() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository.clone());
    
    // Create tasks with different statuses
    let statuses = vec![
        ("Todo Task 1", TaskStatus::Todo),
        ("Todo Task 2", TaskStatus::Todo),
        ("In Progress Task", TaskStatus::InProgress),
        ("Done Task", TaskStatus::Done),
        ("Blocked Task", TaskStatus::Blocked),
    ];
    
    for (title, status) in statuses {
        let mut task = Task::new(title.to_string(), "".to_string());
        task.status = status;
        service.create(task).await.unwrap();
    }
    
    // Test filtering by status
    let todo_filter = TaskFilters {
        status: Some(TaskStatus::Todo),
        ..Default::default()
    };
    let todo_tasks = repository.tasks.list(todo_filter).await.unwrap();
    assert_eq!(todo_tasks.len(), 2);
    
    let done_filter = TaskFilters {
        status: Some(TaskStatus::Done),
        ..Default::default()
    };
    let done_tasks = repository.tasks.list(done_filter).await.unwrap();
    assert_eq!(done_tasks.len(), 1);
}

#[tokio::test]
async fn test_list_view_inline_editing() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create a task
    let original_task = Task::new("Original Title".to_string(), "Original Description".to_string());
    let created = service.create(original_task).await.unwrap();
    
    // Simulate inline editing
    let mut edited_task = created.clone();
    edited_task.title = "Edited Title".to_string();
    edited_task.description = "Edited Description".to_string();
    
    service.update(edited_task).await.unwrap();
    
    // Verify changes
    let fetched = service.get(created.id).await.unwrap().unwrap();
    assert_eq!(fetched.title, "Edited Title");
    assert_eq!(fetched.description, "Edited Description");
}

#[tokio::test]
async fn test_list_view_priority_display() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks with different priorities
    let priorities = vec![
        ("Critical Task", Priority::Critical),
        ("High Priority", Priority::High),
        ("Medium Priority", Priority::Medium),
        ("Low Priority", Priority::Low),
    ];
    
    for (title, priority) in priorities {
        let mut task = Task::new(title.to_string(), "".to_string());
        task.priority = priority;
        service.create(task).await.unwrap();
    }
    
    // Verify all tasks are created with correct priorities
    let all_tasks = service.list_all().await.unwrap();
    assert_eq!(all_tasks.len(), 4);
    
    let critical_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.priority == Priority::Critical)
        .collect();
    assert_eq!(critical_tasks.len(), 1);
    assert_eq!(critical_tasks[0].title, "Critical Task");
}

#[tokio::test]
async fn test_list_view_overdue_highlighting() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create overdue task
    let mut overdue_task = Task::new("Overdue Task".to_string(), "".to_string());
    overdue_task.due_date = Some(Utc::now() - chrono::Duration::days(2));
    service.create(overdue_task.clone()).await.unwrap();
    
    // Create future task
    let mut future_task = Task::new("Future Task".to_string(), "".to_string());
    future_task.due_date = Some(Utc::now() + chrono::Duration::days(7));
    service.create(future_task.clone()).await.unwrap();
    
    // Create task with no due date
    let no_due_task = Task::new("No Due Date".to_string(), "".to_string());
    service.create(no_due_task).await.unwrap();
    
    // Check overdue status
    let all_tasks = service.list_all().await.unwrap();
    
    let overdue_count = all_tasks.iter().filter(|t| t.is_overdue()).count();
    assert_eq!(overdue_count, 1);
    
    let overdue = all_tasks.iter().find(|t| t.is_overdue()).unwrap();
    assert_eq!(overdue.title, "Overdue Task");
}

#[tokio::test]
async fn test_list_view_subtask_progress() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create task with subtasks
    let mut task = Task::new("Main Task".to_string(), "".to_string());
    let sub1_id = task.add_subtask("Subtask 1".to_string());
    let sub2_id = task.add_subtask("Subtask 2".to_string());
    let sub3_id = task.add_subtask("Subtask 3".to_string());
    
    let created = service.create(task).await.unwrap();
    
    // Verify initial progress
    assert_eq!(created.subtask_progress(), (0, 3));
    
    // Complete some subtasks
    let mut updated = created.clone();
    updated.complete_subtask(sub1_id).unwrap();
    updated.complete_subtask(sub3_id).unwrap();
    
    service.update(updated.clone()).await.unwrap();
    
    // Verify progress
    assert_eq!(updated.subtask_progress(), (2, 3));
}

#[tokio::test]
async fn test_list_view_sorting() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks with different attributes
    let mut task1 = Task::new("Alpha Task".to_string(), "".to_string());
    task1.priority = Priority::Low;
    task1.due_date = Some(Utc::now() + chrono::Duration::days(3));
    
    let mut task2 = Task::new("Beta Task".to_string(), "".to_string());
    task2.priority = Priority::Critical;
    task2.due_date = Some(Utc::now() + chrono::Duration::days(1));
    
    let mut task3 = Task::new("Charlie Task".to_string(), "".to_string());
    task3.priority = Priority::High;
    task3.due_date = Some(Utc::now() + chrono::Duration::days(2));
    
    service.create(task1).await.unwrap();
    service.create(task2).await.unwrap();
    service.create(task3).await.unwrap();
    
    let mut all_tasks = service.list_all().await.unwrap();
    
    // Sort by title
    all_tasks.sort_by(|a, b| a.title.cmp(&b.title));
    assert_eq!(all_tasks[0].title, "Alpha Task");
    assert_eq!(all_tasks[1].title, "Beta Task");
    assert_eq!(all_tasks[2].title, "Charlie Task");
    
    // Sort by priority
    all_tasks.sort_by(|a, b| b.priority.cmp(&a.priority)); // Reverse for priority (Critical first)
    assert_eq!(all_tasks[0].title, "Beta Task"); // Critical
    assert_eq!(all_tasks[1].title, "Charlie Task"); // High
    assert_eq!(all_tasks[2].title, "Alpha Task"); // Low
    
    // Sort by due date
    all_tasks.sort_by(|a, b| a.due_date.cmp(&b.due_date));
    assert_eq!(all_tasks[0].title, "Beta Task"); // Due in 1 day
    assert_eq!(all_tasks[1].title, "Charlie Task"); // Due in 2 days
    assert_eq!(all_tasks[2].title, "Alpha Task"); // Due in 3 days
}

#[tokio::test]
async fn test_list_view_batch_operations() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create multiple tasks
    let task_ids: Vec<_> = (0..5)
        .map(|i| {
            let task = Task::new(format!("Batch Task {}", i), "".to_string());
            let handle = service.clone();
            tokio::spawn(async move {
                handle.create(task).await.unwrap().id
            })
        })
        .collect();
    
    let task_ids: Vec<_> = future::join_all(task_ids)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Batch update status
    for id in &task_ids[0..3] {
        let task = service.get(*id).await.unwrap().unwrap();
        let mut updated = task.clone();
        updated.status = TaskStatus::Done;
        service.update(updated).await.unwrap();
    }
    
    // Verify batch update
    let all_tasks = service.list_all().await.unwrap();
    let done_count = all_tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
    assert_eq!(done_count, 3);
}

#[tokio::test]
async fn test_list_view_search_performance() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create 500 tasks
    for i in 0..500 {
        let task = Task::new(
            format!("Performance Task {}", i),
            format!("Description for task {}", i)
        );
        service.create(task).await.unwrap();
    }
    
    // Test search performance
    let start = std::time::Instant::now();
    let all_tasks = service.list_all().await.unwrap();
    
    // Filter tasks containing "5"
    let filtered: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.title.contains("5"))
        .collect();
    
    let duration = start.elapsed();
    
    // Should complete quickly even with 500 tasks
    assert!(duration.as_millis() < 50);
    assert!(filtered.len() > 0);
}

#[tokio::test]
async fn test_list_view_pagination() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository.clone());
    
    // Create 50 tasks
    for i in 0..50 {
        let task = Task::new(format!("Page Task {}", i), "".to_string());
        service.create(task).await.unwrap();
    }
    
    // Simulate pagination
    let page_size = 10;
    let all_tasks = service.list_all().await.unwrap();
    
    // Page 1
    let page1: Vec<_> = all_tasks.iter().take(page_size).collect();
    assert_eq!(page1.len(), 10);
    
    // Page 2
    let page2: Vec<_> = all_tasks.iter().skip(page_size).take(page_size).collect();
    assert_eq!(page2.len(), 10);
    
    // Last page
    let last_page: Vec<_> = all_tasks.iter().skip(40).collect();
    assert_eq!(last_page.len(), 10);
}