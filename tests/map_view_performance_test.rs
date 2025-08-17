use plon::ui::views::map_view::MapView;
use plon::domain::task::Task;
use plon::domain::goal::Goal;
use eframe::egui;
use std::time::{Duration, Instant};

#[test]
fn test_intensive_panning_performance() {
    // Create a test context
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();
    
    // Create a large number of tasks to stress test
    let mut tasks: Vec<Task> = Vec::new();
    for i in 0..100 {
        let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
        task.set_position((i as f64 % 10.0) * 200.0, (i as f64 / 10.0) * 200.0);
        tasks.push(task);
    }
    
    let mut goals: Vec<Goal> = Vec::new();
    
    // Simulate rapid panning operations
    let start_time = Instant::now();
    let mut operations = 0;
    
    // Simulate 1000 pan operations
    for i in 0..1000 {
        let start_pos = egui::Pos2::new(400.0 + (i as f32 % 100.0), 300.0 + (i as f32 % 50.0));
        let end_pos = egui::Pos2::new(500.0 + (i as f32 % 100.0), 400.0 + (i as f32 % 50.0));
        
        // Start pan
        map_view.start_pan(start_pos, egui::PointerButton::Middle);
        
        // Simulate dragging with multiple update calls
        for j in 0..10 {
            let t = j as f32 / 10.0;
            let current_pos = start_pos + (end_pos - start_pos) * t;
            map_view.update_pan(current_pos);
            operations += 1;
        }
        
        // End pan
        map_view.end_pan();
    }
    
    let elapsed = start_time.elapsed();
    println!("Performed {} pan operations in {:?}", operations, elapsed);
    
    // Each operation should take less than 1ms on average
    let avg_time_per_op = elapsed.as_millis() as f64 / operations as f64;
    assert!(avg_time_per_op < 1.0, "Pan operations too slow: {}ms average", avg_time_per_op);
}

#[test]
fn test_trackpad_scroll_performance() {
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();
    
    // Create tasks
    let mut tasks: Vec<Task> = Vec::new();
    for i in 0..100 {
        let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
        task.set_position((i as f64 % 10.0) * 200.0, (i as f64 / 10.0) * 200.0);
        tasks.push(task);
    }
    
    let start_time = Instant::now();
    let mut operations = 0;
    
    // Simulate rapid trackpad scrolling
    for i in 0..5000 {
        let delta = egui::Vec2::new(
            (i as f32 % 20.0) - 10.0,
            (i as f32 % 30.0) - 15.0
        );
        
        map_view.handle_two_finger_pan(delta);
        operations += 1;
        
        // Also test momentum
        if i % 100 == 0 {
            map_view.start_momentum_pan(delta * 10.0);
            for _ in 0..20 {
                map_view.update_momentum(0.016); // 60fps
                operations += 1;
            }
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Performed {} scroll operations in {:?}", operations, elapsed);
    
    // Check performance
    let avg_time_per_op = elapsed.as_millis() as f64 / operations as f64;
    assert!(avg_time_per_op < 0.5, "Scroll operations too slow: {}ms average", avg_time_per_op);
}

#[test]
fn test_zoom_performance() {
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();
    
    // Create tasks
    let mut tasks: Vec<Task> = Vec::new();
    for i in 0..100 {
        let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
        task.set_position((i as f64 % 10.0) * 200.0, (i as f64 / 10.0) * 200.0);
        tasks.push(task);
    }
    
    let start_time = Instant::now();
    let mut operations = 0;
    
    // Simulate rapid zoom operations
    for i in 0..1000 {
        let mouse_pos = egui::Pos2::new(400.0 + (i as f32 % 100.0), 300.0);
        let scroll_delta = if i % 2 == 0 { 120.0 } else { -120.0 };
        
        map_view.handle_scroll(scroll_delta, mouse_pos);
        operations += 1;
        
        // Test smooth zoom animation
        if i % 50 == 0 {
            map_view.start_smooth_zoom(map_view.get_zoom_level(), 2.0, 0.5);
            for _ in 0..30 {
                map_view.update_zoom_animation(0.016);
                operations += 1;
            }
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Performed {} zoom operations in {:?}", operations, elapsed);
    
    // Check performance
    let avg_time_per_op = elapsed.as_millis() as f64 / operations as f64;
    assert!(avg_time_per_op < 0.5, "Zoom operations too slow: {}ms average", avg_time_per_op);
}

#[test]
fn test_combined_stress() {
    // This test simulates real-world usage with combined operations
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();
    
    // Create many tasks
    let mut tasks: Vec<Task> = Vec::new();
    for i in 0..200 {
        let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
        task.set_position((i as f64 % 20.0) * 200.0, (i as f64 / 20.0) * 200.0);
        tasks.push(task);
    }
    
    let mut goals: Vec<Goal> = Vec::new();
    
    let start_time = Instant::now();
    let mut operations = 0;
    
    // Simulate user rapidly panning and zooming
    for cycle in 0..100 {
        // Pan around
        for i in 0..10 {
            let pos = egui::Pos2::new(400.0 + (i as f32 * 10.0), 300.0 + (i as f32 * 5.0));
            map_view.start_pan(pos, egui::PointerButton::Middle);
            
            for j in 0..5 {
                let delta_pos = pos + egui::Vec2::new(j as f32 * 2.0, j as f32 * 1.5);
                map_view.update_pan(delta_pos);
                operations += 1;
            }
            
            map_view.end_pan();
        }
        
        // Zoom in and out
        for i in 0..5 {
            let scroll = if i % 2 == 0 { 120.0 } else { -120.0 };
            map_view.handle_scroll(scroll, egui::Pos2::new(400.0, 300.0));
            operations += 1;
        }
        
        // Trackpad gestures
        for i in 0..20 {
            let delta = egui::Vec2::new((i as f32 % 10.0) - 5.0, (i as f32 % 10.0) - 5.0);
            map_view.handle_two_finger_pan(delta);
            operations += 1;
        }
        
        // Check if we're taking too long (potential beachball)
        let current_elapsed = start_time.elapsed();
        if current_elapsed > Duration::from_secs(5) {
            panic!("Operations taking too long - potential beachball condition detected after {} operations", operations);
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Performed {} combined operations in {:?}", operations, elapsed);
    
    // Should complete all operations in under 2 seconds
    assert!(elapsed < Duration::from_secs(2), "Combined operations too slow: {:?}", elapsed);
}