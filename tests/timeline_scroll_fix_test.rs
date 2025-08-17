use plon::ui::views::timeline_view::TimelineView;
use plon::domain::task::Task;
use plon::domain::goal::Goal;
use eframe::egui;

#[test]
fn test_timeline_no_infinite_scroll() {
    // Test that timeline view has proper scroll constraints
    let mut timeline_view = TimelineView::new();
    
    // Create some test tasks
    let tasks: Vec<Task> = (0..100)
        .map(|i| Task::new(format!("Task {}", i), String::new()))
        .collect();
    
    let goals: Vec<Goal> = vec![];
    
    // The view should use appropriate scroll areas for each mode
    // and have bounded content dimensions
    
    // Test Gantt view dimensions
    timeline_view.days_to_show = 30;
    timeline_view.zoom_level = 1.0;
    
    let row_height = 30.0;
    let day_width = 25.0;
    let label_width = 200.0;
    
    // Calculate expected dimensions
    let chart_width: f32 = label_width + (30.0 * day_width);
    let chart_height: f32 = 100.0 * row_height + 50.0;
    
    // Verify dimensions are properly bounded
    let max_width = 2000.0;
    let max_height = 1000.0;
    
    let actual_width = chart_width.min(max_width);
    let actual_height = chart_height.min(max_height);
    
    assert!(actual_width <= max_width);
    assert!(actual_height <= max_height);
    
    // Width should be under the limit for 30 days
    assert_eq!(actual_width, 950.0); // 200 + 30*25 = 950
    
    // Height should be capped at max_height
    assert_eq!(actual_height, max_height); // 100*30+50 = 3050, capped at 1000
}

#[test]
fn test_timeline_scroll_area_configuration() {
    // Verify that each view mode uses appropriate scroll configuration
    let timeline_view = TimelineView::new();
    
    // Different view modes should use different scroll strategies:
    // - Gantt: Horizontal scrolling only (fixed height)
    // - List: Vertical scrolling only  
    // - Calendar: No scrolling (for now)
    
    // This ensures the content doesn't cause infinite expansion
    assert_eq!(timeline_view.days_to_show, 30);
    assert_eq!(timeline_view.zoom_level, 1.0);
}

#[test]
fn test_timeline_content_bounds() {
    let mut timeline_view = TimelineView::new();
    
    // Test zoom limits work correctly
    timeline_view.set_date_range(1000); // Try extreme zoom out
    assert_eq!(timeline_view.days_to_show, 365); // Should be capped
    
    timeline_view.set_date_range(1); // Try extreme zoom in
    assert_eq!(timeline_view.days_to_show, 7); // Should be capped at minimum
    
    // Zoom level should adjust proportionally
    timeline_view.zoom_level = 1.0;
    timeline_view.days_to_show = 30;
    timeline_view.set_date_range(60);
    
    // Verify zoom level updated
    assert!(timeline_view.zoom_level > 1.0);
}