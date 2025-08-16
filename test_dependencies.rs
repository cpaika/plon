use plon::repository::{Repository, database};
use plon::services::{TaskService, DependencyService};
use plon::domain::task::Task;
use plon::domain::dependency::DependencyType;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Dependency System ===\n");
    
    // Initialize database
    let pool = database::init_database("test_dependencies.db").await?;
    let repository = Arc::new(Repository::new(pool.clone()));
    let task_service = TaskService::new(repository.clone());
    let dependency_service = DependencyService::new(repository.clone());
    
    // Create two tasks
    let task1 = Task::new("Task 1".to_string(), "First task".to_string());
    let task2 = Task::new("Task 2".to_string(), "Second task".to_string());
    
    let task1_id = task1.id;
    let task2_id = task2.id;
    
    println!("Creating tasks...");
    task_service.create(task1).await?;
    task_service.create(task2).await?;
    println!("  ✓ Created Task 1: {}", task1_id);
    println!("  ✓ Created Task 2: {}", task2_id);
    
    // Create a dependency between them
    println!("\nCreating dependency: Task 1 -> Task 2 (FinishToStart)");
    let dependency = dependency_service.create_dependency(
        task1_id,
        task2_id,
        DependencyType::FinishToStart,
    ).await?;
    println!("  ✓ Dependency created: {}", dependency.id);
    
    // Verify dependency exists
    println!("\nVerifying dependencies...");
    let deps_for_task2 = dependency_service.get_dependencies_for_task(task2_id).await?;
    println!("  Dependencies for Task 2: {} found", deps_for_task2.len());
    for dep in &deps_for_task2 {
        println!("    - From Task {} (type: {:?})", dep.from_task_id, dep.dependency_type);
    }
    
    let dependents_of_task1 = dependency_service.get_dependents_for_task(task1_id).await?;
    println!("  Dependents of Task 1: {} found", dependents_of_task1.len());
    for dep in &dependents_of_task1 {
        println!("    - To Task {} (type: {:?})", dep.to_task_id, dep.dependency_type);
    }
    
    // Build dependency graph
    println!("\nBuilding dependency graph...");
    let graph = dependency_service.build_dependency_graph().await?;
    println!("  ✓ Graph built successfully");
    
    // Check for cycles (should be none)
    println!("\nChecking for cycles...");
    let has_cycle = dependency_service.check_for_cycles(task2_id, task1_id).await?;
    if has_cycle {
        println!("  ⚠ Cycle detected (creating reverse dependency would create a cycle)");
    } else {
        println!("  ✓ No cycles detected");
    }
    
    // Clean up
    pool.close().await;
    std::fs::remove_file("test_dependencies.db").ok();
    
    println!("\n=== Test Complete ===");
    Ok(())
}