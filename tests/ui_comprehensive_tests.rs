use plon::domain::task::{Task, TaskStatus, Priority};
use plon::domain::goal::{Goal, GoalStatus};
use plon::domain::resource::Resource;
use plon::domain::dependency::{Dependency, DependencyGraph, DependencyType};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::services::{TaskService, GoalService, ResourceService};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use chrono::{Utc, Duration, NaiveDate};
use uuid::Uuid;

// Helper function to create test repository
async fn create_test_repository() -> Arc<Repository> {
    let pool = init_test_database().await.unwrap();
    Arc::new(Repository::new(pool))
}

// Helper function to create sample tasks
fn create_sample_tasks(count: usize) -> Vec<Task> {
    (0..count).map(|i| {
        let mut task = Task::new(
            format!("Task {}", i),
            format!("Description for task {}", i)
        );
        task.priority = match i % 4 {
            0 => Priority::Critical,
            1 => Priority::High,
            2 => Priority::Medium,
            _ => Priority::Low,
        };
        task.status = match i % 5 {
            0 => TaskStatus::Todo,
            1 => TaskStatus::InProgress,
            2 => TaskStatus::Blocked,
            3 => TaskStatus::Review,
            _ => TaskStatus::Done,
        };
        if i % 3 == 0 {
            task.due_date = Some(Utc::now() + Duration::days((i as i64) - 5));
        }
        task
    }).collect()
}

#[cfg(test)]
mod task_tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test Task".to_string(), "Test Description".to_string());
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "Test Description");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, Priority::Medium);
    }

    #[test]
    fn test_task_subtasks() {
        let mut task = Task::new("Main Task".to_string(), "".to_string());
        
        // Add subtasks
        let id1 = task.add_subtask("Subtask 1".to_string());
        let id2 = task.add_subtask("Subtask 2".to_string());
        let id3 = task.add_subtask("Subtask 3".to_string());
        
        assert_eq!(task.subtasks.len(), 3);
        
        // Complete some subtasks
        assert!(task.complete_subtask(id1).is_ok());
        assert!(task.complete_subtask(id2).is_ok());
        
        let (completed, total) = task.subtask_progress();
        assert_eq!(completed, 2);
        assert_eq!(total, 3);
        
        // Uncomplete a subtask
        assert!(task.uncomplete_subtask(id1).is_ok());
        let (completed, _) = task.subtask_progress();
        assert_eq!(completed, 1);
    }

    #[test]
    fn test_task_overdue() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        // No due date - not overdue
        assert!(!task.is_overdue());
        
        // Past due date - overdue
        task.due_date = Some(Utc::now() - Duration::days(1));
        assert!(task.is_overdue());
        
        // Completed task - not overdue even with past due date
        task.status = TaskStatus::Done;
        assert!(!task.is_overdue());
        
        // Future due date - not overdue
        task.status = TaskStatus::Todo;
        task.due_date = Some(Utc::now() + Duration::days(1));
        assert!(!task.is_overdue());
    }

    #[test]
    fn test_task_tags_and_metadata() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        // Add tags
        task.tags.insert("urgent".to_string());
        task.tags.insert("bug".to_string());
        task.tags.insert("frontend".to_string());
        
        assert_eq!(task.tags.len(), 3);
        assert!(task.tags.contains("urgent"));
        
        // Add metadata
        task.metadata.insert("category".to_string(), "feature".to_string());
        task.metadata.insert("version".to_string(), "1.0".to_string());
        
        assert_eq!(task.metadata.len(), 2);
        assert_eq!(task.metadata.get("category"), Some(&"feature".to_string()));
    }

    #[test]
    fn test_task_status_transitions() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        // Progress through statuses
        assert_eq!(task.status, TaskStatus::Todo);
        
        task.status = TaskStatus::InProgress;
        assert_eq!(task.status, TaskStatus::InProgress);
        
        task.status = TaskStatus::Review;
        assert_eq!(task.status, TaskStatus::Review);
        
        task.status = TaskStatus::Done;
        assert_eq!(task.status, TaskStatus::Done);
        
        // Can also be blocked at any time
        task.status = TaskStatus::Blocked;
        assert_eq!(task.status, TaskStatus::Blocked);
    }

    #[test]
    fn test_task_archiving() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        assert!(!task.is_archived);
        
        task.is_archived = true;
        assert!(task.is_archived);
        
        // Should maintain other properties
        task.status = TaskStatus::Done;
        assert_eq!(task.status, TaskStatus::Done);
        assert!(task.is_archived);
    }
}

