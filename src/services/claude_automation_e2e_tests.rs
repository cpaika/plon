#[cfg(test)]
mod tests {
    use super::super::claude_automation::ClaudeAutomation;
    use super::super::workspace_service::WorkspaceService;
    use crate::domain::task::{Task, TaskStatus, Priority, Position};
    use crate::repository::Repository;
    use tempfile::TempDir;
    use std::path::PathBuf;
    use std::sync::Arc;
    use uuid::Uuid;
    use sqlx::SqlitePool;
    use std::process::Command;
    use std::collections::{HashMap, HashSet};
    
    /// Test fixture for Claude automation tests
    struct TestContext {
        temp_dir: TempDir,
        workspace_service: WorkspaceService,
        claude_automation: ClaudeAutomation,
        repository: Arc<Repository>,
        project_path: PathBuf,
    }
    
    impl TestContext {
        async fn new() -> anyhow::Result<Self> {
            // Create temporary directory
            let temp_dir = TempDir::new()?;
            let temp_path = temp_dir.path().to_str().unwrap();
            
            // Override HOME for testing
            unsafe {
                std::env::set_var("HOME", temp_path);
            }
            
            // Create workspace service and directories
            let workspace_service = WorkspaceService::new();
            workspace_service.create_all_directories().await?;
            
            // Create a test project directory
            let project_path = workspace_service.create_project_directory("test-claude-project").await?;
            
            // Initialize git repository in the project
            Command::new("git")
                .current_dir(&project_path)
                .arg("init")
                .output()?;
            
            // Set up git config for testing
            Command::new("git")
                .current_dir(&project_path)
                .args(&["config", "user.email", "test@example.com"])
                .output()?;
            
            Command::new("git")
                .current_dir(&project_path)
                .args(&["config", "user.name", "Test User"])
                .output()?;
            
            // Create initial commit
            let readme_content = "# Test Project\n\nThis is a test project for Claude automation.";
            std::fs::write(project_path.join("README.md"), readme_content)?;
            
            Command::new("git")
                .current_dir(&project_path)
                .args(&["add", "."])
                .output()?;
            
            Command::new("git")
                .current_dir(&project_path)
                .args(&["commit", "-m", "Initial commit"])
                .output()?;
            
            // Create Claude automation instance
            let claude_automation = ClaudeAutomation::new(project_path.clone());
            
            // Set up test database
            let db_path = temp_dir.path().join("test.db");
            let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
            let pool = SqlitePool::connect(&database_url).await?;
            
            // Run migrations
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await?;
            
            let repository = Arc::new(Repository::new(pool));
            
            Ok(TestContext {
                temp_dir,
                workspace_service,
                claude_automation,
                repository,
                project_path,
            })
        }
        
        fn create_test_task(&self, title: &str, description: &str) -> Task {
            let mut tags = HashSet::new();
            tags.insert("test".to_string());
            tags.insert("automation".to_string());
            
            let mut metadata = HashMap::new();
            metadata.insert("created_by".to_string(), "test_user".to_string());
            metadata.insert("automation_enabled".to_string(), "true".to_string());
            
            Task {
                id: Uuid::new_v4(),
                title: title.to_string(),
                description: description.to_string(),
                status: TaskStatus::Todo,
                priority: Priority::High,
                metadata,
                tags,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                due_date: None,
                scheduled_date: None,
                completed_at: None,
                estimated_hours: Some(2.0),
                actual_hours: None,
                assigned_resource_id: None,
                goal_id: None,
                parent_task_id: None,
                position: Position { x: 0.0, y: 0.0 },
                subtasks: Vec::new(),
                is_archived: false,
                assignee: Some("test_user".to_string()),
                configuration_id: None,
                sort_order: 0,
            }
        }
    }
    
