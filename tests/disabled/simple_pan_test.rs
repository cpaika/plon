use eframe::egui;
use plon::domain::goal::Goal;
use plon::domain::task::Task;
use plon::ui::views::map_view::MapView;
use std::time::Instant;

#[test]
fn test_simple_pan_operations() {
    // Test basic pan operations without UI framework
    let mut map_view = MapView::new();

    println!(
        "Initial camera position: {:?}",
        map_view.get_camera_position()
    );
    println!("Initial zoom level: {}", map_view.get_zoom_level());

    // Test 1: Simple pan
    let start = Instant::now();
    for i in 0..1000 {
        map_view.set_camera_position(egui::Vec2::new(i as f32, i as f32 * 0.5));
    }
    let elapsed = start.elapsed();
    println!("1000 camera position updates took: {:?}", elapsed);
    assert!(elapsed.as_millis() < 10, "Camera updates too slow!");

    // Test 2: Zoom operations
    let start = Instant::now();
    for i in 0..1000 {
        map_view.set_zoom_level(1.0 + (i as f32 * 0.001).sin());
    }
    let elapsed = start.elapsed();
    println!("1000 zoom updates took: {:?}", elapsed);
    assert!(elapsed.as_millis() < 10, "Zoom updates too slow!");

    // Test 3: Pan with mouse simulation
    let start = Instant::now();
    for i in 0..100 {
        let pos = egui::Pos2::new(100.0 + i as f32, 100.0 + i as f32);
        map_view.start_pan(pos, egui::PointerButton::Middle);

        for j in 0..10 {
            let current = pos + egui::Vec2::new(j as f32, j as f32);
            map_view.update_pan(current);
        }

        map_view.end_pan();
    }
    let elapsed = start.elapsed();
    println!("100 pan sequences took: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100, "Pan sequences too slow!");
}

#[test]
fn test_show_method_without_rendering() {
    // Test the show method in isolation
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();

    // Create minimal test data
    let mut tasks: Vec<Task> = vec![
        Task::new("Task 1".to_string(), "Description 1".to_string()),
        Task::new("Task 2".to_string(), "Description 2".to_string()),
    ];
    let mut goals: Vec<Goal> = Vec::new();

    // Measure show method performance
    let mut times = Vec::new();

    for frame in 0..10 {
        ctx.begin_frame(egui::RawInput::default());

        let start = Instant::now();

        egui::CentralPanel::default().show(&ctx, |ui| {
            map_view.show(ui, &mut tasks, &mut goals);
        });

        let elapsed = start.elapsed();
        times.push(elapsed);

        ctx.end_frame();

        println!("Frame {}: {:?}", frame, elapsed);
    }

    let max_time = times.iter().max().unwrap();
    println!("Maximum frame time: {:?}", max_time);

    assert!(
        max_time.as_millis() < 100,
        "Show method taking too long: {:?}",
        max_time
    );
}
