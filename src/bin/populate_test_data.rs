use plon::domain::task::{Task, TaskStatus, Priority, Position};
use plon::repository::Repository;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Populating database with test data...");
    
    // Connect to the actual database
    let pool = SqlitePool::connect("sqlite:plon.db").await?;
    let repo = Repository::new(pool);
    
    // Create many tasks to stress test the list view
    let task_count = 500;
    println!("Creating {} tasks...", task_count);
    
    for i in 0..task_count {
        let task = Task {
            id: Uuid::new_v4(),
            title: format!("Task #{} - {}", i, generate_title(i)),
            description: format!(
                "This is a detailed description for task {}. It contains important information about what needs to be done. {}",
                i,
                generate_description(i)
            ),
            status: match i % 5 {
                0 => TaskStatus::Todo,
                1 => TaskStatus::InProgress,
                2 => TaskStatus::Done,
                3 => TaskStatus::Blocked,
                _ => TaskStatus::Review,
            },
            priority: match i % 4 {
                0 => Priority::Critical,
                1 => Priority::High,
                2 => Priority::Medium,
                _ => Priority::Low,
            },
            position: Position { 
                x: (i as f64 % 10.0) * 100.0, 
                y: (i as f64 / 10.0) * 100.0 
            },
            metadata: HashMap::new(),
            tags: generate_tags(i),
            created_at: Utc::now() - chrono::Duration::days(i as i64 % 30),
            updated_at: Utc::now() - chrono::Duration::hours(i as i64 % 24),
            due_date: if i % 3 == 0 {
                Some(Utc::now() + chrono::Duration::days((i % 14) as i64))
            } else {
                None
            },
            scheduled_date: if i % 4 == 0 {
                Some(Utc::now() + chrono::Duration::days((i % 7) as i64))
            } else {
                None
            },
            completed_at: if i % 5 == 2 {
                Some(Utc::now() - chrono::Duration::days((i % 10) as i64))
            } else {
                None
            },
            estimated_hours: Some(((i % 8) + 1) as f32),
            actual_hours: if i % 5 == 2 {
                Some(((i % 8) + 2) as f32)
            } else {
                None
            },
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            subtasks: vec![],
            is_archived: false,
            assignee: if i % 2 == 0 {
                Some(format!("user{}@example.com", i % 5))
            } else {
                None
            },
            configuration_id: None,
            sort_order: i as i32,
        };
        
        match repo.tasks.create(&task).await {
            Ok(_) => {
                if i % 50 == 0 {
                    println!("Created {} tasks...", i + 1);
                }
            }
            Err(e) => {
                eprintln!("Failed to create task {}: {}", i, e);
            }
        }
    }
    
    println!("✅ Successfully populated database with {} tasks!", task_count);
    println!("You can now test the list view performance with a large dataset.");
    
    Ok(())
}

fn generate_title(index: usize) -> String {
    let titles = vec![
        "Implement new feature",
        "Fix critical bug",
        "Update documentation",
        "Refactor codebase",
        "Write unit tests",
        "Code review",
        "Deploy to production",
        "Optimize performance",
        "Database migration",
        "Security audit",
    ];
    titles[index % titles.len()].to_string()
}

fn generate_description(index: usize) -> String {
    let descriptions = vec![
        "This requires careful attention to detail and thorough testing.",
        "We need to coordinate with the team before proceeding.",
        "This is blocking several other tasks and needs immediate attention.",
        "Consider the performance implications of this change.",
        "Make sure to update all related documentation.",
    ];
    descriptions[index % descriptions.len()].to_string()
}

fn generate_tags(index: usize) -> HashSet<String> {
    let mut tags = HashSet::new();
    tags.insert(format!("tag{}", index % 10));
    tags.insert("common".to_string());
    
    if index % 3 == 0 {
        tags.insert("urgent".to_string());
    }
    if index % 4 == 0 {
        tags.insert("backend".to_string());
    }
    if index % 5 == 0 {
        tags.insert("frontend".to_string());
    }
    
    tags
}