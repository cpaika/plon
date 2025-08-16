use plon::domain::task::{Task, TaskStatus, Priority};
use plon::domain::goal::{Goal, GoalStatus};
use plon::domain::resource::Resource;
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::services::{TaskService, GoalService, ResourceService};
use std::sync::Arc;
use chrono::Utc;

#[tokio::test]
async fn test_dashboard_statistics() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks with various statuses
    let task_distribution = vec![
        (TaskStatus::Todo, 10),
        (TaskStatus::InProgress, 5),
        (TaskStatus::Review, 3),
        (TaskStatus::Done, 15),
        (TaskStatus::Blocked, 2),
    ];
    
    for (status, count) in task_distribution {
        for i in 0..count {
            let mut task = Task::new(format!("{:?} Task {}", status, i), "".to_string());
            task.status = status;
            service.create(task).await.unwrap();
        }
    }
    
    // Calculate statistics
    let all_tasks = service.list_all().await.unwrap();
    let total = all_tasks.len();
    let completed = all_tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
    let in_progress = all_tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
    let blocked = all_tasks.iter().filter(|t| t.status == TaskStatus::Blocked).count();
    let todo = all_tasks.iter().filter(|t| t.status == TaskStatus::Todo).count();
    let review = all_tasks.iter().filter(|t| t.status == TaskStatus::Review).count();
    
    assert_eq!(total, 35);
    assert_eq!(completed, 15);
    assert_eq!(in_progress, 5);
    assert_eq!(blocked, 2);
    assert_eq!(todo, 10);
    assert_eq!(review, 3);
    
    // Calculate completion percentage
    let completion_rate = (completed as f32 / total as f32) * 100.0;
    assert!((completion_rate - 42.857).abs() < 0.01);
}

#[tokio::test]
async fn test_dashboard_overdue_tasks() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks with various due dates
    let mut overdue_task1 = Task::new("Overdue 1".to_string(), "".to_string());
    overdue_task1.due_date = Some(Utc::now() - chrono::Duration::days(5));
    
    let mut overdue_task2 = Task::new("Overdue 2".to_string(), "".to_string());
    overdue_task2.due_date = Some(Utc::now() - chrono::Duration::days(1));
    overdue_task2.status = TaskStatus::InProgress;
    
    let mut upcoming_task = Task::new("Upcoming".to_string(), "".to_string());
    upcoming_task.due_date = Some(Utc::now() + chrono::Duration::days(2));
    
    let mut completed_overdue = Task::new("Completed Overdue".to_string(), "".to_string());
    completed_overdue.due_date = Some(Utc::now() - chrono::Duration::days(3));
    completed_overdue.status = TaskStatus::Done;
    
    service.create(overdue_task1).await.unwrap();
    service.create(overdue_task2).await.unwrap();
    service.create(upcoming_task).await.unwrap();
    service.create(completed_overdue).await.unwrap();
    
    // Check overdue tasks
    let all_tasks = service.list_all().await.unwrap();
    let overdue_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.is_overdue())
        .collect();
    
    assert_eq!(overdue_tasks.len(), 2);
    assert!(overdue_tasks.iter().any(|t| t.title == "Overdue 1"));
    assert!(overdue_tasks.iter().any(|t| t.title == "Overdue 2"));
}

#[tokio::test]
async fn test_dashboard_goal_progress() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let task_service = Arc::new(TaskService::new(repository.clone()));
    let goal_service = GoalService::new(repository);
    
    // Create a goal
    let mut goal = Goal::new("Q1 Objectives".to_string(), "Complete Q1 goals".to_string());
    
    // Create tasks and add to goal
    let task_ids: Vec<_> = (0..5).map(|i| {
        let mut task = Task::new(format!("Goal Task {}", i), "".to_string());
        if i < 3 {
            task.status = TaskStatus::Done;
        }
        task.id
    }).collect();
    
    for (i, id) in task_ids.iter().enumerate() {
        let mut task = Task::new(format!("Goal Task {}", i), "".to_string());
        task.id = *id;
        if i < 3 {
            task.status = TaskStatus::Done;
        }
        task_service.create(task).await.unwrap();
        goal.add_task(*id);
    }
    
    // Calculate goal progress
    let task_statuses: Vec<_> = task_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i < 3))
        .collect();
    
    let progress = goal.calculate_progress(&task_statuses);
    assert!((progress - 60.0).abs() < 0.01);
    
    // Check if goal is at risk
    goal.target_date = Some(Utc::now() + chrono::Duration::days(3));
    assert!(goal.is_at_risk());
    
    goal.target_date = Some(Utc::now() + chrono::Duration::days(30));
    assert!(!goal.is_at_risk());
}

