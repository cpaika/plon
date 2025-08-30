#[path = "./claude_code_test_utils.rs"]
mod claude_code_test_utils;

use anyhow::Result;
use claude_code_test_utils::{ClaudeCodeTestEnvironment, MockClaudeCodeService, fixtures};
use plon::domain::claude_code::{ClaudeCodeConfig, ClaudePromptTemplate, SessionStatus};
use plon::domain::task::Task;
use plon::repository::Repository;
use plon::services::{ClaudeCodeService, command_executor::mock::MockCommandExecutor};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use uuid::Uuid;

#[tokio::test]
async fn test_launch_claude_code_success() -> Result<()> {
    // Setup
    let mut env = ClaudeCodeTestEnvironment::new().await?;
    let task = fixtures::sample_task();

    // Save task to database
    env.repository.tasks.create(&task).await?;

    // Launch Claude Code with mock in success mode
    let session = env.launch_with_mock(&task, "success").await?;

    // Verify initial session state
    assert_eq!(session.task_id, task.id);
    assert_eq!(session.status, SessionStatus::Initializing);
    assert!(session.branch_name.is_some());
    assert!(session.pr_url.is_none());

    // Wait for completion
    let completed_session = env.wait_for_completion(session.id, 10).await?;

    // Verify completed state
    assert_eq!(completed_session.status, SessionStatus::Completed);
    assert!(completed_session.pr_url.is_some());
    assert!(completed_session.pr_number.is_some());
    assert!(completed_session.completed_at.is_some());
    assert!(completed_session.error_message.is_none());

    // Verify logs contain expected messages
    let logs = env.get_session_logs(session.id).await?;
    assert!(logs.contains("Initializing Claude Code session"));
    assert!(logs.contains("Working directory"));
    assert!(logs.contains("Branch name"));

    env.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_launch_claude_code_with_error() -> Result<()> {
    // Setup
    let mut env = ClaudeCodeTestEnvironment::new().await?;
    let task = fixtures::minimal_task();

    // Save task
    env.repository.tasks.create(&task).await?;

    // Launch with error mode
    let session = env.launch_with_mock(&task, "error").await?;

    // Wait for completion
    let completed_session = env.wait_for_completion(session.id, 10).await?;

    // Verify error state
    assert_eq!(completed_session.status, SessionStatus::Failed);
    assert!(completed_session.error_message.is_some());
    assert!(completed_session.pr_url.is_none());

    let error_msg = completed_session.error_message.unwrap();
    assert!(error_msg.contains("Failed to understand task requirements"));

    env.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_multiple_concurrent_sessions() -> Result<()> {
    // Setup
    let mut env = ClaudeCodeTestEnvironment::new().await?;

    // Create multiple tasks
    let tasks = vec![
        fixtures::sample_task(),
        fixtures::complex_task(),
        fixtures::minimal_task(),
    ];

    let mut session_ids = Vec::new();

    // Launch multiple sessions concurrently
    for task in &tasks {
        env.repository.tasks.create(&task).await?;
        let session = env.launch_with_mock(&task, "success").await?;
        session_ids.push(session.id);
    }

    // Verify all sessions are tracked
    let active_sessions = env.repository.claude_code.get_active_sessions().await?;
    assert!(active_sessions.len() >= 3);

    // Wait for all to complete
    for session_id in session_ids {
        let completed = env.wait_for_completion(session_id, 15).await?;
        assert_eq!(completed.status, SessionStatus::Completed);
        assert!(completed.pr_url.is_some());
    }

    env.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_session_cancellation() -> Result<()> {
    // Setup
    let mut env = ClaudeCodeTestEnvironment::new().await?;
    let task = fixtures::complex_task();

    env.repository.tasks.create(&task).await?;

    // Create a long-running mock
    env.mock_executor.add_response_with_delay(
        "claude",
        vec!["code"],
        "Working...",
        "",
        true,
        5000, // 5 second delay
    );

    // Launch Claude Code
    let session = env
        .service
        .launch_claude_code(&task, &env.config, &env.template)
        .await?;

    // Wait a bit for it to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Cancel the session
    env.service.cancel_session(session.id).await?;

    // Verify cancellation
    let cancelled_session = env
        .repository
        .claude_code
        .get_session(session.id)
        .await?
        .expect("Session should exist");

    assert_eq!(cancelled_session.status, SessionStatus::Cancelled);
    assert!(cancelled_session.session_log.contains("cancelled by user"));

    env.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_session_timeout_detection() -> Result<()> {
    // Setup with short timeout
    let mut env = ClaudeCodeTestEnvironment::new().await?;
    env.config.max_session_duration_minutes = 1; // Very short timeout for testing
    env.repository
        .claude_code
        .update_config(&env.config)
        .await?;

    let task = fixtures::sample_task();
    env.repository.tasks.create(&task).await?;

    // Create a session
    let mut session = env
        .service
        .launch_claude_code(&task, &env.config, &env.template)
        .await?;

    // Manually set started_at to past time to simulate timeout
    session.started_at = chrono::Utc::now() - chrono::Duration::minutes(2);
    env.repository.claude_code.update_session(&session).await?;

    // Check timeout detection
    assert!(session.is_timed_out(env.config.max_session_duration_minutes));

    env.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_pr_creation_flow() -> Result<()> {
    // Setup
    let mut env = ClaudeCodeTestEnvironment::new().await?;
    let task = fixtures::sample_task();

    env.repository.tasks.create(&task).await?;

    // Launch with success mode
    let session = env.launch_with_mock(&task, "success").await?;

    // Wait for completion
    let completed = env.wait_for_completion(session.id, 10).await?;

    // Verify PR was created
    assert!(env.verify_pr_created(session.id).await?);

    // Verify PR details
    let pr_url = completed.pr_url.unwrap();
    assert!(pr_url.contains("github.com"));
    assert!(pr_url.contains("/pull/"));

    let pr_number = completed.pr_number.unwrap();
    assert!(pr_number > 0);

    env.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_configuration_management() -> Result<()> {
    // Setup
    let pool = SqlitePool::connect(":memory:").await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let repository = Repository::new(pool);

    // Test creating config
    let config = fixtures::sample_config();
    repository.claude_code.create_config(&config).await?;

    // Test retrieving config
    let retrieved = repository.claude_code.get_config().await?;
    assert!(retrieved.is_some());

    let retrieved_config = retrieved.unwrap();
    assert_eq!(retrieved_config.github_repo, config.github_repo);
    assert_eq!(retrieved_config.github_owner, config.github_owner);
    assert_eq!(retrieved_config.auto_create_pr, config.auto_create_pr);

    // Test updating config
    let mut updated_config = retrieved_config.clone();
    updated_config.github_repo = "new-repo".to_string();
    updated_config.auto_create_pr = false;
    repository
        .claude_code
        .update_config(&updated_config)
        .await?;

    let final_config = repository.claude_code.get_config().await?.unwrap();
    assert_eq!(final_config.github_repo, "new-repo");
    assert!(!final_config.auto_create_pr);

    Ok(())
}

#[tokio::test]
async fn test_template_management() -> Result<()> {
    // Setup
    let pool = SqlitePool::connect(":memory:").await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let repository = Repository::new(pool);

    // Create template
    let template = fixtures::sample_template();
    repository.claude_code.create_template(&template).await?;

    // Retrieve by name
    let retrieved = repository.claude_code.get_template("standard").await?;
    assert!(retrieved.is_some());

    let retrieved_template = retrieved.unwrap();
    assert_eq!(retrieved_template.name, template.name);
    assert!(retrieved_template.template.contains("task_title"));

    // Test variable extraction
    assert!(
        retrieved_template
            .variables
            .contains(&"task_title".to_string())
    );
    assert!(
        retrieved_template
            .variables
            .contains(&"task_description".to_string())
    );
    assert!(
        retrieved_template
            .variables
            .contains(&"estimated_hours".to_string())
    );

    // Test template rendering
    let mut context = std::collections::HashMap::new();
    context.insert("task_title".to_string(), "Test Task".to_string());
    context.insert(
        "task_description".to_string(),
        "Test Description".to_string(),
    );
    context.insert("estimated_hours".to_string(), "5".to_string());
    context.insert("tags".to_string(), "test, demo".to_string());

    let rendered = retrieved_template.render(&context);
    assert!(rendered.contains("Test Task"));
    assert!(rendered.contains("Test Description"));
    assert!(rendered.contains("5"));
    assert!(rendered.contains("test, demo"));

    Ok(())
}

#[tokio::test]
async fn test_session_persistence_and_retrieval() -> Result<()> {
    // Setup
    let pool = SqlitePool::connect(":memory:").await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let repository = Repository::new(pool);

    let task_id = Uuid::new_v4();

    // Create multiple sessions for same task
    let mut session1 = plon::domain::claude_code::ClaudeCodeSession::new(task_id);
    session1.update_status(SessionStatus::Completed);
    session1.set_pr_info("https://github.com/test/repo/pull/1".to_string(), 1);

    let mut session2 = plon::domain::claude_code::ClaudeCodeSession::new(task_id);
    session2.update_status(SessionStatus::Failed);
    session2.set_error("Test error".to_string());

    let mut session3 = plon::domain::claude_code::ClaudeCodeSession::new(task_id);
    session3.update_status(SessionStatus::Working);

    // Save all sessions
    repository.claude_code.create_session(&session1).await?;
    repository.claude_code.create_session(&session2).await?;
    repository.claude_code.create_session(&session3).await?;

    // Retrieve sessions by task
    let task_sessions = repository.claude_code.get_sessions_by_task(task_id).await?;
    assert_eq!(task_sessions.len(), 3);

    // Verify session states
    let completed_count = task_sessions
        .iter()
        .filter(|s| s.status == SessionStatus::Completed)
        .count();
    assert_eq!(completed_count, 1);

    let failed_count = task_sessions
        .iter()
        .filter(|s| s.status == SessionStatus::Failed)
        .count();
    assert_eq!(failed_count, 1);

    // Get active sessions
    let active_sessions = repository.claude_code.get_active_sessions().await?;
    assert_eq!(active_sessions.len(), 1);
    assert_eq!(active_sessions[0].status, SessionStatus::Working);

    Ok(())
}

#[tokio::test]
async fn test_session_log_accumulation() -> Result<()> {
    // Setup
    let pool = SqlitePool::connect(":memory:").await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let repository = Repository::new(pool);

    let task_id = Uuid::new_v4();
    let mut session = plon::domain::claude_code::ClaudeCodeSession::new(task_id);

    // Add multiple log entries
    session.append_log("Starting session");
    session.append_log("Analyzing task");
    session.append_log("Implementing solution");
    session.append_log("Running tests");
    session.append_log("Creating pull request");

    // Save session
    repository.claude_code.create_session(&session).await?;

    // Retrieve and verify logs
    let retrieved = repository
        .claude_code
        .get_session(session.id)
        .await?
        .unwrap();

    assert!(retrieved.session_log.contains("Starting session"));
    assert!(retrieved.session_log.contains("Analyzing task"));
    assert!(retrieved.session_log.contains("Implementing solution"));
    assert!(retrieved.session_log.contains("Running tests"));
    assert!(retrieved.session_log.contains("Creating pull request"));

    // Verify timestamps are included
    let log_lines: Vec<&str> = retrieved.session_log.lines().collect();
    assert!(log_lines.len() >= 5);

    for line in log_lines {
        assert!(line.contains("[20")); // Year timestamp check
    }

    Ok(())
}

#[tokio::test]
async fn test_cleanup_old_sessions() -> Result<()> {
    // Setup
    let pool = SqlitePool::connect(":memory:").await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let repository = Repository::new(pool);

    let task_id = Uuid::new_v4();

    // Create old completed session
    let mut old_session = plon::domain::claude_code::ClaudeCodeSession::new(task_id);
    old_session.update_status(SessionStatus::Completed);
    old_session.completed_at = Some(chrono::Utc::now() - chrono::Duration::days(31));
    repository.claude_code.create_session(&old_session).await?;

    // Create recent completed session
    let mut recent_session = plon::domain::claude_code::ClaudeCodeSession::new(task_id);
    recent_session.update_status(SessionStatus::Completed);
    repository
        .claude_code
        .create_session(&recent_session)
        .await?;

    // Create active session (should not be cleaned up)
    let mut active_session = plon::domain::claude_code::ClaudeCodeSession::new(task_id);
    active_session.update_status(SessionStatus::Working);
    repository
        .claude_code
        .create_session(&active_session)
        .await?;

    // Run cleanup for sessions older than 30 days
    let cutoff = chrono::Utc::now() - chrono::Duration::days(30);
    repository.claude_code.cleanup_old_sessions(cutoff).await?;

    // Verify only old completed session was removed
    let remaining = repository.claude_code.get_sessions_by_task(task_id).await?;
    assert_eq!(remaining.len(), 2);

    // Verify the old session was the one removed
    assert!(!remaining.iter().any(|s| s.id == old_session.id));
    assert!(remaining.iter().any(|s| s.id == recent_session.id));
    assert!(remaining.iter().any(|s| s.id == active_session.id));

    Ok(())
}

#[tokio::test]
async fn test_mock_service_behavior() -> Result<()> {
    // Test the mock service itself
    let mut mock_service = MockClaudeCodeService::new();
    let task = fixtures::sample_task();

    // Test successful launch
    let session = mock_service.mock_launch(&task).await?;
    assert_eq!(session.status, SessionStatus::Working);
    assert!(session.branch_name.is_some());

    // Test PR completion
    mock_service.mock_complete_with_pr(session.id).await?;
    let completed = mock_service
        .sessions
        .iter()
        .find(|s| s.id == session.id)
        .unwrap();
    assert_eq!(completed.status, SessionStatus::Completed);
    assert!(completed.pr_url.is_some());

    // Test failure mode
    let mut failing_service = MockClaudeCodeService::new().with_failure("Simulated failure");
    let result = failing_service.mock_launch(&task).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Simulated failure")
    );

    Ok(())
}