#[cfg(test)]
mod goal_tests {
    use super::*;

    #[test]
    fn test_goal_creation() {
        let goal = Goal::new("Q1 Goals".to_string(), "Complete Q1 objectives".to_string());
        assert_eq!(goal.title, "Q1 Goals");
        assert_eq!(goal.description, "Complete Q1 objectives");
        assert_eq!(goal.status, GoalStatus::NotStarted);
    }

    #[test]
    fn test_goal_progress() {
        let mut goal = Goal::new("Release v1.0".to_string(), "".to_string());
        
        // Add tasks to goal
        for i in 0..5 {
            let task = Task::new(format!("Task {}", i), "".to_string());
            goal.add_task(task.id);
        }
        
        assert_eq!(goal.task_ids.len(), 5);
        
        // Update status
        goal.status = GoalStatus::InProgress;
        assert_eq!(goal.status, GoalStatus::InProgress);
        
        goal.status = GoalStatus::Completed;
        assert_eq!(goal.status, GoalStatus::Completed);
    }

    #[test]
    fn test_goal_hierarchy() {
        let parent_goal = Goal::new("Annual Goals".to_string(), "".to_string());
        let mut child_goal = Goal::new("Q1 Goals".to_string(), "".to_string());
        
        child_goal.parent_goal_id = Some(parent_goal.id);
        assert_eq!(child_goal.parent_goal_id, Some(parent_goal.id));
    }
}

#[cfg(test)]
mod resource_tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new("John Doe".to_string(), "Developer".to_string(), 40.0);
        assert_eq!(resource.name, "John Doe");
        assert_eq!(resource.role, "Developer");
        assert_eq!(resource.weekly_hours, 40.0);
    }

    #[test]
    fn test_resource_skills() {
        let mut resource = Resource::new("Jane Doe".to_string(), "Designer".to_string(), 40.0);
        
        resource.add_skill("UI Design".to_string());
        resource.add_skill("Prototyping".to_string());
        resource.add_skill("User Research".to_string());
        
        assert_eq!(resource.skills.len(), 3);
        assert!(resource.skills.contains(&"UI Design".to_string()));
    }

    #[test]
    fn test_resource_availability() {
        let resource = Resource::new("Test Resource".to_string(), "Tester".to_string(), 40.0);
        
        // Weekday should have 8 hours by default
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(resource.get_availability_for_date(monday), 8.0);
        
        // Weekend should have 0 hours
        let saturday = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        assert_eq!(resource.get_availability_for_date(saturday), 0.0);
    }

    #[test]
    fn test_resource_utilization() {
        let mut resource = Resource::new("Developer".to_string(), "dev".to_string(), 40.0);
        resource.weekly_hours = 40.0;
        resource.current_load = 32.0;
        
        assert_eq!(resource.utilization_percentage(), 80.0);
        assert_eq!(resource.available_hours(), 8.0);
        assert!(resource.current_load < resource.weekly_hours);
        
        // Test overallocation
        resource.current_load = 50.0;
        assert_eq!(resource.utilization_percentage(), 125.0);
        assert!(resource.current_load > resource.weekly_hours);
    }
}

#[cfg(test)]
mod dependency_tests {
    use super::*;

    #[test]
    fn test_dependency_creation() {
        let task1 = Task::new("Task 1".to_string(), "".to_string());
        let task2 = Task::new("Task 2".to_string(), "".to_string());
        
        let dep = Dependency::new(task1.id, task2.id, DependencyType::FinishToStart);
        assert_eq!(dep.from_task_id, task1.id);
        assert_eq!(dep.to_task_id, task2.id);
        assert_eq!(dep.dependency_type, DependencyType::FinishToStart);
    }

