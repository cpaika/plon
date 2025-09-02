use eframe::egui::{self, PointerButton, Pos2, Rect, Vec2};
use plon::domain::goal::Goal;
use plon::domain::task::{Task, TaskStatus};
use plon::ui::views::map_view::MapView;
use std::sync::Arc;

// Test utilities for simulating input events
struct TestContext {
    map_view: MapView,
    tasks: Vec<Task>,
    goals: Vec<Goal>,
}

impl TestContext {
    fn new() -> Self {
        Self {
            map_view: MapView::new(),
            tasks: Vec::new(),
            goals: Vec::new(),
        }
    }

    fn add_test_task(&mut self, title: &str, x: f64, y: f64) -> uuid::Uuid {
        let mut task = Task::new(title.to_string(), "Test task".to_string());
        task.set_position(x, y);
        let id = task.id;
        self.tasks.push(task);
        id
    }
}

// ============================================================================
// Pan Tests
// ============================================================================

#[test]
fn test_pan_with_middle_mouse_button() {
    let mut ctx = TestContext::new();

    // Initial camera position should be at origin
    assert_eq!(ctx.map_view.get_camera_position(), Vec2::ZERO);

    // Simulate middle mouse button drag
    let start_pos = Pos2::new(400.0, 300.0);
    let end_pos = Pos2::new(500.0, 400.0);
    let delta = end_pos - start_pos;

    ctx.map_view.start_pan(start_pos, PointerButton::Middle);
    ctx.map_view.update_pan(end_pos);
    ctx.map_view.end_pan();

    // Camera should have moved by delta / zoom_level
    let expected_camera_pos = delta / ctx.map_view.get_zoom_level();
    assert_eq!(ctx.map_view.get_camera_position(), expected_camera_pos);
}

#[test]
fn test_pan_with_shift_left_click() {
    let mut ctx = TestContext::new();

    // Initial camera position
    assert_eq!(ctx.map_view.get_camera_position(), Vec2::ZERO);

    // Simulate shift+left click drag
    let start_pos = Pos2::new(200.0, 200.0);
    let end_pos = Pos2::new(350.0, 250.0);
    let delta = end_pos - start_pos;

    ctx.map_view
        .start_pan_with_modifiers(start_pos, PointerButton::Primary, true, false);
    ctx.map_view.update_pan(end_pos);
    ctx.map_view.end_pan();

    // Camera should have moved
    let expected_camera_pos = delta / ctx.map_view.get_zoom_level();
    assert_eq!(ctx.map_view.get_camera_position(), expected_camera_pos);
}

#[test]
fn test_pan_does_not_move_tasks() {
    let mut ctx = TestContext::new();

    // Add a task at specific position
    let task_id = ctx.add_test_task("Task 1", 100.0, 100.0);

    // Pan the view
    let start_pos = Pos2::new(300.0, 300.0);
    let end_pos = Pos2::new(400.0, 400.0);

    ctx.map_view.start_pan(start_pos, PointerButton::Middle);
    ctx.map_view.update_pan(end_pos);
    ctx.map_view.end_pan();

    // Task position should remain unchanged
    let task = ctx.tasks.iter().find(|t| t.id == task_id).unwrap();
    assert_eq!(task.position.x, 100.0);
    assert_eq!(task.position.y, 100.0);
}

#[test]
fn test_pan_accumulates_multiple_drags() {
    let mut ctx = TestContext::new();

    // First pan
    ctx.map_view
        .start_pan(Pos2::new(100.0, 100.0), PointerButton::Middle);
    ctx.map_view.update_pan(Pos2::new(150.0, 150.0));
    ctx.map_view.end_pan();

    let camera_after_first = ctx.map_view.get_camera_position();

    // Second pan
    ctx.map_view
        .start_pan(Pos2::new(200.0, 200.0), PointerButton::Middle);
    ctx.map_view.update_pan(Pos2::new(250.0, 250.0));
    ctx.map_view.end_pan();

    // Camera position should accumulate both pans
    let expected_total = camera_after_first + Vec2::new(50.0, 50.0) / ctx.map_view.get_zoom_level();
    assert_eq!(ctx.map_view.get_camera_position(), expected_total);
}

