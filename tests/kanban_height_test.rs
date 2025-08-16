/// Test to verify Kanban view uses full available height
use plon::ui::PlonApp;
use plon::domain::task::{Task, TaskStatus};

#[test]
fn test_kanban_view_uses_full_height() {
    let mut app = PlonApp::new_for_test();
    
    // Add multiple tasks to ensure we have content
    for i in 0..10 {
        let mut task = Task::new(
            format!("Task {}", i),
            format!("Description for task {}", i)
        );
        task.status = match i % 4 {
            0 => TaskStatus::Todo,
            1 => TaskStatus::InProgress,
            2 => TaskStatus::Review,
            _ => TaskStatus::Done,
        };
        app.add_test_task(task);
    }
    
    app.switch_to_kanban_view();
    
    let kanban = app.get_kanban_view();
    
    // Check column bounds height
    for (idx, column) in kanban.columns.iter().enumerate() {
        assert!(
            column.bounds.height() >= 500.0,
            "Column {} ('{}') height is only {} - should use more vertical space",
            idx,
            column.title,
            column.bounds.height()
        );
    }
    
    // The viewport should be using most of the available space
    // Default test height is usually around 600-800px
    let expected_min_height = 500.0;
    for column in &kanban.columns {
        assert!(
            column.bounds.height() >= expected_min_height,
            "Column '{}' should have at least {} height, but has {}",
            column.title,
            expected_min_height,
            column.bounds.height()
        );
    }
}

#[test]
fn test_kanban_scroll_area_has_adequate_height() {
    let mut app = PlonApp::new_for_test();
    
    app.switch_to_kanban_view();
    let kanban = app.get_kanban_view();
    
    // The scroll area max_height in the current implementation is hardcoded to 500.0
    // This test will fail if it's too small
    // We want it to be dynamic based on available space
    
    // Check that columns have reasonable initial bounds
    for column in &kanban.columns {
        assert!(
            column.bounds.height() >= 500.0,
            "Initial column bounds height should be at least 500px, found {}",
            column.bounds.height()
        );
    }
}

#[test]
fn test_kanban_responsive_to_viewport_height() {
    let mut app = PlonApp::new_for_test();
    
    // Add many tasks to create scrollable content
    for i in 0..20 {
        let task = Task::new(
            format!("Task {}", i),
            "This task should be visible when scrolling".to_string()
        );
        app.add_test_task(task);
    }
    
    app.switch_to_kanban_view();
    
    // Update layout with a larger viewport
    let large_viewport_height = 1080.0;
    let kanban = app.get_kanban_view_mut();
    kanban.update_layout(1920.0); // This currently only handles width
    
    // The columns should adapt to use available height
    // Currently they're initialized with fixed 600px height
    for column in &kanban.columns {
        println!("Column {} height: {}", column.title, column.bounds.height());
        // This will likely fail with current implementation
        assert!(
            column.bounds.height() >= 600.0,
            "Column should use available viewport height"
        );
    }
}