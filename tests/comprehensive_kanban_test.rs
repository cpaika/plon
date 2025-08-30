use chrono::Utc;
use eframe::egui::{self, Pos2};
use plon::domain::task::{Priority, Task, TaskStatus};
use plon::ui::PlonApp;

/// Test that verifies the actual drag-and-drop UI interaction works
#[test]
fn test_actual_drag_drop_workflow() {
    let mut app = PlonApp::new_for_test();

    // Create tasks with different statuses
    let mut todo_task = Task::new("Fix bug".to_string(), "Critical bug in login".to_string());
    todo_task.status = TaskStatus::Todo;
    todo_task.priority = Priority::High;

    let mut in_progress_task =
        Task::new("Feature X".to_string(), "Implement feature X".to_string());
    in_progress_task.status = TaskStatus::InProgress;

    let todo_id = todo_task.id;
    let in_progress_id = in_progress_task.id;

    // Add tasks to app and sync with kanban
    app.add_test_task(todo_task);
    app.add_test_task(in_progress_task);

    // Switch to kanban view
    app.switch_to_kanban_view();
    assert!(app.is_kanban_view_active());

    // Verify initial state
    let todo_tasks = app.get_kanban_tasks_in_column(0);
    let in_progress_tasks = app.get_kanban_tasks_in_column(1);
    assert_eq!(todo_tasks.len(), 1, "Should have 1 task in Todo");
    assert_eq!(
        in_progress_tasks.len(),
        1,
        "Should have 1 task in In Progress"
    );

    // CRITICAL TEST: Simulate actual drag and drop
    // 1. Start dragging from Todo column
    app.start_kanban_drag(todo_id, Pos2::new(150.0, 200.0));
    assert!(app.is_kanban_dragging(), "Should be in dragging state");

    // 2. Update drag position to In Progress column
    app.update_kanban_drag(Pos2::new(450.0, 200.0));

    // 3. Complete the drag
    app.complete_kanban_drag(1); // Drop in In Progress column

    // 4. Verify task moved
    assert!(!app.is_kanban_dragging(), "Should no longer be dragging");

    let todo_tasks_after = app.get_kanban_tasks_in_column(0);
    let in_progress_tasks_after = app.get_kanban_tasks_in_column(1);

    assert_eq!(todo_tasks_after.len(), 0, "Todo should be empty");
    assert_eq!(
        in_progress_tasks_after.len(),
        2,
        "In Progress should have 2 tasks"
    );

    // Verify task status changed
    let moved_task = app.get_task(todo_id).expect("Task should exist");
    assert_eq!(
        moved_task.status,
        TaskStatus::InProgress,
        "Task status should be updated"
    );
}

/// Test column interactions and UI state
#[test]
fn test_kanban_column_interactions() {
    let mut app = PlonApp::new_for_test();
    app.switch_to_kanban_view();

    // Test column count
    assert_eq!(
        app.get_kanban_column_count(),
        4,
        "Should have 4 default columns"
    );

    // Test WIP limits
    let kanban = app.get_kanban_view();
    assert!(
        kanban.columns[1].wip_limit.is_some(),
        "In Progress should have WIP limit"
    );
    assert_eq!(
        kanban.columns[1].wip_limit,
        Some(3),
        "In Progress WIP limit should be 3"
    );

    // Add tasks to test WIP limit
    for i in 0..5 {
        let mut task = Task::new(format!("Task {}", i), String::new());
        task.status = TaskStatus::InProgress;
        app.add_test_task(task);
    }

    // Check if column is over WIP limit
    let kanban = app.get_kanban_view();
    assert!(
        kanban.is_column_over_wip_limit(1),
        "Should be over WIP limit"
    );
}

/// Test task selection and multi-select
#[test]
fn test_kanban_selection_and_multi_select() {
    let mut app = PlonApp::new_for_test();

    let task1 = Task::new("Task 1".to_string(), String::new());
    let task2 = Task::new("Task 2".to_string(), String::new());
    let task3 = Task::new("Task 3".to_string(), String::new());

    let id1 = task1.id;
    let id2 = task2.id;
    let id3 = task3.id;

    app.add_test_task(task1);
    app.add_test_task(task2);
    app.add_test_task(task3);

    app.switch_to_kanban_view();

    // Test single selection
    app.select_kanban_task(id1);
    assert_eq!(app.get_selected_kanban_task(), Some(id1));

    // Test adding to selection
    let kanban = app.get_kanban_view_mut();
    kanban.add_to_selection(id2);
    kanban.add_to_selection(id3);

    assert_eq!(
        kanban.selected_tasks.len(),
        3,
        "Should have 3 selected tasks"
    );

    // Test bulk move
    kanban.bulk_move_selected(1); // Move all to In Progress

    // Verify all moved
    let in_progress_tasks = kanban.get_tasks_for_column(1);
    assert_eq!(
        in_progress_tasks.len(),
        3,
        "All tasks should be in In Progress"
    );
}

