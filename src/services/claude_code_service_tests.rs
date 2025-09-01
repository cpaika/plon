#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::domain::claude_code::{
        ClaudeCodeConfig, ClaudeCodeSession, ClaudePromptTemplate, SessionStatus,
    };
    use crate::domain::task::Task;
    use crate::repository::Repository;
    use crate::services::command_executor::mock::MockCommandExecutor;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn setup_test_env() -> (ClaudeCodeService, Repository, TempDir, MockCommandExecutor) {
        // Use in-memory database for tests
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!().run(&pool).await.unwrap();

        let temp_dir = TempDir::new().unwrap();

        let repository = Repository::new(pool);

        let mock = MockCommandExecutor::new();
        mock.mock_git_operations();

        let service = ClaudeCodeService::with_executor(
            repository.claude_code.clone(),
            Arc::new(mock.clone()),
        );

        (service, repository, temp_dir, mock)
    }

    #[tokio::test]
    async fn test_launch_claude_code() {
        let (_service, repository, _temp_dir, mut mock) = setup_test_env().await;

        // Setup mock responses
        mock.mock_claude_success();
        mock.mock_gh_pr_create();

        // Recreate service with updated mock
        let mut service =
            ClaudeCodeService::with_executor(repository.claude_code.clone(), Arc::new(mock));

        // Create config and template
        let config = ClaudeCodeConfig::new("test-repo".to_string(), "test-owner".to_string());
        repository.claude_code.create_config(&config).await.unwrap();

        let template =
            ClaudePromptTemplate::new("test".to_string(), "Task: {{task_title}}".to_string());
        repository
            .claude_code
            .create_template(&template)
            .await
            .unwrap();

        // Create task
        let task = Task::new("Test Task".to_string(), "Test Description".to_string());
        repository.tasks.create(&task).await.unwrap();

        // Launch Claude Code
        let session = service
            .launch_claude_code(&task, &config, &template)
            .await
            .unwrap();

        assert_eq!(session.task_id, task.id);
        assert!(session.branch_name.is_some());
    }

    #[tokio::test]
    async fn test_cancel_session() {
        let (mut service, repository, _temp_dir, _mock) = setup_test_env().await;

        // Create task first
        let task = Task::new("Test Task".to_string(), "Description".to_string());
        repository.tasks.create(&task).await.unwrap();

        let session = ClaudeCodeSession::new(task.id);
        repository
            .claude_code
            .create_session(&session)
            .await
            .unwrap();

        service.cancel_session(session.id).await.unwrap();

        let cancelled = repository
            .claude_code
            .get_session(session.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(cancelled.status, SessionStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_process_log_updates() {
        let (mut service, repository, _temp_dir, _mock) = setup_test_env().await;

        // Create task first
        let task = Task::new("Test Task".to_string(), "Description".to_string());
        repository.tasks.create(&task).await.unwrap();

        let session = ClaudeCodeSession::new(task.id);
        repository
            .claude_code
            .create_session(&session)
            .await
            .unwrap();

        // Send log message
        service
            .log_sender
            .send((session.id, "Test log message".to_string()))
            .await
            .unwrap();

        // Process log updates
        service.process_log_updates().await.unwrap();

        let updated = repository
            .claude_code
            .get_session(session.id)
            .await
            .unwrap()
            .unwrap();
        assert!(updated.session_log.contains("Test log message"));
    }

    #[tokio::test]
    async fn test_generate_branch_name() {
        let (_service, repository, _temp_dir, _mock) = setup_test_env().await;

        let service = ClaudeCodeService::new(repository.claude_code.clone());
        let task = Task::new("Fix Bug #123".to_string(), "Description".to_string());

        let branch_name = service.generate_branch_name(&task);
        assert!(branch_name.starts_with("claude/"));
        assert!(branch_name.contains("fix-bug"));
    }

    #[tokio::test]
    async fn test_render_prompt() {
        let (_service, repository, _temp_dir, _mock) = setup_test_env().await;

        let service = ClaudeCodeService::new(repository.claude_code.clone());

        let template = ClaudePromptTemplate::new(
            "test".to_string(),
            "Task: {{task_title}}\nDescription: {{task_description}}".to_string(),
        );

        let mut task = Task::new("Test Task".to_string(), "Test Description".to_string());
        task.add_tag("test".to_string());

        let prompt = service.render_prompt(&task, &template).unwrap();
        assert!(prompt.contains("Test Task"));
        assert!(prompt.contains("Test Description"));
    }

    #[tokio::test]
    async fn test_mock_command_executor_integration() {
        let mut mock = MockCommandExecutor::new();

        // Test git operations
        mock.mock_git_operations();

        let result = mock
            .execute("git", &["checkout", "-b", "test-branch"], None, None)
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("Switched to a new branch"));

        // Test Claude Code mock
        mock.mock_claude_success();

        let result = mock
            .execute("claude", &["code", "--file", "test.md"], None, None)
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("Task completed successfully"));

        // Test GitHub CLI mock
        mock.mock_gh_pr_create();

        let result = mock
            .execute("gh", &["pr", "create", "--title", "Test"], None, None)
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("github.com"));
        assert!(result.stdout.contains("/pull/"));

        // Verify call history
        assert!(mock.assert_called_with("git", &["checkout"]));
        assert!(mock.assert_called_with("claude", &["code"]));
        assert!(mock.assert_called_with("gh", &["pr", "create"]));
    }

    #[tokio::test]
    async fn test_session_timeout_detection() {
        let (_service, repository, _temp_dir, _mock) = setup_test_env().await;

        // Create task first
        let task = Task::new("Test Task".to_string(), "Description".to_string());
        repository.tasks.create(&task).await.unwrap();

        let mut session = ClaudeCodeSession::new(task.id);

        // Fresh session should not be timed out
        assert!(!session.is_timed_out(60));

        // Manually set old start time
        session.started_at = chrono::Utc::now() - chrono::Duration::minutes(61);

        // Should now be timed out
        assert!(session.is_timed_out(60));
    }

    #[tokio::test]
    async fn test_cleanup_old_sessions() {
        let (_service, repository, _temp_dir, _mock) = setup_test_env().await;

        // Create task first
        let task = Task::new("Test Task".to_string(), "Description".to_string());
        repository.tasks.create(&task).await.unwrap();

        // Create old completed session
        let mut old_session = ClaudeCodeSession::new(task.id);
        old_session.update_status(SessionStatus::Completed);
        old_session.completed_at = Some(chrono::Utc::now() - chrono::Duration::days(31));
        repository
            .claude_code
            .create_session(&old_session)
            .await
            .unwrap();

        // Create recent session
        let recent_session = ClaudeCodeSession::new(task.id);
        repository
            .claude_code
            .create_session(&recent_session)
            .await
            .unwrap();

        // Run cleanup
        let cutoff = chrono::Utc::now() - chrono::Duration::days(30);
        repository
            .claude_code
            .cleanup_old_sessions(cutoff)
            .await
            .unwrap();

        // Verify old session was removed
        let remaining = repository
            .claude_code
            .get_sessions_by_task(task.id)
            .await
            .unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, recent_session.id);
    }
}
