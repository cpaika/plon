use plon::domain::task::{Task, TaskStatus, Priority, Position};
use plon::domain::dependency::{Dependency, DependencyType};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use uuid::Uuid;
use chrono::Utc;
use std::collections::{HashMap, HashSet};

#[tokio::test]
async fn test_task_persistence() {
    let pool = init_test_database().await.unwrap();
    let repo = Repository::new(pool);
    
    // Create a task
    let task = Task {
        id: Uuid::new_v4(),
        title: "Test Task".to_string(),
        description: "Test Description".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: Position { x: 100.0, y: 200.0 },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: HashMap::new(),
        tags: HashSet::new(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 0,
        subtasks: Vec::new(),
    };
    
    // Save task
    repo.tasks.create(&task).await.unwrap();
    
    // Load task
    let loaded = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(loaded.title, "Test Task");
    assert_eq!(loaded.position.x, 100.0);
    assert_eq!(loaded.position.y, 200.0);
}

#[tokio::test]
async fn test_dependency_persistence() {
    let pool = init_test_database().await.unwrap();
    let repo = Repository::new(pool);
    
    // Create two tasks
    let task1 = Task {
        id: Uuid::new_v4(),
        title: "Task 1".to_string(),
        description: String::new(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: Position { x: 100.0, y: 100.0 },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: HashMap::new(),
        tags: HashSet::new(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 0,
        subtasks: Vec::new(),
    };
    
    let task2 = Task {
        id: Uuid::new_v4(),
        title: "Task 2".to_string(),
        description: String::new(),
        status: TaskStatus::InProgress,
        priority: Priority::High,
        position: Position { x: 300.0, y: 100.0 },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: HashMap::new(),
        tags: HashSet::new(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 1,
        subtasks: Vec::new(),
    };
    
    // Save tasks
    repo.tasks.create(&task1).await.unwrap();
    repo.tasks.create(&task2).await.unwrap();
    
    // Create dependency
    let dependency = Dependency {
        id: Uuid::new_v4(),
        from_task_id: task1.id,
        to_task_id: task2.id,
        dependency_type: DependencyType::FinishToStart,
        created_at: Utc::now(),
    };
    
    // Save dependency
    repo.dependencies.create(&dependency).await.unwrap();
    
    // Load dependencies
    let deps = repo.dependencies.list_all().await.unwrap();
    assert!(deps.len() >= 1);
    
    // Verify the dependency exists
    let found = deps.iter().any(|d| {
        d.from_task_id == task1.id && 
        d.to_task_id == task2.id &&
        d.dependency_type == DependencyType::FinishToStart
    });
    assert!(found, "Dependency should be persisted");
}

#[tokio::test]
async fn test_task_position_update() {
    let pool = init_test_database().await.unwrap();
    let repo = Repository::new(pool);
    
    // Create a task
    let mut task = Task {
        id: Uuid::new_v4(),
        title: "Movable Task".to_string(),
        description: String::new(),
        status: TaskStatus::Todo,
        priority: Priority::Low,
        position: Position { x: 50.0, y: 50.0 },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: HashMap::new(),
        tags: HashSet::new(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 0,
        subtasks: Vec::new(),
    };
    
    // Save task
    repo.tasks.create(&task).await.unwrap();
    
    // Update position
    task.position = Position { x: 250.0, y: 350.0 };
    task.updated_at = Utc::now();
    repo.tasks.update(&task).await.unwrap();
    
    // Load and verify
    let loaded = repo.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(loaded.position.x, 250.0);
    assert_eq!(loaded.position.y, 350.0);
}

#[tokio::test]
async fn test_duplicate_dependency_prevention() {
    let pool = init_test_database().await.unwrap();
    let repo = Repository::new(pool);
    
    // Create two tasks
    let task1_id = Uuid::new_v4();
    let task2_id = Uuid::new_v4();
    
    let task1 = Task {
        id: task1_id,
        title: "Task A".to_string(),
        description: String::new(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: Position { x: 0.0, y: 0.0 },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: HashMap::new(),
        tags: HashSet::new(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 0,
        subtasks: Vec::new(),
    };
    
    let task2 = Task {
        id: task2_id,
        title: "Task B".to_string(),
        description: String::new(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: Position { x: 100.0, y: 0.0 },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: HashMap::new(),
        tags: HashSet::new(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 1,
        subtasks: Vec::new(),
    };
    
    repo.tasks.create(&task1).await.unwrap();
    repo.tasks.create(&task2).await.unwrap();
    
    // Create first dependency
    let dep1 = Dependency {
        id: Uuid::new_v4(),
        from_task_id: task1_id,
        to_task_id: task2_id,
        dependency_type: DependencyType::FinishToStart,
        created_at: Utc::now(),
    };
    
    repo.dependencies.create(&dep1).await.unwrap();
    
    // Try to create duplicate (should fail)
    let dep2 = Dependency {
        id: Uuid::new_v4(),
        from_task_id: task1_id,
        to_task_id: task2_id,
        dependency_type: DependencyType::FinishToStart,
        created_at: Utc::now(),
    };
    
    let result = repo.dependencies.create(&dep2).await;
    assert!(result.is_err(), "Should not allow duplicate dependencies");
}