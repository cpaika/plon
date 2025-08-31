use crate::domain::task::{Task, TaskStatus, Position, Priority};
use uuid::Uuid;
use chrono::Utc;

// Simple state without Fermi for now
#[derive(Clone, Debug, PartialEq)]
pub enum ViewType {
    Map,
    List,
    Kanban,
    Timeline,
    Gantt,
    Dashboard,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaskExecutionStatus {
    Running,
    Completed,
    Failed(String),
}

// Create some sample data for demo
pub fn sample_tasks() -> Vec<Task> {
    vec![
        // Todo tasks
        Task {
            id: Uuid::new_v4(),
            title: "Design dashboard layout".to_string(),
            description: "Create mockups for the main dashboard".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            due_date: None,
            position: Position { x: 100.0, y: 100.0 },
            sort_order: 100,
            ..Default::default()
        },
        Task {
            id: Uuid::new_v4(),
            title: "Set up CI/CD pipeline".to_string(),
            description: "".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::Low,
            due_date: None,
            position: Position { x: 100.0, y: 200.0 },
            sort_order: 200,
            ..Default::default()
        },
        
        // In Progress tasks
        Task {
            id: Uuid::new_v4(),
            title: "Implement data models".to_string(),
            description: "Create database schema and ORM models".to_string(),
            status: TaskStatus::InProgress,
            priority: Priority::Critical,
            due_date: Some(Utc::now() + chrono::Duration::days(2)),
            position: Position { x: 300.0, y: 100.0 },
            sort_order: 100,
            ..Default::default()
        },
        Task {
            id: Uuid::new_v4(),
            title: "Implement user authentication".to_string(),
            description: "Add login and registration functionality".to_string(),
            status: TaskStatus::InProgress,
            priority: Priority::High,
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            position: Position { x: 300.0, y: 200.0 },
            sort_order: 200,
            ..Default::default()
        },
        
        // Review task
        Task {
            id: Uuid::new_v4(),
            title: "API documentation".to_string(),
            description: "Document all REST endpoints".to_string(),
            status: TaskStatus::Review,
            priority: Priority::Medium,
            due_date: None,
            position: Position { x: 500.0, y: 100.0 },
            sort_order: 100,
            ..Default::default()
        },
        
        // Done tasks
        Task {
            id: Uuid::new_v4(),
            title: "Project setup".to_string(),
            description: "".to_string(),
            status: TaskStatus::Done,
            priority: Priority::High,
            due_date: None,
            position: Position { x: 700.0, y: 100.0 },
            sort_order: 100,
            ..Default::default()
        },
        Task {
            id: Uuid::new_v4(),
            title: "Requirements gathering".to_string(),
            description: "".to_string(),
            status: TaskStatus::Done,
            priority: Priority::High,
            due_date: None,
            position: Position { x: 700.0, y: 200.0 },
            sort_order: 200,
            ..Default::default()
        },
    ]
}