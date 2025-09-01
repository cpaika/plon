use plon::domain::task::{Task, TaskStatus, Priority, Position};
use plon::domain::task_execution::{TaskExecution, ExecutionStatus};
use plon::repository::{Repository, database::init_database};
use plon::services::ClaudeConsole;
use std::collections::{HashMap, HashSet};
use tempfile::tempdir;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_modal_shows_execution_details() {
    // Setup test database
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create a test task in InProgress state
    let task = Task {
        id: Uuid::new_v4(),
        title: "Test Modal Task".to_string(),
        description: "Testing modal functionality".to_string(),
        status: TaskStatus::InProgress,
        priority: Priority::High,
        position: Position { x: 100.0, y: 100.0 },
        metadata: HashMap::new(),
        tags: HashSet::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: Some(3.0),
        actual_hours: None,
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        subtasks: vec![],
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 0,
    };
    
    let task_id = task.id;
    let task_title = task.title.clone();
    repo.tasks.create(&task).await.unwrap();
    
    // Create an active execution with logs
    let mut execution = TaskExecution::new(task_id, "feature/modal-test".to_string());
    execution.status = ExecutionStatus::Running;
    execution.add_log("Starting task execution...".to_string());
    execution.add_log("Analyzing task requirements...".to_string());
    execution.add_log("Generating implementation plan...".to_string());
    execution.add_log("Writing code...".to_string());
    
    repo.task_executions.create(&execution).await.unwrap();
    
    // Verify the modal would show the right data
    let active_exec = ClaudeConsole::get_active_execution(&repo, task_id)
        .await
        .unwrap()
        .expect("Should have active execution");
    
    // Check modal would display:
    assert_eq!(active_exec.status, ExecutionStatus::Running);
    assert_eq!(active_exec.branch_name, "feature/modal-test");
    assert_eq!(active_exec.output_log.len(), 4);
    
    // Check if logs contain expected content (logs are now stored with timestamps)
    let logs_text = active_exec.output_log.join("\n");
    assert!(logs_text.contains("Starting task execution"));
    assert!(logs_text.contains("Analyzing task requirements"));
    assert!(logs_text.contains("Generating implementation plan"));
    assert!(logs_text.contains("Writing code"));
    
    assert!(active_exec.pr_url.is_none());
    assert!(active_exec.error_message.is_none());
    
    println!("✅ Modal would display:");
    println!("   Task: {}", task_title);
    println!("   Status: {:?}", active_exec.status);
    println!("   Branch: {}", active_exec.branch_name);
    println!("   Logs: {} entries", active_exec.output_log.len());
}

#[tokio::test]
async fn test_modal_shows_completed_execution() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create task
    let task = Task {
        id: Uuid::new_v4(),
        title: "Completed Task".to_string(),
        description: "Test".to_string(),
        status: TaskStatus::Done,
        priority: Priority::Medium,
        position: Position { x: 0.0, y: 0.0 },
        metadata: HashMap::new(),
        tags: HashSet::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: Some(Utc::now()),
        estimated_hours: None,
        actual_hours: None,
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        subtasks: vec![],
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 0,
    };
    
    let task_id = task.id;
    repo.tasks.create(&task).await.unwrap();
    
    // Create completed execution with PR
    let mut execution = TaskExecution::new(task_id, "feature/completed".to_string());
    execution.status = ExecutionStatus::Success;
    execution.completed_at = Some(Utc::now());
    execution.pr_url = Some("https://github.com/user/repo/pull/42".to_string());
    execution.add_log("Task completed successfully!".to_string());
    
    repo.task_executions.create(&execution).await.unwrap();
    
    // Since execution is completed, get_active_execution should return None
    let active = ClaudeConsole::get_active_execution(&repo, task_id)
        .await
        .unwrap();
    assert!(active.is_none());
    
    // But we can still get execution history
    let history = repo.task_executions.list_for_task(task_id).await.unwrap();
    assert_eq!(history.len(), 1);
    
    let exec = &history[0];
    assert_eq!(exec.status, ExecutionStatus::Success);
    assert!(exec.pr_url.is_some());
    assert!(exec.completed_at.is_some());
    assert!(exec.duration().is_some());
    
    println!("✅ Completed execution modal would show:");
    println!("   Status: Success");
    println!("   PR: {}", exec.pr_url.as_ref().unwrap());
    println!("   Duration: {} minutes", exec.duration().unwrap().num_minutes());
}

#[tokio::test]
async fn test_modal_shows_error_execution() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create task
    let task = Task {
        id: Uuid::new_v4(),
        title: "Failed Task".to_string(),
        description: "Test".to_string(),
        status: TaskStatus::Blocked,
        priority: Priority::High,
        position: Position { x: 0.0, y: 0.0 },
        metadata: HashMap::new(),
        tags: HashSet::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        subtasks: vec![],
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 0,
    };
    
    let task_id = task.id;
    repo.tasks.create(&task).await.unwrap();
    
    // Create failed execution with error
    let mut execution = TaskExecution::new(task_id, "feature/failed".to_string());
    execution.status = ExecutionStatus::Failed;
    execution.completed_at = Some(Utc::now());
    execution.error_message = Some("Failed to compile: missing semicolon on line 42".to_string());
    execution.add_log("Starting task...".to_string());
    execution.add_log("ERROR: Compilation failed".to_string());
    
    repo.task_executions.create(&execution).await.unwrap();
    
    // Failed execution is not active
    let active = ClaudeConsole::get_active_execution(&repo, task_id)
        .await
        .unwrap();
    assert!(active.is_none());
    
    // Get from history
    let history = repo.task_executions.list_for_task(task_id).await.unwrap();
    let exec = &history[0];
    
    assert_eq!(exec.status, ExecutionStatus::Failed);
    assert!(exec.error_message.is_some());
    assert_eq!(exec.output_log.len(), 2);
    
    println!("✅ Failed execution modal would show:");
    println!("   Status: Failed");
    println!("   Error: {}", exec.error_message.as_ref().unwrap());
    println!("   Logs showing error details");
}