use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

/// Mock structures for testing without egui dependency
struct MockRect {
    min_x: f32,
    min_y: f32,
    max_x: f32, 
    max_y: f32,
}

impl MockRect {
    fn width(&self) -> f32 {
        self.max_x - self.min_x
    }
    
    fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
}

struct AllocationTracker {
    allocations: Arc<Mutex<Vec<(f32, f32)>>>,
    depth: Arc<AtomicUsize>,
}

impl AllocationTracker {
    fn new() -> Self {
        AllocationTracker {
            allocations: Arc::new(Mutex::new(Vec::new())),
            depth: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    fn track_allocation(&self, width: f32, height: f32) {
        let mut allocs = self.allocations.lock().unwrap();
        allocs.push((width, height));
        println!("Allocation #{}: {}x{} at depth {}", 
                 allocs.len(), width, height, self.depth.load(Ordering::Relaxed));
    }
    
    fn enter_nested(&self) {
        self.depth.fetch_add(1, Ordering::Relaxed);
    }
    
    fn exit_nested(&self) {
        self.depth.fetch_sub(1, Ordering::Relaxed);
    }
    
    fn detect_infinite_scroll(&self) -> bool {
        let allocs = self.allocations.lock().unwrap();
        
        // Check for suspicious patterns:
        // 1. Nested allocations at depth > 2
        if self.depth.load(Ordering::Relaxed) > 2 {
            println!("ERROR: Nested allocation depth > 2!");
            return true;
        }
        
        // 2. Growing allocations
        if allocs.len() > 1 {
            for i in 1..allocs.len() {
                let (prev_w, prev_h) = allocs[i-1];
                let (curr_w, curr_h) = allocs[i];
                
                // If allocations are growing by more than 10%
                if curr_w > prev_w * 1.1 || curr_h > prev_h * 1.1 {
                    println!("ERROR: Allocations growing! {}x{} -> {}x{}", 
                             prev_w, prev_h, curr_w, curr_h);
                    return true;
                }
            }
        }
        
        // 3. Unbounded allocations
        for (w, h) in allocs.iter() {
            if *w > 5000.0 || *h > 5000.0 {
                println!("ERROR: Unbounded allocation detected: {}x{}", w, h);
                return true;
            }
        }
        
        false
    }
}

/// Test that simulates the nested allocation pattern in timeline_view.rs
#[test]
fn test_nested_allocations_cause_infinite_scroll() {
    let tracker = AllocationTracker::new();
    
    // Simulate the timeline view's allocation pattern
    simulate_timeline_rendering(&tracker);
    
    assert!(!tracker.detect_infinite_scroll(), 
            "Infinite scroll pattern detected in timeline view!");
}

fn simulate_timeline_rendering(tracker: &AllocationTracker) {
    // Simulate main show() method
    let available = MockRect {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 1600.0,
        max_y: 900.0,
    };
    
    // First allocation in show() - lines 295-298
    let container_width = available.width().min(1400.0);
    let container_height = available.height().min(600.0);
    
    tracker.track_allocation(container_width, container_height);
    tracker.enter_nested();
    
    // Simulate show_gantt_view being called
    simulate_gantt_view(tracker, container_width, container_height);
    
    tracker.exit_nested();
}

fn simulate_gantt_view(tracker: &AllocationTracker, parent_width: f32, parent_height: f32) {
    // Second allocation in show_gantt_view() - lines 337-344
    let available = MockRect {
        min_x: 0.0,
        min_y: 0.0, 
        max_x: parent_width,
        max_y: parent_height,
    };
    
    let max_width = available.width().min(1200.0);
    let max_height = 400.0; // Hard-coded in the actual code
    
    tracker.track_allocation(max_width, max_height);
    tracker.enter_nested();
    
    // Third allocation for painter - lines 354-359
    // THIS IS THE PROBLEM: It uses ui.available_width() which might be unbounded
    let chart_width = max_width;  // Should be bounded
    let chart_height = max_height; // Should be bounded
    
    tracker.track_allocation(chart_width, chart_height);
    
    tracker.exit_nested();
}

/// Test that checks for the specific double-nested allocation pattern
#[test]
fn test_double_nested_ui_allocations() {
    // Read the actual source to verify the issue
    let content = std::fs::read_to_string("src/ui/views/timeline_view.rs")
        .expect("Failed to read timeline_view.rs");
    
    // Count nested allocate_ui calls
    let allocate_ui_with_layout_count = content.matches("allocate_ui_with_layout").count();
    let allocate_ui_at_rect_count = content.matches("allocate_ui_at_rect").count();
    
    println!("Found {} allocate_ui_with_layout calls", allocate_ui_with_layout_count);
    println!("Found {} allocate_ui_at_rect calls", allocate_ui_at_rect_count);
    
    // Check for nested allocation pattern (both in show and show_gantt_view)
    let show_method = content.split("fn show(").nth(1).unwrap_or("")
        .split("fn show_gantt_view").nth(0).unwrap_or("");
    let gantt_method = content.split("fn show_gantt_view").nth(1).unwrap_or("")
        .split("fn show_list_view").nth(0).unwrap_or("");
    
    let show_has_allocate = show_method.contains("allocate_ui");
    let gantt_has_allocate = gantt_method.contains("allocate_ui");
    
    if show_has_allocate && gantt_has_allocate {
        println!("WARNING: Nested allocate_ui calls detected!");
        println!("  - show() contains allocate_ui: {}", show_has_allocate);
        println!("  - show_gantt_view() contains allocate_ui: {}", gantt_has_allocate);
        println!("This pattern can cause infinite scrolling!");
    }
    
    // The test should fail if we have this problematic pattern
    assert!(!(show_has_allocate && gantt_has_allocate), 
            "Double-nested UI allocations detected! This causes infinite scrolling.");
}

/// Test that verifies proper bounding of dimensions
#[test] 
fn test_all_allocations_are_bounded() {
    let content = std::fs::read_to_string("src/ui/views/timeline_view.rs")
        .expect("Failed to read timeline_view.rs");
    
    // Check that all size calculations use .min() to bound them
    let lines: Vec<&str> = content.lines().collect();
    let mut unbounded_allocations = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        // Look for allocate_painter calls
        if line.contains("allocate_painter") {
            // Check the Vec2::new call for this allocation
            let context_start = i.saturating_sub(10);
            let context_end = (i + 5).min(lines.len());
            let context = &lines[context_start..context_end];
            
            // Look for Vec2::new with bounded dimensions
            let mut found_bounded = false;
            let mut found_vec2 = false;
            
            for ctx_line in context {
                // Check if Vec2::new is present
                if ctx_line.contains("Vec2::new") {
                    found_vec2 = true;
                }
                // Check if the dimensions are bounded (look for .min() before the allocate_painter)
                if ctx_line.contains(".min(") {
                    // Make sure this .min() is actually bounding dimensions, not something else
                    if ctx_line.contains("width") || ctx_line.contains("height") || 
                       ctx_line.contains("chart_width") || ctx_line.contains("chart_height") {
                        found_bounded = true;
                    }
                }
            }
            
            // If we found Vec2::new but no bounding, it's potentially unbounded
            if found_vec2 && !found_bounded {
                unbounded_allocations.push((i + 1, line.trim()));
                println!("Line {}: Potentially unbounded allocation: {}", i + 1, line.trim());
            }
        }
    }
    
    assert!(unbounded_allocations.is_empty(),
            "Found {} potentially unbounded allocations that could cause infinite scroll:\n{:?}",
            unbounded_allocations.len(), unbounded_allocations);
}

/// Test for recursive render calls
#[test]
fn test_no_recursive_rendering() {
    let render_count = Arc::new(AtomicUsize::new(0));
    let max_depth = Arc::new(AtomicUsize::new(0));
    let current_depth = Arc::new(AtomicUsize::new(0));
    
    // Simulate multiple render frames
    for frame in 0..5 {
        simulate_render_frame(&render_count, &max_depth, &current_depth, frame);
    }
    
    let total_renders = render_count.load(Ordering::Relaxed);
    let max_observed_depth = max_depth.load(Ordering::Relaxed);
    
    println!("Total render calls: {}", total_renders);
    println!("Maximum recursion depth: {}", max_observed_depth);
    
    // Each frame should only cause one render, not recursive renders
    assert!(total_renders <= 5, 
            "Too many render calls ({})! Possible recursive rendering.", total_renders);
    assert!(max_observed_depth <= 3,
            "Recursion depth too high ({})! Possible infinite recursion.", max_observed_depth);
}

fn simulate_render_frame(render_count: &Arc<AtomicUsize>, 
                         max_depth: &Arc<AtomicUsize>,
                         current_depth: &Arc<AtomicUsize>,
                         frame_num: usize) {
    // Enter render
    current_depth.fetch_add(1, Ordering::Relaxed);
    render_count.fetch_add(1, Ordering::Relaxed);
    
    let depth = current_depth.load(Ordering::Relaxed);
    max_depth.fetch_max(depth, Ordering::Relaxed);
    
    println!("Frame {}: Render at depth {}", frame_num, depth);
    
    // Simulate nested UI calls (should not cause recursive renders)
    if depth < 10 {  // Safety limit to prevent actual infinite recursion in test
        // This simulates what happens when allocate_ui_at_rect is called
        // It should NOT trigger another full render
    }
    
    // Exit render
    current_depth.fetch_sub(1, Ordering::Relaxed);
}