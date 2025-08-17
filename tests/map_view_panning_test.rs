use eframe::egui;
use plon::ui::views::map_view::MapView;
use plon::domain::task::Task;
use plon::domain::goal::Goal;

/// Test suite for map view panning functionality
#[cfg(test)]
mod map_panning_tests {
    use super::*;
    
    /// Test that panning works with different input methods
    #[test]
    fn test_map_view_panning_methods() {
        let ctx = egui::Context::default();
        let mut map_view = MapView::new();
        
        // Create test data
        let mut tasks: Vec<Task> = (0..10)
            .map(|i| {
                let mut task = Task::new(format!("Task {}", i), String::new());
                task.set_position((i * 100) as f64, (i * 50) as f64);
                task
            })
            .collect();
        
        let mut goals: Vec<Goal> = vec![];
        
        // Record initial camera position
        let initial_camera_pos = map_view.get_camera_position();
        
        // Test different panning methods
        let test_cases = vec![
            ("Middle mouse drag", egui::PointerButton::Middle, false),
            ("Shift + Primary drag", egui::PointerButton::Primary, true),
            ("Two-finger trackpad pan", egui::PointerButton::Primary, false), // Should be supported!
        ];
        
        for (test_name, button, use_shift) in test_cases {
            println!("Testing: {}", test_name);
            
            ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Simulate input
                    ctx.input_mut(|i| {
                        i.pointer.button_down[button as usize] = true;
                        i.modifiers.shift = use_shift;
                        
                        // Simulate drag motion
                        i.pointer.delta = egui::Vec2::new(50.0, 30.0);
                    });
                    
                    // Show map view
                    map_view.show(ui, &mut tasks, &mut goals);
                    
                    // Check if camera moved
                    let camera_moved = map_view.get_camera_position() != initial_camera_pos;
                    
                    println!("  Camera position: {:?}", map_view.get_camera_position());
                    println!("  Camera moved: {}", camera_moved);
                    
                    // For trackpad on Mac, we expect panning to work
                    if test_name.contains("trackpad") {
                        assert!(camera_moved, 
                            "Trackpad panning should work but camera didn't move!");
                    }
                });
            });
        }
    }
    
    /// Test trackpad-specific gestures
    #[test]
    fn test_trackpad_gestures() {
        let ctx = egui::Context::default();
        let mut map_view = MapView::new();
        let mut tasks: Vec<Task> = vec![];
        let mut goals: Vec<Goal> = vec![];
        
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Test two-finger pan (scroll delta)
                ctx.input_mut(|i| {
                    // On Mac trackpad, two-finger pan generates scroll deltas
                    i.scroll_delta = egui::Vec2::new(30.0, 20.0);
                });
                
                let initial_pos = map_view.get_camera_position();
                map_view.show(ui, &mut tasks, &mut goals);
                
                // Camera should move with scroll delta
                assert_ne!(map_view.get_camera_position(), initial_pos,
                    "Two-finger trackpad pan should move camera");
            });
        });
        
        // Test pinch zoom
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ctx.input_mut(|i| {
                    // Pinch zoom typically modifies zoom delta
                    i.zoom_delta = 1.2; // Zoom in
                });
                
                let initial_zoom = map_view.get_zoom_level();
                map_view.show(ui, &mut tasks, &mut goals);
                
                // Zoom should change
                assert_ne!(map_view.get_zoom_level(), initial_zoom,
                    "Pinch zoom should change zoom level");
            });
        });
    }
    
    /// Test that panning doesn't interfere with other interactions
    #[test]
    fn test_panning_isolation() {
        let ctx = egui::Context::default();
        let mut map_view = MapView::new();
        
        let mut tasks: Vec<Task> = vec![
            {
                let mut task = Task::new("Test Task".to_string(), String::new());
                task.set_position(100.0, 100.0);
                task
            }
        ];
        let mut goals: Vec<Goal> = vec![];
        
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Test that panning state is properly managed
                assert!(!map_view.is_panning(), "Should not be panning initially");
                
                // Start pan
                ctx.input_mut(|i| {
                    i.pointer.button_down[egui::PointerButton::Middle as usize] = true;
                    i.pointer.delta = egui::Vec2::new(10.0, 10.0);
                });
                
                map_view.show(ui, &mut tasks, &mut goals);
                assert!(map_view.is_panning(), "Should be panning when dragging");
                
                // Release
                ctx.input_mut(|i| {
                    i.pointer.button_down[egui::PointerButton::Middle as usize] = false;
                });
                
                map_view.show(ui, &mut tasks, &mut goals);
                assert!(!map_view.is_panning(), "Should stop panning when released");
            });
        });
    }
}