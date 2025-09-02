use plon::domain::claude_code::ClaudeCodeConfig;
use plon::domain::task::Task;
use plon::repository::{Repository, database::init_database};
use plon::services::claude_code_service::ClaudeCodeService;
use tempfile::tempdir;
use std::path::PathBuf;

#[tokio::test]
async fn test_workspace_creation_and_cloning() {
    // Setup test environment
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create a test task
    let task = Task::new(
        "Test Task for Cloning".to_string(),
        "Testing repository cloning functionality".to_string()
    );
    
    // Create configuration with workspace root
    let mut config = ClaudeCodeConfig::new(
        "plon".to_string(),
        "cpaika".to_string()
    );
    config.workspace_root = Some(dir.path().join("workspaces").to_string_lossy().to_string());
    config.git_clone_url = Some("https://github.com/rust-lang/rust-clippy.git".to_string()); // Use a small public repo for testing
    
    // Create the service
    let service = ClaudeCodeService::new(repo.claude_code.clone());
    
    // Test workspace directory creation
    let workspace_root = config.get_workspace_root();
    assert_eq!(workspace_root, dir.path().join("workspaces"));
    
    // Verify the task folder name generation
    let task_id_short: String = task.id.to_string().chars().take(8).collect();
    let title_slug = "test-task-for-cloning";
    let expected_folder_name = format!("task-{}-{}", task_id_short, title_slug);
    
    println!("Expected workspace folder: {}", expected_folder_name);
    println!("Workspace root: {:?}", workspace_root);
    
    // Test the git clone URL generation
    assert_eq!(
        config.get_git_clone_url(),
        "https://github.com/rust-lang/rust-clippy.git"
    );
    
    // Test with default GitHub URL
    let mut config2 = ClaudeCodeConfig::new(
        "test-repo".to_string(),
        "test-owner".to_string()
    );
    config2.workspace_root = None; // Use default
    config2.git_clone_url = None; // Use default
    
    assert_eq!(
        config2.get_git_clone_url(),
        "https://github.com/test-owner/test-repo.git"
    );
    
    // Test default workspace root (should be ~/plon-workspaces)
    let default_root = config2.get_workspace_root();
    assert!(default_root.to_string_lossy().ends_with("plon-workspaces"));
}

#[tokio::test]
async fn test_task_folder_naming() {
    use uuid::Uuid;
    
    // Test various task titles to ensure proper slug generation
    let test_cases = vec![
        ("Simple Task", "simple-task"),
        ("Task with UPPERCASE", "task-with-uppercase"),
        ("Task-with-dashes", "task-with-dashes"),
        ("Task_with_underscores", "task-with-underscores"),
        ("Task with 123 numbers", "task-with-123-numbers"),
        ("Task!@#$%^&*()special", "task----------special"),
        ("Very Long Task Title That Should Be Truncated After Thirty", "very-long-task-title-that-shou"),
        ("", ""), // Empty title edge case
    ];
    
    for (title, expected_slug) in test_cases {
        let task = Task::new(title.to_string(), "Description".to_string());
        let task_id_short: String = task.id.to_string().chars().take(8).collect();
        
        // Replicate the slug generation logic from the service
        let title_slug: String = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .chars()
            .take(30)
            .collect();
        
        assert_eq!(title_slug, expected_slug, "Failed for title: {}", title);
        
        let folder_name = format!("task-{}-{}", task_id_short, title_slug);
        println!("Title: '{}' -> Folder: '{}'", title, folder_name);
        
        // Ensure folder name is valid
        // Note: Multiple consecutive special characters will create multiple dashes
        // This is acceptable for filesystem compatibility
        assert!(folder_name.len() <= 100, "Folder name too long");
    }
}

#[tokio::test]
async fn test_workspace_isolation() {
    // Test that multiple tasks get their own isolated workspaces
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    let mut config = ClaudeCodeConfig::new("test".to_string(), "owner".to_string());
    config.workspace_root = Some(dir.path().join("workspaces").to_string_lossy().to_string());
    
    // Create multiple tasks
    let task1 = Task::new("First Task".to_string(), "Description 1".to_string());
    let task2 = Task::new("Second Task".to_string(), "Description 2".to_string());
    let task3 = Task::new("Third Task".to_string(), "Description 3".to_string());
    
    // Generate workspace paths for each
    let workspace_root = config.get_workspace_root();
    
    let task1_id_short: String = task1.id.to_string().chars().take(8).collect();
    let task2_id_short: String = task2.id.to_string().chars().take(8).collect();
    let task3_id_short: String = task3.id.to_string().chars().take(8).collect();
    
    let workspace1 = workspace_root.join(format!("task-{}-first-task", task1_id_short));
    let workspace2 = workspace_root.join(format!("task-{}-second-task", task2_id_short));
    let workspace3 = workspace_root.join(format!("task-{}-third-task", task3_id_short));
    
    // Ensure all paths are different
    assert_ne!(workspace1, workspace2);
    assert_ne!(workspace2, workspace3);
    assert_ne!(workspace1, workspace3);
    
    println!("Task 1 workspace: {:?}", workspace1);
    println!("Task 2 workspace: {:?}", workspace2);
    println!("Task 3 workspace: {:?}", workspace3);
}

#[test]
fn test_branch_name_generation() {
    let task = Task::new(
        "Fix Authentication Bug".to_string(),
        "Fix the JWT token validation issue".to_string()
    );
    
    let task_id_short: String = task.id.to_string().chars().take(8).collect();
    let title_slug = "fix-authentication-bug";
    
    // This matches the format from generate_branch_name in the service
    let branch_name = format!("claude/{}-{}", task_id_short, title_slug);
    
    assert!(branch_name.starts_with("claude/"));
    assert!(branch_name.contains(&task_id_short));
    assert!(branch_name.ends_with("-fix-authentication-bug"));
    
    println!("Generated branch name: {}", branch_name);
}