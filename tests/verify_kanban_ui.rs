/// This test verifies that the Kanban UI is properly integrated
/// and has all the expected drag-and-drop features
use plon::ui::PlonApp;
use plon::domain::task::{Task, TaskStatus};

#[test]
fn test_kanban_ui_has_drag_drop_capability() {
    let mut app = PlonApp::new_for_test();
    
    // Add some test tasks
    let task1 = Task::new("Card 1".to_string(), "This should be draggable".to_string());
    let task2 = Task::new("Card 2".to_string(), "This should also be draggable".to_string());
    
    app.add_test_task(task1.clone());
    app.add_test_task(task2.clone());
    
    // Switch to Kanban view
    app.switch_to_kanban_view();
    
    // Verify kanban view has the improved features
    let kanban = app.get_kanban_view();
    
    // Check that columns are properly initialized
    assert_eq!(kanban.columns.len(), 4, "Should have 4 columns");
    assert_eq!(kanban.columns[0].title, "To Do");
    assert_eq!(kanban.columns[1].title, "In Progress");
    assert_eq!(kanban.columns[2].title, "Review");
    assert_eq!(kanban.columns[3].title, "Done");
    
    // Check that columns have bounds set (for drag detection)
    for column in &kanban.columns {
        assert!(column.bounds.width() > 0.0, "Column {} should have width", column.title);
        assert!(column.bounds.height() > 0.0, "Column {} should have height", column.title);
    }
    
    // Check drag functionality exists
    assert!(!kanban.is_dragging(), "Should not be dragging initially");
    
    // Verify we can start a drag
    app.start_kanban_drag(task1.id, eframe::egui::Pos2::new(100.0, 100.0));
    assert!(app.is_kanban_dragging(), "Should be able to start dragging");
    
    // Verify drag context is properly set up
    let kanban = app.get_kanban_view();
    assert!(kanban.drag_context.is_some(), "Drag context should be set");
    
    let drag_ctx = kanban.drag_context.as_ref().unwrap();
    assert_eq!(drag_ctx.task_id, task1.id, "Should be dragging the correct task");
    
    // Complete the drag
    app.complete_kanban_drag(1);
    assert!(!app.is_kanban_dragging(), "Should stop dragging after completion");
    
    // Verify task moved
    let task = app.get_task(task1.id).unwrap();
    assert_eq!(task.status, TaskStatus::InProgress, "Task should have moved to In Progress");
}

#[test]
fn test_kanban_cards_are_interactive() {
    let mut app = PlonApp::new_for_test();
    
    let mut task = Task::new("Interactive Card".to_string(), "Should respond to clicks".to_string());
    task.description = "This card should be clickable and draggable".to_string();
    task.tags.insert("important".to_string());
    task.priority = plon::domain::task::Priority::High;
    
    let task_id = task.id;
    app.add_test_task(task);
    app.switch_to_kanban_view();
    
    // Test selection
    app.select_kanban_task(task_id);
    assert_eq!(app.get_selected_kanban_task(), Some(task_id), "Should be able to select card");
    
    // Test that card has proper rendering info
    let kanban = app.get_kanban_view();
    let card_height = kanban.calculate_card_height(&kanban.tasks[0]);
    assert!(card_height > 0.0, "Card should have height");
    
    let card_color = kanban.get_card_color(&kanban.tasks[0]);
    // High priority should have specific color
    assert_eq!(card_color.r(), 255);
    assert_eq!(card_color.g(), 150);
    assert_eq!(card_color.b(), 100);
}

#[test]
fn test_kanban_columns_are_drop_zones() {
    let mut app = PlonApp::new_for_test();
    
    let task = Task::new("Test Task".to_string(), "".to_string());
    app.add_test_task(task.clone());
    app.switch_to_kanban_view();
    
    // Update layout first
    {
        let kanban = app.get_kanban_view_mut();
        kanban.update_layout(1200.0);
    }
    
    let kanban = app.get_kanban_view();
    
    // Check column 0 (Todo)
    let todo_center = kanban.columns[0].bounds.center();
    assert!(kanban.is_over_column(todo_center, 0), "Should detect position over Todo column");
    
    // Check column 1 (In Progress)
    let progress_center = kanban.columns[1].bounds.center();
    assert!(kanban.is_over_column(progress_center, 1), "Should detect position over In Progress column");
    
    // Check that position outside columns is not detected
    let outside = eframe::egui::Pos2::new(-100.0, -100.0);
    assert!(!kanban.is_over_column(outside, 0), "Should not detect position outside columns");
}

#[test]
fn test_improved_kanban_is_active() {
    // This test verifies we're using the improved version
    let app = PlonApp::new_for_test();
    
    // The improved kanban view should have these methods
    let kanban = app.get_kanban_view();
    
    // Check for improved features
    assert_eq!(kanban.columns[1].wip_limit, Some(3), "Should have WIP limits");
    assert!(kanban.quick_add_states.is_empty(), "Should have quick add capability");
    assert!(kanban.selected_tasks.is_empty(), "Should have multi-select capability");
    assert_eq!(kanban.viewport_width, 1200.0, "Should have responsive layout");
}