/// Test search and filtering
#[test]
fn test_kanban_search_and_filter() {
    let mut app = PlonApp::new_for_test();

    app.add_test_task(Task::new(
        "Fix login bug".to_string(),
        "Critical issue".to_string(),
    ));
    app.add_test_task(Task::new(
        "Add feature".to_string(),
        "New feature request".to_string(),
    ));
    app.add_test_task(Task::new(
        "Update docs".to_string(),
        "Documentation update".to_string(),
    ));

    app.switch_to_kanban_view();

    let kanban = app.get_kanban_view_mut();

    // Test search
    kanban.set_search_filter("bug");
    let visible = kanban.get_visible_tasks();
    assert_eq!(visible.len(), 1, "Should only show bug-related task");
    assert!(visible[0].title.contains("bug"));

    // Clear search
    kanban.set_search_filter("");
    let all_visible = kanban.get_visible_tasks();
    assert_eq!(all_visible.len(), 3, "Should show all tasks");
}

/// Test quick add functionality
#[test]
fn test_kanban_quick_add_comprehensive() {
    let mut app = PlonApp::new_for_test();
    app.switch_to_kanban_view();

    // Test quick add for each column
    for column_idx in 0..4 {
        app.enable_kanban_quick_add(column_idx);
        assert!(app.is_kanban_quick_add_active(column_idx));

        let title = format!("Quick task for column {}", column_idx);
        app.kanban_quick_add_task(column_idx, title.clone());

        // Verify task was added
        let tasks = app.get_kanban_tasks_in_column(column_idx);
        assert!(
            tasks.iter().any(|t| t.title == title),
            "Should have added task to column {}",
            column_idx
        );
    }
}

/// Test keyboard shortcuts
#[test]
fn test_kanban_keyboard_navigation() {
    let mut app = PlonApp::new_for_test();

    let task = Task::new("Navigate me".to_string(), String::new());
    let task_id = task.id;
    app.add_test_task(task);

    app.switch_to_kanban_view();
    app.select_kanban_task(task_id);

    // Test arrow right to move to next column
    {
        let kanban = app.get_kanban_view_mut();
        kanban.handle_keyboard_shortcut(egui::Key::ArrowRight, egui::Modifiers::NONE);
    }

    // Check task moved
    {
        let kanban = app.get_kanban_view();
        let task_after_right = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(
            task_after_right.status,
            TaskStatus::InProgress,
            "Should move to In Progress"
        );
    }

    // Test arrow left to move back
    {
        let kanban = app.get_kanban_view_mut();
        kanban.handle_keyboard_shortcut(egui::Key::ArrowLeft, egui::Modifiers::NONE);
    }

    // Check task moved back
    {
        let kanban = app.get_kanban_view();
        let task_after_left = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(
            task_after_left.status,
            TaskStatus::Todo,
            "Should move back to Todo"
        );
    }
}

/// Test column collapse/expand
#[test]
fn test_kanban_column_collapse() {
    let mut app = PlonApp::new_for_test();
    app.switch_to_kanban_view();

    let kanban = app.get_kanban_view_mut();

    // Test toggle collapse
    assert!(
        !kanban.is_column_collapsed(0),
        "Should not be collapsed initially"
    );

    kanban.toggle_column_collapse(0);
    assert!(kanban.is_column_collapsed(0), "Should be collapsed");

    kanban.toggle_column_collapse(0);
    assert!(!kanban.is_column_collapsed(0), "Should be expanded again");
}

/// Test task details and metadata
#[test]
fn test_kanban_task_details() {
    let mut app = PlonApp::new_for_test();

    let mut task = Task::new(
        "Detailed task".to_string(),
        "With lots of metadata".to_string(),
    );
    task.priority = Priority::Critical;
    task.due_date = Some(Utc::now() + chrono::Duration::days(3));
    task.tags.insert("urgent".to_string());
    task.tags.insert("backend".to_string());
    task.estimated_hours = Some(8.0);

    app.add_test_task(task.clone());
    app.switch_to_kanban_view();

    let kanban = app.get_kanban_view();

    // Test priority color
    let color = kanban.get_card_color(&task);
    // Color32 doesn't implement PartialEq, so compare components
    let expected = egui::Color32::from_rgb(255, 100, 100);
    assert_eq!(color.r(), expected.r());
    assert_eq!(color.g(), expected.g());
    assert_eq!(color.b(), expected.b(), "Critical priority should be red");

    // Test overdue highlighting
    let mut overdue_task = task.clone();
    overdue_task.due_date = Some(Utc::now() - chrono::Duration::days(1));
    assert!(
        kanban.should_highlight_as_overdue(&overdue_task),
        "Should highlight overdue"
    );
}

