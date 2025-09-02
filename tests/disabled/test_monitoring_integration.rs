use plon::domain::task::{Task, TaskStatus, Priority, Position};
use plon::domain::task_execution::{TaskExecution, ExecutionStatus};
use plon::repository::{Repository, database::init_database};
use plon::services::{ClaudeConsole, ClaudeAutomation};
use std::collections::{HashMap, HashSet};
use tempfile::tempdir;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_monitoring_flow_integration() {
    // Setup test database
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create a test task
    let task = Task {
        id: Uuid::new_v4(),
        title: "Test Task for Monitoring".to_string(),
        description: "Test monitoring features".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: Position { x: 100.0, y: 200.0 },
        metadata: HashMap::new(),
        tags: HashSet::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: Some(2.0),
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
    
    // Verify no active execution initially
    let active = ClaudeConsole::get_active_execution(&repo, task_id).await.unwrap();
    assert!(active.is_none());
    
    // Create an execution
    let mut execution = TaskExecution::new(task_id, "feature/test-branch".to_string());
    execution.status = ExecutionStatus::Running;
    repo.task_executions.create(&execution).await.unwrap();
    
    // Update task status to InProgress
    let mut updated_task = repo.tasks.get(task_id).await.unwrap().unwrap();
    updated_task.status = TaskStatus::InProgress;
    repo.tasks.update(&updated_task).await.unwrap();
    
    // Verify active execution can be found
    let active = ClaudeConsole::get_active_execution(&repo, task_id).await.unwrap();
    assert!(active.is_some());
    assert_eq!(active.as_ref().unwrap().status, ExecutionStatus::Running);
    
    // Check execution status
    let status = ClaudeConsole::get_execution_status(&repo, execution.id).await.unwrap();
    assert_eq!(status, Some(ExecutionStatus::Running));
    
    // Simulate completion
    let mut exec = repo.task_executions.get(execution.id).await.unwrap().unwrap();
    exec.status = ExecutionStatus::Success;
    exec.completed_at = Some(Utc::now());
    exec.pr_url = Some("https://github.com/user/repo/pull/123".to_string());
    repo.task_executions.update(&exec).await.unwrap();
    
    // Verify no longer active
    let active = ClaudeConsole::get_active_execution(&repo, task_id).await.unwrap();
    assert!(active.is_none());
    
    // Verify status is updated
    let status = ClaudeConsole::get_execution_status(&repo, execution.id).await.unwrap();
    assert_eq!(status, Some(ExecutionStatus::Success));
    
    // List executions for task
    let all_execs = repo.task_executions.list_for_task(task_id).await.unwrap();
    assert_eq!(all_execs.len(), 1);
    assert_eq!(all_execs[0].status, ExecutionStatus::Success);
    assert!(all_execs[0].pr_url.is_some());
}

#[tokio::test]
async fn test_multiple_executions_only_one_active() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create task
    let task = Task {
        id: Uuid::new_v4(),
        title: "Test Task".to_string(),
        description: "Test".to_string(),
        status: TaskStatus::InProgress,
        priority: Priority::Medium,
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
    
    // Create completed execution
    let mut exec1 = TaskExecution::new(task_id, "feature/old".to_string());
    exec1.status = ExecutionStatus::Success;
    exec1.completed_at = Some(Utc::now());
    repo.task_executions.create(&exec1).await.unwrap();
    
    // Create running execution
    let exec2 = TaskExecution::new(task_id, "feature/current".to_string());
    repo.task_executions.create(&exec2).await.unwrap();
    
    // Only the running execution should be active
    let active = ClaudeConsole::get_active_execution(&repo, task_id).await.unwrap();
    assert!(active.is_some());
    assert_eq!(active.unwrap().branch_name, "feature/current");
    
    // List should show both
    let all = repo.task_executions.list_for_task(task_id).await.unwrap();
    assert_eq!(all.len(), 2);
}