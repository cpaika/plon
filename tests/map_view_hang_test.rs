use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use plon::ui::views::map_view::MapView;
use plon::ui::app::PlonApp;
use plon::domain::task::Task;
use plon::domain::goal::Goal;

#[test]
fn test_map_view_panning_hang() {
    // Set up a flag to track if the test completed
    let test_completed = Arc::new(AtomicBool::new(false));
    let test_completed_clone = Arc::clone(&test_completed);
    
    // Set up a flag to track if a hang was detected
    let hang_detected = Arc::new(AtomicBool::new(false));
    let hang_detected_clone = Arc::clone(&hang_detected);
    
    // Spawn a watchdog thread that will detect hangs
    let watchdog = thread::spawn(move || {
        let start = Instant::now();
        let timeout = Duration::from_secs(10); // 10 second timeout
        
        while !test_completed_clone.load(Ordering::Relaxed) {
            if start.elapsed() > timeout {
                println!("HANG DETECTED: Test has been running for over 10 seconds!");
                hang_detected_clone.store(true, Ordering::Relaxed);
                
                // Force exit to prevent infinite hang
                std::process::exit(1);
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    // Run the actual test
    let result = std::panic::catch_unwind(|| {
        run_panning_simulation();
    });
    
    // Mark test as completed
    test_completed.store(true, Ordering::Relaxed);
    
    // Wait for watchdog to finish
    let _ = watchdog.join();
    
    // Check results
    if hang_detected.load(Ordering::Relaxed) {
        panic!("Test failed: Hang detected during panning simulation!");
    }
    
    if let Err(e) = result {
        panic!("Test panicked: {:?}", e);
    }
}

fn run_panning_simulation() {
    // Create the egui context
    let ctx = egui::Context::default();
    
    // Create map view with many tasks to stress test
    let mut map_view = MapView::new();
    
    // Create many tasks spread across the map
    let mut tasks: Vec<Task> = Vec::new();
    for i in 0..200 {
        let mut task = Task::new(
            format!("Task {}", i),
            format!("This is task {} with a longer description to simulate real data", i)
        );
        task.set_position(
            (i as f64 % 20.0) * 300.0,
            (i as f64 / 20.0) * 300.0
        );
        
        // Add subtasks to some tasks
        if i % 5 == 0 {
            for j in 0..3 {
                task.add_subtask(format!("Subtask {}-{}", i, j));
            }
        }
        
        tasks.push(task);
    }
    
    let mut goals: Vec<Goal> = Vec::new();
    
    // Create a mock UI
    let mut output = egui::FullOutput::default();
    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::Vec2::new(1024.0, 768.0)
    ));
    
    println!("Starting panning simulation with {} tasks", tasks.len());
    
    // Simulate rapid panning that would cause the beachball
    for frame in 0..100 {
        let frame_start = Instant::now();
        
        // Begin frame
        ctx.begin_frame(raw_input.clone());
        
        // Simulate the UI rendering
        egui::CentralPanel::default().show(&ctx, |ui| {
            // Call the map view show method - this is where the hang occurs
            map_view.show(ui, &mut tasks, &mut goals);
        });
        
        // End frame
        output = ctx.end_frame();
        
        // Simulate mouse movement and dragging (panning)
        if frame % 2 == 0 {
            // Start drag
            raw_input.events.push(egui::Event::PointerButton {
                pos: egui::Pos2::new(500.0 + frame as f32, 400.0 + frame as f32),
                button: egui::PointerButton::Middle,
                pressed: true,
                modifiers: Default::default(),
            });
        } else {
            // Continue drag
            raw_input.events.push(egui::Event::PointerMoved(
                egui::Pos2::new(500.0 + frame as f32 * 2.0, 400.0 + frame as f32 * 1.5)
            ));
        }
        
        // Also simulate scroll events (trackpad panning)
        if frame % 5 == 0 {
            raw_input.events.push(egui::Event::Scroll(egui::Vec2::new(
                (frame as f32 % 20.0) - 10.0,
                (frame as f32 % 30.0) - 15.0
            )));
        }
        
        let frame_time = frame_start.elapsed();
        
        // Check if frame took too long (potential hang)
        if frame_time > Duration::from_millis(100) {
            println!("WARNING: Frame {} took {:?} (> 100ms)", frame, frame_time);
            
            if frame_time > Duration::from_millis(500) {
                panic!("Frame {} took {:?} - this would cause a beachball!", frame, frame_time);
            }
        }
        
        // Clear events for next frame
        raw_input.events.clear();
        
        // Small delay to simulate frame timing
        thread::sleep(Duration::from_millis(16)); // ~60fps
    }
    
    println!("Panning simulation completed successfully");
}

#[test] 
fn test_map_view_show_frame_timing() {
    // This test measures the actual frame time of the show method
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();
    
    // Create test data
    let mut tasks: Vec<Task> = Vec::new();
    for i in 0..100 {
        let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
        task.set_position((i as f64 % 10.0) * 200.0, (i as f64 / 10.0) * 200.0);
        tasks.push(task);
    }
    let mut goals = Vec::new();
    
    // Measure multiple frames
    let mut frame_times = Vec::new();
    
    for frame in 0..50 {
        let raw_input = egui::RawInput::default();
        ctx.begin_frame(raw_input);
        
        let frame_start = Instant::now();
        
        egui::CentralPanel::default().show(&ctx, |ui| {
            map_view.show(ui, &mut tasks, &mut goals);
        });
        
        let frame_time = frame_start.elapsed();
        frame_times.push(frame_time);
        
        ctx.end_frame();
        
        // Simulate some panning
        if frame > 10 {
            map_view.set_camera_position(egui::Vec2::new(
                (frame as f32 * 5.0).sin() * 100.0,
                (frame as f32 * 3.0).cos() * 100.0
            ));
        }
    }
    
    // Analyze frame times
    let max_frame_time = frame_times.iter().max().unwrap();
    let avg_frame_time: Duration = frame_times.iter().sum::<Duration>() / frame_times.len() as u32;
    
    println!("Frame time analysis:");
    println!("  Average: {:?}", avg_frame_time);
    println!("  Maximum: {:?}", max_frame_time);
    
    // Check for performance issues
    if *max_frame_time > Duration::from_millis(100) {
        println!("WARNING: Maximum frame time exceeds 100ms!");
        println!("Frame times: {:?}", frame_times);
    }
    
    assert!(
        *max_frame_time < Duration::from_millis(500),
        "Frame time too high: {:?} - this would cause UI freezing!",
        max_frame_time
    );
    
    assert!(
        avg_frame_time < Duration::from_millis(50),
        "Average frame time too high: {:?} - UI would feel sluggish!",
        avg_frame_time
    );
}

#[test]
fn test_blocking_operations_detection() {
    // This test specifically looks for blocking operations
    use std::sync::Mutex;
    use std::sync::Arc;
    
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();
    
    // Track if any blocking operations occur
    let blocking_detected = Arc::new(Mutex::new(Vec::new()));
    let blocking_detected_clone = Arc::clone(&blocking_detected);
    
    // Monitor thread to detect blocking
    let main_thread_id = thread::current().id();
    let monitor = thread::spawn(move || {
        let mut last_check = Instant::now();
        
        for _ in 0..100 {
            thread::sleep(Duration::from_millis(10));
            
            let elapsed = last_check.elapsed();
            if elapsed > Duration::from_millis(50) {
                let mut detections = blocking_detected_clone.lock().unwrap();
                detections.push(format!(
                    "Potential blocking detected: {:?} since last check",
                    elapsed
                ));
            }
            
            last_check = Instant::now();
        }
    });
    
    // Create test data
    let mut tasks: Vec<Task> = Vec::new();
    for i in 0..50 {
        let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
        task.set_position((i as f64 % 10.0) * 200.0, (i as f64 / 10.0) * 200.0);
        tasks.push(task);
    }
    let mut goals = Vec::new();
    
    // Run multiple frames with various operations
    for frame in 0..20 {
        let raw_input = egui::RawInput::default();
        ctx.begin_frame(raw_input);
        
        egui::CentralPanel::default().show(&ctx, |ui| {
            // This is where blocking would occur
            map_view.show(ui, &mut tasks, &mut goals);
        });
        
        ctx.end_frame();
        
        // Simulate user interactions
        map_view.set_camera_position(egui::Vec2::new(frame as f32 * 10.0, frame as f32 * 5.0));
        map_view.set_zoom_level(1.0 + (frame as f32 * 0.1).sin());
    }
    
    // Wait for monitor to finish
    let _ = monitor.join();
    
    // Check for blocking detections
    let detections = blocking_detected.lock().unwrap();
    if !detections.is_empty() {
        println!("Blocking operations detected:");
        for detection in detections.iter() {
            println!("  - {}", detection);
        }
        panic!("Blocking operations detected that could cause beachball!");
    }
}