    #[test]
    fn test_dependency_graph() {
        let task1 = Task::new("Task 1".to_string(), "".to_string());
        let task2 = Task::new("Task 2".to_string(), "".to_string());
        let task3 = Task::new("Task 3".to_string(), "".to_string());
        
        let mut graph = DependencyGraph::new();
        
        // Add dependencies
        assert!(graph.add_dependency(&Dependency::new(
            task1.id, task2.id, DependencyType::FinishToStart
        )).is_ok());
        
        assert!(graph.add_dependency(&Dependency::new(
            task2.id, task3.id, DependencyType::FinishToStart
        )).is_ok());
        
        // Should prevent circular dependencies
        assert!(graph.add_dependency(&Dependency::new(
            task3.id, task1.id, DependencyType::FinishToStart
        )).is_err());
        
        // Check dependencies and dependents
        // task1 -> task2 means task2 depends on task1
        let deps = graph.get_dependencies(task2.id);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, task1.id);
        
        // task2 -> task3 means task3 depends on task2
        let deps_task3 = graph.get_dependencies(task3.id);
        assert_eq!(deps_task3.len(), 1);
        assert_eq!(deps_task3[0].0, task2.id);
        
        // Check dependents - task1 has task2 as dependent
        let dependents = graph.get_dependents(task1.id);
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0].0, task2.id);
    }

    #[test]
    fn test_topological_sort() {
        let tasks: Vec<_> = (0..5).map(|i| {
            Task::new(format!("Task {}", i), "".to_string())
        }).collect();
        
        let mut graph = DependencyGraph::new();
        
        // Create chain: 0 -> 1 -> 2 -> 3 -> 4
        for i in 0..4 {
            graph.add_dependency(&Dependency::new(
                tasks[i].id, 
                tasks[i + 1].id, 
                DependencyType::FinishToStart
            )).unwrap();
        }
        
        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 5);
        
        // First task should come before last task in the sorted order
        let first_pos = sorted.iter().position(|&id| id == tasks[0].id).unwrap();
        let last_pos = sorted.iter().position(|&id| id == tasks[4].id).unwrap();
        assert!(first_pos < last_pos);
    }

    #[test]
    fn test_critical_path() {
        let tasks: Vec<_> = (0..3).map(|i| {
            Task::new(format!("Task {}", i), "".to_string())
        }).collect();
        
        let mut graph = DependencyGraph::new();
        
        // Create simple chain
        graph.add_dependency(&Dependency::new(
            tasks[0].id, tasks[1].id, DependencyType::FinishToStart
        )).unwrap();
        
        graph.add_dependency(&Dependency::new(
            tasks[1].id, tasks[2].id, DependencyType::FinishToStart
        )).unwrap();
        
        let estimates: HashMap<Uuid, f32> = tasks.iter()
            .map(|t| (t.id, 8.0))
            .collect();
        
        let critical_path = graph.get_critical_path(&estimates);
        assert_eq!(critical_path.len(), 3);
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_collections() {
        let tasks: Vec<Task> = Vec::new();
        assert_eq!(tasks.len(), 0);
        
        let goals: Vec<Goal> = Vec::new();
        assert_eq!(goals.len(), 0);
        
        let resources: Vec<Resource> = Vec::new();
        assert_eq!(resources.len(), 0);
    }

    #[test]
    fn test_large_datasets() {
        let tasks = create_sample_tasks(1000);
        assert_eq!(tasks.len(), 1000);
        
        // Check that all tasks have unique IDs
        let mut ids = HashSet::new();
        for task in &tasks {
            assert!(ids.insert(task.id));
        }
    }

    #[test]
    fn test_unicode_handling() {
        let task = Task::new(
            "测试任务 🚀 тест τεστ".to_string(),
            "Description with émojis 😀😁😂".to_string()
        );
        
        assert!(task.title.contains("🚀"));
        assert!(task.description.contains("😀"));
        
        let goal = Goal::new(
            "目标 🎯".to_string(),
            "Цель με emoji 🏆".to_string()
        );
        
        assert!(goal.title.contains("🎯"));
        assert!(goal.description.contains("🏆"));
    }

    #[test]
    fn test_extreme_dates() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        // Far future date
        task.due_date = Some(Utc::now() + Duration::days(365 * 100));
        assert!(!task.is_overdue());
        
        // Far past date
        task.due_date = Some(Utc::now() - Duration::days(365 * 50));
        task.status = TaskStatus::Todo;
        assert!(task.is_overdue());
    }

    #[test]
    fn test_maximum_string_lengths() {
        let long_title = "A".repeat(10000);
        let long_desc = "B".repeat(100000);
        
        let task = Task::new(long_title.clone(), long_desc.clone());
        assert_eq!(task.title.len(), 10000);
        assert_eq!(task.description.len(), 100000);
    }

    #[test]
    fn test_special_characters() {
        let task = Task::new(
            "Title with <>&\"' special chars".to_string(),
            "Description\nwith\nnewlines\tand\ttabs".to_string()
        );
        
        assert!(task.title.contains("<>&\"'"));
        assert!(task.description.contains("\n"));
        assert!(task.description.contains("\t"));
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_task_creation_performance() {
        let start = Instant::now();
        let tasks = create_sample_tasks(10000);
        let duration = start.elapsed();
        
        assert_eq!(tasks.len(), 10000);
        // Should create 10000 tasks in under 1 second
        assert!(duration.as_secs() < 1);
    }

    #[test]
    fn test_dependency_graph_performance() {
        let tasks = create_sample_tasks(100);
        let mut graph = DependencyGraph::new();
        
        let start = Instant::now();
        
        // Add dependencies in a chain
        for i in 0..99 {
            graph.add_dependency(&Dependency::new(
                tasks[i].id,
                tasks[i + 1].id,
                DependencyType::FinishToStart
            )).ok();
        }
        
        // Perform topological sort
        let sorted = graph.topological_sort();
        let duration = start.elapsed();
        
        assert!(sorted.is_ok());
        // Should complete in under 100ms
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_large_metadata_performance() {
        let start = Instant::now();
        
        let mut task = Task::new("Test".to_string(), "".to_string());
        
        // Add many metadata entries
        for i in 0..1000 {
            task.metadata.insert(format!("key_{}", i), format!("value_{}", i));
        }
        
        // Add many tags
        for i in 0..1000 {
            task.tags.insert(format!("tag_{}", i));
        }
        
        let duration = start.elapsed();
        
        assert_eq!(task.metadata.len(), 1000);
        assert_eq!(task.tags.len(), 1000);
        // Should complete in under 100ms
        assert!(duration.as_millis() < 100);
    }
}

#[tokio::test]
async fn test_database_integration() {
    let repository = create_test_repository().await;
    let task_service = Arc::new(TaskService::new(repository.clone()));
    
    // Create and save tasks
    let mut task = Task::new("Test Task".to_string(), "Test Description".to_string());
    task.priority = Priority::High;
    task.tags.insert("test".to_string());
    task.metadata.insert("category".to_string(), "integration".to_string());
    
    let task_result = task_service.create(task.clone()).await;
    assert!(task_result.is_ok());
    
    // Retrieve and verify
    let retrieved_task = task_service.get(task.id).await;
    assert!(retrieved_task.is_ok());
    
    if let Ok(Some(t)) = retrieved_task {
        assert_eq!(t.title, "Test Task");
        assert_eq!(t.priority, Priority::High);
        assert!(t.tags.contains("test"));
        assert_eq!(t.metadata.get("category"), Some(&"integration".to_string()));
    }
    
    // Update task
    if let Ok(mut updated_task) = task_result {
        updated_task.status = TaskStatus::InProgress;
        let update_result = task_service.update(updated_task).await;
        assert!(update_result.is_ok());
    }
    
    // List all tasks
    let all_tasks = task_service.list_all().await;
    assert!(all_tasks.is_ok());
    if let Ok(tasks) = all_tasks {
        assert!(!tasks.is_empty());
    }
}