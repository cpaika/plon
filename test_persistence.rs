use plon::repository::{Repository, database};
use plon::services::TaskService;
use plon::domain::task::Task;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Task Persistence ===\n");
    
    // Step 1: Initialize database and create a task
    println!("Step 1: Creating database and adding a task...");
    let task_id = {
        let pool = database::init_database("plon.db").await?;
        let repository = Arc::new(Repository::new(pool.clone()));
        let service = TaskService::new(repository);
        
        let task = Task::new("Test Task".to_string(), "This should persist".to_string());
        let task_id = task.id;
        
        service.create(task).await?;
        println!("  ✓ Created task with ID: {}", task_id);
        
        // Verify it was saved
        let loaded = service.get(task_id).await?;
        if loaded.is_some() {
            println!("  ✓ Task found in database immediately after creation");
        } else {
            println!("  ✗ Task NOT found after creation!");
        }
        
        pool.close().await;
        task_id
    };
    
    println!("\n--- Simulating application restart ---\n");
    
    // Step 2: Reconnect to database and verify task persists
    println!("Step 2: Reopening database and checking for task...");
    {
        let pool = database::init_database("plon.db").await?;
        let repository = Arc::new(Repository::new(pool.clone()));
        let service = TaskService::new(repository);
        
        // List all tasks
        let all_tasks = service.list_all().await?;
        println!("  Found {} total tasks in database", all_tasks.len());
        
        // Try to find our specific task
        let loaded = service.get(task_id).await?;
        if let Some(task) = loaded {
            println!("  ✓ Task persisted after restart!");
            println!("    - Title: {}", task.title);
            println!("    - Description: {}", task.description);
        } else {
            println!("  ✗ Task was NOT persisted!");
        }
        
        pool.close().await;
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}