#[tokio::test]
async fn test_dashboard_resource_utilization() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let resource_service = ResourceService::new(repository);
    
    // Create resources with different utilization levels
    let mut developer1 = Resource::new("Alice".to_string(), "Developer".to_string(), 40.0);
    developer1.current_load = 35.0;
    
    let mut developer2 = Resource::new("Bob".to_string(), "Developer".to_string(), 40.0);
    developer2.current_load = 45.0; // Overloaded
    
    let mut designer = Resource::new("Carol".to_string(), "Designer".to_string(), 30.0);
    designer.current_load = 15.0;
    
    resource_service.create(developer1.clone()).await.unwrap();
    resource_service.create(developer2.clone()).await.unwrap();
    resource_service.create(designer.clone()).await.unwrap();
    
    // Calculate utilization metrics
    let all_resources = resource_service.list_all().await.unwrap();
    
    let total_capacity: f32 = all_resources.iter().map(|r| r.weekly_hours).sum();
    let total_load: f32 = all_resources.iter().map(|r| r.current_load).sum();
    let overall_utilization = (total_load / total_capacity) * 100.0;
    
    assert_eq!(total_capacity, 110.0);
    assert_eq!(total_load, 95.0);
    assert!((overall_utilization - 86.36).abs() < 0.01);
    
    // Check overloaded resources
    let overloaded: Vec<_> = all_resources
        .iter()
        .filter(|r| r.is_overloaded())
        .collect();
    
    assert_eq!(overloaded.len(), 1);
    assert_eq!(overloaded[0].name, "Bob");
}

#[tokio::test]
async fn test_dashboard_priority_distribution() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks with different priorities
    let priority_distribution = vec![
        (Priority::Critical, 2),
        (Priority::High, 5),
        (Priority::Medium, 10),
        (Priority::Low, 8),
    ];
    
    for (priority, count) in priority_distribution {
        for i in 0..count {
            let mut task = Task::new(format!("{:?} Priority {}", priority, i), "".to_string());
            task.priority = priority;
            service.create(task).await.unwrap();
        }
    }
    
    // Analyze priority distribution
    let all_tasks = service.list_all().await.unwrap();
    let mut priority_counts = std::collections::HashMap::new();
    
    for task in &all_tasks {
        *priority_counts.entry(task.priority).or_insert(0) += 1;
    }
    
    assert_eq!(priority_counts.get(&Priority::Critical), Some(&2));
    assert_eq!(priority_counts.get(&Priority::High), Some(&5));
    assert_eq!(priority_counts.get(&Priority::Medium), Some(&10));
    assert_eq!(priority_counts.get(&Priority::Low), Some(&8));
}

#[tokio::test]
async fn test_dashboard_upcoming_deadlines() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks with upcoming deadlines
    let deadlines = vec![
        ("Today", chrono::Duration::hours(5)),
        ("Tomorrow", chrono::Duration::days(1)),
        ("This Week 1", chrono::Duration::days(3)),
        ("This Week 2", chrono::Duration::days(5)),
        ("Next Week", chrono::Duration::days(8)),
        ("Next Month", chrono::Duration::days(35)),
    ];
    
    for (name, duration) in deadlines {
        let mut task = Task::new(format!("Due: {}", name), "".to_string());
        task.due_date = Some(Utc::now() + duration);
        service.create(task).await.unwrap();
    }
    
    // Group by timeframe
    let all_tasks = service.list_all().await.unwrap();
    let now = Utc::now();
    
    let today: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.due_date.map_or(false, |d| d.date_naive() == now.date_naive()))
        .collect();
    
    let this_week: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.due_date.map_or(false, |d| {
            let days_until = (d - now).num_days();
            days_until >= 0 && days_until <= 7
        }))
        .collect();
    
    assert!(today.len() <= 1);
    assert!(this_week.len() >= 3);
}

