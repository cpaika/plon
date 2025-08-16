use plon::ui::PlonApp;
use plon::domain::task::{Task, TaskStatus};
use eframe::egui;

#[test]
fn test_kanban_view_is_using_improved_version() {
    // Create the app
    let app = PlonApp::new_for_test();
    
    // The kanban view should be the improved version with proper methods
    assert!(app.has_improved_kanban_view(), "App should use the improved Kanban view");
}

#[test]
fn test_kanban_drag_drop_in_main_app() {
    let mut app = PlonApp::new_for_test();
    
    // Add some test tasks
    let task1 = Task::new("Test Task 1".to_string(), "Description 1".to_string());
    let task2 = Task::new("Test Task 2".to_string(), "Description 2".to_string());
    let task3 = Task::new("Test Task 3".to_string(), "Description 3".to_string());
    
    app.add_test_task(task1.clone());
    app.add_test_task(task2.clone());
    app.add_test_task(task3.clone());
    
    // Switch to Kanban view
    app.switch_to_kanban_view();
    
    // Verify kanban view is active
    assert!(app.is_kanban_view_active(), "Kanban view should be active");
    
    // Test that drag and drop methods exist and work
    assert!(app.can_start_drag_in_kanban(task1.id), "Should be able to start drag");
    
    // Start dragging task1
    app.start_kanban_drag(task1.id, egui::Pos2::new(100.0, 100.0));
    assert!(app.is_kanban_dragging(), "Should be dragging in Kanban view");
    
    // Move to different column
    app.update_kanban_drag(egui::Pos2::new(400.0, 100.0));
    app.complete_kanban_drag(1); // Move to column 1 (In Progress)
    
    // Verify task moved
    let task = app.get_task(task1.id).expect("Task should exist");
    assert_eq!(task.status, TaskStatus::InProgress, "Task should have moved to In Progress");
}

#[test]
fn test_kanban_view_renders_properly() {
    let mut app = PlonApp::new_for_test();
    
    // Add tasks with different statuses
    let mut todo_task = Task::new("Todo Task".to_string(), "".to_string());
    todo_task.status = TaskStatus::Todo;
    
    let mut progress_task = Task::new("Progress Task".to_string(), "".to_string());
    progress_task.status = TaskStatus::InProgress;
    
    let mut done_task = Task::new("Done Task".to_string(), "".to_string());
    done_task.status = TaskStatus::Done;
    
    app.add_test_task(todo_task);
    app.add_test_task(progress_task);
    app.add_test_task(done_task);
    
    app.switch_to_kanban_view();
    
    // Verify columns exist and have correct tasks
    assert_eq!(app.get_kanban_column_count(), 4, "Should have 4 columns");
    assert_eq!(app.get_kanban_tasks_in_column(0).len(), 1, "Todo column should have 1 task");
    assert_eq!(app.get_kanban_tasks_in_column(1).len(), 1, "In Progress column should have 1 task");
    assert_eq!(app.get_kanban_tasks_in_column(3).len(), 1, "Done column should have 1 task");
}

#[test]
fn test_kanban_quick_add_in_app() {
    let mut app = PlonApp::new_for_test();
    app.switch_to_kanban_view();
    
    // Enable quick add for Todo column
    app.enable_kanban_quick_add(0);
    assert!(app.is_kanban_quick_add_active(0), "Quick add should be active");
    
    // Add a task via quick add
    app.kanban_quick_add_task(0, "Quick Task".to_string());
    
    // Verify task was added
    let tasks = app.get_kanban_tasks_in_column(0);
    assert_eq!(tasks.len(), 1, "Should have added task");
    assert_eq!(tasks[0].title, "Quick Task", "Task should have correct title");
}

#[test]
fn test_kanban_selection_in_app() {
    let mut app = PlonApp::new_for_test();
    
    let task = Task::new("Test Task".to_string(), "".to_string());
    let task_id = task.id;
    app.add_test_task(task);
    
    app.switch_to_kanban_view();
    
    // Select task
    app.select_kanban_task(task_id);
    assert_eq!(app.get_selected_kanban_task(), Some(task_id), "Task should be selected");
    
    // Clear selection
    app.clear_kanban_selection();
    assert_eq!(app.get_selected_kanban_task(), None, "No task should be selected");
}