use chrono::Utc;
use plon::domain::task::{Task, TaskStatus};
use plon::ui::views::kanban_view_improved::{KanbanColumn, KanbanView};
use std::collections::HashMap;
use uuid::Uuid;

fn create_test_task(title: &str, status: TaskStatus) -> Task {
    let mut task = Task::new(title.to_string(), "Test description".to_string());
    task.status = status;
    task
}

#[cfg(test)]
mod kanban_drag_drop_tests {
    use super::*;

    #[test]
    fn test_kanban_initialization() {
        let kanban = KanbanView::new();

        // Should have default columns
        assert!(
            !kanban.columns.is_empty(),
            "Kanban should have default columns"
        );

        // Should have standard columns
        let column_titles: Vec<String> = kanban.columns.iter().map(|c| c.title.clone()).collect();

        assert!(
            column_titles.contains(&"To Do".to_string()),
            "Should have To Do column"
        );
        assert!(
            column_titles.contains(&"In Progress".to_string()),
            "Should have In Progress column"
        );
        assert!(
            column_titles.contains(&"Done".to_string()),
            "Should have Done column"
        );
    }

    #[test]
    fn test_drag_start() {
        let mut kanban = KanbanView::new();
        let task = create_test_task("Test Task", TaskStatus::Todo);
        kanban.add_task(task.clone());

        // Start dragging
        kanban.start_drag(task.id, egui::Pos2::new(100.0, 100.0));

        assert!(kanban.is_dragging(), "Should be in dragging state");
        assert_eq!(
            kanban.get_dragging_task_id(),
            Some(task.id),
            "Should track dragging task ID"
        );
    }

    #[test]
    fn test_drag_update() {
        let mut kanban = KanbanView::new();
        let task = create_test_task("Test Task", TaskStatus::Todo);
        kanban.add_task(task.clone());

        // Start and update drag
        kanban.start_drag(task.id, egui::Pos2::new(100.0, 100.0));
        kanban.update_drag_position(egui::Pos2::new(200.0, 150.0));

        let drag_pos = kanban.get_drag_position();
        assert!(drag_pos.is_some(), "Should have drag position");
        assert_eq!(drag_pos.unwrap(), egui::Pos2::new(200.0, 150.0));
    }

    #[test]
    fn test_drag_to_different_column() {
        let mut kanban = KanbanView::new();
        let mut task = create_test_task("Test Task", TaskStatus::Todo);
        kanban.add_task(task.clone());

        // Ensure we have the columns set up
        assert!(kanban.columns.len() >= 3, "Should have at least 3 columns");

        // Start drag from Todo column
        kanban.start_drag(task.id, egui::Pos2::new(100.0, 100.0));

        // Drag to In Progress column area
        kanban.update_drag_position(egui::Pos2::new(400.0, 100.0));

        // Complete the drag
        kanban.complete_drag(1); // Index 1 should be In Progress

        // Task should have moved to In Progress
        let updated_task = kanban
            .tasks
            .iter()
            .find(|t| t.id == task.id)
            .expect("Task should exist");

        assert_eq!(
            updated_task.status,
            TaskStatus::InProgress,
            "Task should be in In Progress status"
        );
        assert!(!kanban.is_dragging(), "Should no longer be dragging");
    }

    #[test]
    fn test_cancel_drag() {
        let mut kanban = KanbanView::new();
        let task = create_test_task("Test Task", TaskStatus::Todo);
        kanban.add_task(task.clone());

        kanban.start_drag(task.id, egui::Pos2::new(100.0, 100.0));
        assert!(kanban.is_dragging());

        kanban.cancel_drag();
        assert!(
            !kanban.is_dragging(),
            "Should no longer be dragging after cancel"
        );
    }

    #[test]
    fn test_drop_zones() {
        let kanban = KanbanView::new();

        // Test if position is in Todo column
        assert!(
            kanban.is_over_column(egui::Pos2::new(150.0, 200.0), 0),
            "Should be over Todo column"
        );

        // Test if position is in In Progress column
        assert!(
            kanban.is_over_column(egui::Pos2::new(450.0, 200.0), 1),
            "Should be over In Progress column"
        );

        // Test if position is outside any column
        assert!(
            !kanban.is_over_column(egui::Pos2::new(-50.0, 200.0), 0),
            "Should not be over any column"
        );
    }

    #[test]
    fn test_multiple_tasks_in_column() {
        let mut kanban = KanbanView::new();

        // Add multiple tasks to Todo
        for i in 0..5 {
            let task = create_test_task(&format!("Task {}", i), TaskStatus::Todo);
            kanban.add_task(task);
        }

        let todo_tasks = kanban.get_tasks_for_column(0);
        assert_eq!(todo_tasks.len(), 5, "Should have 5 tasks in Todo column");
    }

