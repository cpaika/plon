use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::domain::task::{Task, TaskStatus, Priority};
use plon::domain::goal::Goal;
use plon::domain::resource::Resource;
use plon::domain::comment::{Comment, EntityType};
use plon::domain::dependency::{Dependency, DependencyType, DependencyGraph};
use plon::services::{TaskService, GoalService, ResourceService};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_database_initialization() {
    let pool = init_test_database().await.expect("Failed to init database");
    
    // Verify tables exist
    let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
        .fetch_all(&pool)
        .await
        .expect("Failed to query tables");
    
    assert!(result.len() > 0);
}

#[tokio::test]
async fn test_task_repository_crud() {
    let pool = init_test_database().await.unwrap();
    let repository = Repository::new(pool);
    
    // Create task
    let mut task = Task::new("Integration Test Task".to_string(), "Test Description".to_string());
    task.set_position(100.0, 200.0);
    task.add_subtask("Subtask 1".to_string());
    task.add_subtask("Subtask 2".to_string());
    
    repository.tasks.create(&task).await.unwrap();
    
    // Read task
    let fetched = repository.tasks.get(task.id).await.unwrap();
    assert!(fetched.is_some());
    
    let fetched_task = fetched.unwrap();
    assert_eq!(fetched_task.title, "Integration Test Task");
    assert_eq!(fetched_task.position.x, 100.0);
    assert_eq!(fetched_task.position.y, 200.0);
    assert_eq!(fetched_task.subtasks.len(), 2);
    
    // Update task
    let mut updated_task = fetched_task.clone();
    updated_task.title = "Updated Task".to_string();
    updated_task.update_status(TaskStatus::InProgress);
    repository.tasks.update(&updated_task).await.unwrap();
    
    let fetched_updated = repository.tasks.get(task.id).await.unwrap().unwrap();
    assert_eq!(fetched_updated.title, "Updated Task");
    assert_eq!(fetched_updated.status, TaskStatus::InProgress);
    
    // Delete task
    let deleted = repository.tasks.delete(task.id).await.unwrap();
    assert!(deleted);
    
    let fetched_deleted = repository.tasks.get(task.id).await.unwrap();
    assert!(fetched_deleted.is_none());
}

#[tokio::test]
async fn test_task_list_with_filters() {
    let pool = init_test_database().await.unwrap();
    let repository = Repository::new(pool);
    
    // Create multiple tasks with different statuses
    let mut task1 = Task::new("Todo Task".to_string(), "".to_string());
    task1.status = TaskStatus::Todo;
    
    let mut task2 = Task::new("In Progress Task".to_string(), "".to_string());
    task2.status = TaskStatus::InProgress;
    
    let mut task3 = Task::new("Done Task".to_string(), "".to_string());
    task3.status = TaskStatus::Done;
    
    repository.tasks.create(&task1).await.unwrap();
    repository.tasks.create(&task2).await.unwrap();
    repository.tasks.create(&task3).await.unwrap();
    
    // Test filtering by status
    use plon::repository::task_repository::TaskFilters;
    
    let filters = TaskFilters {
        status: Some(TaskStatus::InProgress),
        ..Default::default()
    };
    
    let tasks = repository.tasks.list(filters).await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "In Progress Task");
    
    // Test listing all tasks
    let all_tasks = repository.tasks.list(TaskFilters::default()).await.unwrap();
    assert_eq!(all_tasks.len(), 3);
}

#[tokio::test]
async fn test_spatial_queries() {
    let pool = init_test_database().await.unwrap();
    let repository = Repository::new(pool);
    
    // Create tasks at different positions
    let mut task1 = Task::new("Task 1".to_string(), "".to_string());
    task1.set_position(10.0, 10.0);
    
    let mut task2 = Task::new("Task 2".to_string(), "".to_string());
    task2.set_position(50.0, 50.0);
    
    let mut task3 = Task::new("Task 3".to_string(), "".to_string());
    task3.set_position(100.0, 100.0);
    
    repository.tasks.create(&task1).await.unwrap();
    repository.tasks.create(&task2).await.unwrap();
    repository.tasks.create(&task3).await.unwrap();
    
    // Query tasks in area
    let tasks_in_area = repository.tasks.find_in_area(0.0, 60.0, 0.0, 60.0).await.unwrap();
    assert_eq!(tasks_in_area.len(), 2);
}

