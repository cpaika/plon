#[cfg(test)]
mod claude_hanging_tests {
    use plon::services::ClaudeAutomation;
    use plon::domain::task::Task;
    use std::time::{Duration, Instant};
    use std::path::PathBuf;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_claude_command_should_not_hang() {
        // Create a test task
        let task = Task::new(
            "Test Task".to_string(),
            "Test description".to_string()
        );
        
        // Use temp dir as workspace
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().to_path_buf();
        
        // Initialize git in temp dir to avoid git-related errors
        std::process::Command::new("git")
            .current_dir(&workspace_dir)
            .arg("init")
            .output()
            .ok();
        
        // Create automation service
        let automation = ClaudeAutomation::new(workspace_dir);
        
        // Try to execute task with timeout - should complete within 5 seconds
        let start = Instant::now();
        let result = timeout(
            Duration::from_secs(5),
            automation.execute_task(&task, "https://github.com/test/repo")
        ).await;
        
        let elapsed = start.elapsed();
        
        // Test assertions
        match result {
            Ok(_) => {
                // Command completed (either success or error)
                println!("Command completed in {:?}", elapsed);
                assert!(elapsed < Duration::from_secs(5), "Command took too long");
            }
            Err(_) => {
                // Timeout occurred - this means the command hung
                panic!("Claude command hung for more than 5 seconds! This indicates the --print flag might be waiting for input.");
            }
        }
    }

    #[tokio::test] 
    async fn test_claude_command_with_mock() {
        // Test what command is actually being executed
        let task = Task::new(
            "Test Task".to_string(), 
            "Test description".to_string()
        );
        
        let temp_dir = tempfile::TempDir::new().unwrap();
        
        // Create a mock claude script that logs what it receives
        let mock_claude = temp_dir.path().join("claude");
        std::fs::write(&mock_claude, r#"#!/bin/bash
echo "Mock claude called with args: $@" > /tmp/claude_test_args.txt
echo "Mock response"
exit 0
"#).unwrap();
        
        // Make it executable
        std::process::Command::new("chmod")
            .arg("+x")
            .arg(&mock_claude)
            .output()
            .unwrap();
        
        // We need to test what arguments are being passed
        // This will help us understand why it might be hanging
    }
}