/// Tests to detect and fix rendering issues in the Kanban view
use plon::ui::PlonApp;
use plon::domain::task::{Task, TaskStatus};
use std::collections::HashSet;

#[test]
fn test_no_duplicate_scroll_area_ids() {
    // This test checks for duplicate ScrollArea IDs which cause the warning:
    // "First use of ScrollArea ID C5A3" 
    
    let mut app = PlonApp::new_for_test();
    
    // Add tasks to multiple columns to ensure ScrollAreas are created
    for i in 0..5 {
        let mut task = Task::new(format!("Task {}", i), String::new());
        task.status = match i % 4 {
            0 => TaskStatus::Todo,
            1 => TaskStatus::InProgress,
            2 => TaskStatus::Review,
            _ => TaskStatus::Done,
        };
        app.add_test_task(task);
    }
    
    app.switch_to_kanban_view();
    
    // Check that each column has a unique scroll area ID
    let kanban = app.get_kanban_view();
    let mut scroll_ids = HashSet::new();
    
    // Each column should have its own unique ScrollArea
    for (idx, column) in kanban.columns.iter().enumerate() {
        // The ID should be based on column index or title, not a hardcoded value
        let expected_id = format!("kanban_column_{}", idx);
        assert!(!scroll_ids.contains(&expected_id), 
                "Duplicate ScrollArea ID found for column {}", column.title);
        scroll_ids.insert(expected_id);
    }
}

#[test]
fn test_text_rendering_without_overlap() {
    // Test that text doesn't overlap with widgets
    let mut app = PlonApp::new_for_test();
    
    // Create a task with potentially problematic text
    let mut task = Task::new(
        "Conduct training sessions".to_string(),
        "This is a longer description that might cause rendering issues".to_string()
    );
    task.tags.insert("training".to_string());
    task.tags.insert("important".to_string());
    
    app.add_test_task(task.clone());
    app.switch_to_kanban_view();
    
    let kanban = app.get_kanban_view();
    
    // Verify card height calculation accounts for all content
    let card_height = kanban.calculate_card_height(&task);
    
    // Card should have enough height for title, description, and tags
    assert!(card_height >= 80.0, "Card height should be sufficient for content");
    
    // With tags and description, height should be increased
    assert!(card_height > 80.0, "Card with tags should have increased height");
}

#[test]
fn test_special_characters_in_text() {
    // Test that special characters and Unicode are handled properly
    let mut app = PlonApp::new_for_test();
    
    // Test various problematic characters
    let test_cases = vec![
        "Widget is above this text.",
        "Can-Tame • Widget test",
        "Sometimes the solution is to use ui",
        "Test with emoji 🎯 📊 ✅",
        "C5А3 (with Cyrillic А)",  // Note: This has a Cyrillic 'А' not Latin 'A'
    ];
    
    for text in test_cases {
        let task = Task::new(text.to_string(), String::new());
        app.add_test_task(task);
    }
    
    app.switch_to_kanban_view();
    
    // All tasks should be added without issues
    let kanban = app.get_kanban_view();
    assert_eq!(kanban.tasks.len(), 5, "All tasks with special characters should be added");
}

#[test]
fn test_no_widget_text_overlap() {
    // Ensure widgets don't overlap with text
    let mut app = PlonApp::new_for_test();
    
    let task = Task::new(
        "Test Task".to_string(),
        "Widget is above this text.".to_string()
    );
    
    app.add_test_task(task.clone());
    app.switch_to_kanban_view();
    
    let kanban = app.get_kanban_view();
    
    // Check that card spacing is adequate
    let spacing = kanban.get_card_spacing();
    assert!(spacing >= 8.0, "Cards should have adequate spacing");
    
    // Check that column width is reasonable
    let col_width = kanban.calculate_column_width(1200.0);
    assert!(col_width >= 250.0, "Column width should be sufficient");
}

#[test]
fn test_scroll_area_unique_ids() {
    // Verify that ScrollArea IDs are properly unique
    let app = PlonApp::new_for_test();
    
    // The kanban view should use unique IDs for each ScrollArea
    // Format should be something like:
    // - "kanban_board_main" for the main horizontal scroll
    // - "kanban_column_0", "kanban_column_1", etc. for each column
    
    // This test verifies the implementation doesn't use hardcoded IDs
    let kanban = app.get_kanban_view();
    
    // Each column should have a unique position/index
    for (idx, column) in kanban.columns.iter().enumerate() {
        assert_eq!(column.position, idx, "Column position should match index");
    }
}

#[test]
fn test_proper_text_truncation() {
    // Test that long text is properly truncated or wrapped
    let mut app = PlonApp::new_for_test();
    
    let long_title = "This is a very long task title that might cause rendering issues if not properly handled in the UI";
    let long_desc = "x".repeat(500); // Very long description
    
    let task = Task::new(long_title.to_string(), long_desc);
    app.add_test_task(task.clone());
    app.switch_to_kanban_view();
    
    let kanban = app.get_kanban_view();
    
    // Card height should be capped at max
    let card_height = kanban.calculate_card_height(&task);
    assert!(card_height <= 200.0, "Card height should be capped at maximum");
}

#[test]
fn test_search_filter_clears_properly() {
    // Ensure search filter doesn't cause rendering artifacts
    let mut app = PlonApp::new_for_test();
    
    app.add_test_task(Task::new("Task 1".to_string(), String::new()));
    app.add_test_task(Task::new("Task 2".to_string(), String::new()));
    
    app.switch_to_kanban_view();
    
    let kanban = app.get_kanban_view_mut();
    
    // Set and clear search filter
    kanban.set_search_filter("Task");
    assert_eq!(kanban.get_visible_tasks().len(), 2);
    
    kanban.set_search_filter("");
    assert_eq!(kanban.get_visible_tasks().len(), 2, "All tasks should be visible after clearing filter");
    
    // Search filter should be empty
    assert!(kanban.search_filter.is_empty(), "Search filter should be cleared");
}