#[tokio::test]
async fn test_task_service() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create task through service
    let task = Task::new("Service Task".to_string(), "Created through service".to_string());
    let created = service.create(task.clone()).await.unwrap();
    assert_eq!(created.title, "Service Task");
    
    // Get task through service
    let fetched = service.get(created.id).await.unwrap();
    assert!(fetched.is_some());
    
    // List all tasks
    let all_tasks = service.list_all().await.unwrap();
    assert_eq!(all_tasks.len(), 1);
    
    // Delete task
    let deleted = service.delete(created.id).await.unwrap();
    assert!(deleted);
}

#[tokio::test]
async fn test_dependency_graph() {
    let mut graph = DependencyGraph::new();
    
    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();
    
    graph.add_task(task1);
    graph.add_task(task2);
    graph.add_task(task3);
    
    // Create dependencies: task1 -> task2 -> task3
    let dep1 = Dependency::new(task1, task2, DependencyType::FinishToStart);
    let dep2 = Dependency::new(task2, task3, DependencyType::FinishToStart);
    
    assert!(graph.add_dependency(&dep1).is_ok());
    assert!(graph.add_dependency(&dep2).is_ok());
    
    // Check for cycles
    assert!(!graph.has_cycle());
    
    // Try to create a cycle: task3 -> task1
    let dep_cycle = Dependency::new(task3, task1, DependencyType::FinishToStart);
    assert!(graph.add_dependency(&dep_cycle).is_err());
    
    // Test topological sort
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0], task1);
    assert_eq!(sorted[1], task2);
    assert_eq!(sorted[2], task3);
    
    // Test dependency queries
    let task2_deps = graph.get_dependencies(task2);
    assert_eq!(task2_deps.len(), 1);
    assert_eq!(task2_deps[0].0, task1);
    
    let task2_dependents = graph.get_dependents(task2);
    assert_eq!(task2_dependents.len(), 1);
    assert_eq!(task2_dependents[0].0, task3);
}

#[tokio::test]
async fn test_goal_with_tasks() {
    let mut goal = Goal::new("Q1 Goals".to_string(), "First quarter goals".to_string());
    
    let task1_id = Uuid::new_v4();
    let task2_id = Uuid::new_v4();
    let task3_id = Uuid::new_v4();
    
    goal.add_task(task1_id);
    goal.add_task(task2_id);
    goal.add_task(task3_id);
    
    assert_eq!(goal.task_ids.len(), 3);
    
    // Test progress calculation
    let task_statuses = vec![
        (task1_id, true),  // completed
        (task2_id, false), // not completed
        (task3_id, true),  // completed
    ];
    
    let progress = goal.calculate_progress(&task_statuses);
    assert!((progress - 66.66667).abs() < 0.001);
    
    // Test removing task
    assert!(goal.remove_task(&task2_id));
    assert_eq!(goal.task_ids.len(), 2);
    assert!(!goal.remove_task(&task2_id)); // Already removed
}

#[tokio::test]
async fn test_resource_allocation() {
    let mut resource = Resource::new("Developer".to_string(), "Engineer".to_string(), 40.0);
    
    // Test skill management
    resource.add_skill("Rust".to_string());
    resource.add_skill("TypeScript".to_string());
    assert_eq!(resource.skills.len(), 2);
    
    // Test metadata filters
    resource.add_metadata_filter("team".to_string(), "backend".to_string());
    
    let mut task_metadata = std::collections::HashMap::new();
    task_metadata.insert("team".to_string(), "backend".to_string());
    assert!(resource.can_work_on_task(&task_metadata));
    
    task_metadata.insert("team".to_string(), "frontend".to_string());
    assert!(!resource.can_work_on_task(&task_metadata));
    
    // Test utilization
    assert_eq!(resource.utilization_percentage(), 0.0);
    
    resource.current_load = 30.0;
    assert_eq!(resource.utilization_percentage(), 75.0);
    assert_eq!(resource.available_hours(), 10.0);
    assert!(!resource.is_overloaded());
    
    resource.current_load = 50.0;
    assert!(resource.is_overloaded());
}

