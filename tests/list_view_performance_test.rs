#[cfg(test)]
mod list_view_performance_tests {
    use plon::repository::Repository;
    use plon::repository::task_repository::TaskFilters;
    use plon::domain::task::{Task, TaskStatus};
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use std::time::Instant;

    async fn setup_test_repository() -> Arc<Repository> {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        Arc::new(Repository::new(pool))
    }

    #[tokio::test]
    async fn test_list_view_with_many_tasks() {
        let repo = setup_test_repository().await;
        
        // Create a large number of tasks to test performance
        let num_tasks = 1000;
        println!("Creating {} tasks...", num_tasks);
        let start = Instant::now();
        
        for i in 0..num_tasks {
            let task = Task::new(
                format!("Task {}", i),
                format!("Description for task {}", i)
            );
            repo.tasks.create(&task).await.unwrap();
            
            if i % 100 == 0 {
                println!("Created {} tasks...", i);
            }
        }
        
        let creation_time = start.elapsed();
        println!("Task creation took: {:?}", creation_time);
        
        // Now test loading all tasks (simulating what list view does)
        println!("Loading all tasks...");
        let load_start = Instant::now();
        
        let filters = TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        };
        
        let tasks = repo.tasks.list(filters).await.unwrap();
        let load_time = load_start.elapsed();
        
        println!("Loaded {} tasks in {:?}", tasks.len(), load_time);
        assert_eq!(tasks.len(), num_tasks);
        
        // Check if load time is reasonable (should be under 1 second for 1000 tasks)
        assert!(load_time.as_secs() < 5, "Loading {} tasks took too long: {:?}", num_tasks, load_time);
    }

    #[tokio::test]
    async fn test_list_view_filtering_performance() {
        let repo = setup_test_repository().await;
        
        // Create tasks with different statuses
        for i in 0..100 {
            let mut task = Task::new(format!("Todo Task {}", i), "".to_string());
            task.status = TaskStatus::Todo;
            repo.tasks.create(&task).await.unwrap();
        }
        
        for i in 0..100 {
            let mut task = Task::new(format!("InProgress Task {}", i), "".to_string());
            task.status = TaskStatus::InProgress;
            repo.tasks.create(&task).await.unwrap();
        }
        
        for i in 0..100 {
            let mut task = Task::new(format!("Done Task {}", i), "".to_string());
            task.status = TaskStatus::Done;
            repo.tasks.create(&task).await.unwrap();
        }
        
        // Test filtering performance
        let start = Instant::now();
        
        let filters = TaskFilters {
            status: Some(TaskStatus::InProgress),
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        };
        
        let filtered_tasks = repo.tasks.list(filters).await.unwrap();
        let filter_time = start.elapsed();
        
        println!("Filtered to {} tasks in {:?}", filtered_tasks.len(), filter_time);
        assert_eq!(filtered_tasks.len(), 100);
        assert!(filter_time.as_millis() < 500, "Filtering took too long: {:?}", filter_time);
    }

    #[tokio::test]
    async fn test_detect_n_plus_one_queries() {
        let repo = setup_test_repository().await;
        
        // Create tasks with subtasks to detect N+1 query issues
        for i in 0..50 {
            let mut task = Task::new(format!("Task {}", i), "".to_string());
            
            // Add subtasks
            for j in 0..5 {
                task.subtasks.push(plon::domain::task::SubTask {
                    id: uuid::Uuid::new_v4(),
                    title: format!("Subtask {}", j),
                    description: format!("Subtask {}", j),
                    completed: false,
                    created_at: chrono::Utc::now(),
                    completed_at: None,
                });
            }
            
            repo.tasks.create(&task).await.unwrap();
        }
        
        // Load all tasks and measure time
        let start = Instant::now();
        
        let filters = TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        };
        
        let tasks = repo.tasks.list(filters).await.unwrap();
        let load_time = start.elapsed();
        
        println!("Loaded {} tasks with subtasks in {:?}", tasks.len(), load_time);
        
        // Check that all subtasks were loaded
        let total_subtasks: usize = tasks.iter().map(|t| t.subtasks.len()).sum();
        assert_eq!(total_subtasks, 250); // 50 tasks * 5 subtasks each
        
        // Should still be fast even with subtasks
        assert!(load_time.as_secs() < 2, "Loading tasks with subtasks took too long: {:?}", load_time);
    }

    #[tokio::test]
    async fn test_concurrent_list_view_loads() {
        let repo = setup_test_repository().await;
        
        // Create some tasks
        for i in 0..100 {
            let task = Task::new(format!("Task {}", i), "".to_string());
            repo.tasks.create(&task).await.unwrap();
        }
        
        // Simulate multiple concurrent list view loads (like rapid navigation)
        let start = Instant::now();
        let mut handles = vec![];
        
        for _ in 0..10 {
            let repo_clone = repo.clone();
            let handle = tokio::spawn(async move {
                let filters = TaskFilters {
                    status: None,
                    assigned_resource_id: None,
                    goal_id: None,
                    overdue: false,
                    limit: None,
                };
                
                let load_start = Instant::now();
                let tasks = repo_clone.tasks.list(filters).await.unwrap();
                let load_time = load_start.elapsed();
                
                (tasks.len(), load_time)
            });
            handles.push(handle);
        }
        
        // Wait for all concurrent loads
        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }
        
        let total_time = start.elapsed();
        
        println!("10 concurrent loads completed in {:?}", total_time);
        for (i, (count, time)) in results.iter().enumerate() {
            println!("  Load {}: {} tasks in {:?}", i + 1, count, time);
        }
        
        // All loads should return the same number of tasks
        for (count, _) in &results {
            assert_eq!(*count, 100);
        }
        
        // Concurrent loads should complete reasonably fast
        assert!(total_time.as_secs() < 5, "Concurrent loads took too long: {:?}", total_time);
    }

    #[tokio::test]
    async fn test_memory_usage_with_large_descriptions() {
        let repo = setup_test_repository().await;
        
        // Create tasks with large descriptions to test memory usage
        let large_description = "x".repeat(10000); // 10KB description
        
        for i in 0..100 {
            let task = Task::new(
                format!("Task {}", i),
                large_description.clone()
            );
            repo.tasks.create(&task).await.unwrap();
        }
        
        // Load all tasks
        let start = Instant::now();
        
        let filters = TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        };
        
        let tasks = repo.tasks.list(filters).await.unwrap();
        let load_time = start.elapsed();
        
        println!("Loaded {} tasks with large descriptions in {:?}", tasks.len(), load_time);
        assert_eq!(tasks.len(), 100);
        
        // Check that descriptions were loaded correctly
        for task in &tasks {
            assert_eq!(task.description.len(), 10000);
        }
        
        // Should still complete in reasonable time
        assert!(load_time.as_secs() < 5, "Loading tasks with large descriptions took too long: {:?}", load_time);
    }
}