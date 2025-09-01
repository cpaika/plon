use enigo::{Enigo, Mouse, Keyboard, Settings, Button, Key, Coordinate, Direction};
use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use image::GenericImageView;

struct AppHandle {
    process: Child,
    enigo: Enigo,
}

impl AppHandle {
    fn new() -> Self {
        // Start the desktop application
        let process = Command::new("cargo")
            .args(&["run", "--bin", "plon-desktop"])
            .spawn()
            .expect("Failed to start application");
        
        // Wait for the app to start
        thread::sleep(Duration::from_secs(3));
        
        Self {
            process,
            enigo: Enigo::new(&Settings::default()).expect("Failed to create Enigo"),
        }
    }
    
    fn click_at(&mut self, x: f64, y: f64) {
        let _ = self.enigo.move_mouse(x as i32, y as i32, Coordinate::Abs);
        thread::sleep(Duration::from_millis(100));
        let _ = self.enigo.button(Button::Left, Direction::Click);
        thread::sleep(Duration::from_millis(100));
    }
    
    fn drag_from_to(&mut self, from_x: f64, from_y: f64, to_x: f64, to_y: f64) {
        // Move to start position
        let _ = self.enigo.move_mouse(from_x as i32, from_y as i32, Coordinate::Abs);
        thread::sleep(Duration::from_millis(100));
        
        // Press mouse button
        let _ = self.enigo.button(Button::Left, Direction::Press);
        thread::sleep(Duration::from_millis(100));
        
        // Drag to end position
        let _ = self.enigo.move_mouse(to_x as i32, to_y as i32, Coordinate::Abs);
        thread::sleep(Duration::from_millis(100));
        
        // Release mouse button
        let _ = self.enigo.button(Button::Left, Direction::Release);
        thread::sleep(Duration::from_millis(100));
    }
    
    fn type_text(&mut self, text: &str) {
        let _ = self.enigo.text(text);
        thread::sleep(Duration::from_millis(50));
    }
    
    fn take_screenshot(&self, name: &str) {
        // Use screencapture on macOS
        Command::new("screencapture")
            .args(&["-x", &format!("test-screenshots/{}.png", name)])
            .output()
            .ok();
    }
}