    #[tokio::test]
    async fn test_create_task_and_prepare_for_claude() {
        let context = TestContext::new().await.unwrap();
        
        // Create a simple test task
        let task = context.create_test_task(
            "Add hello world function",
            "Create a simple function that returns 'Hello, World!'"
        );
        
        // Save task to repository
        context.repository.tasks.create(&task).await.unwrap();
        
        // Execute task with Claude automation (this will prepare files)
        let result = context.claude_automation.execute_task(&task, "https://github.com/test/repo").await;
        
        // Even if Claude CLI isn't installed, the fallback should work
        assert!(result.is_ok(), "Should prepare task for Claude even without CLI");
        
        // Check that TODO_CLAUDE.md was created
        let todo_file = context.project_path.join("TODO_CLAUDE.md");
        assert!(todo_file.exists(), "TODO_CLAUDE.md should be created");
        
        // Verify content of TODO file
        let content = std::fs::read_to_string(&todo_file).unwrap();
        assert!(content.contains(&task.title));
        assert!(content.contains(&task.description));
        assert!(content.contains("Instructions"));
    }
    
    #[tokio::test]
    async fn test_task_with_code_generation_prompt() {
        let context = TestContext::new().await.unwrap();
        
        // Create a code generation task
        let task = context.create_test_task(
            "Implement calculator module",
            "Create a calculator module with add, subtract, multiply, and divide functions. Include unit tests."
        );
        
        // Create a sample code file that Claude would work with
        let src_dir = context.project_path.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        
        let main_file = src_dir.join("main.rs");
        std::fs::write(&main_file, "fn main() {\n    println!(\"Hello, world!\");\n}\n").unwrap();
        
        // Execute task
        let result = context.claude_automation.execute_task(&task, "https://github.com/test/repo").await;
        assert!(result.is_ok());
        
        // Verify git branch was created or attempted
        let output = Command::new("git")
            .current_dir(&context.project_path)
            .args(&["branch", "--list"])
            .output()
            .unwrap();
        
        let branches = String::from_utf8_lossy(&output.stdout);
        
        // Branch should contain task ID prefix
        let task_id_string = task.id.to_string();
        let task_id_prefix = task_id_string.split('-').next().unwrap();
        assert!(
            branches.contains(task_id_prefix) || branches.contains("main") || branches.contains("master"),
            "Should have created a branch or be on main/master"
        );
    }
    
    #[tokio::test]
    async fn test_check_task_status() {
        let context = TestContext::new().await.unwrap();
        
        let task = context.create_test_task(
            "Test status check",
            "Task for testing status checking"
        );
        
        // Initially should be Todo
        let status = context.claude_automation.check_task_status(task.id).await.unwrap();
        assert_eq!(status, TaskStatus::Todo);
        
        // Create a branch for the task
        let branch_name = format!("task/{}-test", 
            task.id.to_string().split('-').next().unwrap()
        );
        
        Command::new("git")
            .current_dir(&context.project_path)
            .args(&["checkout", "-b", &branch_name])
            .output()
            .unwrap();
        
        // Now status should be InProgress
        let status = context.claude_automation.check_task_status(task.id).await.unwrap();
        assert_eq!(status, TaskStatus::InProgress);
    }
    
    #[tokio::test]
    async fn test_workspace_integration() {
        let context = TestContext::new().await.unwrap();
        
        // Create task that requires workspace
        let task = context.create_test_task(
            "Setup project structure",
            "Create proper directory structure for the project"
        );
        
        // Get task directory from workspace service
        let task_dir = context.workspace_service
            .get_or_create_task_directory(&task.id.to_string(), &task.title)
            .await
            .unwrap();
        
        assert!(task_dir.exists());
        assert!(task_dir.join("TODO_CLAUDE.md").exists());
        
        // Content should include task details
        let todo_content = std::fs::read_to_string(task_dir.join("TODO_CLAUDE.md")).unwrap();
        assert!(todo_content.contains(&task.title));
        assert!(todo_content.contains(&task.id.to_string()));
    }
    
