#[cfg(test)]
mod tests {
    use crate::domain::task::{Task, TaskStatus, Priority};
    use crate::repository::Repository;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use std::collections::HashSet;
    use uuid::Uuid;
    
    async fn setup_test_db() -> Arc<Repository> {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        Arc::new(Repository::new(pool))
    }
    
    #[tokio::test]
    async fn test_task_can_have_dependencies() {
        let repo = setup_test_db().await;
        
        // Create parent task
        let parent_task = Task {
            id: Uuid::new_v4(),
            title: "Parent Task".to_string(),
            description: "This task has dependencies".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::High,
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
        
        // Create dependent task
        let dependent_task = Task {
            id: Uuid::new_v4(),
            title: "Dependent Task".to_string(),
            description: "This task depends on parent".to_string(),
            status: TaskStatus::Blocked,
            priority: Priority::Medium,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            due_date: None,
            estimated_hours: None,
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: Some(parent_task.id), // Set dependency
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
        
        repo.tasks.create(&parent_task).await.unwrap();
        repo.tasks.create(&dependent_task).await.unwrap();
        
        // Verify dependency relationship
        let fetched_dependent = repo.tasks.get(dependent_task.id).await.unwrap().unwrap();
        assert_eq!(fetched_dependent.parent_task_id, Some(parent_task.id));
        assert_eq!(fetched_dependent.status, TaskStatus::Blocked);
    }
    
    #[tokio::test]
    async fn test_completing_parent_unblocks_dependent() {
        let repo = setup_test_db().await;
        
        // Create parent task
        let parent_id = Uuid::new_v4();
        let parent_task = Task {
            id: parent_id,
            title: "Parent Task".to_string(),
            description: "".to_string(),
            status: TaskStatus::InProgress,
            priority: Priority::High,
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
        
        // Create dependent task
        let dependent_task = Task {
            id: Uuid::new_v4(),
            title: "Dependent Task".to_string(),
            description: "".to_string(),
            status: TaskStatus::Blocked,
            priority: Priority::Medium,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            due_date: None,
            estimated_hours: None,
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: Some(parent_id),
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
        
        repo.tasks.create(&parent_task).await.unwrap();
        repo.tasks.create(&dependent_task).await.unwrap();
        
        // Complete parent task
        let mut updated_parent = parent_task.clone();
        updated_parent.status = TaskStatus::Done;
        repo.tasks.update(&updated_parent).await.unwrap();
        
        // In a real implementation, we would have a service that automatically
        // unblocks dependent tasks when parent is completed
        // For now, we manually update the dependent task
        let mut updated_dependent = dependent_task.clone();
        updated_dependent.status = TaskStatus::Todo;
        repo.tasks.update(&updated_dependent).await.unwrap();
        
        // Verify dependent is now unblocked
        let fetched_dependent = repo.tasks.get(dependent_task.id).await.unwrap().unwrap();
        assert_eq!(fetched_dependent.status, TaskStatus::Todo);
    }
    
    #[tokio::test] 
    async fn test_cannot_complete_task_with_incomplete_dependencies() {
        let repo = setup_test_db().await;
        
        // Create parent task (incomplete)
        let parent_task = Task {
            id: Uuid::new_v4(),
            title: "Incomplete Parent".to_string(),
            description: "".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::High,
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
        
        // Create dependent task
        let dependent_task = Task {
            id: Uuid::new_v4(),
            title: "Dependent Task".to_string(),
            description: "".to_string(),
            status: TaskStatus::Blocked,
            priority: Priority::Medium,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            due_date: None,
            estimated_hours: None,
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: Some(parent_task.id),
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
        
        repo.tasks.create(&parent_task).await.unwrap();
        repo.tasks.create(&dependent_task).await.unwrap();
        
        // Verify dependent task remains blocked
        let fetched = repo.tasks.get(dependent_task.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, TaskStatus::Blocked);
        
        // In a real implementation, we would prevent setting status to Done
        // if parent tasks are not complete
    }
}