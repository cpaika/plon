#[cfg(test)]
mod gui_beachball_tests {
    use dioxus::prelude::*;
    use plon::repository::Repository;
    use plon::ui_dioxus::views::ListView;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    
    async fn setup_test_db_with_tasks() -> Arc<Repository> {
        // Use the actual database with test data
        let pool = SqlitePool::connect("sqlite:plon.db").await.unwrap();
        Arc::new(Repository::new(pool))
    }

    #[tokio::test]
    async fn test_list_view_initial_load_performance() {
        println!("\n=== TESTING INITIAL LOAD PERFORMANCE ===\n");
        
        let repo = setup_test_db_with_tasks().await;
        
        // Count tasks in DB
        let task_count = repo.tasks.list(Default::default()).await.unwrap().len();
        println!("Testing with {} tasks in database", task_count);
        
        // Create the app
        let mut vdom = VirtualDom::new_with_props(
            move |_| {
                use_context_provider(|| repo.clone());
                rsx! { ListView {} }
            },
            ()
        );
        
        // Measure initial render
        let start = Instant::now();
        vdom.rebuild_in_place();
        let initial_time = start.elapsed();
        
        println!("Initial render time: {:?}", initial_time);
        
        if initial_time > Duration::from_millis(500) {
            println!("⚠️ WARNING: Initial render is slow!");
            println!("This could cause beachballing on real hardware");
        }
        
        // Simulate multiple quick re-renders (like what happens in the real app)
        println!("\nSimulating rapid re-renders...");
        let start = Instant::now();
        for i in 0..10 {
            vdom.wait_for_work().await;
            let render_start = Instant::now();
            vdom.rebuild_in_place();
            let render_time = render_start.elapsed();
            
            if render_time > Duration::from_millis(50) {
                println!("  Render {}: {:?} - TOO SLOW!", i, render_time);
            }
        }
        let total_time = start.elapsed();
        
        println!("Total time for 10 re-renders: {:?}", total_time);
        
        if total_time > Duration::from_secs(1) {
            panic!("❌ BEACHBALL DETECTED: Re-renders are too slow!");
        }
    }

    #[tokio::test]
    async fn test_find_beachball_cause() {
        println!("\n=== ANALYZING BEACHBALL ROOT CAUSE ===\n");
        
        // Let's trace exactly what happens when the list view loads
        let repo = setup_test_db_with_tasks().await;
        
        println!("Step 1: Creating VirtualDom...");
        let start = Instant::now();
        
        let mut vdom = VirtualDom::new_with_props(
            move |_| {
                println!("  Component function called");
                use_context_provider(|| {
                    println!("  Context provider initialized");
                    repo.clone()
                });
                
                // Add instrumentation to see what's happening
                let render_count = use_signal(|| 0);
                
                use_effect(move || {
                    let count = render_count() + 1;
                    render_count.set(count);
                    println!("  Effect triggered - render #{}", count);
                });
                
                println!("  Rendering ListView component");
                rsx! { ListView {} }
            },
            ()
        );
        
        println!("VirtualDom created in {:?}", start.elapsed());
        
        println!("\nStep 2: Initial rebuild...");
        let start = Instant::now();
        vdom.rebuild_in_place();
        println!("Initial rebuild took {:?}", start.elapsed());
        
        println!("\nStep 3: Waiting for async work...");
        let start = Instant::now();
        vdom.wait_for_work().await;
        println!("Async work completed in {:?}", start.elapsed());
        
        println!("\nStep 4: Second rebuild...");
        let start = Instant::now();
        vdom.rebuild_in_place();
        println!("Second rebuild took {:?}", start.elapsed());
        
        // Check for infinite loops
        println!("\nStep 5: Checking for render loops...");
        for i in 0..5 {
            vdom.wait_for_work().await;
            println!("  Checking for work iteration #{}", i);
            vdom.rebuild_in_place();
        }
    }

    #[test]
    fn test_identify_performance_bottlenecks() {
        println!("\n=== PERFORMANCE BOTTLENECK ANALYSIS ===\n");
        
        println!("POTENTIAL CAUSES OF BEACHBALLING:");
        println!("─────────────────────────────────");
        println!();
        
        println!("1. INFINITE RENDER LOOP");
        println!("   Symptom: Component re-renders continuously");
        println!("   Check: use_effect reading signals incorrectly");
        println!();
        
        println!("2. SYNCHRONOUS DATABASE CALLS");
        println!("   Symptom: UI blocks waiting for DB");
        println!("   Check: Any blocking .await in render");
        println!();
        
        println!("3. EXCESSIVE MEMO RECALCULATION");
        println!("   Symptom: use_memo dependencies changing too often");
        println!("   Check: Dependencies that change on every render");
        println!();
        
        println!("4. HEAVY COMPUTATION IN RENDER");
        println!("   Symptom: CPU spikes during render");
        println!("   Check: Unmemoized filtering/sorting");
        println!();
        
        println!("5. SIGNAL SUBSCRIPTION STORMS");
        println!("   Symptom: Too many signal updates");
        println!("   Check: Signals triggering cascading updates");
        println!();
        
        // Check the actual code
        let code = std::fs::read_to_string("src/ui_dioxus/views/list_view_simple.rs")
            .expect("Could not read list view");
        
        println!("CODE ANALYSIS:");
        println!("─────────────");
        
        // Check for potential issues
        if code.contains("use_effect") && code.contains("filter_status()") {
            println!("⚠️  Found use_effect with filter_status() - potential loop");
        }
        
        // Count use_memo calls
        let memo_count = code.matches("use_memo").count();
        println!("  Found {} use_memo hooks", memo_count);
        
        // Count use_effect calls
        let effect_count = code.matches("use_effect").count();
        println!("  Found {} use_effect hooks", effect_count);
        
        // Check for async in render
        if code.contains("spawn") && code.contains("await") {
            println!("  Found async operations - check if they're properly handled");
        }
        
        // Look for problematic patterns
        let lines: Vec<&str> = code.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            // Check for reading signals in effects
            if line.contains("use_effect") {
                // Look ahead for signal reads
                for j in i+1..std::cmp::min(i+10, lines.len()) {
                    if lines[j].contains("()") && !lines[j].contains("move") {
                        if lines[j].contains("filter") || lines[j].contains("tasks") || lines[j].contains("search") {
                            println!("  ⚠️ Line {}: Possible signal read in effect", j+1);
                        }
                    }
                }
            }
        }
    }
}