#[tokio::test]
async fn test_comment_system() {
    let task_id = Uuid::new_v4();
    let mut comment = Comment::new(
        task_id,
        EntityType::Task,
        "John Doe".to_string(),
        "This task looks good!".to_string()
    );
    
    assert_eq!(comment.entity_id, task_id);
    assert_eq!(comment.content, "This task looks good!");
    assert!(!comment.edited);
    
    // Test editing
    comment.edit("Actually, needs more work.".to_string());
    assert_eq!(comment.content, "Actually, needs more work.");
    assert!(comment.edited);
    
    // Test attachments
    comment.add_attachment(
        "screenshot.png".to_string(),
        "image/png".to_string(),
        1024000,
        "/uploads/screenshot.png".to_string()
    );
    
    assert_eq!(comment.attachments.len(), 1);
    assert_eq!(comment.attachments[0].filename, "screenshot.png");
    
    let attachment_id = comment.attachments[0].id;
    assert!(comment.remove_attachment(attachment_id));
    assert_eq!(comment.attachments.len(), 0);
}

#[tokio::test]
async fn test_recurring_tasks() {
    use plon::domain::recurring::{RecurringTaskTemplate, RecurrenceRule, RecurrencePattern};
    use chrono::NaiveTime;
    
    let rule = RecurrenceRule {
        pattern: RecurrencePattern::Daily,
        interval: 1,
        days_of_week: vec![],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: Some(5),
        occurrences_count: 0,
    };
    
    let mut template = RecurringTaskTemplate::new(
        "Daily Standup".to_string(),
        "Team sync meeting".to_string(),
        rule
    );
    
    // Generate tasks
    let mut generated_tasks = Vec::new();
    for _ in 0..5 {
        if let Some(task) = template.generate_task() {
            generated_tasks.push(task);
        }
    }
    
    assert_eq!(generated_tasks.len(), 5);
    assert_eq!(template.recurrence_rule.occurrences_count, 5);
    
    // Should not generate more tasks after max occurrences
    assert!(template.generate_task().is_none());
    assert!(!template.active);
}

#[tokio::test]
async fn test_markdown_subtask_extraction() {
    let mut task = Task::new(
        "Task with subtasks".to_string(),
        r#"
# Task Description

Here are the subtasks:
- [ ] First subtask
- [ ] Second subtask
- [x] This should be ignored (already done)
- Regular bullet point (not a task)
- [ ] Third subtask
"#.to_string()
    );
    
    task.extract_subtasks_from_markdown();
    
    assert_eq!(task.subtasks.len(), 3);
    assert_eq!(task.subtasks[0].description, "First subtask");
    assert_eq!(task.subtasks[1].description, "Second subtask");
    assert_eq!(task.subtasks[2].description, "Third subtask");
}

#[tokio::test]
async fn test_critical_path_calculation() {
    let mut graph = DependencyGraph::new();
    
    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();
    let task4 = Uuid::new_v4();
    
    // Create a diamond dependency pattern
    // task1 -> task2 -> task4
    //      \-> task3 ->/
    
    graph.add_dependency(&Dependency::new(task1, task2, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(task1, task3, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(task2, task4, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(task3, task4, DependencyType::FinishToStart)).unwrap();
    
    let mut estimates = std::collections::HashMap::new();
    estimates.insert(task1, 2.0);
    estimates.insert(task2, 5.0); // Longer path through task2
    estimates.insert(task3, 1.0);
    estimates.insert(task4, 1.0);
    
    let critical_path = graph.get_critical_path(&estimates);
    
    assert_eq!(critical_path.len(), 3);
    assert_eq!(critical_path[0], task1);
    assert_eq!(critical_path[1], task2); // Should choose longer path
    assert_eq!(critical_path[2], task4);
}