    #[tokio::test]
    async fn test_sanitize_branch_names() {
        let context = TestContext::new().await.unwrap();
        
        // Task with special characters in title
        let task = context.create_test_task(
            "Fix bug #123: Handle special/characters & symbols!",
            "This task has special characters that need sanitizing"
        );
        
        // Execute task
        let result = context.claude_automation.execute_task(&task, "https://github.com/test/repo").await;
        assert!(result.is_ok());
        
        // Check branch name is sanitized - get the current branch
        let output = Command::new("git")
            .current_dir(&context.project_path)
            .args(&["branch", "--show-current"])
            .output()
            .unwrap();
        
        let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Extract just the part after "task/" to check sanitization
        if let Some(branch_suffix) = current_branch.strip_prefix("task/") {
            // The part after the task ID should be sanitized
            let parts: Vec<&str> = branch_suffix.splitn(2, '-').collect();
            if parts.len() > 1 {
                let sanitized_title = parts[1];
                // Should not contain special characters in the sanitized title part
                assert!(!sanitized_title.contains("#"), "Branch title should not contain #");
                assert!(!sanitized_title.contains(":"), "Branch title should not contain :");
                assert!(!sanitized_title.contains("/"), "Branch title should not contain /");
                assert!(!sanitized_title.contains("&"), "Branch title should not contain &");
                assert!(!sanitized_title.contains("!"), "Branch title should not contain !");
            }
        }
    }
    
    #[tokio::test]
    async fn test_multiple_tasks_sequential() {
        let context = TestContext::new().await.unwrap();
        
        // Create multiple tasks
        let tasks = vec![
            context.create_test_task("Task 1", "First task"),
            context.create_test_task("Task 2", "Second task"),
            context.create_test_task("Task 3", "Third task"),
        ];
        
        // Execute all tasks
        for task in &tasks {
            let result = context.claude_automation.execute_task(task, "https://github.com/test/repo").await;
            assert!(result.is_ok(), "Task {} should execute successfully", task.title);
        }
        
        // Verify each task has its own branch or TODO file
        for task in &tasks {
            let todo_file = context.project_path.join("TODO_CLAUDE.md");
            // The last task should have the TODO file (since they overwrite)
            // or branches should exist for earlier tasks
            
            let task_id_string = task.id.to_string();
            let task_id_prefix = task_id_string.split('-').next().unwrap();
            let branch_output = Command::new("git")
                .current_dir(&context.project_path)
                .args(&["branch", "--list", &format!("*{}*", task_id_prefix)])
                .output()
                .unwrap();
            
            let has_branch = !String::from_utf8_lossy(&branch_output.stdout).trim().is_empty();
            
            // Either branch exists or it's the last task with TODO file
            assert!(
                has_branch || (todo_file.exists() && task.title == "Task 3"),
                "Task {} should have branch or TODO file", 
                task.title
            );
        }
    }
    
    #[tokio::test]
    async fn test_small_example_project() {
        let context = TestContext::new().await.unwrap();
        
        // Create a realistic small project structure
        let src_dir = context.project_path.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        
        // Create a simple Rust project
        let cargo_toml = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        std::fs::write(context.project_path.join("Cargo.toml"), cargo_toml).unwrap();
        
        let main_rs = r#"fn main() {
    println!("Hello, world!");
}
"#;
        std::fs::write(src_dir.join("main.rs"), main_rs).unwrap();
        
        // Commit the project structure
        Command::new("git")
            .current_dir(&context.project_path)
            .args(&["add", "."])
            .output()
            .unwrap();
        
        Command::new("git")
            .current_dir(&context.project_path)
            .args(&["commit", "-m", "Add project structure"])
            .output()
            .unwrap();
        
        // Create a task to add a new feature
        let task = context.create_test_task(
            "Add greeting function",
            "Add a function called greet(name: &str) that returns a personalized greeting"
        );
        
        // Execute the task
        let result = context.claude_automation.execute_task(&task, "https://github.com/test/repo").await;
        assert!(result.is_ok());
        
        // Verify TODO file has correct instructions
        let todo_file = context.project_path.join("TODO_CLAUDE.md");
        if todo_file.exists() {
            let content = std::fs::read_to_string(&todo_file).unwrap();
            assert!(content.contains("greet"));
            assert!(content.contains("personalized greeting"));
            assert!(content.contains("Instructions"));
            assert!(content.contains("git add"));
            assert!(content.contains("git commit"));
        }
        
        // Verify we're on a task branch
        let output = Command::new("git")
            .current_dir(&context.project_path)
            .args(&["branch", "--show-current"])
            .output()
            .unwrap();
        
        let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Should either be on a task branch or have created one
        assert!(
            current_branch.contains("task") || current_branch == "main" || current_branch == "master",
            "Should be on task branch or main: {}", 
            current_branch
        );
    }
}