    #[test]
    fn test_drag_reordering_within_column() {
        let mut kanban = KanbanView::new();

        // Add tasks using the proper method
        let task1 = create_test_task("Task 1", TaskStatus::Todo);
        let task2 = create_test_task("Task 2", TaskStatus::Todo);
        let task3 = create_test_task("Task 3", TaskStatus::Todo);

        kanban.add_task(task1.clone());
        kanban.add_task(task2.clone());
        kanban.add_task(task3.clone());

        // Drag task3 to top of Todo column
        kanban.start_drag(task3.id, egui::Pos2::new(150.0, 300.0));
        kanban.update_drag_position(egui::Pos2::new(150.0, 50.0));
        kanban.complete_drag_with_reorder(0, 0); // Column 0, position 0

        let todo_tasks = kanban.get_tasks_for_column(0);
        assert_eq!(todo_tasks[0].id, task3.id, "Task 3 should be first");
    }
}

#[cfg(test)]
mod kanban_rendering_tests {
    use super::*;

    #[test]
    fn test_column_width_calculation() {
        let kanban = KanbanView::new();
        let available_width = 1200.0;

        let column_width = kanban.calculate_column_width(available_width);

        // With 3 default columns and some spacing
        assert!(column_width > 300.0, "Column width should be reasonable");
        assert!(column_width < 500.0, "Column width should not be too large");
    }

    #[test]
    fn test_card_height_calculation() {
        let kanban = KanbanView::new();
        let task = create_test_task(
            "Test task with a longer description that might wrap",
            TaskStatus::Todo,
        );

        let card_height = kanban.calculate_card_height(&task);

        assert!(card_height >= 60.0, "Card should have minimum height");
        assert!(card_height <= 200.0, "Card should not be too tall");
    }

    #[test]
    fn test_column_header_rendering() {
        let kanban = KanbanView::new();

        for column in &kanban.columns {
            assert!(!column.title.is_empty(), "Column should have title");
            assert!(
                column.color != egui::Color32::BLACK,
                "Column should have color"
            );
        }
    }

    #[test]
    fn test_task_count_in_column_header() {
        let mut kanban = KanbanView::new();

        // Add tasks to different columns
        kanban.add_task(create_test_task("Task 1", TaskStatus::Todo));
        kanban.add_task(create_test_task("Task 2", TaskStatus::Todo));
        kanban.add_task(create_test_task("Task 3", TaskStatus::InProgress));

        let todo_count = kanban.get_column_task_count(0);
        let progress_count = kanban.get_column_task_count(1);

        assert_eq!(todo_count, 2, "Todo should have 2 tasks");
        assert_eq!(progress_count, 1, "In Progress should have 1 task");
    }

    #[test]
    fn test_wip_limits() {
        let mut kanban = KanbanView::new();

        // Set WIP limit for In Progress
        kanban.set_wip_limit(1, 3); // Column 1, limit 3

        // Add 4 tasks to In Progress
        for i in 0..4 {
            kanban.add_task(create_test_task(
                &format!("Task {}", i),
                TaskStatus::InProgress,
            ));
        }

        assert!(
            kanban.is_column_over_wip_limit(1),
            "Column should be over WIP limit"
        );
    }

    #[test]
    fn test_empty_column_message() {
        let kanban = KanbanView::new();

        let todo_tasks = kanban.get_tasks_for_column(0);
        assert_eq!(todo_tasks.len(), 0, "Column should be empty initially");

        let empty_message = kanban.get_empty_column_message(0);
        assert!(
            !empty_message.is_empty(),
            "Should have empty column message"
        );
    }

    #[test]
    fn test_card_spacing() {
        let kanban = KanbanView::new();

        let spacing = kanban.get_card_spacing();
        assert!(spacing > 0.0, "Cards should have spacing");
        assert!(spacing < 20.0, "Spacing should not be too large");
    }

    #[test]
    fn test_responsive_columns() {
        let mut kanban = KanbanView::new();

        // Test with narrow screen
        kanban.update_layout(400.0);
        assert!(
            kanban.should_stack_columns(),
            "Should stack columns on narrow screen"
        );

        // Test with wide screen
        kanban.update_layout(1400.0);
        assert!(
            !kanban.should_stack_columns(),
            "Should not stack columns on wide screen"
        );
    }

