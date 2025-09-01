use plon::services::ClaudeAutomation;
use plon::domain::task::{Task, TaskStatus, Priority};
use std::env::current_dir;

#[tokio::main]
async fn main() {
    println!("Testing Claude Code integration...");
    
    // Create a test task
    let task = Task::new(
        "Test Claude Integration".to_string(),
        "This is a test task to verify Claude Code CLI integration works correctly.".to_string()
    );
    
    // Get current directory as workspace
    let workspace_dir = current_dir().unwrap();
    println!("Workspace: {:?}", workspace_dir);
    
    // Create automation service
    let automation = ClaudeAutomation::new(workspace_dir);
    
    // Try to execute the task
    println!("\nAttempting to launch Claude Code for task: {}", task.title);
    match automation.execute_task(&task, "https://github.com/test/repo").await {
        Ok(execution_id) => {
            println!("✅ SUCCESS! Claude Code launched successfully");
            println!("Execution ID: {}", execution_id);
        }
        Err(e) => {
            println!("❌ ERROR: {}", e);
        }
    }
}