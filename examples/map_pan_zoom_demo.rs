// Demonstration of Map View Pan and Zoom functionality
// This example shows how the MapView pan and zoom features work

use eframe::egui::{PointerButton, Pos2, Rect, Vec2};
use plon::ui::views::map_view::MapView;

fn main() {
    println!("Map View Pan and Zoom Demo");
    println!("==========================\n");

    let mut map_view = MapView::new();

    // Demonstrate Pan functionality
    println!("1. PAN FUNCTIONALITY");
    println!(
        "   Initial camera position: {:?}",
        map_view.get_camera_position()
    );

    // Simulate middle mouse drag
    let start = Pos2::new(100.0, 100.0);
    let end = Pos2::new(200.0, 150.0);

    map_view.start_pan(start, PointerButton::Middle);
    map_view.update_pan(end);
    map_view.end_pan();

    println!("   After panning from {:?} to {:?}:", start, end);
    println!(
        "   New camera position: {:?}\n",
        map_view.get_camera_position()
    );

    // Demonstrate Zoom functionality
    println!("2. ZOOM FUNCTIONALITY");
    println!("   Initial zoom level: {}", map_view.get_zoom_level());

    map_view.zoom_in();
    println!("   After zoom in: {}", map_view.get_zoom_level());

    map_view.zoom_out();
    map_view.zoom_out();
    println!("   After 2x zoom out: {}", map_view.get_zoom_level());

    map_view.set_zoom_level(2.5);
    println!("   Set zoom to 2.5: {}", map_view.get_zoom_level());

    map_view.set_zoom_level(10.0); // Should clamp to max
    println!(
        "   Try to set zoom to 10.0 (clamped): {}\n",
        map_view.get_zoom_level()
    );

    // Demonstrate Trackpad Gestures
    println!("3. TRACKPAD GESTURES");
    map_view.reset_view();
    println!(
        "   Reset view - zoom: {}, camera: {:?}",
        map_view.get_zoom_level(),
        map_view.get_camera_position()
    );

    // Pinch zoom
    map_view.handle_pinch_gesture(Pos2::new(400.0, 300.0), 100.0, 150.0);
    println!(
        "   After pinch zoom (100->150): {}",
        map_view.get_zoom_level()
    );

    // Two-finger pan
    map_view.handle_two_finger_pan(Vec2::new(50.0, -30.0));
    println!(
        "   After two-finger pan: {:?}\n",
        map_view.get_camera_position()
    );

    // Demonstrate Coordinate Transformations
    println!("4. COORDINATE TRANSFORMATIONS");
    map_view.set_camera_position(Vec2::new(100.0, 50.0));
    map_view.set_zoom_level(2.0);

    let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
    let world_pos = Vec2::new(200.0, 150.0);

    let screen_pos = map_view.world_to_screen(world_pos, viewport);
    println!("   World {:?} -> Screen {:?}", world_pos, screen_pos);

    let back_to_world = map_view.screen_to_world(screen_pos, viewport);
    println!("   Screen {:?} -> World {:?}", screen_pos, back_to_world);
    println!(
        "   Round-trip accurate: {}\n",
        (back_to_world - world_pos).length() < 0.01
    );

    // Demonstrate Momentum Scrolling
    println!("5. MOMENTUM SCROLLING");
    map_view.reset_view();

    let initial_velocity = Vec2::new(100.0, 50.0);
    map_view.start_momentum_pan(initial_velocity);
    println!("   Started momentum with velocity: {:?}", initial_velocity);

    // Simulate a few frames
    for i in 0..5 {
        map_view.update_momentum(0.016); // 60fps
        println!(
            "   Frame {}: camera={:?}, velocity={:?}",
            i + 1,
            map_view.get_camera_position(),
            map_view.get_momentum_velocity()
        );
    }

    println!("\n✅ All pan and zoom features demonstrated successfully!");
}
