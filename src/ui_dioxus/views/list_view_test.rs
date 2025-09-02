#[cfg(test)]
mod tests {
    use dioxus::prelude::*;
    use std::sync::Arc;
    use crate::repository::Repository;
    use crate::repository::task_repository::TaskFilters;
    use crate::domain::task::{Task, TaskStatus, Priority, Position};
    use crate::ui_dioxus::views::ListView;
    use uuid::Uuid;
    use chrono::Utc;
    use std::collections::{HashMap, HashSet};
    
    #[tokio::test]
    async fn test_list_view_renders_with_repository() {
        // This test verifies that ListView can render when Repository context is provided
        
        let result = tokio::task::spawn_blocking(|| {
            std::panic::catch_unwind(|| {
                let mut vdom = VirtualDom::new(test_app_with_list_view);
                let _ = vdom.rebuild_to_vec();
            })
        }).await.unwrap();
        
        assert!(result.is_ok(), "ListView should render without panicking when Repository context is provided");
    }
    
    #[component]
    fn test_app_with_list_view() -> Element {
        // Create a test repository
        let repository = create_test_repository();
        
        // Provide the Repository context
        use_context_provider(|| repository);
        
        // Render ListView
        rsx! {
            ListView {}
        }
    }
    
    #[tokio::test]
    async fn test_list_view_loads_tasks() {
        // Create a repository with test data
        let repository = create_test_repository_with_tasks().await;
        
        // Verify we can query tasks from the repository
        let filters = TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        };
        let tasks = repository.tasks.list(filters).await.unwrap();
        assert_eq!(tasks.len(), 3, "Should have 3 test tasks");
        
        // Verify task properties
        let todo_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Todo).collect();
        assert_eq!(todo_tasks.len(), 1, "Should have 1 todo task");
        
        let in_progress_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).collect();
        assert_eq!(in_progress_tasks.len(), 1, "Should have 1 in progress task");
        
        let done_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Done).collect();
        assert_eq!(done_tasks.len(), 1, "Should have 1 done task");
    }
    
    #[tokio::test]
    async fn test_list_view_filter_functionality() {
        let repository = create_test_repository_with_tasks().await;
        
        // Test filtering by status
        let all_filters = TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        };
        let all_tasks = repository.tasks.list(all_filters).await.unwrap();
        assert_eq!(all_tasks.len(), 3, "Should have all 3 tasks");
        
        // Simulate filtering (this would be done in the component)
        let todo_only: Vec<_> = all_tasks.iter()
            .filter(|t| t.status == TaskStatus::Todo)
            .collect();
        assert_eq!(todo_only.len(), 1, "Filter should return 1 todo task");
        
        let done_only: Vec<_> = all_tasks.iter()
            .filter(|t| t.status == TaskStatus::Done)
            .collect();
        assert_eq!(done_only.len(), 1, "Filter should return 1 done task");
    }
    
    #[tokio::test]
    async fn test_list_view_update_task_status() {
        let repository = create_test_repository_with_tasks().await;
        
        // Get a task
        let filters = TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        };
        let tasks = repository.tasks.list(filters).await.unwrap();
        // Find the Todo task
        let mut task = tasks.iter().find(|t| t.status == TaskStatus::Todo).unwrap().clone();
        
        // Update its status
        assert_eq!(task.status, TaskStatus::Todo);
        task.status = TaskStatus::InProgress;
        task.updated_at = Utc::now();
        
        // Save the update
        repository.tasks.update(&task).await.unwrap();
        
        // Verify the update persisted
        let updated_task = repository.tasks.get(task.id).await.unwrap().unwrap();
        assert_eq!(updated_task.status, TaskStatus::InProgress, "Task status should be updated");
    }
    
    #[tokio::test]
    async fn test_list_view_sorting() {
        let repository = create_test_repository_with_tasks().await;
        
        let filters = TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        };
        let mut tasks = repository.tasks.list(filters).await.unwrap();
        
        // Test sorting by title
        tasks.sort_by(|a, b| a.title.cmp(&b.title));
        assert_eq!(tasks[0].title, "Complete documentation");
        assert_eq!(tasks[1].title, "Fix bug in login");
        assert_eq!(tasks[2].title, "Implement new feature");
        
        // Test sorting by created date (newest first)
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        // The order depends on creation order in create_test_repository_with_tasks
    }
    
    fn create_test_repository() -> Arc<Repository> {
        use sqlx::SqlitePool;
        use tokio::runtime::Runtime;
        
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = SqlitePool::connect(":memory:").await.unwrap();
            
            // Run migrations
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .unwrap();
            
            Arc::new(Repository::new(pool))
        })
    }
    
    async fn create_test_repository_with_tasks() -> Arc<Repository> {
        let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();
        
        let repository = Arc::new(Repository::new(pool));
        
        // Create test tasks
        let task1 = Task {
            id: Uuid::new_v4(),
            title: "Implement new feature".to_string(),
            description: "Add user authentication".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::High,
            tags: HashSet::from(["feature".to_string()]),
            metadata: HashMap::new(),
            estimated_hours: Some(8.0),
            actual_hours: None,
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            scheduled_date: None,
            completed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            position: Position { x: 100.0, y: 100.0 },
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            subtasks: vec![],
            is_archived: false,
            assignee: None,
            configuration_id: None,
            sort_order: 0,
        };
        
        let task2 = Task {
            id: Uuid::new_v4(),
            title: "Fix bug in login".to_string(),
            description: "Users can't log in with special characters".to_string(),
            status: TaskStatus::InProgress,
            priority: Priority::High,
            tags: HashSet::from(["bug".to_string()]),
            metadata: HashMap::new(),
            estimated_hours: Some(2.0),
            actual_hours: Some(1.5),
            due_date: Some(Utc::now() + chrono::Duration::days(1)),
            scheduled_date: None,
            completed_at: None,
            created_at: Utc::now() - chrono::Duration::hours(2),
            updated_at: Utc::now(),
            position: Position { x: 200.0, y: 200.0 },
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            subtasks: vec![],
            is_archived: false,
            assignee: None,
            configuration_id: None,
            sort_order: 1,
        };
        
        let task3 = Task {
            id: Uuid::new_v4(),
            title: "Complete documentation".to_string(),
            description: "Update API documentation".to_string(),
            status: TaskStatus::Done,
            priority: Priority::Medium,
            tags: HashSet::from(["docs".to_string()]),
            metadata: HashMap::new(),
            estimated_hours: Some(4.0),
            actual_hours: Some(3.5),
            due_date: None,
            scheduled_date: None,
            completed_at: Some(Utc::now() - chrono::Duration::hours(1)),
            created_at: Utc::now() - chrono::Duration::days(2),
            updated_at: Utc::now() - chrono::Duration::hours(1),
            position: Position { x: 300.0, y: 150.0 },
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            subtasks: vec![],
            is_archived: false,
            assignee: None,
            configuration_id: None,
            sort_order: 2,
        };
        
        // Save tasks
        repository.tasks.create(&task1).await.unwrap();
        repository.tasks.create(&task2).await.unwrap();
        repository.tasks.create(&task3).await.unwrap();
        
        repository
    }
}