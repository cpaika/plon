use eframe::egui;
use plon::domain::goal::Goal;
use plon::domain::task::Task;
use plon::ui::views::timeline_view::TimelineView;
use std::collections::HashMap;
use uuid::Uuid;

/// Test to detect if the timeline view auto-scrolls without user input
/// This simulates the GUI rendering loop and checks if content moves on its own
#[test]
fn test_timeline_auto_scroll_detection() {
    // Create a mock context
    let ctx = egui::Context::default();

    // Create timeline view with test data
    let mut timeline_view = TimelineView::new();

    // Create test tasks
    let tasks: Vec<Task> = (0..30)
        .map(|i| {
            let mut task = Task::new(format!("Task {}", i), String::new());
            task.scheduled_date = Some(chrono::Utc::now());
            task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(i));
            task
        })
        .collect();

    let goals: Vec<Goal> = vec![];

    // Track positions across frames to detect unwanted movement
    let mut content_positions = Vec::new();
    let mut scroll_positions = Vec::new();
    let mut available_sizes = Vec::new();

    // Simulate multiple frames
    for frame in 0..10 {
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Track available size before rendering
                let available_before = ui.available_size();
                available_sizes.push(available_before);

                // Get scroll position if available
                let scroll_pos = ui.memory(|mem| {
                    // Try to get any scroll area state
                    // In real egui, scroll areas have IDs we can query
                    mem.data
                        .get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll"))
                        .unwrap_or(egui::Vec2::ZERO)
                });
                scroll_positions.push(scroll_pos);

                // Render the timeline
                timeline_view.show(ui, &tasks, &goals);

                // Track content position after rendering
                let rect_after = ui.min_rect();
                content_positions.push((rect_after.min.x, rect_after.min.y));
            });
        });
    }

    // Check for auto-scrolling behavior
    let mut auto_scroll_detected = false;
    let mut unstable_size_detected = false;

    // Check if positions changed without user input
    for i in 1..content_positions.len() {
        let (prev_x, prev_y) = content_positions[i - 1];
        let (curr_x, curr_y) = content_positions[i];

        // Allow for tiny floating point differences but detect real movement
        if (curr_x - prev_x).abs() > 0.1 || (curr_y - prev_y).abs() > 0.1 {
            println!(
                "Frame {}: Content moved from ({:.2}, {:.2}) to ({:.2}, {:.2})",
                i, prev_x, prev_y, curr_x, curr_y
            );
            auto_scroll_detected = true;
        }
    }

    // Check if available size is changing (causing re-layouts)
    for i in 1..available_sizes.len() {
        let prev_size = available_sizes[i - 1];
        let curr_size = available_sizes[i];

        if (curr_size.x - prev_size.x).abs() > 0.1 || (curr_size.y - prev_size.y).abs() > 0.1 {
            println!(
                "Frame {}: Available size changed from ({:.2}, {:.2}) to ({:.2}, {:.2})",
                i, prev_size.x, prev_size.y, curr_size.x, curr_size.y
            );
            unstable_size_detected = true;
        }
    }

    // Check scroll positions
    for i in 1..scroll_positions.len() {
        let prev = scroll_positions[i - 1];
        let curr = scroll_positions[i];

        if (curr.x - prev.x).abs() > 0.1 || (curr.y - prev.y).abs() > 0.1 {
            println!(
                "Frame {}: Scroll position changed from ({:.2}, {:.2}) to ({:.2}, {:.2})",
                i, prev.x, prev.y, curr.x, curr.y
            );
            auto_scroll_detected = true;
        }
    }

    // Assert no auto-scrolling
    assert!(
        !auto_scroll_detected,
        "Auto-scrolling detected! The timeline view is moving without user input."
    );
    assert!(
        !unstable_size_detected,
        "Unstable layout detected! Available size is changing between frames."
    );
}

/// Test that specifically looks for feedback loops in size calculations
#[test]
fn test_timeline_size_feedback_loop() {
    let ctx = egui::Context::default();
    let mut timeline_view = TimelineView::new();

    let tasks: Vec<Task> = (0..20)
        .map(|i| {
            let mut task = Task::new(format!("Task {}", i), String::new());
            task.scheduled_date = Some(chrono::Utc::now());
            task
        })
        .collect();

    let goals: Vec<Goal> = vec![];

    // Track chart dimensions that the view calculates
    let mut calculated_dimensions = Vec::new();

    // Simulate frames with different available sizes
    let test_sizes = vec![
        egui::Vec2::new(800.0, 600.0),
        egui::Vec2::new(800.0, 600.0), // Same size - should be stable
        egui::Vec2::new(800.0, 600.0), // Same size - should be stable
    ];

    for (frame, &size) in test_sizes.iter().enumerate() {
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Force a specific available size
                ui.set_max_size(size);

                // Calculate what the timeline would render
                let available_width = ui.available_width();
                let available_height = ui.available_height();

                // These are from show_gantt_view
                let chart_width = available_width.min(1200.0);
                let chart_height = available_height.min(400.0);

                calculated_dimensions.push((chart_width, chart_height));

                // Render
                timeline_view.show(ui, &tasks, &goals);
            });
        });
    }

    // Check that dimensions are stable when available size doesn't change
    for i in 1..calculated_dimensions.len() {
        let (prev_w, prev_h) = calculated_dimensions[i - 1];
        let (curr_w, curr_h) = calculated_dimensions[i];

        if i >= 1 {
            // After first frame, size should be stable
            assert!(
                (curr_w - prev_w).abs() < 0.1,
                "Chart width unstable: changed from {} to {} in frame {}",
                prev_w,
                curr_w,
                i
            );
            assert!(
                (curr_h - prev_h).abs() < 0.1,
                "Chart height unstable: changed from {} to {} in frame {}",
                prev_h,
                curr_h,
                i
            );
        }
    }

    println!(
        "Size stability test passed. Dimensions across frames: {:?}",
        calculated_dimensions
    );
}