#[test]
fn test_pan_cancelled_on_escape() {
    let mut ctx = TestContext::new();

    // Start panning
    ctx.map_view
        .start_pan(Pos2::new(100.0, 100.0), PointerButton::Middle);
    ctx.map_view.update_pan(Pos2::new(200.0, 200.0));

    // Cancel pan with escape
    ctx.map_view.cancel_pan();

    // Camera should remain at origin
    assert_eq!(ctx.map_view.get_camera_position(), Vec2::ZERO);
    assert!(!ctx.map_view.is_panning());
}

// ============================================================================
// Zoom Tests
// ============================================================================

#[test]
fn test_zoom_in_with_scroll() {
    let mut ctx = TestContext::new();

    // Initial zoom level
    assert_eq!(ctx.map_view.get_zoom_level(), 1.0);

    // Simulate scroll up
    let scroll_delta = 120.0; // Typical scroll wheel delta
    ctx.map_view
        .handle_scroll(scroll_delta, Pos2::new(400.0, 300.0));

    // Zoom should increase
    assert!(ctx.map_view.get_zoom_level() > 1.0);
    assert!(ctx.map_view.get_zoom_level() <= 5.0); // Within max limit
}

#[test]
fn test_zoom_out_with_scroll() {
    let mut ctx = TestContext::new();

    // Initial zoom level
    assert_eq!(ctx.map_view.get_zoom_level(), 1.0);

    // Simulate scroll down
    let scroll_delta = -120.0;
    ctx.map_view
        .handle_scroll(scroll_delta, Pos2::new(400.0, 300.0));

    // Zoom should decrease
    assert!(ctx.map_view.get_zoom_level() < 1.0);
    assert!(ctx.map_view.get_zoom_level() >= 0.1); // Within min limit
}

#[test]
fn test_zoom_limits() {
    let mut ctx = TestContext::new();

    // Zoom in to maximum
    for _ in 0..20 {
        ctx.map_view.handle_scroll(500.0, Pos2::new(400.0, 300.0));
    }
    assert_eq!(ctx.map_view.get_zoom_level(), 5.0);

    // Reset
    ctx.map_view.reset_view();
    assert_eq!(ctx.map_view.get_zoom_level(), 1.0);

    // Zoom out to minimum
    for _ in 0..20 {
        ctx.map_view.handle_scroll(-500.0, Pos2::new(400.0, 300.0));
    }
    assert_eq!(ctx.map_view.get_zoom_level(), 0.1);
}

#[test]
fn test_zoom_centered_on_mouse_position() {
    let mut ctx = TestContext::new();

    // Add a task at a specific world position
    let task_id = ctx.add_test_task("Task", 200.0, 200.0);

    // Zoom in centered at a specific point
    let zoom_center = Pos2::new(600.0, 400.0);
    let initial_camera = ctx.map_view.get_camera_position();

    ctx.map_view.handle_scroll(120.0, zoom_center);

    // The point under the mouse should remain at the same screen position
    // This requires the camera to adjust based on zoom center
    let new_camera = ctx.map_view.get_camera_position();
    assert_ne!(initial_camera, new_camera);
}

#[test]
fn test_zoom_with_buttons() {
    let mut ctx = TestContext::new();

    // Test zoom in button
    ctx.map_view.zoom_in();
    assert_eq!(ctx.map_view.get_zoom_level(), 1.2);

    // Test zoom out button
    ctx.map_view.zoom_out();
    assert_eq!(ctx.map_view.get_zoom_level(), 1.0);
}

#[test]
fn test_reset_view() {
    let mut ctx = TestContext::new();

    // Change camera and zoom
    ctx.map_view.set_camera_position(Vec2::new(100.0, 200.0));
    ctx.map_view.set_zoom_level(2.5);

    // Reset
    ctx.map_view.reset_view();

    // Should return to defaults
    assert_eq!(ctx.map_view.get_camera_position(), Vec2::ZERO);
    assert_eq!(ctx.map_view.get_zoom_level(), 1.0);
}

// ============================================================================
// Trackpad Gesture Tests
// ============================================================================

#[test]
fn test_trackpad_pinch_zoom() {
    let mut ctx = TestContext::new();

    // Initial zoom
    assert_eq!(ctx.map_view.get_zoom_level(), 1.0);

    // Simulate pinch gesture (two fingers moving apart)
    let pinch_center = Pos2::new(400.0, 300.0);
    let initial_distance = 100.0;
    let final_distance = 150.0;

    ctx.map_view
        .handle_pinch_gesture(pinch_center, initial_distance, final_distance);

    // Zoom should increase proportionally
    let expected_zoom = 1.0 * (final_distance / initial_distance);
    assert!((ctx.map_view.get_zoom_level() - expected_zoom).abs() < 0.01);
}

