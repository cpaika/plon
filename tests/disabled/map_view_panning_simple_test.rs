use eframe::egui;
use plon::domain::goal::Goal;
use plon::domain::task::Task;
use plon::ui::views::map_view::MapView;

/// Simple test to verify map view panning methods exist and work
#[test]
fn test_map_view_has_panning_methods() {
    let map_view = MapView::new();

    // Test that getter methods exist
    let _camera_pos = map_view.get_camera_position();
    let _zoom = map_view.get_zoom_level();
    let _is_panning = map_view.is_panning();

    // Verify initial state
    assert!(!map_view.is_panning(), "Should not be panning initially");
    assert!(
        map_view.get_zoom_level() > 0.0,
        "Zoom level should be positive"
    );
}

/// Test that map view can be created and shown
#[test]
fn test_map_view_basic_functionality() {
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();

    let mut tasks: Vec<Task> = vec![
        {
            let mut task = Task::new("Test Task 1".to_string(), String::new());
            task.set_position(100.0, 100.0);
            task
        },
        {
            let mut task = Task::new("Test Task 2".to_string(), String::new());
            task.set_position(200.0, 200.0);
            task
        },
    ];

    let mut goals: Vec<Goal> = vec![];

    // Run the map view
    ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Just verify it doesn't panic
            map_view.show(ui, &mut tasks, &mut goals);
        });
    });

    // The map view should have handled the rendering without issues
    assert!(true, "Map view rendered successfully");
}

/// Test zoom level boundaries
#[test]
fn test_map_view_zoom_limits() {
    let mut map_view = MapView::new();

    // Test setting extreme zoom values
    map_view.set_zoom_level(0.01); // Very small
    assert!(
        map_view.get_zoom_level() >= 0.1,
        "Zoom should be clamped to minimum"
    );

    map_view.set_zoom_level(100.0); // Very large
    assert!(
        map_view.get_zoom_level() <= 5.0,
        "Zoom should be clamped to maximum"
    );

    // Test normal zoom value
    map_view.set_zoom_level(2.0);
    assert_eq!(
        map_view.get_zoom_level(),
        2.0,
        "Normal zoom should be set correctly"
    );
}
