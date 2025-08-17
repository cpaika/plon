use egui_kittest::{kittest::Harness, TestApp};
use plon::ui::views::map_view::MapView;
use plon::domain::task::Task;
use plon::domain::goal::Goal;
use eframe::egui;

/// Test that reproduces the bug where panning stops when hovering over tasks
#[test]
fn test_panning_stops_over_tasks() {
    // Create a test harness
    let mut harness = TestApp::new().harness();
    
    // Store test state
    let mut map_view = MapView::new();
    let mut tasks = vec![];
    let mut goals = vec![];
    
    // Add some tasks at known positions
    for i in 0..5 {
        let mut task = Task::new(
            format!("Task {}", i),
            format!("Description {}", i)
        );
        task.set_position((i * 200) as f64, 200.0);
        tasks.push(task);
    }
    
    // Add a goal
    let mut goal = Goal::new("Test Goal".to_string());
    goal.position_x = 100.0;
    goal.position_y = 400.0;
    goal.position_width = 300.0;
    goal.position_height = 200.0;
    goals.push(goal);
    
    // Initial camera position
    let initial_camera = map_view.get_camera_position();
    
    // Run the test
    harness.run(&mut map_view, |ctx, map_view| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show the map view
            map_view.show(ui, &mut tasks, &mut goals);
            
            // Simulate middle mouse button press for panning
            ctx.input_mut(|input| {
                // Start middle mouse drag
                input.events.push(egui::Event::PointerButton {
                    pos: egui::Pos2::new(400.0, 300.0),
                    button: egui::PointerButton::Middle,
                    pressed: true,
                    modifiers: Default::default(),
                });
            });
        });
    });
    
    // Move the mouse while holding middle button (panning)
    harness.run(&mut map_view, |ctx, map_view| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input_mut(|input| {
                // Move mouse to the right (should pan)
                input.events.push(egui::Event::PointerMoved(egui::Pos2::new(500.0, 300.0)));
            });
            
            map_view.show(ui, &mut tasks, &mut goals);
        });
    });
    
    // Check that camera moved (panning is working)
    let camera_after_first_move = map_view.get_camera_position();
    assert_ne!(
        camera_after_first_move, initial_camera,
        "Camera should have moved after first pan"
    );
    
    // Now move mouse over a task position while still holding middle button
    // Task 1 is at world position (200, 200), which maps to screen position
    // based on camera and zoom
    harness.run(&mut map_view, |ctx, map_view| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input_mut(|input| {
                // Move mouse over task area
                input.events.push(egui::Event::PointerMoved(egui::Pos2::new(200.0, 200.0)));
            });
            
            map_view.show(ui, &mut tasks, &mut goals);
        });
    });
    
    // Check if panning continued or stopped
    let camera_after_task_hover = map_view.get_camera_position();
    
    // The bug would cause camera_after_task_hover == camera_after_first_move
    // because panning stops when hovering over a task
    assert_ne!(
        camera_after_task_hover, camera_after_first_move,
        "PANNING BUG: Camera should continue moving when mouse hovers over task!"
    );
    
    // Release middle mouse button
    harness.run(&mut map_view, |ctx, map_view| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input_mut(|input| {
                input.events.push(egui::Event::PointerButton {
                    pos: egui::Pos2::new(200.0, 200.0),
                    button: egui::PointerButton::Middle,
                    pressed: false,
                    modifiers: Default::default(),
                });
            });
            
            map_view.show(ui, &mut tasks, &mut goals);
        });
    });
    
    // Verify panning has stopped
    assert!(!map_view.is_panning(), "Should not be panning after release");
}

/// Test trackpad two-finger pan stopping over tasks
#[test]
fn test_trackpad_pan_stops_over_tasks() {
    let mut harness = TestApp::new().harness();
    
    let mut map_view = MapView::new();
    let mut tasks = vec![];
    let mut goals = vec![];
    
    // Add tasks
    for i in 0..3 {
        let mut task = Task::new(format!("Task {}", i), String::new());
        task.set_position((i * 150) as f64, 150.0);
        tasks.push(task);
    }
    
    let initial_camera = map_view.get_camera_position();
    
    // Simulate trackpad pan (scroll delta)
    harness.run(&mut map_view, |ctx, map_view| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input_mut(|input| {
                // Trackpad two-finger pan generates scroll events
                input.events.push(egui::Event::Scroll(egui::Vec2::new(50.0, 30.0)));
            });
            
            map_view.show(ui, &mut tasks, &mut goals);
        });
    });
    
    // Camera should have moved
    let camera_after_scroll = map_view.get_camera_position();
    assert_ne!(
        camera_after_scroll, initial_camera,
        "Camera should move with trackpad scroll"
    );
    
    // Continue scrolling while mouse is over task area
    harness.run(&mut map_view, |ctx, map_view| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input_mut(|input| {
                // Move pointer over task area
                input.events.push(egui::Event::PointerMoved(egui::Pos2::new(150.0, 150.0)));
                // Try to continue scrolling
                input.events.push(egui::Event::Scroll(egui::Vec2::new(50.0, 30.0)));
            });
            
            map_view.show(ui, &mut tasks, &mut goals);
        });
    });
    
    let camera_after_task_scroll = map_view.get_camera_position();
    
    // The bug would cause panning to stop when over a task
    assert_ne!(
        camera_after_task_scroll, camera_after_scroll,
        "TRACKPAD BUG: Camera should continue moving when scrolling over task!"
    );
}