#[tokio::test]
async fn test_dashboard_project_velocity() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Simulate completed tasks over time
    let mut completed_tasks = Vec::new();
    
    for week in 0..4 {
        for i in 0..5 {
            let mut task = Task::new(format!("Week {} Task {}", week, i), "".to_string());
            task.status = TaskStatus::Done;
            task.completed_at = Some(Utc::now() - chrono::Duration::weeks(week));
            task.estimated_hours = Some(4.0);
            completed_tasks.push(task);
        }
    }
    
    for task in completed_tasks {
        service.create(task).await.unwrap();
    }
    
    // Calculate velocity
    let all_tasks = service.list_all().await.unwrap();
    let completed = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .collect::<Vec<_>>();
    
    let total_points = completed.len();
    let weeks = 4;
    let velocity = total_points / weeks;
    
    assert_eq!(velocity, 5);
}

#[tokio::test]
async fn test_dashboard_burndown_chart_data() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create sprint tasks
    let sprint_start = Utc::now() - chrono::Duration::days(7);
    let sprint_end = Utc::now() + chrono::Duration::days(7);
    
    let mut sprint_tasks = Vec::new();
    for i in 0..10 {
        let mut task = Task::new(format!("Sprint Task {}", i), "".to_string());
        task.scheduled_date = Some(sprint_start);
        task.due_date = Some(sprint_end);
        task.estimated_hours = Some(4.0);
        
        // Complete some tasks
        if i < 4 {
            task.status = TaskStatus::Done;
            task.completed_at = Some(sprint_start + chrono::Duration::days(i as i64));
        }
        
        sprint_tasks.push(task);
    }
    
    for task in sprint_tasks {
        service.create(task).await.unwrap();
    }
    
    // Calculate burndown data
    let all_tasks = service.list_all().await.unwrap();
    let sprint_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.scheduled_date.map_or(false, |d| d >= sprint_start))
        .collect();
    
    let total_work = sprint_tasks.len();
    let completed_work = sprint_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();
    let remaining_work = total_work - completed_work;
    
    assert_eq!(total_work, 10);
    assert_eq!(completed_work, 4);
    assert_eq!(remaining_work, 6);
}

#[tokio::test]
async fn test_dashboard_task_age_analysis() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks with different ages
    let task_ages = vec![
        ("Fresh Task", 0),
        ("Week Old", 7),
        ("Two Weeks Old", 14),
        ("Month Old", 30),
        ("Stale Task", 60),
    ];
    
    for (name, days_old) in task_ages {
        let mut task = Task::new(name.to_string(), "".to_string());
        task.created_at = Utc::now() - chrono::Duration::days(days_old);
        task.status = TaskStatus::InProgress;
        service.create(task).await.unwrap();
    }
    
    // Analyze task age
    let all_tasks = service.list_all().await.unwrap();
    let now = Utc::now();
    
    let stale_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| {
            t.status != TaskStatus::Done &&
            now.signed_duration_since(t.created_at).num_days() > 30
        })
        .collect();
    
    assert_eq!(stale_tasks.len(), 1);
    assert_eq!(stale_tasks[0].title, "Stale Task");
}

#[tokio::test]
async fn test_dashboard_refresh() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Initial state
    let initial_count = service.list_all().await.unwrap().len();
    assert_eq!(initial_count, 0);
    
    // Add tasks
    for i in 0..5 {
        let task = Task::new(format!("Task {}", i), "".to_string());
        service.create(task).await.unwrap();
    }
    
    // Refresh and verify new count
    let updated_count = service.list_all().await.unwrap().len();
    assert_eq!(updated_count, 5);
    
    // Simulate concurrent update
    let task = Task::new("Concurrent Task".to_string(), "".to_string());
    service.create(task).await.unwrap();
    
    // Refresh again
    let final_count = service.list_all().await.unwrap().len();
    assert_eq!(final_count, 6);
}