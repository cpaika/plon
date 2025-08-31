use plon::repository::{Repository, database::init_database};
use plon::domain::task::{Task, TaskStatus, Position, Priority};
use plon::domain::dependency::{Dependency, DependencyType};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    println!("Testing persistence...");
    
    // Initialize database
    let pool = init_database("test_plon.db").await.expect("Failed to init database");
    let repo = Repository::new(pool);
    
    // Create a task
    let task1 = Task {
        id: Uuid::new_v4(),
        title: "Task 1".to_string(),
        description: "First task".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::High,
        position: Position { x: 100.0, y: 100.0 },
        ..Default::default()
    };
    
    let task2 = Task {
        id: Uuid::new_v4(),
        title: "Task 2".to_string(),
        description: "Second task".to_string(),
        status: TaskStatus::InProgress,
        priority: Priority::Medium,
        position: Position { x: 300.0, y: 100.0 },
        ..Default::default()
    };
    
    // Save tasks
    println!("Creating tasks...");
    repo.tasks.create(&task1).await.expect("Failed to create task1");
    repo.tasks.create(&task2).await.expect("Failed to create task2");
    
    // Create dependency
    let dep = Dependency::new(task1.id, task2.id, DependencyType::FinishToStart);
    println!("Creating dependency from {} to {}", task1.id, task2.id);
    repo.dependencies.create(&dep).await.expect("Failed to create dependency");
    
    // List all tasks
    use plon::repository::task_repository::TaskFilters;
    let tasks = repo.tasks.list(TaskFilters::default()).await.expect("Failed to list tasks");
    println!("\nTasks in database: {}", tasks.len());
    for task in &tasks {
        println!("  - {} at ({}, {})", task.title, task.position.x, task.position.y);
    }
    
    // List all dependencies
    let deps = repo.dependencies.list_all().await.expect("Failed to list dependencies");
    println!("\nDependencies in database: {}", deps.len());
    for dep in &deps {
        println!("  - {} -> {}", dep.from_task_id, dep.to_task_id);
    }
    
    println!("\n✅ Persistence test successful!");
}