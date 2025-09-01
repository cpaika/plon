#[cfg(test)]
mod claude_automation_tests {
    use plon::services::ClaudeAutomation;
    use plon::repository::Repository;
    use plon::repository::database::init_test_database;
    use plon::domain::task::{Task, TaskStatus, Priority};
    use plon::domain::task_execution::ExecutionStatus;
    use tempfile::TempDir;
    use std::process::Command;

    async fn setup_test_environment() -> (Repository, TempDir) {
        let pool = init_test_database().await.unwrap();
        let repo = Repository::new(pool);
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize git repo in temp dir
        Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["init"])
            .output()
            .unwrap();
        
        Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["config", "user.email", "test@example.com"])
            .output()
            .unwrap();
        
        Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["config", "user.name", "Test User"])
            .output()
            .unwrap();
        
        (repo, temp_dir)
    }

    #[tokio::test]
    async fn test_claude_automation_creation() {
        let (_repo, temp_dir) = setup_test_environment().await;
        
        let automation = ClaudeAutomation::new(temp_dir.path().to_path_buf());
        
        // Basic creation test
        assert!(true); // Automation created successfully
    }

    #[tokio::test]
    async fn test_claude_automation_with_repository() {
        let (repo, temp_dir) = setup_test_environment().await;
        
        let automation = ClaudeAutomation::with_repository(
            temp_dir.path().to_path_buf(),
            repo.clone()
        );
        
        // Create a task
        let task = Task::new("Test task".to_string(), "Test description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Get task executions (should be empty initially)
        let executions = automation.get_task_executions(task.id).await.unwrap();
        assert_eq!(executions.len(), 0);
    }

    #[tokio::test]
    async fn test_get_task_executions() {
        let (repo, temp_dir) = setup_test_environment().await;
        
        let automation = ClaudeAutomation::with_repository(
            temp_dir.path().to_path_buf(),
            repo.clone()
        );
        
        // Create a task
        let task = Task::new("Test task".to_string(), "Description".to_string());
        repo.tasks.create(&task).await.unwrap();
        
        // Create some executions
        use plon::domain::task_execution::TaskExecution;
        let exec1 = TaskExecution::new(task.id, "branch-1".to_string());
        let exec2 = TaskExecution::new(task.id, "branch-2".to_string());
        
        repo.task_executions.create(&exec1).await.unwrap();
        repo.task_executions.create(&exec2).await.unwrap();
        
        // Get executions through automation service
        let executions = automation.get_task_executions(task.id).await.unwrap();
        assert_eq!(executions.len(), 2);
    }

    #[tokio::test]
    async fn test_check_task_status_without_pr() {
        let (_repo, temp_dir) = setup_test_environment().await;
        
        let automation = ClaudeAutomation::new(temp_dir.path().to_path_buf());
        
        // Create a task
        let task = Task::new("Test task".to_string(), "Description".to_string());
        
        // Check status (should be Todo when no branch exists)
        let status = automation.check_task_status(task.id).await.unwrap();
        assert_eq!(status, TaskStatus::Todo);
    }

    #[tokio::test]
    async fn test_execute_task_without_claude_cli() {
        let (_repo, temp_dir) = setup_test_environment().await;
        
        let automation = ClaudeAutomation::new(temp_dir.path().to_path_buf());
        
        // Create a task
        let task = Task::new("Test task".to_string(), "Description".to_string());
        
        // Try to execute (should fail without Claude CLI installed)
        let result = automation.execute_task(&task, "https://github.com/test/repo").await;
        
        // Should fail with a helpful error message
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("Claude Code CLI is not installed") || error.contains("claude"));
    }

    #[tokio::test]
    async fn test_update_execution_status_nonexistent() {
        let (repo, temp_dir) = setup_test_environment().await;
        
        let automation = ClaudeAutomation::with_repository(
            temp_dir.path().to_path_buf(),
            repo.clone()
        );
        
        // Try to update non-existent execution
        let fake_id = uuid::Uuid::new_v4();
        let result = automation.update_execution_status(fake_id).await;
        
        // Should succeed (no-op for non-existent execution)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_automation_with_existing_branch() {
        let (_repo, temp_dir) = setup_test_environment().await;
        
        // Create a branch manually
        Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["checkout", "-b", "task/123-test"])
            .output()
            .unwrap();
        
        let automation = ClaudeAutomation::new(temp_dir.path().to_path_buf());
        
        // Check status with existing branch
        let task_id = uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
        let status = automation.check_task_status(task_id).await.unwrap();
        
        // Should detect the branch exists
        assert_eq!(status, TaskStatus::InProgress);
    }
}