#[test]
fn test_trackpad_pinch_zoom_in() {
    let mut ctx = TestContext::new();

    // Simulate pinch out (zoom in)
    let pinch_center = Pos2::new(400.0, 300.0);
    ctx.map_view.handle_pinch_gesture(pinch_center, 50.0, 100.0);

    assert_eq!(ctx.map_view.get_zoom_level(), 2.0);
}

#[test]
fn test_trackpad_pinch_zoom_out() {
    let mut ctx = TestContext::new();

    // Start at zoom level 2
    ctx.map_view.set_zoom_level(2.0);

    // Simulate pinch in (zoom out)
    let pinch_center = Pos2::new(400.0, 300.0);
    ctx.map_view.handle_pinch_gesture(pinch_center, 100.0, 50.0);

    assert_eq!(ctx.map_view.get_zoom_level(), 1.0);
}

#[test]
fn test_trackpad_two_finger_pan() {
    let mut ctx = TestContext::new();

    // Initial camera position
    assert_eq!(ctx.map_view.get_camera_position(), Vec2::ZERO);

    // Simulate two-finger pan gesture
    let delta = Vec2::new(50.0, 30.0);
    ctx.map_view.handle_two_finger_pan(delta);

    // Camera should move by delta / zoom_level
    let expected_camera = delta / ctx.map_view.get_zoom_level();
    assert_eq!(ctx.map_view.get_camera_position(), expected_camera);
}

#[test]
fn test_trackpad_gesture_momentum() {
    let mut ctx = TestContext::new();

    // Start a pan with velocity
    let initial_velocity = Vec2::new(10.0, 5.0);
    ctx.map_view.start_momentum_pan(initial_velocity);

    // Update several times to simulate momentum decay
    for _ in 0..10 {
        ctx.map_view.update_momentum(0.016); // ~60fps
    }

    // Camera should have moved but velocity should decay
    assert_ne!(ctx.map_view.get_camera_position(), Vec2::ZERO);

    // After enough time, momentum should stop
    for _ in 0..100 {
        ctx.map_view.update_momentum(0.016);
    }

    assert!(ctx.map_view.get_momentum_velocity().length() < 0.1);
}

// ============================================================================
// Coordinate Transformation Tests
// ============================================================================

#[test]
fn test_world_to_screen_transformation() {
    let mut ctx = TestContext::new();

    // Set known camera and zoom
    ctx.map_view.set_camera_position(Vec2::new(100.0, 50.0));
    ctx.map_view.set_zoom_level(2.0);

    let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
    let world_pos = Vec2::new(200.0, 150.0);

    let screen_pos = ctx.map_view.world_to_screen(world_pos, viewport);

    // Expected: center + (world_pos + camera_pos) * zoom
    let expected = viewport.center() + (world_pos + ctx.map_view.get_camera_position()) * 2.0;
    assert_eq!(screen_pos, expected);
}

