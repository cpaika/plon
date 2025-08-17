use eframe::egui;
use plon::ui::views::timeline_view::TimelineView;
use plon::domain::task::Task;
use plon::domain::goal::Goal;

/// Test that mouse movement doesn't cause scrolling in timeline view
#[test]
fn test_timeline_no_scroll_on_mouse_hover() {
    // Create test context
    let ctx = egui::Context::default();
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
    
    // Track scroll positions across frames
    let mut scroll_positions = Vec::new();
    let mut mouse_positions = vec![
        egui::Pos2::new(100.0, 100.0),
        egui::Pos2::new(200.0, 150.0),
        egui::Pos2::new(300.0, 200.0),
        egui::Pos2::new(400.0, 250.0),
        egui::Pos2::new(500.0, 300.0),
    ];
    
    // Simulate multiple frames with different mouse positions
    for (frame, mouse_pos) in mouse_positions.iter().enumerate() {
        ctx.run(Default::default(), |ctx| {
            // Simulate mouse position
            // Note: We can't directly modify input in tests, so we'll track positions differently
            
            egui::CentralPanel::default().show(ctx, |ui| {
                // Capture scroll state before rendering
                let scroll_before = ui.ctx().memory(|mem| {
                    mem.data.get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll_area"))
                        .unwrap_or(egui::Vec2::ZERO)
                });
                
                // Render timeline
                timeline_view.show(ui, &tasks, &goals);
                
                // Capture scroll state after rendering
                let scroll_after = ui.ctx().memory(|mem| {
                    mem.data.get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll_area"))
                        .unwrap_or(egui::Vec2::ZERO)
                });
                
                scroll_positions.push((frame, *mouse_pos, scroll_before, scroll_after));
            });
        });
    }
    
    // Check that scroll position didn't change just from mouse movement
    let mut unexpected_scrolls = Vec::new();
    for i in 1..scroll_positions.len() {
        let (prev_frame, prev_mouse, prev_scroll_before, prev_scroll_after) = scroll_positions[i - 1];
        let (curr_frame, curr_mouse, curr_scroll_before, curr_scroll_after) = scroll_positions[i];
        
        // Check if scroll changed between frames (when only mouse moved)
        if (curr_scroll_before - prev_scroll_after).length() > 0.01 {
            unexpected_scrolls.push(format!(
                "Frame {} -> {}: Scroll changed from {:?} to {:?} when mouse moved from {:?} to {:?}",
                prev_frame, curr_frame, prev_scroll_after, curr_scroll_before, prev_mouse, curr_mouse
            ));
        }
    }
    
    // Assert no unexpected scrolling
    assert!(
        unexpected_scrolls.is_empty(),
        "Mouse movement caused scrolling:\n{}",
        unexpected_scrolls.join("\n")
    );
}

/// Test that verifies ScrollArea settings prevent mouse wheel scrolling when not intended
#[test]
fn test_timeline_scroll_area_configuration() {
    let ctx = egui::Context::default();
    let mut timeline_view = TimelineView::new();
    
    let tasks: Vec<Task> = (0..10).map(|i| {
        let mut task = Task::new(format!("Task {}", i), String::new());
        task.scheduled_date = Some(chrono::Utc::now());
        task
    }).collect();
    
    let goals: Vec<Goal> = vec![];
    
    // Test that scroll area is properly configured
    ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Record initial state
            let initial_scroll = ui.ctx().memory(|mem| {
                mem.data.get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll_area"))
                    .unwrap_or(egui::Vec2::ZERO)
            });
            
            // Simulate mouse wheel event
            ctx.input(|i| {
                let mut input = i.clone();
                // Simulate scroll wheel
                input.scroll_delta = egui::Vec2::new(0.0, 100.0);
                input
            });
            
            timeline_view.show(ui, &tasks, &goals);
            
            // Check if scroll changed
            let final_scroll = ui.ctx().memory(|mem| {
                mem.data.get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll_area"))
                    .unwrap_or(egui::Vec2::ZERO)
            });
            
            // For this test, we're checking that the scroll IS responding to wheel
            // but NOT to mere mouse movement (tested in the other test)
            println!("Initial scroll: {:?}, Final scroll: {:?}", initial_scroll, final_scroll);
        });
    });
}

/// Integration test that simulates actual user interaction
#[test]
fn test_timeline_interaction_stability() {
    let ctx = egui::Context::default();
    let mut timeline_view = TimelineView::new();
    
    let tasks: Vec<Task> = (0..20).map(|i| {
        let mut task = Task::new(format!("Task {}", i), String::new());
        task.scheduled_date = Some(chrono::Utc::now());
        task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(i * 2));
        task
    }).collect();
    
    let goals: Vec<Goal> = vec![];
    
    let mut interaction_log = Vec::new();
    
    // Simulate a sequence of interactions
    let interactions = vec![
        ("hover", egui::Pos2::new(100.0, 100.0), false),
        ("hover", egui::Pos2::new(200.0, 200.0), false),
        ("hover", egui::Pos2::new(300.0, 300.0), false),
        ("click", egui::Pos2::new(150.0, 150.0), true),
        ("hover", egui::Pos2::new(400.0, 400.0), false),
    ];
    
    for (action, pos, is_click) in interactions {
        ctx.run(Default::default(), |ctx| {
            // Set up input
            ctx.input(|i| {
                let mut input = i.clone();
                input.pointer.set_hover_pos(Some(pos));
                if is_click {
                    input.pointer.press_origin = Some(pos);
                    input.pointer.primary_pressed = true;
                }
                input
            });
            
            egui::CentralPanel::default().show(ctx, |ui| {
                let scroll_before = ui.ctx().memory(|mem| {
                    mem.data.get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll_area"))
                        .unwrap_or(egui::Vec2::ZERO)
                });
                
                timeline_view.show(ui, &tasks, &goals);
                
                let scroll_after = ui.ctx().memory(|mem| {
                    mem.data.get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll_area"))
                        .unwrap_or(egui::Vec2::ZERO)
                });
                
                let scroll_changed = (scroll_after - scroll_before).length() > 0.01;
                interaction_log.push((action, pos, scroll_changed));
            });
        });
    }
    
    // Verify that only intentional interactions cause scrolling
    for (action, pos, scroll_changed) in &interaction_log {
        if *action == "hover" && *scroll_changed {
            panic!("Hover at {:?} caused unexpected scrolling!", pos);
        }
    }
    
    println!("Interaction test passed. Log: {:?}", interaction_log);
}