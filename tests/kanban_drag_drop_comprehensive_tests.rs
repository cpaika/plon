use plon::domain::task::{Priority, Task, TaskStatus};
use plon::ui::views::kanban_view::{DragContext, KanbanColumn, KanbanView};
// use plon::ui::views::kanban_view_extensions::KanbanViewExtensions;
use eframe::egui::{Context, Event, Key, Modifiers, MouseButton, Pos2, Response, Ui};
use std::collections::HashSet;
use uuid::Uuid;

#[cfg(test)]
mod drag_drop_tests {
    use super::*;

    fn create_test_kanban() -> KanbanView {
        let mut kanban = KanbanView::new();
        
        // Create test tasks
        for i in 0..5 {
            let mut task = Task::new_simple(format!("Task {}", i));
            task.id = Uuid::new_v4();
            task.status = if i < 2 {
                TaskStatus::Todo
            } else if i < 4 {
                TaskStatus::InProgress
            } else {
                TaskStatus::Done
            };
            task.priority = match i % 3 {
                0 => Priority::High,
                1 => Priority::Medium,
                _ => Priority::Low,
            };
            kanban.tasks.push(task);
        }
        
        kanban
    }

    #[test]
    fn test_drag_initialization() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let start_pos = Pos2::new(100.0, 100.0);
        
        // Start drag
        kanban.start_drag(task_id, start_pos);
        
        assert!(kanban.drag_context.is_some());
        
