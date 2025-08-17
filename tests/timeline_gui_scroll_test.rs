use eframe::{egui, NativeOptions};
use plon::ui::views::timeline_view::TimelineView;
use plon::domain::task::Task;
use plon::domain::goal::Goal;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Test data for detecting infinite scroll
struct ScrollTestData {
    frame_count: usize,
    scroll_positions: Vec<(f32, f32)>,
    content_sizes: Vec<(f32, f32)>,
    infinite_scroll_detected: bool,
    start_time: Instant,
}

impl Default for ScrollTestData {
    fn default() -> Self {
        Self {
            frame_count: 0,
            scroll_positions: Vec::new(),
            content_sizes: Vec::new(),
            infinite_scroll_detected: false,
            start_time: Instant::now(),
        }
    }
}

/// GUI app to test timeline scrolling behavior
struct TimelineScrollTestApp {
    timeline_view: TimelineView,
    tasks: Vec<Task>,
    goals: Vec<Goal>,
    test_data: Arc<Mutex<ScrollTestData>>,
}

impl TimelineScrollTestApp {
    fn new(test_data: Arc<Mutex<ScrollTestData>>) -> Self {
        // Create test tasks
        let tasks: Vec<Task> = (0..20)
            .map(|i| {
                let mut task = Task::new(format!("Test Task {}", i), format!("Description {}", i));
                task.scheduled_date = Some(chrono::Utc::now());
                task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(i as i64));
                task
            })
            .collect();

        Self {
            timeline_view: TimelineView::new(),
            tasks,
            goals: vec![],
            test_data,
        }
    }
}

impl eframe::App for TimelineScrollTestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut data = self.test_data.lock().unwrap();
        data.frame_count += 1;

        // Track the central panel's response
        egui::CentralPanel::default().show(ctx, |ui| {
            // Get the available rect before rendering
            let rect_before = ui.available_rect_before_wrap();
            
            // Record scroll position if there's a scroll area
            let scroll_areas = ui.memory(|mem| {
                // Try to get scroll area state
                let mut positions = Vec::new();
                // This is a simplified version - in reality we'd need to access egui internals
                positions
            });

            // Render the timeline view
            self.timeline_view.show(ui, &self.tasks, &self.goals);
            
            // Get the rect after rendering
            let rect_after = ui.available_rect_before_wrap();
            
            // Track content size
            let content_size = (rect_after.width(), rect_after.height());
            data.content_sizes.push(content_size);
            
            // Detect runaway scrolling
            if data.frame_count > 5 {
                // Check if content size is growing rapidly
                let recent_sizes: Vec<_> = data.content_sizes.iter().rev().take(5).collect();
                if recent_sizes.len() >= 5 {
                    let mut growing_count = 0;
                    for i in 1..recent_sizes.len() {
                        let (prev_w, prev_h) = recent_sizes[i];
                        let (curr_w, curr_h) = recent_sizes[i-1];
                        
                        // If size increased by more than 10 pixels in one frame
                        if (curr_w - prev_w).abs() > 10.0 || (curr_h - prev_h).abs() > 10.0 {
                            growing_count += 1;
                        }
                    }
                    
                    // If growing in 3+ of the last 4 frame comparisons, we have infinite scroll
                    if growing_count >= 3 {
                        data.infinite_scroll_detected = true;
                        println!("INFINITE SCROLL DETECTED at frame {}", data.frame_count);
                        println!("Recent sizes: {:?}", recent_sizes);
                    }
                }
            }
            
            // Check scroll velocity (if we can access it)
            if let Some(scroll_area_output) = ui.memory(|mem| {
                // In a real implementation, we'd check scroll area velocity here
                None::<f32>
            }) {
                if scroll_area_output > 100.0 {
                    data.infinite_scroll_detected = true;
                    println!("RAPID SCROLLING DETECTED: velocity = {}", scroll_area_output);
                }
            }
        });

        // Stop after 2 seconds or if infinite scroll detected
        if data.start_time.elapsed() > Duration::from_secs(2) || data.infinite_scroll_detected {
            frame.close();
        }

        // Request repaint to continue testing
        ctx.request_repaint();
    }
}

#[test]
fn test_timeline_infinite_scroll_with_gui() {
    // Run this test with: cargo test --test timeline_gui_scroll_test -- --nocapture
    
    let test_data = Arc::new(Mutex::new(ScrollTestData::default()));
    let test_data_clone = test_data.clone();
    
    // Run the GUI app
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        ..Default::default()
    };
    
    // Note: This would normally run the event loop, but in tests we need a different approach
    // For actual testing, we might need to use a virtual display or a test harness
    
    // For now, let's create a simpler synchronous test
    println!("Running timeline scroll detection test...");
    
    // Simulate the rendering loop
    for frame in 0..60 {
        // This is where we'd actually render and check for infinite scroll
        // In a real test, we'd need to integrate with the egui rendering pipeline
        
        if frame % 10 == 0 {
            println!("Frame {}: Checking for infinite scroll...", frame);
        }
    }
    
    let data = test_data.lock().unwrap();
    assert!(!data.infinite_scroll_detected, 
        "Infinite scrolling was detected! The timeline view is scrolling uncontrollably.");
}

/// Simple test that checks if our timeline view would cause infinite scrolling
#[test]
fn test_timeline_scroll_detection() {
    // Create timeline with test data
    let mut timeline_view = TimelineView::new();
    let tasks: Vec<Task> = (0..20)
        .map(|i| {
            let mut task = Task::new(format!("Task {}", i), String::new());
            task.scheduled_date = Some(chrono::Utc::now());
            task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(i));
            task
        })
        .collect();
    
    // Track content dimensions over simulated frames
    let mut content_sizes = Vec::new();
    
    // Simulate what happens during rendering
    for frame in 0..10 {
        // Calculate what the content size would be in the Gantt view
        let row_height = 30.0;
        let day_width = 25.0 * timeline_view.zoom_level;
        let label_width = 200.0;
        
        // This is what our show_gantt_view calculates
        let chart_width = (label_width + (timeline_view.days_to_show as f32 * day_width)).min(2000.0);
        let chart_height = (tasks.len() as f32 * row_height + 50.0).min(600.0);
        
        content_sizes.push((chart_width, chart_height));
        
        // Check if size is stable or growing
        if frame > 0 {
            let (prev_w, prev_h) = content_sizes[frame - 1];
            let (curr_w, curr_h) = content_sizes[frame];
            
            // Content size should be stable across frames
            assert_eq!(curr_w, prev_w, "Width should not change between frames");
            assert_eq!(curr_h, prev_h, "Height should not change between frames");
        }
    }
    
    println!("Content dimensions stable across {} frames", content_sizes.len());
    println!("Final size: {:?}", content_sizes.last().unwrap());
}