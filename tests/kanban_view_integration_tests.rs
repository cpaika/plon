use plon::domain::task::{Task, TaskStatus, Priority};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::services::TaskService;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::test]
async fn test_kanban_column_distribution() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks for each column
    let columns = vec![
        ("Todo Task 1", TaskStatus::Todo),
        ("Todo Task 2", TaskStatus::Todo),
        ("In Progress Task 1", TaskStatus::InProgress),
        ("In Progress Task 2", TaskStatus::InProgress),
        ("Review Task", TaskStatus::Review),
        ("Done Task 1", TaskStatus::Done),
        ("Done Task 2", TaskStatus::Done),
        ("Done Task 3", TaskStatus::Done),
    ];
    
    for (title, status) in columns {
        let mut task = Task::new(title.to_string(), "".to_string());
        task.status = status;
        service.create(task).await.unwrap();
    }
    
    // Verify column distribution
    let all_tasks = service.list_all().await.unwrap();
    
    let mut status_counts = HashMap::new();
    for task in &all_tasks {
        *status_counts.entry(task.status).or_insert(0) += 1;
    }
    
    assert_eq!(status_counts.get(&TaskStatus::Todo), Some(&2));
    assert_eq!(status_counts.get(&TaskStatus::InProgress), Some(&2));
    assert_eq!(status_counts.get(&TaskStatus::Review), Some(&1));
    assert_eq!(status_counts.get(&TaskStatus::Done), Some(&3));
}

#[tokio::test]
async fn test_kanban_drag_and_drop() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create a task in Todo column
    let mut task = Task::new("Draggable Task".to_string(), "".to_string());
    task.status = TaskStatus::Todo;
    let created = service.create(task).await.unwrap();
    
    // Simulate drag to In Progress
    let mut moved_task = created.clone();
    moved_task.update_status(TaskStatus::InProgress);
    service.update(moved_task.clone()).await.unwrap();
    
    // Verify task moved
    let fetched = service.get(created.id).await.unwrap().unwrap();
    assert_eq!(fetched.status, TaskStatus::InProgress);
    
    // Simulate drag to Done
    let mut done_task = fetched.clone();
    done_task.update_status(TaskStatus::Done);
    service.update(done_task).await.unwrap();
    
    // Verify final status
    let final_task = service.get(created.id).await.unwrap().unwrap();
    assert_eq!(final_task.status, TaskStatus::Done);
    assert!(final_task.completed_at.is_some());
}

#[tokio::test]
async fn test_kanban_wip_limits() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Define WIP limits
    let wip_limits = HashMap::from([
        (TaskStatus::InProgress, 3),
        (TaskStatus::Review, 2),
    ]);
    
    // Create tasks up to and over WIP limit
    for i in 0..5 {
        let mut task = Task::new(format!("Task {}", i), "".to_string());
        task.status = TaskStatus::InProgress;
        service.create(task).await.unwrap();
    }
    
    // Check WIP limit violation
    let all_tasks = service.list_all().await.unwrap();
    let in_progress_count = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .count();
    
    let wip_limit = wip_limits.get(&TaskStatus::InProgress).unwrap_or(&0);
    let is_over_limit = in_progress_count > *wip_limit;
    
    assert!(is_over_limit);
    assert_eq!(in_progress_count, 5);
}

#[tokio::test]
async fn test_kanban_card_details() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create task with full details
    let mut task = Task::new("Detailed Task".to_string(), "Task with all details".to_string());
    task.priority = Priority::High;
    task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(3));
    task.add_subtask("Subtask 1".to_string());
    task.add_subtask("Subtask 2".to_string());
    task.add_tag("frontend".to_string());
    task.add_tag("urgent".to_string());
    task.estimated_hours = Some(8.0);
    
    let created = service.create(task).await.unwrap();
    
    // Verify all details are preserved
    assert_eq!(created.priority, Priority::High);
    assert!(created.due_date.is_some());
    assert_eq!(created.subtasks.len(), 2);
    assert_eq!(created.tags.len(), 2);
    assert_eq!(created.estimated_hours, Some(8.0));
    assert_eq!(created.subtask_progress(), (0, 2));
}

#[tokio::test]
async fn test_kanban_column_customization() {
    // Test custom column configuration
    #[derive(Debug, Clone, PartialEq)]
    struct KanbanColumn {
        title: String,
        status: TaskStatus,
        color: (u8, u8, u8),
        wip_limit: Option<usize>,
    }
    
    let custom_columns = vec![
        KanbanColumn {
            title: "Backlog".to_string(),
            status: TaskStatus::Todo,
            color: (128, 128, 128),
            wip_limit: None,
        },
        KanbanColumn {
            title: "Development".to_string(),
            status: TaskStatus::InProgress,
            color: (100, 150, 255),
            wip_limit: Some(3),
        },
        KanbanColumn {
            title: "Testing".to_string(),
            status: TaskStatus::Review,
            color: (255, 200, 100),
            wip_limit: Some(2),
        },
        KanbanColumn {
            title: "Deployed".to_string(),
            status: TaskStatus::Done,
            color: (100, 255, 100),
            wip_limit: None,
        },
    ];
    
    assert_eq!(custom_columns.len(), 4);
    assert_eq!(custom_columns[1].wip_limit, Some(3));
}

