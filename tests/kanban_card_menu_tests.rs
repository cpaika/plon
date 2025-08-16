use plon::domain::task::{Task, TaskStatus, Priority};
use plon::ui::views::kanban_view::{KanbanView, CardAction, ContextMenu};
use eframe::egui::{Pos2, Vec2};
use uuid::Uuid;

#[cfg(test)]
mod card_menu_tests {
    use super::*;

    fn create_test_kanban() -> KanbanView {
        let mut kanban = KanbanView::new();
        
        // Add test tasks
        for i in 0..5 {
            let mut task = Task::new_simple(format!("Task {}", i));
            task.id = Uuid::new_v4();
            task.status = match i {
                0..=1 => TaskStatus::Todo,
                2..=3 => TaskStatus::InProgress,
                _ => TaskStatus::Done,
            };
            kanban.tasks.push(task);
        }
        
        kanban
    }

    #[test]
    fn test_context_menu_activation() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let menu_pos = Pos2::new(100.0, 100.0);
        
        // Right-click to open context menu
        kanban.open_context_menu(task_id, menu_pos);
        
        assert!(kanban.is_context_menu_open());
        assert_eq!(kanban.context_menu_task_id, Some(task_id));
        assert_eq!(kanban.context_menu_position, menu_pos);
    }

    #[test]
    fn test_copy_task_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let original_count = kanban.tasks.len();
        
        // Execute copy action
        kanban.execute_card_action(task_id, CardAction::Copy);
        
        // Should have one more task
        assert_eq!(kanban.tasks.len(), original_count + 1);
        
        // Find copied task
        let copied_task = kanban.tasks.last().unwrap();
        assert_ne!(copied_task.id, task_id);
        assert!(copied_task.title.contains("(Copy)"));
        
        // Should be in same status as original
        let original_task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(copied_task.status, original_task.status);
    }

    #[test]
    fn test_duplicate_task_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let original_task = kanban.tasks[0].clone();
        
        // Execute duplicate action
        kanban.execute_card_action(task_id, CardAction::Duplicate);
        
        // Find duplicated task
        let duplicated_task = kanban.tasks.iter()
            .find(|t| t.id != task_id && t.title == original_task.title)
            .unwrap();
        
        // Should have same properties except ID
        assert_eq!(duplicated_task.status, original_task.status);
        assert_eq!(duplicated_task.priority, original_task.priority);
        assert_eq!(duplicated_task.description, original_task.description);
    }

    #[test]
    fn test_move_to_column_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Move from Todo to Done
        kanban.execute_card_action(task_id, CardAction::MoveTo(TaskStatus::Done));
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.status, TaskStatus::Done);
    }

    #[test]
    fn test_archive_task_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Archive task
        kanban.execute_card_action(task_id, CardAction::Archive);
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(task.is_archived);
        
        // Should not be visible by default
        assert!(!kanban.is_task_visible(task_id));
        
        // Should be visible when showing archived
        kanban.show_archived = true;
        assert!(kanban.is_task_visible(task_id));
    }

    #[test]
    fn test_delete_task_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let original_count = kanban.tasks.len();
        
        // Delete task (with confirmation)
        kanban.execute_card_action(task_id, CardAction::Delete);
        
        // Should show confirmation dialog
        assert!(kanban.is_showing_delete_confirmation());
        assert_eq!(kanban.delete_confirmation_task_id, Some(task_id));
        
        // Confirm deletion
        kanban.confirm_delete();
        
        // Task should be removed
        assert_eq!(kanban.tasks.len(), original_count - 1);
        assert!(kanban.tasks.iter().find(|t| t.id == task_id).is_none());
    }

    #[test]
    fn test_convert_to_subtask_action() {
        let mut kanban = create_test_kanban();
        let parent_id = kanban.tasks[0].id;
        let child_id = kanban.tasks[1].id;
        
        // Convert second task to subtask of first
        kanban.select_parent_task(parent_id);
        kanban.execute_card_action(child_id, CardAction::ConvertToSubtask);
        
        // Child should be removed from main tasks
        assert!(kanban.tasks.iter().find(|t| t.id == child_id).is_none());
        
        // Parent should have new subtask
        let parent = kanban.tasks.iter().find(|t| t.id == parent_id).unwrap();
        assert!(!parent.subtasks.is_empty());
        assert!(parent.subtasks.iter().any(|st| st.title.contains("Task 1")));
    }

    #[test]
    fn test_assign_to_user_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Assign to user
        kanban.execute_card_action(task_id, CardAction::AssignTo("Alice".to_string()));
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.assignee, Some("Alice".to_string()));
    }

    #[test]
    fn test_set_priority_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Set priority to Critical
        kanban.execute_card_action(task_id, CardAction::SetPriority(Priority::Critical));
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.priority, Priority::Critical);
    }

    #[test]
    fn test_add_tag_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Add tags
        kanban.execute_card_action(task_id, CardAction::AddTag("urgent".to_string()));
        kanban.execute_card_action(task_id, CardAction::AddTag("bug".to_string()));
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(task.tags.contains(&"urgent".to_string()));
        assert!(task.tags.contains(&"bug".to_string()));
    }

    #[test]
    fn test_remove_tag_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Add then remove tag
        kanban.execute_card_action(task_id, CardAction::AddTag("test".to_string()));
        kanban.execute_card_action(task_id, CardAction::RemoveTag("test".to_string()));
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(!task.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_set_due_date_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let due_date = chrono::Utc::now() + chrono::Duration::days(7);
        
        // Set due date
        kanban.execute_card_action(task_id, CardAction::SetDueDate(due_date));
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(task.due_date.is_some());
    }

    #[test]
    fn test_quick_actions_menu() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Show quick actions (without right-click)
        kanban.show_quick_actions(task_id);
        
        assert!(kanban.is_quick_actions_visible(task_id));
        
        // Get available quick actions
        let actions = kanban.get_quick_actions(task_id);
        assert!(actions.contains(&CardAction::Edit));
        assert!(actions.contains(&CardAction::Archive));
        assert!(actions.contains(&CardAction::Delete));
    }

    #[test]
    fn test_bulk_actions() {
        let mut kanban = create_test_kanban();
        
        // Select multiple tasks
        kanban.selected_tasks.insert(kanban.tasks[0].id);
        kanban.selected_tasks.insert(kanban.tasks[1].id);
        kanban.selected_tasks.insert(kanban.tasks[2].id);
        
        // Execute bulk action
        kanban.execute_bulk_action(CardAction::MoveTo(TaskStatus::Review));
        
        // All selected tasks should be moved
        for task_id in &kanban.selected_tasks {
            let task = kanban.tasks.iter().find(|t| &t.id == task_id).unwrap();
            assert_eq!(task.status, TaskStatus::Review);
        }
    }

    #[test]
    fn test_context_menu_keyboard_navigation() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        kanban.open_context_menu(task_id, Pos2::new(100.0, 100.0));
        
        // Navigate with arrow keys
        assert_eq!(kanban.context_menu_selected_index, 0);
        
        kanban.handle_context_menu_key(egui::Key::ArrowDown);
        assert_eq!(kanban.context_menu_selected_index, 1);
        
        kanban.handle_context_menu_key(egui::Key::ArrowUp);
        assert_eq!(kanban.context_menu_selected_index, 0);
        
        // Execute with Enter
        kanban.handle_context_menu_key(egui::Key::Enter);
        assert!(!kanban.is_context_menu_open());
    }

    #[test]
    fn test_context_menu_close_on_escape() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        kanban.open_context_menu(task_id, Pos2::new(100.0, 100.0));
        assert!(kanban.is_context_menu_open());
        
        kanban.handle_context_menu_key(egui::Key::Escape);
        assert!(!kanban.is_context_menu_open());
    }

    #[test]
    fn test_action_history() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Perform several actions
        kanban.execute_card_action(task_id, CardAction::SetPriority(Priority::High));
        kanban.execute_card_action(task_id, CardAction::AddTag("important".to_string()));
        kanban.execute_card_action(task_id, CardAction::MoveTo(TaskStatus::InProgress));
        
        // Check action history
        let history = kanban.get_action_history(task_id);
        assert_eq!(history.len(), 3);
        assert!(matches!(history[0], CardAction::SetPriority(_)));
        assert!(matches!(history[1], CardAction::AddTag(_)));
        assert!(matches!(history[2], CardAction::MoveTo(_)));
    }

    #[test]
    fn test_undo_card_action() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let original_status = kanban.tasks[0].status.clone();
        
        // Move task
        kanban.execute_card_action(task_id, CardAction::MoveTo(TaskStatus::Done));
        
        // Undo
        kanban.undo_last_action();
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.status, original_status);
    }

    #[test]
    fn test_custom_actions() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Register custom action
        kanban.register_custom_action(
            "Send to Slack",
            Box::new(|task| {
                // Custom action implementation
                task.tags.push("sent-to-slack".to_string());
            })
        );
        
        // Execute custom action
        kanban.execute_custom_action(task_id, "Send to Slack");
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(task.tags.contains(&"sent-to-slack".to_string()));
    }

    #[test]
    fn test_action_permissions() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Set task as locked
        kanban.lock_task(task_id);
        
        // Try to execute action on locked task
        let result = kanban.try_execute_action(task_id, CardAction::Delete);
        assert!(!result);
        
        // Unlock and try again
        kanban.unlock_task(task_id);
        let result = kanban.try_execute_action(task_id, CardAction::Delete);
        assert!(result);
    }

    #[test]
    fn test_action_shortcuts() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Select task
        kanban.select_task(task_id);
        
        // Test keyboard shortcuts
        kanban.handle_keyboard_shortcut(egui::Key::D, egui::Modifiers::CTRL); // Duplicate
        assert_eq!(kanban.tasks.len(), 6); // One more task
        
        kanban.handle_keyboard_shortcut(egui::Key::Delete, egui::Modifiers::NONE); // Delete
        assert!(kanban.is_showing_delete_confirmation());
        
        kanban.handle_keyboard_shortcut(egui::Key::A, egui::Modifiers::CTRL); // Archive
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(task.is_archived);
    }
}