/// Test responsive layout
#[test]
fn test_kanban_responsive_layout() {
    let mut app = PlonApp::new_for_test();
    app.switch_to_kanban_view();

    let kanban = app.get_kanban_view_mut();

    // Test desktop layout
    kanban.update_layout(1400.0);
    assert!(
        !kanban.should_stack_columns(),
        "Should not stack on wide screen"
    );

    let width = kanban.calculate_column_width(1400.0);
    assert!(
        width >= 320.0 && width <= 400.0,
        "Column width should be in reasonable range"
    );

    // Test mobile layout
    kanban.update_layout(600.0);
    assert!(
        kanban.should_stack_columns(),
        "Should stack on narrow screen"
    );
}

/// Test drag and drop with position tracking
#[test]
fn test_kanban_drag_position_tracking() {
    let mut app = PlonApp::new_for_test();

    let task = Task::new("Drag me".to_string(), String::new());
    let task_id = task.id;
    app.add_test_task(task);

    app.switch_to_kanban_view();

    let kanban = app.get_kanban_view_mut();

    // Start drag
    kanban.start_drag(task_id, Pos2::new(100.0, 100.0));

    // Track drag position
    kanban.update_drag_position(Pos2::new(200.0, 150.0));
    assert_eq!(kanban.get_drag_position(), Some(Pos2::new(200.0, 150.0)));

    // Update position again
    kanban.update_drag_position(Pos2::new(400.0, 200.0));
    assert_eq!(kanban.get_drag_position(), Some(Pos2::new(400.0, 200.0)));

    // Check hover column detection
    let hover_col = kanban.get_column_at_position(Pos2::new(400.0, 200.0));
    assert!(hover_col.is_some(), "Should detect column at position");

    // Cancel drag
    kanban.cancel_drag();
    assert!(!kanban.is_dragging(), "Should no longer be dragging");
}

/// Test reordering within column
#[test]
fn test_kanban_reorder_within_column() {
    let mut app = PlonApp::new_for_test();

    let task1 = Task::new("First".to_string(), String::new());
    let task2 = Task::new("Second".to_string(), String::new());
    let task3 = Task::new("Third".to_string(), String::new());

    let id1 = task1.id;
    let id2 = task2.id;
    let id3 = task3.id;

    app.add_test_task(task1);
    app.add_test_task(task2);
    app.add_test_task(task3);

    app.switch_to_kanban_view();

    let kanban = app.get_kanban_view_mut();

    // Reorder task3 to position 0
    kanban.start_drag(id3, Pos2::new(150.0, 300.0));
    kanban.complete_drag_with_reorder(0, 0);

    let tasks = kanban.get_tasks_for_column(0);
    assert_eq!(tasks[0].id, id3, "Third task should be first");
    assert_eq!(tasks[1].id, id1, "First task should be second");
    assert_eq!(tasks[2].id, id2, "Second task should be third");
}

/// Integration test that simulates real user workflow
#[test]
fn test_kanban_full_user_workflow() {
    let mut app = PlonApp::new_for_test();

    // User opens app and switches to Kanban view
    app.switch_to_kanban_view();

    // User creates tasks using quick add
    app.enable_kanban_quick_add(0);
    app.kanban_quick_add_task(0, "Write tests".to_string());
    app.kanban_quick_add_task(0, "Fix bugs".to_string());
    app.kanban_quick_add_task(0, "Review PR".to_string());

    // User searches for specific task
    let kanban = app.get_kanban_view_mut();
    kanban.set_search_filter("bug");
    let results = kanban.get_visible_tasks();
    assert_eq!(results.len(), 1);

    // Clear search
    kanban.set_search_filter("");

    // User drags task to In Progress
    let bug_task = app
        .get_tasks()
        .iter()
        .find(|t| t.title.contains("bug"))
        .unwrap();
    let bug_id = bug_task.id;

    app.start_kanban_drag(bug_id, Pos2::new(150.0, 200.0));
    app.update_kanban_drag(Pos2::new(450.0, 200.0));
    app.complete_kanban_drag(1);

    // User checks progress
    let in_progress = app.get_kanban_tasks_in_column(1);
    assert_eq!(in_progress.len(), 1);
    assert_eq!(in_progress[0].title, "Fix bugs");

    // User completes task
    app.start_kanban_drag(bug_id, Pos2::new(450.0, 200.0));
    app.complete_kanban_drag(3); // Done column

    let done = app.get_kanban_tasks_in_column(3);
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].title, "Fix bugs");
}
