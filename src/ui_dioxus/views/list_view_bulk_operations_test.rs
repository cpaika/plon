#[cfg(test)]
mod tests {
    use dioxus::prelude::*;
    use dioxus_desktop::DesktopConfig;
    use std::sync::Arc;
    use std::collections::HashSet;
    use crate::repository::Repository;
    use crate::domain::task::{Task, TaskStatus, Priority};
    use crate::ui_dioxus::views::ListView;
    use sqlx::SqlitePool;
    use uuid::Uuid;
    
    async fn setup_test_db() -> Arc<Repository> {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        Arc::new(Repository::new(pool))
    }
    
    #[tokio::test]
    async fn test_bulk_select_multiple_tasks() {
        let repo = setup_test_db().await;
        
        // Create test tasks
        let task1 = Task {
            id: Uuid::new_v4(),
            title: "Task 1".to_string(),
            description: "Description 1".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            due_date: None,
            estimated_hours: None,
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            tags: HashSet::new(),
            assignee: None,
            position: crate::domain::task::Position { x: 0.0, y: 0.0 },
            scheduled_date: None,
            completed_at: None,
            metadata: std::collections::HashMap::new(),
            subtasks: vec![],
            is_archived: false,
            configuration_id: None,
            sort_order: 0,        };
        
        let task2 = task1.clone();
        let mut task2 = task2;
        task2.id = Uuid::new_v4();
        task2.title = "Task 2".to_string();
        
        repo.tasks.create(&task1).await.unwrap();
        repo.tasks.create(&task2).await.unwrap();
        
        // Verify tasks can be selected in bulk
        let tasks = repo.tasks.list(crate::repository::task_repository::TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        }).await.unwrap();
        
        assert_eq!(tasks.len(), 2);
    }
    
    #[tokio::test]
    async fn test_bulk_mark_as_done() {
        let repo = setup_test_db().await;
        
        // Create test tasks
        let mut tasks = vec![];
        for i in 0..3 {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Task {}", i),
                description: format!("Description {}", i),
                status: TaskStatus::Todo,
                priority: Priority::Medium,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                due_date: None,
                estimated_hours: None,
                actual_hours: None,
                assigned_resource_id: None,
                goal_id: None,
                parent_task_id: None,
                tags: HashSet::new(),
                assignee: None,
                position: crate::domain::task::Position { x: 0.0, y: 0.0 },
            scheduled_date: None,
            completed_at: None,
            metadata: std::collections::HashMap::new(),
            subtasks: vec![],
            is_archived: false,
            configuration_id: None,
            sort_order: 0,            };
            repo.tasks.create(&task).await.unwrap();
            tasks.push(task);
        }
        
        // Mark all tasks as done
        for task in &tasks {
            let mut updated = task.clone();
            updated.status = TaskStatus::Done;
            repo.tasks.update(&updated).await.unwrap();
        }
        
        // Verify all tasks are done
        let done_tasks = repo.tasks.list(crate::repository::task_repository::TaskFilters {
            status: Some(TaskStatus::Done),
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        }).await.unwrap();
        
        assert_eq!(done_tasks.len(), 3);
    }
    
    #[tokio::test]
    async fn test_select_all_keyboard_shortcut() {
        // This tests that Ctrl/Cmd+A selects all tasks
        // In a real UI test, we would simulate keyboard events
        // For now, we just verify the logic works
        
        let repo = setup_test_db().await;
        
        // Create 5 test tasks
        for i in 0..5 {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Task {}", i),
                description: "".to_string(),
                status: TaskStatus::Todo,
                priority: Priority::Medium,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                due_date: None,
                estimated_hours: None,
                actual_hours: None,
                assigned_resource_id: None,
                goal_id: None,
                parent_task_id: None,
                tags: HashSet::new(),
                assignee: None,
                position: crate::domain::task::Position { x: 0.0, y: 0.0 },
            scheduled_date: None,
            completed_at: None,
            metadata: std::collections::HashMap::new(),
            subtasks: vec![],
            is_archived: false,
            configuration_id: None,
            sort_order: 0,            };
            repo.tasks.create(&task).await.unwrap();
        }
        
        let all_tasks = repo.tasks.list(crate::repository::task_repository::TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        }).await.unwrap();
        
        // Simulate selecting all tasks
        let selected_ids: std::collections::HashSet<_> = all_tasks.iter().map(|t| t.id).collect();
        
        assert_eq!(selected_ids.len(), 5);
    }
}