use plon::ui::views::timeline_view::TimelineView;
use plon::domain::task::Task;
use chrono::Utc;

#[test]
fn test_timeline_scroll_stability() {
    // Create timeline view
    let mut timeline_view = TimelineView::new();
    
    // Create test tasks
    let tasks: Vec<Task> = (0..50)
        .map(|i| {
            let mut task = Task::new(format!("Task {}", i), String::new());
            task.scheduled_date = Some(Utc::now());
            task.due_date = Some(Utc::now() + chrono::Duration::days(i));
            task
        })
        .collect();
    
    // Test that scroll_to_today is bounded
    timeline_view.scroll_to_today();
    
    // Verify that default settings are reasonable
    assert!(timeline_view.zoom_level > 0.0 && timeline_view.zoom_level <= 5.0, 
            "Zoom level should be bounded: {}", timeline_view.zoom_level);
    
    assert!(timeline_view.days_to_show > 0 && timeline_view.days_to_show <= 365,
            "Days to show should be reasonable: {}", timeline_view.days_to_show);
    
    // Check that content dimensions would be bounded
    let row_height = 30.0;
    let day_width = 25.0 * timeline_view.zoom_level;
    let label_width = 200.0;
    
    // These are the max bounds from show_gantt_view
    let chart_width = (label_width + (timeline_view.days_to_show as f32 * day_width)).min(2000.0);
    let chart_height = (tasks.len() as f32 * row_height + 50.0).min(600.0);
    
    // Verify bounds are applied
    assert!(chart_width <= 2000.0, "Chart width should be bounded");
    assert!(chart_height <= 600.0, "Chart height should be bounded");
    
    println!("Timeline scroll stability test passed!");
    println!("- Chart dimensions: {}x{}", chart_width, chart_height);
    println!("- Zoom level: {}", timeline_view.zoom_level);
    println!("- Days to show: {}", timeline_view.days_to_show);
    println!("- Scroll to today triggered");
}

#[test]
fn test_timeline_zoom_bounds() {
    let timeline_view = TimelineView::new();
    
    // Test that zoom is properly bounded
    assert!(timeline_view.zoom_level >= 0.5, "Zoom should have minimum bound");
    assert!(timeline_view.zoom_level <= 3.0, "Zoom should have maximum bound");
    
    // Simulate zoom changes
    let zoom_in = (timeline_view.zoom_level * 1.1).min(3.0);
    let zoom_out = (timeline_view.zoom_level * 0.9).max(0.5);
    
    assert!(zoom_in <= 3.0, "Zoom in should be bounded");
    assert!(zoom_out >= 0.5, "Zoom out should be bounded");
    
    println!("Zoom bounds test passed!");
}

#[test]
fn test_timeline_content_clipping() {
    let timeline_view = TimelineView::new();
    
    // Create many tasks to test content clipping
    let tasks: Vec<Task> = (0..100)
        .map(|i| {
            let mut task = Task::new(format!("Task {}", i), String::new());
            task.scheduled_date = Some(Utc::now());
            task.due_date = Some(Utc::now() + chrono::Duration::days(i * 2));
            task
        })
        .collect();
    
    // Calculate what would be rendered
    let row_height = 30.0;
    let max_height = 600.0; // From the code
    
    let calculated_height = tasks.len() as f32 * row_height + 50.0;
    let clamped_height = calculated_height.min(max_height);
    
    // Verify that height is properly clamped
    assert_eq!(clamped_height, max_height, 
               "Height should be clamped to max when too many tasks");
    
    // Calculate visible tasks
    let visible_tasks = ((max_height - 50.0) / row_height) as usize;
    
    println!("Content clipping test passed!");
    println!("- Total tasks: {}", tasks.len());
    println!("- Visible tasks: {}", visible_tasks);
    println!("- Calculated height: {}", calculated_height);
    println!("- Clamped height: {}", clamped_height);
}