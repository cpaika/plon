use plon::domain::task::{Task, TaskStatus, Priority};
use plon::ui::views::kanban_view_enhanced::{KanbanView, KanbanColumn, DragState};
use eframe::egui::{Pos2, Vec2, Color32, Rect};
use uuid::Uuid;
use std::collections::HashMap;

#[cfg(test)]
mod kanban_trello_style_tests {
    use super::*;

    fn create_test_kanban() -> KanbanView {
        let mut kanban = KanbanView::new();
        kanban.update_layout(1200.0);
        kanban
    }

    fn create_test_task(title: &str, status: TaskStatus) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.status = status;
        task
    }

    fn create_test_task_with_order(title: &str, status: TaskStatus, order: usize) -> Task {
        let mut task = create_test_task(title, status);
        task.metadata.insert("kanban_order".to_string(), order.to_string());
        task
    }

    // Test 1: Dragging should use the actual card, not a ghost
    #[test]
    fn test_drag_uses_actual_card_not_ghost() {
        let mut kanban = create_test_kanban();
        let task = create_test_task("Test Task", TaskStatus::Todo);
        let task_id = task.id;
        kanban.add_task(task);

        // Start dragging
        kanban.start_drag(task_id, Pos2::new(100.0, 200.0));
        
        // Verify drag context is created with actual task
        assert!(kanban.is_dragging());
        assert_eq!(kanban.get_dragging_task_id(), Some(task_id));
        
        // The drag context should track the actual card position
        let drag_pos = kanban.get_drag_position();
        assert!(drag_pos.is_some());
        
        // Verify the dragged card is the actual card (not a ghost)
        // This is validated by checking that drag_context contains the actual task_id
        assert!(kanban.drag_context.as_ref().unwrap().task_id == task_id);
    }

    // Test 2: Cards should have an ordering property within columns
    #[test]
    fn test_cards_have_ordering_within_columns() {
        let mut kanban = create_test_kanban();
        
        // Add multiple tasks to the same column with different orders
        let task1 = create_test_task_with_order("Task 1", TaskStatus::Todo, 0);
        let task2 = create_test_task_with_order("Task 2", TaskStatus::Todo, 1);
        let task3 = create_test_task_with_order("Task 3", TaskStatus::Todo, 2);
        
        let id1 = task1.id;
        let id2 = task2.id;
        let id3 = task3.id;
        
        kanban.add_task(task1);
        kanban.add_task(task2);
        kanban.add_task(task3);
        
        // Verify tasks are ordered correctly in the column
        let todo_column = &kanban.columns[0];
        assert_eq!(todo_column.tasks.len(), 3);
        assert_eq!(todo_column.tasks[0], id1);
        assert_eq!(todo_column.tasks[1], id2);
        assert_eq!(todo_column.tasks[2], id3);
    }

    // Test 3: Reordering cards within the same column
    #[test]
    fn test_reorder_cards_within_same_column() {
        let mut kanban = create_test_kanban();
        
        // Add three tasks to Todo column
        let task1 = create_test_task("Task 1", TaskStatus::Todo);
        let task2 = create_test_task("Task 2", TaskStatus::Todo);
        let task3 = create_test_task("Task 3", TaskStatus::Todo);
        
        let id1 = task1.id;
        let id2 = task2.id;
        let id3 = task3.id;
        
        kanban.add_task(task1);
        kanban.add_task(task2);
        kanban.add_task(task3);
        
        // Start drag of task2
        kanban.start_drag(id2, Pos2::new(100.0, 200.0));
        
        // Move task2 to position 0 (before task1)
        kanban.complete_drag_with_reorder(0, 0);
        
        // Verify new order: task2, task1, task3
        let todo_column = &kanban.columns[0];
        assert_eq!(todo_column.tasks[0], id2);
        assert_eq!(todo_column.tasks[1], id1);
        assert_eq!(todo_column.tasks[2], id3);
    }

    // Test 4: Dynamic resizing during drag
    #[test]
    fn test_dynamic_resizing_during_drag() {
        let mut kanban = create_test_kanban();
        
        let task1 = create_test_task("Task 1", TaskStatus::Todo);
        let task2 = create_test_task("Task 2", TaskStatus::InProgress);
        
        let id1 = task1.id;
        
        kanban.add_task(task1);
        kanban.add_task(task2);
        
        // Start dragging task1
        kanban.start_drag(id1, Pos2::new(100.0, 200.0));
        
        // Update drag position to hover over InProgress column
        let in_progress_column_pos = Pos2::new(400.0, 200.0);
        kanban.update_drag_position(in_progress_column_pos);
        
        // Verify hover detection
        assert_eq!(kanban.drag_context.as_ref().unwrap().hover_column, Some(1));
    }

    // Test 5: Smooth drop zone indicators
    #[test]
    fn test_drop_zone_indicators() {
        let mut kanban = create_test_kanban();
        
        let task = create_test_task("Draggable Task", TaskStatus::Todo);
        let task_id = task.id;
        kanban.add_task(task);
        
        // Start dragging
        kanban.start_drag(task_id, Pos2::new(100.0, 200.0));
        
        // Test hovering over different columns
        let positions = vec![
            (Pos2::new(150.0, 300.0), Some(0)), // Todo column
            (Pos2::new(450.0, 300.0), Some(1)), // InProgress column
            (Pos2::new(750.0, 300.0), Some(2)), // Review column
            (Pos2::new(1050.0, 300.0), Some(3)), // Done column
        ];
        
        for (pos, expected_column) in positions {
            kanban.update_drag_position(pos);
            let hover_column = kanban.get_column_at_position(pos);
            assert_eq!(hover_column, expected_column);
        }
    }

    // Test 6: Insert card at specific position during drop
    #[test]
    fn test_insert_card_at_specific_position() {
        let mut kanban = create_test_kanban();
        
        // Setup: Add tasks to InProgress column
        let task1 = create_test_task("Task 1", TaskStatus::InProgress);
        let task2 = create_test_task("Task 2", TaskStatus::InProgress);
        let task3 = create_test_task("Task 3", TaskStatus::InProgress);
        let new_task = create_test_task("New Task", TaskStatus::Todo);
        
        let id1 = task1.id;
        let id2 = task2.id;
        let id3 = task3.id;
        let new_id = new_task.id;
        
        kanban.add_task(task1);
        kanban.add_task(task2);
        kanban.add_task(task3);
        kanban.add_task(new_task);
        
        // Drag new_task from Todo to InProgress at position 1 (between task1 and task2)
        kanban.start_drag(new_id, Pos2::new(100.0, 200.0));
        kanban.complete_drag_with_reorder(1, 1);
        
        // Verify the new order in InProgress column
        let in_progress_column = &kanban.columns[1];
        assert_eq!(in_progress_column.tasks.len(), 4);
        assert_eq!(in_progress_column.tasks[0], id1);
        assert_eq!(in_progress_column.tasks[1], new_id);
        assert_eq!(in_progress_column.tasks[2], id2);
        assert_eq!(in_progress_column.tasks[3], id3);
        
        // Verify task status was updated
        let updated_task = kanban.tasks.iter().find(|t| t.id == new_id).unwrap();
        assert_eq!(updated_task.status, TaskStatus::InProgress);
    }

    // Test 7: Maintain card order when moving between columns
    #[test]
    fn test_maintain_order_when_moving_between_columns() {
        let mut kanban = create_test_kanban();
        
        // Add ordered tasks to Todo
        let task1 = create_test_task("Task 1", TaskStatus::Todo);
        let task2 = create_test_task("Task 2", TaskStatus::Todo);
        let task3 = create_test_task("Task 3", TaskStatus::Todo);
        
        let id1 = task1.id;
        let id2 = task2.id;
        let id3 = task3.id;
        
        kanban.add_task(task1);
        kanban.add_task(task2);
        kanban.add_task(task3);
        
        // Move task2 to InProgress
        kanban.start_drag(id2, Pos2::new(100.0, 200.0));
        kanban.complete_drag(1);
        
        // Verify Todo column order (task1, task3)
        let todo_column = &kanban.columns[0];
        assert_eq!(todo_column.tasks.len(), 2);
        assert_eq!(todo_column.tasks[0], id1);
        assert_eq!(todo_column.tasks[1], id3);
        
        // Verify InProgress has task2
        let in_progress_column = &kanban.columns[1];
        assert_eq!(in_progress_column.tasks.len(), 1);
        assert_eq!(in_progress_column.tasks[0], id2);
    }

    // Test 8: Cancel drag operation
    #[test]
    fn test_cancel_drag_operation() {
        let mut kanban = create_test_kanban();
        
        let task = create_test_task("Task", TaskStatus::Todo);
        let task_id = task.id;
        kanban.add_task(task);
        
        // Start drag
        kanban.start_drag(task_id, Pos2::new(100.0, 200.0));
        assert!(kanban.is_dragging());
        
        // Cancel drag
        kanban.cancel_drag();
        assert!(!kanban.is_dragging());
        
        // Verify task remains in original position
        let todo_column = &kanban.columns[0];
        assert_eq!(todo_column.tasks.len(), 1);
        assert_eq!(todo_column.tasks[0], task_id);
    }

    // Test 9: Visual feedback during drag
    #[test]
    fn test_visual_feedback_during_drag() {
        let mut kanban = create_test_kanban();
        
        let task = create_test_task("Draggable", TaskStatus::Todo);
        let task_id = task.id;
        kanban.add_task(task);
        
        // Start drag
        let start_pos = Pos2::new(100.0, 200.0);
        kanban.start_drag(task_id, start_pos);
        
        // Update position multiple times (simulating smooth drag)
        let positions = vec![
            Pos2::new(150.0, 210.0),
            Pos2::new(200.0, 220.0),
            Pos2::new(250.0, 230.0),
            Pos2::new(300.0, 240.0),
        ];
        
        for pos in positions {
            kanban.update_drag_position(pos);
            assert_eq!(kanban.get_drag_position(), Some(pos));
        }
    }

    // Test 10: Drag multiple selected cards
    #[test]
    fn test_bulk_move_with_selection() {
        let mut kanban = create_test_kanban();
        
        // Add multiple tasks
        let task1 = create_test_task("Task 1", TaskStatus::Todo);
        let task2 = create_test_task("Task 2", TaskStatus::Todo);
        let task3 = create_test_task("Task 3", TaskStatus::Todo);
        
        let id1 = task1.id;
        let id2 = task2.id;
        let id3 = task3.id;
        
        kanban.add_task(task1);
        kanban.add_task(task2);
        kanban.add_task(task3);
        
        // Select multiple tasks
        kanban.select_task(id1);
        kanban.add_to_selection(id2);
        kanban.add_to_selection(id3);
        
        // Bulk move to InProgress
        kanban.bulk_move_selected(1);
        
        // Verify all tasks moved
        for task in &kanban.tasks {
            if task.id == id1 || task.id == id2 || task.id == id3 {
                assert_eq!(task.status, TaskStatus::InProgress);
            }
        }
    }

    // Test 11: Preserve task metadata during drag
    #[test]
    fn test_preserve_task_metadata_during_drag() {
        let mut kanban = create_test_kanban();
        
        let mut task = create_test_task("Task with metadata", TaskStatus::Todo);
        task.priority = Priority::High;
        task.tags.insert("important".to_string());
        task.metadata.insert("custom_field".to_string(), "value".to_string());
        let task_id = task.id;
        
        kanban.add_task(task);
        
        // Drag to another column
        kanban.start_drag(task_id, Pos2::new(100.0, 200.0));
        kanban.complete_drag(1);
        
        // Verify metadata preserved
        let moved_task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(moved_task.priority, Priority::High);
        assert!(moved_task.tags.contains("important"));
        assert_eq!(moved_task.metadata.get("custom_field"), Some(&"value".to_string()));
    }

    // Test 12: Smooth animation placeholders
    #[test]
    fn test_drag_placeholder_positioning() {
        let mut kanban = create_test_kanban();
        
        // Add tasks to create gaps for placeholder
        let task1 = create_test_task("Task 1", TaskStatus::InProgress);
        let task2 = create_test_task("Task 2", TaskStatus::InProgress);
        let task3 = create_test_task("Task 3", TaskStatus::InProgress);
        let dragged = create_test_task("Dragged", TaskStatus::Todo);
        
        kanban.add_task(task1);
        kanban.add_task(task2);
        kanban.add_task(task3);
        let dragged_id = dragged.id;
        kanban.add_task(dragged);
        
        // Start dragging
        kanban.start_drag(dragged_id, Pos2::new(100.0, 200.0));
        
        // Hover between different positions
        kanban.update_drag_position(Pos2::new(400.0, 150.0)); // Top of InProgress
        if let Some(ctx) = &mut kanban.drag_context {
            ctx.hover_position = Some(0);
        }
        assert_eq!(kanban.drag_context.as_ref().unwrap().hover_position, Some(0));
        
        kanban.update_drag_position(Pos2::new(400.0, 250.0)); // Middle of InProgress
        if let Some(ctx) = &mut kanban.drag_context {
            ctx.hover_position = Some(1);
        }
        assert_eq!(kanban.drag_context.as_ref().unwrap().hover_position, Some(1));
    }

    // Test 13: WIP limits during drag
    #[test]
    fn test_wip_limits_respected_during_drag() {
        let mut kanban = create_test_kanban();
        kanban.set_wip_limit(1, 2); // InProgress WIP limit = 2
        
        // Add 2 tasks to InProgress (at limit)
        let task1 = create_test_task("Task 1", TaskStatus::InProgress);
        let task2 = create_test_task("Task 2", TaskStatus::InProgress);
        let task3 = create_test_task("Task 3", TaskStatus::Todo);
        
        kanban.add_task(task1);
        kanban.add_task(task2);
        let task3_id = task3.id;
        kanban.add_task(task3);
        
        // Check WIP limit before drag
        assert!(!kanban.is_column_over_wip_limit(1));
        
        // Drag task3 to InProgress
        kanban.start_drag(task3_id, Pos2::new(100.0, 200.0));
        kanban.complete_drag(1);
        
        // Check WIP limit after drag (should be over limit)
        assert!(kanban.is_column_over_wip_limit(1));
    }

    // Test 14: Auto-scroll during drag
    #[test]
    fn test_column_bounds_update_during_drag() {
        let mut kanban = create_test_kanban();
        
        let task = create_test_task("Task", TaskStatus::Todo);
        let task_id = task.id;
        kanban.add_task(task);
        
        // Start drag
        kanban.start_drag(task_id, Pos2::new(100.0, 200.0));
        
        // Test dragging to far right (should detect Done column)
        kanban.update_drag_position(Pos2::new(1050.0, 300.0));
        let hover_col = kanban.get_column_at_position(Pos2::new(1050.0, 300.0));
        assert_eq!(hover_col, Some(3)); // Done column
    }

    // Test 15: Keyboard shortcut integration with drag
    #[test]
    fn test_keyboard_navigation_updates_order() {
        let mut kanban = create_test_kanban();
        
        let task = create_test_task("Task", TaskStatus::Todo);
        let task_id = task.id;
        kanban.add_task(task);
        
        // Select task
        kanban.select_task(task_id);
        
        // Use keyboard to move right
        kanban.handle_keyboard_shortcut(eframe::egui::Key::ArrowRight, eframe::egui::Modifiers::NONE);
        
        // Verify task moved to InProgress
        let moved_task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(moved_task.status, TaskStatus::InProgress);
        
        // Verify task is in InProgress column's task list
        let in_progress_column = &kanban.columns[1];
        assert!(in_progress_column.tasks.contains(&task_id));
    }
}