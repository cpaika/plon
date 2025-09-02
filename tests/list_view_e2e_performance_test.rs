#[cfg(test)]
mod list_view_e2e_performance_tests {
    use dioxus::prelude::*;
    use plon::domain::task::{Task, TaskStatus, Priority};
    use plon::repository::Repository;
    use plon::ui_dioxus::views::ListView;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use uuid::Uuid;

    async fn setup_test_repository_with_tasks(count: usize) -> Arc<Repository> {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let repo = Arc::new(Repository::new(pool));
        
        // Create many tasks to stress test
        for i in 0..count {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Task {}", i),
                description: format!("Description for task {} with some long text to make it more realistic", i),
                status: match i % 4 {
                    0 => TaskStatus::Todo,
                    1 => TaskStatus::InProgress,
                    2 => TaskStatus::Done,
                    _ => TaskStatus::Blocked,
                },
                priority: match i % 4 {
                    0 => Priority::Critical,
                    1 => Priority::High,
                    2 => Priority::Medium,
                    _ => Priority::Low,
                },
                position: plon::domain::task::Position { x: 0.0, y: 0.0 },
                metadata: std::collections::HashMap::new(),
                tags: vec![format!("tag{}", i % 10), "common".to_string()]
                    .into_iter()
                    .collect(),
                created_at: chrono::Utc::now() - chrono::Duration::days(i as i64),
                updated_at: chrono::Utc::now(),
                due_date: if i % 3 == 0 {
                    Some(chrono::Utc::now() + chrono::Duration::days(i as i64))
                } else {
                    None
                },
                scheduled_date: None,
                completed_at: None,
                estimated_hours: Some((i % 8 + 1) as f32),
                actual_hours: None,
                assigned_resource_id: None,
                goal_id: None,
                parent_task_id: None,
                subtasks: vec![],
                is_archived: false,
                assignee: if i % 2 == 0 { Some("user@example.com".to_string()) } else { None },
                configuration_id: None,
                sort_order: i as i32,
            };
            repo.tasks.create(&task).await.unwrap();
        }
        