    #[test]
    fn test_card_colors_by_priority() {
        let kanban = KanbanView::new();

        let mut high_priority_task = create_test_task("Urgent", TaskStatus::Todo);
        high_priority_task.priority = plon::domain::task::Priority::High;

        let mut low_priority_task = create_test_task("Can wait", TaskStatus::Todo);
        low_priority_task.priority = plon::domain::task::Priority::Low;

        let high_color = kanban.get_card_color(&high_priority_task);
        let low_color = kanban.get_card_color(&low_priority_task);

        assert_ne!(
            high_color, low_color,
            "Different priorities should have different colors"
        );
    }

    #[test]
    fn test_overdue_task_highlighting() {
        let kanban = KanbanView::new();

        let mut overdue_task = create_test_task("Overdue", TaskStatus::InProgress);
        overdue_task.due_date = Some(Utc::now() - chrono::Duration::days(1));

        assert!(
            kanban.should_highlight_as_overdue(&overdue_task),
            "Should highlight overdue task"
        );

        let mut future_task = create_test_task("Future", TaskStatus::Todo);
        future_task.due_date = Some(Utc::now() + chrono::Duration::days(7));

        assert!(
            !kanban.should_highlight_as_overdue(&future_task),
            "Should not highlight future task"
        );
    }
}

#[cfg(test)]
mod kanban_interaction_tests {
    use super::*;

    #[test]
    fn test_task_selection() {
        let mut kanban = KanbanView::new();
        let task = create_test_task("Test Task", TaskStatus::Todo);
        kanban.add_task(task.clone());

        kanban.select_task(task.id);
        assert_eq!(kanban.get_selected_task_id(), Some(task.id));

        kanban.clear_selection();
        assert_eq!(kanban.get_selected_task_id(), None);
    }

    #[test]
    fn test_quick_add_task() {
        let mut kanban = KanbanView::new();

        // Enable quick add for Todo column
        kanban.enable_quick_add(0);
        assert!(kanban.is_quick_add_active(0));

        // Add task via quick add
        kanban.quick_add_task(0, "New Task".to_string());

        let todo_tasks = kanban.get_tasks_for_column(0);
        assert_eq!(todo_tasks.len(), 1, "Should have added task");
        assert_eq!(todo_tasks[0].title, "New Task");
    }

    #[test]
    fn test_column_collapse() {
        let mut kanban = KanbanView::new();

        assert!(
            !kanban.is_column_collapsed(0),
            "Column should not be collapsed initially"
        );

        kanban.toggle_column_collapse(0);
        assert!(kanban.is_column_collapsed(0), "Column should be collapsed");

        kanban.toggle_column_collapse(0);
        assert!(
            !kanban.is_column_collapsed(0),
            "Column should be expanded again"
        );
    }

    #[test]
    fn test_bulk_move() {
        let mut kanban = KanbanView::new();

        // Add multiple tasks to Todo
        let task1 = create_test_task("Task 1", TaskStatus::Todo);
        let task2 = create_test_task("Task 2", TaskStatus::Todo);
        let task3 = create_test_task("Task 3", TaskStatus::Todo);

        kanban.add_task(task1.clone());
        kanban.add_task(task2.clone());
        kanban.add_task(task3.clone());

        // Select multiple tasks
        kanban.select_task(task1.id);
        kanban.add_to_selection(task2.id);

        // Move selected tasks to In Progress
        kanban.bulk_move_selected(1); // Column 1 = In Progress

        let in_progress_tasks = kanban.get_tasks_for_column(1);
        assert_eq!(in_progress_tasks.len(), 2, "Should have moved 2 tasks");
    }

    #[test]
    fn test_search_filter() {
        let mut kanban = KanbanView::new();

        kanban.add_task(create_test_task("Fix bug in login", TaskStatus::Todo));
        kanban.add_task(create_test_task("Add new feature", TaskStatus::Todo));
        kanban.add_task(create_test_task("Update documentation", TaskStatus::Todo));

        kanban.set_search_filter("bug");

        let visible_tasks = kanban.get_visible_tasks();
        assert_eq!(
            visible_tasks.len(),
            1,
            "Should only show tasks matching search"
        );
        assert!(visible_tasks[0].title.contains("bug"));
    }

    #[test]
    fn test_keyboard_shortcuts() {
        let mut kanban = KanbanView::new();
        let task = create_test_task("Test Task", TaskStatus::Todo);
        kanban.add_task(task.clone());

        kanban.select_task(task.id);

        // Test move right (to In Progress)
        kanban.handle_keyboard_shortcut(egui::Key::ArrowRight, egui::Modifiers::NONE);

        let updated_task = kanban.tasks.iter().find(|t| t.id == task.id).unwrap();

        assert_eq!(
            updated_task.status,
            TaskStatus::InProgress,
            "Task should move right"
        );
    }
}
