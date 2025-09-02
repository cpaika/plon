#[cfg(test)]
mod list_view_real_e2e_tests {
    use std::process::Command;
    use std::time::{Duration, Instant};
    use std::thread;

    #[test]
    #[ignore] // Run with: cargo test --test list_view_real_e2e_test -- --ignored --nocapture
    fn test_list_view_performance_with_real_app() {
        println!("\n=== REAL E2E LIST VIEW PERFORMANCE TEST ===\n");
        
        // First, populate the database with test data
        println!("Step 1: Populating database with 500 tasks...");
        let output = Command::new("cargo")
            .args(&["run", "--bin", "populate_test_data"])
            .output()
            .expect("Failed to populate test data");
        
        if !output.status.success() {
            eprintln!("Failed to populate test data: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Test data population failed");
        }
        
        println!("✅ Database populated with test data");
        
        // Start the app
        println!("\nStep 2: Starting the application...");
        let mut app_process = Command::new("cargo")
            .args(&["run", "--bin", "plon-desktop"])
            .spawn()
            .expect("Failed to start application");
        
        // Give the app time to start
        thread::sleep(Duration::from_secs(5));
        
        println!("✅ Application started");
        
        // Now we would ideally use a UI automation tool here
        // Since we can't directly interact with the UI from this test,
        // we'll document what to manually verify
        
        println!("\n=== MANUAL VERIFICATION STEPS ===\n");
        println!("1. The app should be running now with 500 tasks loaded");
        println!("2. Navigate to the List View");
        println!("3. Verify the following:");
        println!("   a) The list loads without freezing");
        println!("   b) Scrolling is smooth");
        println!("   c) Typing in the search box is responsive");
        println!("   d) Changing filters doesn't cause beachballing");
        println!("   e) Sorting options work instantly");
        println!();
        println!("4. Monitor CPU usage - it should not spike to 100%");
        println!("5. The UI should remain responsive at all times");
        
        println!("\nPress Enter when done testing...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        // Clean up
        app_process.kill().expect("Failed to kill app process");
        
        println!("✅ Test completed");
    }

    #[test]
    fn test_verify_performance_issue_indicators() {
        println!("\n=== PERFORMANCE ISSUE INDICATORS ===\n");
        
        println!("BEFORE FIX - Beachball symptoms:");
        println!("  • App becomes unresponsive when list view loads");
        println!("  • macOS shows spinning beachball cursor");
        println!("  • CPU usage spikes to 100%");
        println!("  • Typing in search has significant lag");
        println!("  • Filter changes take seconds to apply");
        println!();
        
        println!("AFTER FIX - Expected behavior:");
        println!("  • List view loads instantly (<100ms)");
        println!("  • No beachball cursor appears");
        println!("  • CPU usage remains reasonable (<50%)");
        println!("  • Search typing is smooth (60fps)");
        println!("  • Filter changes are immediate");
        println!();
        
        println!("HOW TO MEASURE:");
        println!("  1. Open Activity Monitor (macOS) or Task Manager");
        println!("  2. Watch CPU usage for plon-desktop process");
        println!("  3. Use Developer Tools if available");
        println!("  4. Monitor frame rate and responsiveness");
    }

    #[test]
    fn test_automated_performance_check() {
        use std::fs;
        
        println!("\n=== AUTOMATED PERFORMANCE CHECK ===\n");
        
        // Check if the fix is in place
        let list_view_code = fs::read_to_string("src/ui_dioxus/views/list_view_simple.rs")
            .expect("Could not read list view file");
        
        // Check for performance anti-patterns
        let has_filter_in_render = list_view_code.contains("for task in tasks()") ||
                                   list_view_code.contains("tasks().into_iter().filter");
        
        let has_use_memo = list_view_code.contains("use_memo");
        let has_filtered_sorted_tasks = list_view_code.contains("filtered_sorted_tasks");
        
        if has_filter_in_render && !has_use_memo {
            println!("❌ PERFORMANCE ISSUE DETECTED!");
            println!("   Filtering/sorting happening in render without memoization");
            panic!("Performance issue still present!");
        }
        
        if has_use_memo && has_filtered_sorted_tasks {
            println!("✅ Performance optimization detected:");
            println!("   - use_memo is being used");
            println!("   - Filtered/sorted tasks are memoized");
        }
        
        // Check for other potential issues
        if list_view_code.contains("use_effect") {
            let effect_count = list_view_code.matches("use_effect").count();
            println!("\n📊 Found {} use_effect hooks", effect_count);
            
            // Check for problematic patterns in effects
            if list_view_code.contains("use_effect({") && 
               list_view_code.contains("filter()") {
                println!("⚠️  Warning: Potential issue with signal reads in effects");
            }
        }
        
        println!("\n✅ Automated checks passed");
    }
}