impl Drop for AppHandle {
    fn drop(&mut self) {
        // Kill the application when the test ends
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

#[test]
#[ignore] // Run with: cargo test test_create_task_via_ui -- --ignored
fn test_create_task_via_ui() {
    let mut app = AppHandle::new();
    
    // Click on Map view (approximate position)
    app.click_at(300.0, 100.0);
    thread::sleep(Duration::from_millis(500));
    
    // Click Add Task button (approximate position)
    app.click_at(150.0, 200.0);
    thread::sleep(Duration::from_millis(500));
    
    // Take screenshot to verify
    app.take_screenshot("after_add_task");
    
    // The task should be created and visible
    // We can verify this by checking the screenshot manually
    // or by implementing image comparison
}

#[test]
#[ignore] // Run with: cargo test test_create_dependency_via_drag -- --ignored
fn test_create_dependency_via_drag() {
    let mut app = AppHandle::new();
    
    // Navigate to Map view
    app.click_at(300.0, 100.0);
    thread::sleep(Duration::from_secs(1));
    
    // Create first task
    app.click_at(150.0, 200.0); // Add Task button
    thread::sleep(Duration::from_millis(500));
    
    // Create second task
    app.click_at(150.0, 200.0); // Add Task button again
    thread::sleep(Duration::from_millis(500));
    
    // Move second task to a different position
    app.drag_from_to(400.0, 300.0, 600.0, 300.0);
    thread::sleep(Duration::from_millis(500));
    
    // Create dependency: drag from right node of first task to left node of second task
    // Assuming tasks are at approximate positions
    app.drag_from_to(
        450.0, 300.0,  // Right node of first task
        550.0, 300.0   // Left node of second task
    );
    thread::sleep(Duration::from_millis(500));
    
    // Take screenshot to verify dependency line
    app.take_screenshot("dependency_created");
}

#[test]
#[ignore] // Run with: cargo test test_task_persistence -- --ignored
fn test_task_persistence() {
    // First session: create tasks
    {
        let mut app = AppHandle::new();
        
        // Navigate to Map view
        app.click_at(300.0, 100.0);
        thread::sleep(Duration::from_secs(1));
        
        // Create a task
        app.click_at(150.0, 200.0); // Add Task button
        thread::sleep(Duration::from_millis(500));
        
        // Move the task to a specific position
        app.drag_from_to(400.0, 300.0, 500.0, 400.0);
        thread::sleep(Duration::from_millis(500));
        
        app.take_screenshot("before_restart");
        
        // App will be killed when it goes out of scope
    }
    
    thread::sleep(Duration::from_secs(2));
    
    // Second session: verify tasks persist
    {
        let mut app = AppHandle::new();
        
        // Navigate to Map view
        app.click_at(300.0, 100.0);
        thread::sleep(Duration::from_secs(1));
        
        // Take screenshot to verify task is still there
        app.take_screenshot("after_restart");
        
        // The task should be at the same position
        // This can be verified by comparing screenshots
    }
}

#[test]
#[ignore] // Run with: cargo test test_kanban_drag_drop -- --ignored
fn test_kanban_drag_drop() {
    let mut app = AppHandle::new();
    
    // Navigate to Kanban view
    app.click_at(200.0, 100.0); // Kanban button position
    thread::sleep(Duration::from_secs(1));
    
    // Create a task in TODO column
    app.click_at(200.0, 200.0); // Add task in TODO
    thread::sleep(Duration::from_millis(500));
    
    // Drag task from TODO to In Progress
    app.drag_from_to(
        200.0, 300.0,  // Task in TODO column
        500.0, 300.0   // In Progress column
    );
    thread::sleep(Duration::from_millis(500));
    
    app.take_screenshot("kanban_after_drag");
}

#[test]
#[ignore] // Run with: cargo test test_map_view_zoom_pan -- --ignored
fn test_map_view_zoom_pan() {
    let mut app = AppHandle::new();
    
    // Navigate to Map view
    app.click_at(300.0, 100.0);
    thread::sleep(Duration::from_secs(1));
    
    // Create some tasks
    for _ in 0..3 {
        app.click_at(150.0, 200.0); // Add Task button
        thread::sleep(Duration::from_millis(300));
    }
    
    // Test zooming with scroll (ctrl + scroll on most platforms)
    let _ = app.enigo.key(Key::Control, Direction::Press);
    let _ = app.enigo.scroll(5, enigo::Axis::Vertical);
    thread::sleep(Duration::from_millis(500));
    let _ = app.enigo.key(Key::Control, Direction::Release);
    
    app.take_screenshot("after_zoom_in");
    
    // Test panning by dragging with middle mouse button
    let _ = app.enigo.move_mouse(400, 400, Coordinate::Abs);
    let _ = app.enigo.button(Button::Middle, Direction::Press);
    let _ = app.enigo.move_mouse(500, 300, Coordinate::Abs);
    let _ = app.enigo.button(Button::Middle, Direction::Release);
    thread::sleep(Duration::from_millis(500));
    
    app.take_screenshot("after_pan");
}

// Helper function to compare screenshots (basic implementation)
#[allow(dead_code)]
fn compare_screenshots(path1: &str, path2: &str) -> bool {
    use image::io::Reader as ImageReader;
    
    let Ok(img1) = ImageReader::open(path1) else { return false; };
    let Ok(img1) = img1.decode() else { return false; };
    
    let Ok(img2) = ImageReader::open(path2) else { return false; };
    let Ok(img2) = img2.decode() else { return false; };
    
    if img1.dimensions() != img2.dimensions() {
        return false;
    }
    
    // Basic pixel comparison (could be enhanced with tolerance)
    let diff = img1.to_rgba8()
        .pixels()
        .zip(img2.to_rgba8().pixels())
        .filter(|(p1, p2)| p1 != p2)
        .count();
    
    // Allow some differences (UI animations, anti-aliasing, etc.)
    diff < 1000
}

#[test]
#[ignore]
fn test_full_workflow() {
    let mut app = AppHandle::new();
    
    println!("Starting full workflow test...");
    
    // 1. Navigate to Map view
    app.click_at(300.0, 100.0);
    thread::sleep(Duration::from_secs(1));
    
    // 2. Create multiple tasks
    println!("Creating tasks...");
    for i in 0..3 {
        app.click_at(150.0, 200.0); // Add Task button
        thread::sleep(Duration::from_millis(500));
        
        // Position tasks in different locations
        let x = 300.0 + (i as f64 * 200.0);
        app.drag_from_to(400.0, 300.0, x, 400.0);
        thread::sleep(Duration::from_millis(300));
    }
    
    // 3. Create dependencies between tasks
    println!("Creating dependencies...");
    app.drag_from_to(350.0, 400.0, 450.0, 400.0); // Task 1 -> Task 2
    thread::sleep(Duration::from_millis(500));
    app.drag_from_to(550.0, 400.0, 650.0, 400.0); // Task 2 -> Task 3
    thread::sleep(Duration::from_millis(500));
    
    // 4. Switch to Kanban view
    println!("Switching to Kanban view...");
    app.click_at(200.0, 100.0);
    thread::sleep(Duration::from_secs(1));
    
    // 5. Move a task between columns
    app.drag_from_to(200.0, 300.0, 500.0, 300.0);
    thread::sleep(Duration::from_millis(500));
    
    // 6. Switch back to Map view to verify
    app.click_at(300.0, 100.0);
    thread::sleep(Duration::from_secs(1));
    
    app.take_screenshot("full_workflow_complete");
    
    println!("Full workflow test completed!");
}

#[test]
#[ignore]
fn test_dependency_persistence_across_tabs() {
    let mut app = AppHandle::new();
    
    println!("Testing dependency persistence across tab navigation...");
    
    // 1. Navigate to Map view
    println!("Step 1: Navigating to Map view...");
    app.click_at(300.0, 100.0);
    thread::sleep(Duration::from_secs(2));
    
    // 2. Create two tasks
    println!("Step 2: Creating first task...");
    app.click_at(150.0, 200.0); // Add Task button
    thread::sleep(Duration::from_millis(500));
    // Move first task to a specific position
    app.drag_from_to(400.0, 300.0, 300.0, 400.0);
    thread::sleep(Duration::from_millis(500));
    
    println!("Step 3: Creating second task...");
    app.click_at(150.0, 200.0); // Add Task button
    thread::sleep(Duration::from_millis(500));
    // Move second task to a different position
    app.drag_from_to(400.0, 300.0, 600.0, 400.0);
    thread::sleep(Duration::from_millis(500));
    
    // 3. Create a dependency between tasks (drag from right node of first to left node of second)
    println!("Step 4: Creating dependency from task 1 to task 2...");
    // Assuming tasks have connection nodes at their edges
    // Right node of first task (approximately)
    let first_task_right_x = 350.0;
    let first_task_right_y = 400.0;
    // Left node of second task (approximately)
    let second_task_left_x = 550.0;
    let second_task_left_y = 400.0;
    
    app.drag_from_to(first_task_right_x, first_task_right_y, 
                     second_task_left_x, second_task_left_y);
    thread::sleep(Duration::from_secs(1));
    
    // Take screenshot of initial state with dependency
    app.take_screenshot("dependencies_before_tab_switch");
    
    // 4. Switch to Kanban view
    println!("Step 5: Switching to Kanban view...");
    app.click_at(200.0, 100.0);
    thread::sleep(Duration::from_secs(2));
    
    // 5. Switch to Timeline view
    println!("Step 6: Switching to Timeline view...");
    app.click_at(100.0, 100.0);
    thread::sleep(Duration::from_secs(2));
    
    // 6. Switch back to Map view
    println!("Step 7: Switching back to Map view...");
    app.click_at(300.0, 100.0);
    thread::sleep(Duration::from_secs(2));
    
    // Take screenshot to verify dependencies are still there
    app.take_screenshot("dependencies_after_tab_switches");
    
    // 7. Restart the app to verify persistence in database
    println!("Step 8: Restarting app to verify database persistence...");
    drop(app);
    thread::sleep(Duration::from_secs(1));
    
    let mut app = AppHandle::new();
    
    // Navigate to Map view
    println!("Step 9: Navigating to Map view after restart...");
    app.click_at(300.0, 100.0);
    thread::sleep(Duration::from_secs(2));
    
    // Take final screenshot to verify dependencies persisted
    app.take_screenshot("dependencies_after_restart");
    
    println!("✅ Dependency persistence test completed!");
    println!("Check screenshots in test-screenshots/ to verify:");
    println!("  - dependencies_before_tab_switch.png");
    println!("  - dependencies_after_tab_switches.png");
    println!("  - dependencies_after_restart.png");
}