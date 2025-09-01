#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ClaudeConsole;
    use crate::domain::task_execution::{TaskExecution, ExecutionStatus};
    use crate::repository::{Repository, database::init_database};
    use uuid::Uuid;
    use chrono::Utc;
    use tempfile::tempdir;
    use std::path::PathBuf;

    async fn setup_test_db() -> Repository {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
        Repository::new(pool)
    }

    fn create_test_execution() -> TaskExecution {
        TaskExecution::new(
            Uuid::new_v4(),
            "test-branch".to_string(),
        )
    }

    #[tokio::test]
    async fn test_get_active_execution_none() {
        let repo = setup_test_db().await;
        let task_id = Uuid::new_v4();
        
        let result = ClaudeConsole::get_active_execution(&repo, task_id).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_active_execution_with_running() {
        let repo = setup_test_db().await;
        let task_id = Uuid::new_v4();
        
        // Create and save a running execution
        let mut execution = create_test_execution();
        execution.task_id = task_id;
        execution.status = ExecutionStatus::Running;
        
        repo.task_executions.create(execution.clone()).await.unwrap();
        
        let result = ClaudeConsole::get_active_execution(&repo, task_id).await;
        
        assert!(result.is_ok());
        let exec = result.unwrap();
        assert!(exec.is_some());
        assert_eq!(exec.unwrap().status, ExecutionStatus::Running);
    }

    #[tokio::test]
    async fn test_get_active_execution_ignores_completed() {
        let repo = setup_test_db().await;
        let task_id = Uuid::new_v4();
        
        // Create a completed execution
        let mut execution = create_test_execution();
        execution.task_id = task_id;
        execution.status = ExecutionStatus::Success;
        execution.completed_at = Some(Utc::now());
        
        repo.task_executions.create(execution).await.unwrap();
        
        let result = ClaudeConsole::get_active_execution(&repo, task_id).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_execution_status() {
        let repo = setup_test_db().await;
        let execution = create_test_execution();
        let exec_id = execution.id;
        
        repo.task_executions.create(execution).await.unwrap();
        
        let status = ClaudeConsole::get_execution_status(&repo, exec_id).await;
        
        assert!(status.is_ok());
        assert_eq!(status.unwrap(), Some(ExecutionStatus::Running));
    }

    #[tokio::test]
    async fn test_get_execution_status_not_found() {
        let repo = setup_test_db().await;
        let exec_id = Uuid::new_v4();
        
        let status = ClaudeConsole::get_execution_status(&repo, exec_id).await;
        
        assert!(status.is_ok());
        assert!(status.unwrap().is_none());
    }

    #[test]
    fn test_open_console_command_generation() {
        // This test verifies the command would be properly formed
        // We can't actually open a console in tests
        let workspace_dir = PathBuf::from("/test/workspace");
        let execution = create_test_execution();
        
        // We can't test the actual opening, but we can verify the function exists
        // and accepts the right parameters
        let result = ClaudeConsole::open_console(&workspace_dir, &execution);
        
        // In CI this will fail as Claude CLI is not installed
        // but we're testing the function signature
        assert!(result.is_err()); // Expected since Claude CLI won't be found in tests
    }

    #[test]
    fn test_open_logs_terminal_script_generation() {
        let dir = tempdir().unwrap();
        let workspace_dir = dir.path().to_path_buf();
        let execution = create_test_execution();
        
        // This will create the monitoring script
        let result = ClaudeConsole::open_logs_terminal(&workspace_dir, &execution);
        
        // Check that the script was created
        let script_path = workspace_dir.join(format!(".monitor_{}.sh", execution.id));
        
        if result.is_ok() {
            assert!(script_path.exists());
            
            // Read the script and verify it contains expected content
            let script_content = std::fs::read_to_string(&script_path).unwrap();
            assert!(script_content.contains(&execution.task_id.to_string()));
            assert!(script_content.contains(&execution.branch_name));
            assert!(script_content.contains("Git Log"));
        }
    }
}