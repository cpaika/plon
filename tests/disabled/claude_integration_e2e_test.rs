#[cfg(test)]
mod claude_integration_e2e_tests {
    use plon::services::ClaudeAutomation;
    use plon::domain::task::{Task, TaskStatus};
    use plon::repository::Repository;
    use plon::repository::database::init_test_database;
    use std::time::Duration;
    use tokio::time::sleep;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_full_claude_integration_flow() {
        // Setup
        let pool = init_test_database().await.unwrap();
        let repo = Repository::new(pool);
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize git
        std::process::Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["init"])
            .output()
            .unwrap();
        
        // Create a task
        let task = Task::new(
            "Integration Test Task".to_string(),
            "Test the full Claude integration flow".to_string()
        );
        repo.tasks.create(&task).await.unwrap();
        
        // Create automation service with repository
        let automation = ClaudeAutomation::with_repository(
            temp_dir.path().to_path_buf(),
            repo.clone()
        );
        
        // Execute the task
        println!("Executing task: {}", task.title);
        let result = automation.execute_task(&task, "https://github.com/test/repo").await;
        
        match result {
            Ok(execution_id) => {
                println!("✅ Task execution started successfully");
                
                // Verify execution was created in database
                let execution = repo.task_executions.get(execution_id).await.unwrap();
                assert!(execution.is_some(), "Execution should be saved in database");
                
                let exec = execution.unwrap();
                assert_eq!(exec.task_id, task.id);
                assert!(exec.branch_name.contains(&task.id.to_string()[..8]));
                
                // Check execution status
                use plon::domain::task_execution::ExecutionStatus;
                assert_eq!(exec.status, ExecutionStatus::Running, "Should be marked as running");
                
                // Verify prompt file was created
                let prompt_file = temp_dir.path().join(format!(".claude_task_{}.md", task.id));
                assert!(prompt_file.exists(), "Prompt file should be created");
                
                // Read and verify prompt content
                let prompt_content = std::fs::read_to_string(&prompt_file).unwrap();
                assert!(prompt_content.contains(&task.title));
                assert!(prompt_content.contains(&task.description));
                
                println!("✅ All integration checks passed!");
            }
            Err(e) => {
                // It's ok if Claude is not installed, but the error should be clear
                println!("Expected error (Claude not installed): {}", e);
                assert!(
                    e.to_string().contains("Claude") || e.to_string().contains("claude"),
                    "Error message should mention Claude"
                );
            }
        }
    }
    
    #[tokio::test]
    async fn test_multiple_task_executions_dont_block() {
        let pool = init_test_database().await.unwrap();
        let repo = Repository::new(pool);
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize git
        std::process::Command::new("git")
            .current_dir(temp_dir.path())
            .args(&["init"])
            .output()
            .unwrap();
        
        let automation = ClaudeAutomation::with_repository(
            temp_dir.path().to_path_buf(),
            repo.clone()
        );
        
        // Create multiple tasks
        let tasks: Vec<Task> = (0..3).map(|i| {
            Task::new(
                format!("Task {}", i),
                format!("Description for task {}", i)
            )
        }).collect();
        
        // Execute all tasks rapidly without blocking
        let start = std::time::Instant::now();
        
        for task in &tasks {
            repo.tasks.create(task).await.unwrap();
            let _ = automation.execute_task(task, "https://github.com/test/repo").await;
            // Small delay to avoid git conflicts
            sleep(Duration::from_millis(100)).await;
        }
        
        let elapsed = start.elapsed();
        
        // All tasks should be launched within a few seconds
        assert!(
            elapsed < Duration::from_secs(5),
            "Launching 3 tasks took {:?}, should be quick",
            elapsed
        );
        
        println!("✅ Launched {} tasks in {:?}", tasks.len(), elapsed);
    }
}