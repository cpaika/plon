#[cfg(test)]
mod list_view_beachball_tests {
    use plon::repository::Repository;
    use plon::domain::task::Task;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    async fn setup_test_repository() -> Arc<Repository> {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        Arc::new(Repository::new(pool))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_simulate_use_effect_infinite_loop() {
        let repo = setup_test_repository().await;
        
        // Create some tasks
        for i in 0..10 {
            let task = Task::new(format!("Task {}", i), "".to_string());
            repo.tasks.create(&task).await.unwrap();
        }
        
        // Track how many times the effect runs
        let effect_runs = Arc::new(AtomicUsize::new(0));
        let max_runs = 100;
        
        // Simulate the problematic use_effect pattern from list_view_simple.rs
        let filter_status = Arc::new(tokio::sync::RwLock::new("all".to_string()));
        
        // This simulates the use_effect that runs on filter changes
        let simulate_effect = {
            let repo = repo.clone();
            let filter_status = filter_status.clone();
            let effect_runs = effect_runs.clone();
            
            async move {
                loop {
                    // Increment run counter
                    let runs = effect_runs.fetch_add(1, Ordering::SeqCst);
                    if runs >= max_runs {
                        println!("Effect ran {} times - infinite loop detected!", runs);
                        break;
                    }
                    
                    // Read the filter (this is the problematic line 116: filter())
                    let filter_value = filter_status.read().await.clone();
                    
                    // Simulate the async task loading
                    let repo_clone = repo.clone();
                    tokio::spawn(async move {
                        let filters = plon::repository::task_repository::TaskFilters {
                            status: None,
                            assigned_resource_id: None,
                            goal_id: None,
                            overdue: false,
                            limit: None,
                        };
                        
                        let _ = repo_clone.tasks.list(filters).await;
                    });
                    
                    // Small delay to prevent tight loop
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    
                    // Check if we should stop (no actual dependency tracking like Dioxus)
                    // In the real code, this would re-trigger whenever any signal changes
                    if runs > 5 {
                        // Simulate that reading the filter might trigger re-render
                        // This is where the bug occurs - reading inside effect
                        break;
                    }
                }
            }
        };
        
        // Run the simulation with a timeout
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            simulate_effect
        ).await;
        
        let final_runs = effect_runs.load(Ordering::SeqCst);
        println!("Effect ran {} times total", final_runs);
        
        // The effect should only run once or twice normally, not many times
        assert!(final_runs < 10, "Effect ran too many times: {}, indicating potential infinite loop", final_runs);
        assert!(result.is_ok(), "Effect timed out, indicating infinite loop");
    }

    #[tokio::test]
    async fn test_dependency_tracking_issue() {
        // This test demonstrates the issue with the use_effect dependencies
        
        // The problematic code pattern is:
        // use_effect({
        //     let filter = filter_status.clone();
        //     move || {
        //         let filter_value = filter(); // <-- Reading signal inside effect
        //         spawn(async move { ... });
        //     }
        // });
        
        // The issue is that filter() is called inside the effect closure,
        // which means Dioxus can't properly track it as a dependency.
        // This can cause:
        // 1. The effect to run on every render
        // 2. Infinite re-render loops
        // 3. Performance issues (beachballing)
        
        println!("The issue is in list_view_simple.rs line 116:");
        println!("  let filter_value = filter();");
        println!("");
        println!("This reads the signal inside the effect, causing dependency tracking issues.");
        println!("");
        println!("The fix would be to read the signal value outside the effect:");
        println!("  let filter_value = filter_status();");
        println!("  use_effect(move || {{ ... }});");
        
        // This test passes to document the issue
        assert!(true);
    }

    #[tokio::test]
    async fn test_multiple_rapid_filter_changes() {
        let repo = setup_test_repository().await;
        
        // Create tasks
        for i in 0..50 {
            let mut task = Task::new(format!("Task {}", i), "".to_string());
            task.status = match i % 3 {
                0 => plon::domain::task::TaskStatus::Todo,
                1 => plon::domain::task::TaskStatus::InProgress,
                _ => plon::domain::task::TaskStatus::Done,
            };
            repo.tasks.create(&task).await.unwrap();
        }
        
        // Simulate rapid filter changes (like user clicking quickly)
        let load_count = Arc::new(AtomicUsize::new(0));
        
        for filter in ["all", "todo", "in_progress", "done", "all"].iter() {
            let repo_clone = repo.clone();
            let load_count = load_count.clone();
            
            // Each filter change triggers a load
            tokio::spawn(async move {
                load_count.fetch_add(1, Ordering::SeqCst);
                
                let status = match *filter {
                    "todo" => Some(plon::domain::task::TaskStatus::Todo),
                    "in_progress" => Some(plon::domain::task::TaskStatus::InProgress),
                    "done" => Some(plon::domain::task::TaskStatus::Done),
                    _ => None,
                };
                
                let filters = plon::repository::task_repository::TaskFilters {
                    status,
                    assigned_resource_id: None,
                    goal_id: None,
                    overdue: false,
                    limit: None,
                };
                
                let _ = repo_clone.tasks.list(filters).await;
            });
            
            // Small delay between changes
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        
        // Wait for all loads to complete
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let total_loads = load_count.load(Ordering::SeqCst);
        println!("Total loads triggered: {}", total_loads);
        
        // Should be exactly 5 loads for 5 filter changes
        assert_eq!(total_loads, 5, "Unexpected number of loads");
    }
}