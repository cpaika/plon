#[cfg(test)]
mod claude_launch_tests {
    use plon::services::ClaudeAutomation;
    use plon::domain::task::Task;
    use plon::repository::Repository;
    use plon::repository::database::init_test_database;
    use std::time::{Duration, Instant};
    use tokio::time::timeout;
    use tempfile::TempDir;

    async fn setup_test_environment() -> (Repository, TempDir) {
        let pool = init_test_database().await.unwrap();
        let repo = Repository::new(pool);
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize git repo in temp dir
        std::process::Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["init"])
            .output()
            .unwrap();
        
        std::process::Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["config", "user.email", "test@example.com"])
            .output()
            .unwrap();
        
        std::process::Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["config", "user.name", "Test User"])
            .output()
            .unwrap();
        
        (repo, temp_dir)
    }

    #[tokio::test]
    async fn test_execute_task_should_not_block() {
        let (repo, temp_dir) = setup_test_environment().await;
        
        // Create a test task
        let task = Task::new(
            "Test Task".to_string(),
            "Test description".to_string()
        );
        repo.tasks.create(&task).await.unwrap();
        
        // Create automation service with repository
        let automation = ClaudeAutomation::with_repository(
            temp_dir.path().to_path_buf(),
            repo.clone()
        );
        
        // Execute task should complete quickly (not wait for Claude to finish)
        let start = Instant::now();
        let result = timeout(
            Duration::from_secs(3),
            automation.execute_task(&task, "https://github.com/test/repo")
        ).await;
        
        let elapsed = start.elapsed();
        
        // Should complete within 3 seconds
        match result {
            Ok(Ok(execution_id)) => {
                println!("Task execution started in {:?}", elapsed);
                assert!(elapsed < Duration::from_secs(3), "Should not wait for Claude to complete");
                
                // Verify execution was recorded
                let execution = repo.task_executions.get(execution_id).await.unwrap();
                assert!(execution.is_some());
            }
            Ok(Err(e)) => {
                // It's ok if Claude is not installed, but it shouldn't hang
                println!("Expected error (Claude not installed): {}", e);
                assert!(elapsed < Duration::from_secs(3), "Should fail quickly if Claude not found");
            }
            Err(_) => {
                panic!("execute_task timed out! It should return quickly without waiting for Claude to complete.");
            }
        }
    }

    #[tokio::test]
    async fn test_launch_claude_in_background() {
        // Test that Claude is launched in background, not blocking
        let (_repo, temp_dir) = setup_test_environment().await;
        
        let task = Task::new(
            "Background Test".to_string(),
            "Should launch in background".to_string()
        );
        
        let automation = ClaudeAutomation::new(temp_dir.path().to_path_buf());
        
        // This should return immediately (or fail fast if Claude not found)
        let start = Instant::now();
        let _result = automation.execute_task(&task, "https://github.com/test/repo").await;
        let elapsed = start.elapsed();
        
        // Should complete very quickly
        assert!(
            elapsed < Duration::from_secs(2),
            "Claude launch took {:?}, should be nearly instant",
            elapsed
        );
    }
}