        let drag_ctx = kanban.drag_context.as_ref().unwrap();
        assert_eq!(drag_ctx.task_id, task_id);
        assert_eq!(drag_ctx.start_position, start_pos);
        assert_eq!(drag_ctx.current_position, start_pos);
        assert!(!drag_ctx.is_multi_select);
        assert_eq!(drag_ctx.selected_tasks.len(), 1);
        assert!(drag_ctx.selected_tasks.contains(&task_id));
    }

    #[test]
    fn test_multi_select_drag() {
        let mut kanban = create_test_kanban();
        let task1_id = kanban.tasks[0].id;
        let task2_id = kanban.tasks[1].id;
        let task3_id = kanban.tasks[2].id;
        
        // Select multiple tasks
        kanban.selected_tasks.insert(task1_id);
        kanban.selected_tasks.insert(task2_id);
        kanban.selected_tasks.insert(task3_id);
        
        // Start multi-select drag
        kanban.start_multi_drag(task1_id, Pos2::new(100.0, 100.0));
        
        assert!(kanban.drag_context.is_some());
        
        let drag_ctx = kanban.drag_context.as_ref().unwrap();
        assert!(drag_ctx.is_multi_select);
        assert_eq!(drag_ctx.selected_tasks.len(), 3);
        assert!(drag_ctx.selected_tasks.contains(&task1_id));
        assert!(drag_ctx.selected_tasks.contains(&task2_id));
        assert!(drag_ctx.selected_tasks.contains(&task3_id));
    }

    #[test]
    fn test_drag_update_position() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let start_pos = Pos2::new(100.0, 100.0);
        let new_pos = Pos2::new(200.0, 150.0);
        
        kanban.start_drag(task_id, start_pos);
        kanban.update_drag_position(new_pos);
        
        let drag_ctx = kanban.drag_context.as_ref().unwrap();
        assert_eq!(drag_ctx.current_position, new_pos);
        assert_eq!(drag_ctx.offset, new_pos - start_pos);
    }

    #[test]
    fn test_drop_zone_detection() {
        let mut kanban = create_test_kanban();
        
        // Define column bounds
        kanban.columns[0].bounds = egui::Rect::from_min_size(
            Pos2::new(0.0, 0.0),
            egui::Vec2::new(300.0, 600.0)
        );
        kanban.columns[1].bounds = egui::Rect::from_min_size(
            Pos2::new(300.0, 0.0),
            egui::Vec2::new(300.0, 600.0)
        );
        
        // Test position in first column
        let target_col = kanban.get_column_at_position(Pos2::new(150.0, 300.0));
        assert!(target_col.is_some());
        assert_eq!(target_col.unwrap(), 0);
        
        // Test position in second column
        let target_col = kanban.get_column_at_position(Pos2::new(450.0, 300.0));
        assert!(target_col.is_some());
        assert_eq!(target_col.unwrap(), 1);
        
        // Test position outside columns
        let target_col = kanban.get_column_at_position(Pos2::new(700.0, 300.0));
        assert!(target_col.is_none());
    }

    #[test]
    fn test_drop_task_to_new_column() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Verify initial status
        assert_eq!(kanban.tasks[0].status, TaskStatus::Todo);
        
        // Simulate drag and drop to InProgress column
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        kanban.drop_task_at_column(1); // Drop at InProgress column
        
        // Verify task status changed
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(kanban.drag_context.is_none());
    }

    #[test]
    fn test_drop_with_position_sorting() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Drop task at specific position in column
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        kanban.drop_task_at_position(1, 2); // Drop at position 2 in InProgress
        
        // Verify task is at correct position
        let in_progress_tasks: Vec<_> = kanban.tasks.iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .collect();
        
        assert_eq!(in_progress_tasks[2].id, task_id);
    }

    #[test]
    fn test_auto_scroll_trigger() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        kanban.view_bounds = egui::Rect::from_min_size(
            Pos2::new(0.0, 0.0),
            egui::Vec2::new(1200.0, 800.0)
        );
        
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        
        // Test near top edge (should trigger upward scroll)
        kanban.update_drag_position(Pos2::new(100.0, 20.0));
        assert!(kanban.should_auto_scroll());
        assert_eq!(kanban.get_scroll_direction(), egui::Vec2::new(0.0, -1.0));
        
        // Test near bottom edge (should trigger downward scroll)
        kanban.update_drag_position(Pos2::new(100.0, 780.0));
        assert!(kanban.should_auto_scroll());
        assert_eq!(kanban.get_scroll_direction(), egui::Vec2::new(0.0, 1.0));
        
        // Test near left edge (should trigger leftward scroll)
        kanban.update_drag_position(Pos2::new(20.0, 400.0));
        assert!(kanban.should_auto_scroll());
        assert_eq!(kanban.get_scroll_direction(), egui::Vec2::new(-1.0, 0.0));
        
        // Test near right edge (should trigger rightward scroll)
        kanban.update_drag_position(Pos2::new(1180.0, 400.0));
        assert!(kanban.should_auto_scroll());
        assert_eq!(kanban.get_scroll_direction(), egui::Vec2::new(1.0, 0.0));
        
        // Test center (should not trigger scroll)
        kanban.update_drag_position(Pos2::new(600.0, 400.0));
        assert!(!kanban.should_auto_scroll());
    }

    #[test]
    fn test_drag_preview_rendering() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        
        // Verify drag preview properties
        assert!(kanban.should_show_drag_preview());
        assert_eq!(kanban.get_drag_preview_opacity(), 0.7);
        assert!(kanban.get_drag_preview_bounds().is_some());
    }

    #[test]
    fn test_escape_key_cancels_drag() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let original_status = kanban.tasks[0].status.clone();
        
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        kanban.update_drag_position(Pos2::new(400.0, 100.0));
        
        // Simulate escape key press
        kanban.handle_escape_key();
        
        // Verify drag cancelled and task unchanged
        assert!(kanban.drag_context.is_none());
        assert_eq!(kanban.tasks[0].status, original_status);
    }

    #[test]
    fn test_drop_indicator_visibility() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Set up column bounds
        for (i, column) in kanban.columns.iter_mut().enumerate() {
            column.bounds = egui::Rect::from_min_size(
                Pos2::new(i as f32 * 300.0, 0.0),
                egui::Vec2::new(300.0, 600.0)
            );
        }
        
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        
        // Move to valid drop zone
        kanban.update_drag_position(Pos2::new(450.0, 300.0));
        assert!(kanban.should_show_drop_indicator());
        assert_eq!(kanban.get_drop_indicator_column(), Some(1));
        
        // Move to invalid drop zone
        kanban.update_drag_position(Pos2::new(1000.0, 300.0));
        assert!(!kanban.should_show_drop_indicator());
    }

    #[test]
    fn test_multi_task_drop() {
        let mut kanban = create_test_kanban();
        let task1_id = kanban.tasks[0].id;
        let task2_id = kanban.tasks[1].id;
        
        // Select and drag multiple tasks
        kanban.selected_tasks.insert(task1_id);
        kanban.selected_tasks.insert(task2_id);
        kanban.start_multi_drag(task1_id, Pos2::new(100.0, 100.0));
        
        // Drop in new column
        kanban.drop_tasks_at_column(2); // Drop in Review column
        
        // Verify all selected tasks moved
        for task in &kanban.tasks {
            if task.id == task1_id || task.id == task2_id {
                assert_eq!(task.status, TaskStatus::Review);
            }
        }
        
        assert!(kanban.drag_context.is_none());
        assert!(kanban.selected_tasks.is_empty());
    }

    #[test]
    fn test_drag_constraints_with_wip_limits() {
        let mut kanban = create_test_kanban();
        
        // Set WIP limit for InProgress column
        kanban.columns[1].wip_limit = Some(2);
        
        // Fill InProgress to WIP limit
        for task in kanban.tasks.iter_mut().take(2) {
            task.status = TaskStatus::InProgress;
        }
        
        let task_id = kanban.tasks[2].id;
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        
        // Try to drop in InProgress column (should be rejected)
        let can_drop = kanban.can_drop_in_column(1);
        assert!(!can_drop);
        
        // Should show WIP limit warning
        assert!(kanban.should_show_wip_warning(1));
    }

    #[test]
    fn test_drag_animation_frames() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        
        // Simulate animation frames
        for frame in 0..10 {
            let t = frame as f32 / 10.0;
            let interpolated_pos = kanban.get_animated_drag_position(t);
            
            // Verify smooth interpolation
            assert!(interpolated_pos.x >= 100.0);
            assert!(interpolated_pos.y >= 100.0);
        }
    }

    #[test]
    fn test_touch_drag_support() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Simulate touch start
        kanban.handle_touch_start(task_id, Pos2::new(100.0, 100.0));
        assert!(kanban.drag_context.is_some());
        
        // Simulate touch move
        kanban.handle_touch_move(Pos2::new(200.0, 150.0));
        let drag_ctx = kanban.drag_context.as_ref().unwrap();
        assert_eq!(drag_ctx.current_position, Pos2::new(200.0, 150.0));
        
        // Simulate touch end
        kanban.handle_touch_end(Pos2::new(450.0, 300.0));
        assert!(kanban.drag_context.is_none());
    }

    #[test]
    fn test_drag_momentum() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        
        // Simulate quick drag motion
        let positions = vec![
            Pos2::new(100.0, 100.0),
            Pos2::new(120.0, 105.0),
            Pos2::new(150.0, 112.0),
            Pos2::new(200.0, 125.0),
        ];
        
        for pos in positions {
            kanban.update_drag_position(pos);
            kanban.record_drag_velocity();
        }
        
        // Check momentum is calculated
        let velocity = kanban.get_drag_velocity();
        assert!(velocity.x > 0.0);
        assert!(velocity.y > 0.0);
    }

    #[test]
    fn test_drop_between_cards() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[4].id; // Last task
        
        kanban.start_drag(task_id, Pos2::new(100.0, 500.0));
        
        // Drop between first and second task in Todo column
        kanban.drop_task_between(0, 0, 1);
        
        // Verify task is now between first and second
        let todo_tasks: Vec<_> = kanban.tasks.iter()
            .filter(|t| t.status == TaskStatus::Todo)
            .collect();
        
        assert_eq!(todo_tasks[1].id, task_id);
    }

    #[test]
    fn test_keyboard_navigation_during_drag() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        
        // Simulate arrow key navigation
        kanban.handle_key_press(Key::ArrowRight);
        assert_eq!(kanban.get_target_column(), Some(1));
        
        kanban.handle_key_press(Key::ArrowLeft);
        assert_eq!(kanban.get_target_column(), Some(0));
        
        kanban.handle_key_press(Key::ArrowDown);
        assert_eq!(kanban.get_target_position_in_column(), Some(1));
        
        // Enter to confirm drop
        kanban.handle_key_press(Key::Enter);
        assert!(kanban.drag_context.is_none());
    }

    #[test]
    fn test_accessibility_announcements() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        kanban.start_drag(task_id, Pos2::new(100.0, 100.0));
        
        // Check accessibility announcement for drag start
        let announcement = kanban.get_accessibility_announcement();
        assert!(announcement.contains("Dragging task"));
        
        // Move to new column
        kanban.update_drag_position(Pos2::new(450.0, 300.0));
        let announcement = kanban.get_accessibility_announcement();
        assert!(announcement.contains("Over column"));
        
        // Drop task
        kanban.drop_task_at_column(1);
        let announcement = kanban.get_accessibility_announcement();
        assert!(announcement.contains("Dropped task"));
    }
}