#[tokio::test]
async fn test_kanban_swimlanes() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks with different priorities (swimlanes)
    let swimlanes = vec![
        ("Critical Bug", Priority::Critical),
        ("High Priority Feature", Priority::High),
        ("Medium Task", Priority::Medium),
        ("Low Priority Cleanup", Priority::Low),
    ];
    
    for (title, priority) in swimlanes {
        let mut task = Task::new(title.to_string(), "".to_string());
        task.priority = priority;
        task.status = TaskStatus::InProgress;
        service.create(task).await.unwrap();
    }
    
    // Group tasks by priority (swimlane)
    let all_tasks = service.list_all().await.unwrap();
    let mut priority_groups: HashMap<Priority, Vec<Task>> = HashMap::new();
    
    for task in all_tasks {
        priority_groups.entry(task.priority).or_insert(Vec::new()).push(task);
    }
    
    assert_eq!(priority_groups.get(&Priority::Critical).unwrap().len(), 1);
    assert_eq!(priority_groups.get(&Priority::High).unwrap().len(), 1);
    assert_eq!(priority_groups.get(&Priority::Medium).unwrap().len(), 1);
    assert_eq!(priority_groups.get(&Priority::Low).unwrap().len(), 1);
}

#[tokio::test]
async fn test_kanban_quick_add() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Simulate quick add in each column
    let columns = vec![
        TaskStatus::Todo,
        TaskStatus::InProgress,
        TaskStatus::Review,
        TaskStatus::Done,
    ];
    
    for (i, status) in columns.iter().enumerate() {
        let mut task = Task::new(format!("Quick Add {}", i), "".to_string());
        task.status = *status;
        service.create(task).await.unwrap();
    }
    
    // Verify tasks were added to correct columns
    let all_tasks = service.list_all().await.unwrap();
    
    for status in columns {
        let column_tasks: Vec<_> = all_tasks
            .iter()
            .filter(|t| t.status == status)
            .collect();
        assert!(column_tasks.len() >= 1);
    }
}

#[tokio::test]
async fn test_kanban_filtering() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks with tags
    let tasks_with_tags = vec![
        ("Frontend Task", vec!["frontend", "ui"]),
        ("Backend Task", vec!["backend", "api"]),
        ("Full Stack Task", vec!["frontend", "backend"]),
        ("DevOps Task", vec!["devops", "ci"]),
    ];
    
    for (title, tags) in tasks_with_tags {
        let mut task = Task::new(title.to_string(), "".to_string());
        for tag in tags {
            task.add_tag(tag.to_string());
        }
        service.create(task).await.unwrap();
    }
    
    // Filter by tag
    let all_tasks = service.list_all().await.unwrap();
    
    let frontend_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.tags.contains(&"frontend".to_string()))
        .collect();
    assert_eq!(frontend_tasks.len(), 2);
    
    let backend_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.tags.contains(&"backend".to_string()))
        .collect();
    assert_eq!(backend_tasks.len(), 2);
}

#[tokio::test]
async fn test_kanban_column_scroll() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create many tasks in one column to test scrolling
    for i in 0..20 {
        let mut task = Task::new(format!("Todo Task {}", i), "".to_string());
        task.status = TaskStatus::Todo;
        service.create(task).await.unwrap();
    }
    
    // Verify column has many tasks
    let all_tasks = service.list_all().await.unwrap();
    let todo_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Todo)
        .collect();
    
    assert_eq!(todo_tasks.len(), 20);
    
    // Simulate viewport with limited height
    let viewport_height = 600.0;
    let card_height = 80.0;
    let visible_cards = (viewport_height / card_height) as usize;
    
    assert!(todo_tasks.len() > visible_cards); // Scrolling needed
}

#[tokio::test]
async fn test_kanban_blocked_indicator() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create blocked tasks
    let mut blocked_task = Task::new("Blocked Task".to_string(), "Waiting for API".to_string());
    blocked_task.status = TaskStatus::Blocked;
    blocked_task.add_tag("blocked".to_string());
    
    let mut normal_task = Task::new("Normal Task".to_string(), "".to_string());
    normal_task.status = TaskStatus::InProgress;
    
    service.create(blocked_task).await.unwrap();
    service.create(normal_task).await.unwrap();
    
    // Check blocked status
    let all_tasks = service.list_all().await.unwrap();
    
    let blocked_count = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Blocked)
        .count();
    assert_eq!(blocked_count, 1);
    
    let blocked = all_tasks
        .iter()
        .find(|t| t.status == TaskStatus::Blocked)
        .unwrap();
    assert!(blocked.tags.contains(&"blocked".to_string()));
}

#[tokio::test]
async fn test_kanban_performance_with_many_cards() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create 200 tasks distributed across columns
    let statuses = vec![
        TaskStatus::Todo,
        TaskStatus::InProgress,
        TaskStatus::Review,
        TaskStatus::Done,
    ];
    
    let start = std::time::Instant::now();
    
    for i in 0..200 {
        let mut task = Task::new(format!("Task {}", i), "".to_string());
        task.status = statuses[i % statuses.len()];
        service.create(task).await.unwrap();
    }
    
    let creation_time = start.elapsed();
    
    // Query and group tasks
    let query_start = std::time::Instant::now();
    let all_tasks = service.list_all().await.unwrap();
    
    let mut column_tasks: HashMap<TaskStatus, Vec<&Task>> = HashMap::new();
    for task in &all_tasks {
        column_tasks.entry(task.status).or_insert(Vec::new()).push(task);
    }
    
    let query_time = query_start.elapsed();
    
    assert_eq!(all_tasks.len(), 200);
    assert!(creation_time.as_secs() < 10);
    assert!(query_time.as_millis() < 200);
    
    // Each column should have roughly 50 tasks
    for status in statuses {
        let count = column_tasks.get(&status).map(|v| v.len()).unwrap_or(0);
        assert!(count >= 45 && count <= 55);
    }
}