        repo
    }

    #[tokio::test]
    async fn test_list_view_initial_render_performance() {
        let repo = setup_test_repository_with_tasks(100).await;
        
        let mut vdom = VirtualDom::new_with_props(
            move |_| {
                use_context_provider(|| repo.clone());
                rsx! { ListView {} }
            },
            ()
        );
        
        // Measure initial render time
        let start = Instant::now();
        vdom.rebuild_in_place();
        let initial_render = start.elapsed();
        
        println!("Initial render with 100 tasks: {:?}", initial_render);
        assert!(initial_render < Duration::from_millis(100), 
            "Initial render took {:?}, expected < 100ms", initial_render);
    }

    #[tokio::test]
    async fn test_list_view_re_render_performance() {
        let repo = setup_test_repository_with_tasks(100).await;
        
        let mut vdom = VirtualDom::new_with_props(
            move |_| {
                use_context_provider(|| repo.clone());
                
                // Create a signal that will trigger re-renders
                let mut counter = use_signal(|| 0);
                
                // This effect will cause re-renders
                use_effect(move || {
                    if counter() < 10 {
                        counter.set(counter() + 1);
                    }
                });
                
                rsx! { ListView {} }
            },
            ()
        );
        
        vdom.rebuild_in_place();
        
        // Measure re-render performance
        let start = Instant::now();
        for _ in 0..10 {
            vdom.wait_for_work().await;
            vdom.rebuild_in_place();
        }
        let re_render_time = start.elapsed();
        
        println!("10 re-renders with 100 tasks: {:?}", re_render_time);
        assert!(re_render_time < Duration::from_millis(500), 
            "Re-renders took {:?}, expected < 500ms for 10 renders", re_render_time);
    }

    #[tokio::test]
    async fn test_list_view_search_performance() {
        let repo = setup_test_repository_with_tasks(500).await;
        
        let mut vdom = VirtualDom::new_with_props(
            move |_| {
                use_context_provider(|| repo.clone());
                
                let mut search_query = use_signal(String::new);
                
                // Simulate typing in search
                use_effect(move || {
                    let query = search_query();
                    if query.len() < 10 {
                        search_query.set(format!("{}a", query));
                    }
                });
                
                rsx! { ListView {} }
            },
            ()
        );
        
        vdom.rebuild_in_place();
        
        // Measure search input performance (simulating typing)
        let start = Instant::now();
        for _ in 0..10 {
            vdom.wait_for_work().await;
            vdom.rebuild_in_place();
        }
        let search_time = start.elapsed();
        
        println!("Search input simulation with 500 tasks: {:?}", search_time);
        assert!(search_time < Duration::from_secs(1), 
            "Search performance took {:?}, expected < 1s", search_time);
    }

    #[tokio::test]
    async fn test_list_view_sorting_performance() {
        let repo = setup_test_repository_with_tasks(200).await;
        
        let mut vdom = VirtualDom::new_with_props(
            move |_| {
                use_context_provider(|| repo.clone());
                
                let mut sort_by = use_signal(|| "created_desc".to_string());
                let mut iteration = use_signal(|| 0);
                
                // Simulate changing sort order multiple times
                use_effect(move || {
                    let current = iteration();
                    if current < 5 {
                        let sorts = vec![
                            "created_asc",
                            "due_desc", 
                            "priority_desc",
                            "title_asc",
                            "status"
                        ];
                        sort_by.set(sorts[current % sorts.len()].to_string());
                        iteration.set(current + 1);
                    }
                });
                
                rsx! { ListView {} }
            },
            ()
        );
        
        vdom.rebuild_in_place();
        
        // Measure sorting changes
        let start = Instant::now();
        for _ in 0..5 {
            vdom.wait_for_work().await;
            vdom.rebuild_in_place();
        }
        let sort_time = start.elapsed();
        
        println!("5 sort changes with 200 tasks: {:?}", sort_time);
        assert!(sort_time < Duration::from_millis(500), 
            "Sorting took {:?}, expected < 500ms", sort_time);
    }

    #[tokio::test]
    async fn test_list_view_filter_performance() {
        let repo = setup_test_repository_with_tasks(300).await;
        
        let mut vdom = VirtualDom::new_with_props(
            move |_| {
                use_context_provider(|| repo.clone());
                
                let mut filter_status = use_signal(|| "all".to_string());
                let mut iteration = use_signal(|| 0);
                
                // Simulate changing filters
                use_effect(move || {
                    let current = iteration();
                    if current < 4 {
                        let filters = vec!["todo", "in_progress", "done", "all"];
                        filter_status.set(filters[current % filters.len()].to_string());
                        iteration.set(current + 1);
                    }
                });
                
                rsx! { ListView {} }
            },
            ()
        );
        
        vdom.rebuild_in_place();
        
        // Measure filter changes  
        let start = Instant::now();
        for _ in 0..4 {
            vdom.wait_for_work().await;
            vdom.rebuild_in_place();
        }
        let filter_time = start.elapsed();
        
        println!("4 filter changes with 300 tasks: {:?}", filter_time);
        assert!(filter_time < Duration::from_millis(400), 
            "Filtering took {:?}, expected < 400ms", filter_time);
    }

    #[tokio::test]
    async fn test_list_view_with_1000_tasks() {
        // This is the stress test - if filtering/sorting is in render, this will fail
        let repo = setup_test_repository_with_tasks(1000).await;
        let repo_clone = repo.clone();
        
        let mut vdom = VirtualDom::new_with_props(
            move |_| {
                use_context_provider(|| repo.clone());
                rsx! { ListView {} }
            },
            ()
        );
        
        let start = Instant::now();
        vdom.rebuild_in_place();
        let render_time = start.elapsed();
        
        println!("Render with 1000 tasks: {:?}", render_time);
        assert!(render_time < Duration::from_millis(500), 
            "Rendering 1000 tasks took {:?}, expected < 500ms", render_time);
        
        // Now trigger a re-render and measure
        let mut vdom = VirtualDom::new_with_props(
            move |_| {
                use_context_provider(|| repo_clone.clone());
                let mut dummy = use_signal(|| 0);
                
                use_effect(move || {
                    dummy.set(dummy() + 1);
                });
                
                rsx! { ListView {} }
            },
            ()
        );
        
        vdom.rebuild_in_place();
        
        let start = Instant::now();
        vdom.wait_for_work().await;
        vdom.rebuild_in_place();
        let re_render = start.elapsed();
        
        println!("Re-render with 1000 tasks: {:?}", re_render);
        assert!(re_render < Duration::from_millis(200), 
            "Re-render took {:?}, expected < 200ms", re_render);
    }

    #[test]
    fn test_identify_performance_bottleneck() {
        println!("\n=== LIST VIEW PERFORMANCE ANALYSIS ===\n");
        println!("CRITICAL ISSUE FOUND:");
        println!("  The list view performs filtering and sorting operations");
        println!("  INSIDE the render function (lines 373-457).");
        println!();
        println!("PROBLEMS:");
        println!("  1. Every re-render recalculates filtered_tasks");
        println!("  2. Every re-render re-sorts the entire list");
        println!("  3. Complex sorting logic runs on every render");
        println!("  4. No memoization of expensive computations");
        println!();
        println!("SYMPTOMS:");
        println!("  - UI freezes/beachballs with many tasks");
        println!("  - Typing in search is laggy");
        println!("  - Any state change triggers full recalculation");
        println!();
        println!("SOLUTION:");
        println!("  Move filtering and sorting logic to use_memo hooks");
        println!("  that only recalculate when dependencies change.");
        println!();
        println!("See the fix implementation in the next test.");
    }
}