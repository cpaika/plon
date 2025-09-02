use plon::domain::dependency::{Dependency, DependencyType};
use plon::domain::task::{Task, TaskStatus, Priority};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_dependency_creation_and_persistence() {
    // Initialize test database
    let pool = init_test_database().await.unwrap();
    let repo = Repository::new(pool);
    
    // Create two tasks
    let task1 = Task {
        id: Uuid::new_v4(),
        title: "Task 1".to_string(),
        description: "First task".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: (100.0, 100.0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec![],
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
    };
    
    let task2 = Task {
        id: Uuid::new_v4(),
        title: "Task 2".to_string(),
        description: "Second task".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: (300.0, 100.0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec![],
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
    };
    
    // Save tasks
    repo.tasks.create(&task1).await.unwrap();
    repo.tasks.create(&task2).await.unwrap();
    
    // Create dependency from task1 to task2
    let dependency = Dependency {
        id: Uuid::new_v4(),
        from_task_id: task1.id,
        to_task_id: task2.id,
        dependency_type: DependencyType::FinishToStart,
        created_at: Utc::now(),
    };
    
    // Save dependency
    repo.dependencies.create(&dependency).await.unwrap();
    
    // Verify dependency was saved
    let loaded_deps = repo.dependencies.get_by_task(task1.id).await.unwrap();
    assert_eq!(loaded_deps.len(), 1);
    assert_eq!(loaded_deps[0].from_task_id, task1.id);
    assert_eq!(loaded_deps[0].to_task_id, task2.id);
    
    // Verify all dependencies can be loaded
    let all_deps = repo.dependencies.get_all().await.unwrap();
    assert!(all_deps.len() >= 1);
    
    // Verify tasks can be loaded with their positions
    let loaded_task1 = repo.tasks.get_by_id(task1.id).await.unwrap().unwrap();
    assert_eq!(loaded_task1.position, (100.0, 100.0));
    
    let loaded_task2 = repo.tasks.get_by_id(task2.id).await.unwrap().unwrap();
    assert_eq!(loaded_task2.position, (300.0, 100.0));
}

#[tokio::test]
async fn test_task_position_persistence() {
    // Initialize test database
    let pool = init_test_database().await.unwrap();
    let repo = Repository::new(pool);
    
    // Create a task
    let mut task = Task {
        id: Uuid::new_v4(),
        title: "Movable Task".to_string(),
        description: "Task that moves".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: (50.0, 50.0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec![],
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
    };
    
    // Save task
    repo.tasks.create(&task).await.unwrap();
    
    // Update position
    task.position = (250.0, 350.0);
    task.updated_at = Utc::now();
    repo.tasks.update(&task).await.unwrap();
    
    // Load and verify position
    let loaded_task = repo.tasks.get_by_id(task.id).await.unwrap().unwrap();
    assert_eq!(loaded_task.position, (250.0, 350.0));
}

#[tokio::test]
async fn test_duplicate_dependency_prevention() {
    // Initialize test database
    let pool = init_test_database().await.unwrap();
    let repo = Repository::new(pool);
    
    // Create two tasks
    let task1 = Task {
        id: Uuid::new_v4(),
        title: "Task A".to_string(),
        description: "".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: (0.0, 0.0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec![],
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
    };
    
    let task2 = Task {
        id: Uuid::new_v4(),
        title: "Task B".to_string(),
        description: "".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: (100.0, 0.0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec![],
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
    };
    
    repo.tasks.create(&task1).await.unwrap();
    repo.tasks.create(&task2).await.unwrap();
    
    // Create first dependency
    let dep1 = Dependency {
        id: Uuid::new_v4(),
        from_task_id: task1.id,
        to_task_id: task2.id,
        dependency_type: DependencyType::FinishToStart,
        created_at: Utc::now(),
    };
    
    repo.dependencies.create(&dep1).await.unwrap();
    
    // Try to create duplicate dependency (should fail due to unique constraint)
    let dep2 = Dependency {
        id: Uuid::new_v4(),
        from_task_id: task1.id,
        to_task_id: task2.id,
        dependency_type: DependencyType::FinishToStart,
        created_at: Utc::now(),
    };
    
    let result = repo.dependencies.create(&dep2).await;
    assert!(result.is_err(), "Should not allow duplicate dependencies");
    
    // Verify only one dependency exists
    let deps = repo.dependencies.get_by_task(task1.id).await.unwrap();
    assert_eq!(deps.len(), 1);
}