#[test]
fn test_screen_to_world_transformation() {
    let mut ctx = TestContext::new();

    // Set known camera and zoom
    ctx.map_view.set_camera_position(Vec2::new(100.0, 50.0));
    ctx.map_view.set_zoom_level(2.0);

    let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
    let screen_pos = Pos2::new(500.0, 400.0);

    let world_pos = ctx.map_view.screen_to_world(screen_pos, viewport);

    // Should be inverse of world_to_screen
    let back_to_screen = ctx.map_view.world_to_screen(world_pos, viewport);
    assert!((back_to_screen.x - screen_pos.x).abs() < 0.01);
    assert!((back_to_screen.y - screen_pos.y).abs() < 0.01);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_pan_and_zoom_together() {
    let mut ctx = TestContext::new();

    // Add some tasks
    ctx.add_test_task("Task 1", 0.0, 0.0);
    ctx.add_test_task("Task 2", 100.0, 100.0);

    // Pan the view
    ctx.map_view
        .start_pan(Pos2::new(200.0, 200.0), PointerButton::Middle);
    ctx.map_view.update_pan(Pos2::new(300.0, 300.0));
    ctx.map_view.end_pan();

    let camera_after_pan = ctx.map_view.get_camera_position();

    // Zoom in
    ctx.map_view.handle_scroll(120.0, Pos2::new(400.0, 300.0));

    // Camera position should be adjusted for zoom center
    assert_ne!(ctx.map_view.get_camera_position(), camera_after_pan);
    assert!(ctx.map_view.get_zoom_level() > 1.0);
}

#[test]
fn test_smooth_zoom_animation() {
    let mut ctx = TestContext::new();

    // Start smooth zoom animation
    ctx.map_view.start_smooth_zoom(1.0, 2.0, 0.5); // From 1x to 2x over 0.5 seconds

    // Update animation
    ctx.map_view.update_zoom_animation(0.25); // Halfway through

    // Should be approximately at 1.5x zoom
    assert!((ctx.map_view.get_zoom_level() - 1.5).abs() < 0.1);

    // Complete animation
    ctx.map_view.update_zoom_animation(0.25);

    // Should be at target zoom
    assert_eq!(ctx.map_view.get_zoom_level(), 2.0);
}

#[test]
fn test_viewport_culling_with_pan_zoom() {
    let mut ctx = TestContext::new();

    // Add tasks at various positions
    let task1 = ctx.add_test_task("Visible", 100.0, 100.0);
    let task2 = ctx.add_test_task("Far Away", 5000.0, 5000.0);

    let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));

    // Task 1 should be visible, task 2 should not
    assert!(ctx.map_view.is_task_visible(&ctx.tasks[0], viewport));
    assert!(!ctx.map_view.is_task_visible(&ctx.tasks[1], viewport));

    // Pan to task 2
    ctx.map_view
        .set_camera_position(Vec2::new(-4600.0, -4700.0));

    // Now task 2 should be visible, task 1 should not
    assert!(!ctx.map_view.is_task_visible(&ctx.tasks[0], viewport));
    assert!(ctx.map_view.is_task_visible(&ctx.tasks[1], viewport));
}

#[test]
fn test_zoom_affects_task_selection() {
    let mut ctx = TestContext::new();

    // Add a task
    let task_id = ctx.add_test_task("Task", 100.0, 100.0);

    let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));

    // At normal zoom, clicking at task position should select it
    let world_pos = Vec2::new(100.0, 100.0);
    let screen_pos = ctx.map_view.world_to_screen(world_pos, viewport);

    let hit = ctx.map_view.hit_test_task(screen_pos, &ctx.tasks, viewport);
    assert_eq!(hit, Some(task_id));

    // Zoom out significantly
    ctx.map_view.set_zoom_level(0.2);

    // Task hit area should be smaller
    let far_pos = screen_pos + Vec2::new(100.0, 100.0);
    let hit = ctx.map_view.hit_test_task(far_pos, &ctx.tasks, viewport);
    assert_eq!(hit, None);
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_pan_performance_with_many_tasks() {
    let mut ctx = TestContext::new();

    // Add 1000 tasks
    for i in 0..1000 {
        ctx.add_test_task(&format!("Task {}", i), (i as f64) * 10.0, (i as f64) * 10.0);
    }

    let start = std::time::Instant::now();

    // Perform multiple pan operations
    for i in 0..100 {
        let pos = Pos2::new((i as f32) * 5.0, (i as f32) * 5.0);
        if i == 0 {
            ctx.map_view.start_pan(pos, PointerButton::Middle);
        } else {
            ctx.map_view.update_pan(pos);
        }
    }
    ctx.map_view.end_pan();

    let elapsed = start.elapsed();

    // Should complete in reasonable time
    assert!(
        elapsed.as_millis() < 100,
        "Pan operations took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_zoom_performance_with_many_tasks() {
    let mut ctx = TestContext::new();

    // Add 1000 tasks
    for i in 0..1000 {
        ctx.add_test_task(&format!("Task {}", i), (i as f64) * 10.0, (i as f64) * 10.0);
    }

    let start = std::time::Instant::now();

    // Perform multiple zoom operations
    for _ in 0..100 {
        ctx.map_view.handle_scroll(10.0, Pos2::new(400.0, 300.0));
    }

    let elapsed = start.elapsed();

    // Should complete in reasonable time
    assert!(
        elapsed.as_millis() < 100,
        "Zoom operations took too long: {:?}",
        elapsed
    );
}
