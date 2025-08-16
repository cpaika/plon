use plon::domain::task::{Task, TaskStatus, Priority};
use plon::ui::views::kanban_view::{KanbanView, EditMode};
use eframe::egui::{self, Context, Event, Key, Modifiers, TextEdit};
use uuid::Uuid;

#[cfg(test)]
mod inline_editing_tests {
    use super::*;

    fn create_test_kanban() -> KanbanView {
        let mut kanban = KanbanView::new();
        
        // Add test tasks
        for i in 0..3 {
            let mut task = Task::new(format!("Task {}", i), format!("Description for task {}", i));
            task.id = Uuid::new_v4();
            task.status = TaskStatus::Todo;
            kanban.tasks.push(task);
        }
        
        kanban
    }

    #[test]
    fn test_inline_edit_activation() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Double-click to activate inline edit
        kanban.handle_double_click(task_id);
        
        assert!(kanban.is_editing_task(task_id));
        assert_eq!(kanban.edit_mode, Some(EditMode::TaskTitle));
        assert_eq!(kanban.editing_task_id, Some(task_id));
    }

    #[test]
    fn test_inline_edit_text_buffer() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let original_title = kanban.tasks[0].title.clone();
        
        // Start editing
        kanban.start_inline_edit(task_id, EditMode::TaskTitle);
        
        // Check edit buffer initialized with current value
        assert_eq!(kanban.edit_buffer, original_title);
        
        // Modify buffer
        kanban.edit_buffer = "Updated Task Title".to_string();
        
        // Commit changes
        kanban.commit_inline_edit();
        
        // Verify task updated
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.title, "Updated Task Title");
    }

    #[test]
    fn test_inline_edit_cancel() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let original_title = kanban.tasks[0].title.clone();
        
        // Start editing
        kanban.start_inline_edit(task_id, EditMode::TaskTitle);
        
        // Modify buffer
        kanban.edit_buffer = "Changed Title".to_string();
        
        // Cancel edit (ESC key)
        kanban.cancel_inline_edit();
        
        // Verify task unchanged
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.title, original_title);
        assert!(kanban.editing_task_id.is_none());
    }

    #[test]
    fn test_inline_edit_description() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start editing description
        kanban.start_inline_edit(task_id, EditMode::TaskDescription);
        
        assert_eq!(kanban.edit_mode, Some(EditMode::TaskDescription));
        assert!(kanban.edit_buffer.contains("Description for task 0"));
        
        // Update description
        kanban.edit_buffer = "New detailed description".to_string();
        kanban.commit_inline_edit();
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.description, Some("New detailed description".to_string()));
    }

    #[test]
    fn test_column_title_inline_edit() {
        let mut kanban = create_test_kanban();
        let column_id = kanban.columns[0].id;
        
        // Start editing column title
        kanban.start_column_title_edit(column_id);
        
        assert!(kanban.is_editing_column(column_id));
        assert_eq!(kanban.edit_buffer, "To Do");
        
        // Change column title
        kanban.edit_buffer = "Backlog".to_string();
        kanban.commit_column_edit();
        
        let column = kanban.columns.iter().find(|c| c.id == column_id).unwrap();
        assert_eq!(column.title, "Backlog");
    }

    #[test]
    fn test_inline_edit_validation() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start editing
        kanban.start_inline_edit(task_id, EditMode::TaskTitle);
        
        // Try to set empty title
        kanban.edit_buffer = "".to_string();
        kanban.commit_inline_edit();
        
        // Should not allow empty title
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_ne!(task.title, "");
        
        // Should show validation error
        assert!(kanban.has_validation_error());
        assert_eq!(kanban.validation_error_message, "Title cannot be empty");
    }

    #[test]
    fn test_tab_key_navigation() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start editing title
        kanban.start_inline_edit(task_id, EditMode::TaskTitle);
        
        // Press Tab to move to description
        kanban.handle_tab_key();
        
        assert_eq!(kanban.edit_mode, Some(EditMode::TaskDescription));
        assert_eq!(kanban.editing_task_id, Some(task_id));
        
        // Press Tab again to move to next field
        kanban.handle_tab_key();
        assert_eq!(kanban.edit_mode, Some(EditMode::TaskTags));
    }

    #[test]
    fn test_inline_edit_auto_save() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Enable auto-save
        kanban.enable_auto_save = true;
        kanban.auto_save_delay_ms = 1000;
        
        // Start editing
        kanban.start_inline_edit(task_id, EditMode::TaskTitle);
        kanban.edit_buffer = "Auto-saved title".to_string();
        
        // Simulate time passing
        kanban.trigger_auto_save();
        
        // Verify saved without explicit commit
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.title, "Auto-saved title");
    }

    #[test]
    fn test_inline_edit_tag_addition() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start editing tags
        kanban.start_inline_edit(task_id, EditMode::TaskTags);
        
        // Add new tags
        kanban.add_tag_inline("urgent");
        kanban.add_tag_inline("bug");
        kanban.commit_inline_edit();
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(task.tags.contains(&"urgent".to_string()));
        assert!(task.tags.contains(&"bug".to_string()));
    }

    #[test]
    fn test_inline_edit_priority_change() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start editing priority
        kanban.start_inline_edit(task_id, EditMode::TaskPriority);
        
        // Change priority
        kanban.set_priority_inline(Priority::Critical);
        kanban.commit_inline_edit();
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.priority, Priority::Critical);
    }

    #[test]
    fn test_inline_edit_assignee() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start editing assignee
        kanban.start_inline_edit(task_id, EditMode::TaskAssignee);
        
        // Set assignee with autocomplete
        kanban.edit_buffer = "John".to_string();
        let suggestions = kanban.get_assignee_suggestions("John");
        assert!(!suggestions.is_empty());
        
        kanban.select_assignee_suggestion("John Doe");
        kanban.commit_inline_edit();
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.assignee, Some("John Doe".to_string()));
    }

    #[test]
    fn test_inline_edit_due_date() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start editing due date
        kanban.start_inline_edit(task_id, EditMode::TaskDueDate);
        
        // Set due date
        let due_date = chrono::Utc::now() + chrono::Duration::days(7);
        kanban.set_due_date_inline(due_date);
        kanban.commit_inline_edit();
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(task.due_date.is_some());
    }

    #[test]
    fn test_multi_line_edit() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start editing description (multi-line)
        kanban.start_inline_edit(task_id, EditMode::TaskDescription);
        
        // Add multi-line text
        kanban.edit_buffer = "Line 1\nLine 2\nLine 3".to_string();
        
        // Verify multi-line support
        assert!(kanban.is_multiline_edit_mode());
        assert_eq!(kanban.get_edit_lines_count(), 3);
        
        kanban.commit_inline_edit();
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(task.description.as_ref().unwrap().contains("\n"));
    }

    #[test]
    fn test_edit_mode_switching() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start with title edit
        kanban.start_inline_edit(task_id, EditMode::TaskTitle);
        kanban.edit_buffer = "Modified Title".to_string();
        
        // Switch to description without committing
        kanban.switch_edit_mode(EditMode::TaskDescription);
        
        // Previous changes should be auto-saved
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.title, "Modified Title");
        
        // Now editing description
        assert_eq!(kanban.edit_mode, Some(EditMode::TaskDescription));
    }

    #[test]
    fn test_inline_edit_keyboard_shortcuts() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // F2 to start editing
        kanban.handle_key_shortcut(Key::F2, Modifiers::NONE, task_id);
        assert!(kanban.is_editing_task(task_id));
        
        // Enter to commit
        kanban.edit_buffer = "Updated via shortcut".to_string();
        kanban.handle_key_shortcut(Key::Enter, Modifiers::NONE, task_id);
        
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.title, "Updated via shortcut");
        assert!(kanban.editing_task_id.is_none());
    }

    #[test]
    fn test_edit_focus_management() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Start editing
        kanban.start_inline_edit(task_id, EditMode::TaskTitle);
        
        // Focus should be on edit field
        assert!(kanban.has_edit_focus());
        
        // Click outside should commit
        kanban.handle_click_outside();
        
        assert!(!kanban.has_edit_focus());
        assert!(kanban.editing_task_id.is_none());
    }

    #[test]
    fn test_inline_edit_undo_redo() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        let original_title = kanban.tasks[0].title.clone();
        
        // Make edit
        kanban.start_inline_edit(task_id, EditMode::TaskTitle);
        kanban.edit_buffer = "First Edit".to_string();
        kanban.commit_inline_edit();
        
        // Make another edit
        kanban.start_inline_edit(task_id, EditMode::TaskTitle);
        kanban.edit_buffer = "Second Edit".to_string();
        kanban.commit_inline_edit();
        
        // Undo
        kanban.undo_last_edit();
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.title, "First Edit");
        
        // Undo again
        kanban.undo_last_edit();
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.title, original_title);
        
        // Redo
        kanban.redo_last_edit();
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert_eq!(task.title, "First Edit");
    }

    #[test]
    fn test_bulk_inline_edit() {
        let mut kanban = create_test_kanban();
        
        // Select multiple tasks
        kanban.selected_tasks.insert(kanban.tasks[0].id);
        kanban.selected_tasks.insert(kanban.tasks[1].id);
        
        // Start bulk edit
        kanban.start_bulk_inline_edit(EditMode::TaskPriority);
        
        // Set priority for all selected
        kanban.set_bulk_priority(Priority::High);
        kanban.commit_bulk_edit();
        
        // Verify all selected tasks updated
        for task_id in &kanban.selected_tasks {
            let task = kanban.tasks.iter().find(|t| &t.id == task_id).unwrap();
            assert_eq!(task.priority, Priority::High);
        }
    }

    #[test]
    fn test_inline_edit_with_markdown() {
        let mut kanban = create_test_kanban();
        let task_id = kanban.tasks[0].id;
        
        // Edit description with markdown
        kanban.start_inline_edit(task_id, EditMode::TaskDescription);
        kanban.edit_buffer = "# Header\n**Bold** and *italic* text".to_string();
        kanban.commit_inline_edit();
        
        // Verify markdown preserved
        let task = kanban.tasks.iter().find(|t| t.id == task_id).unwrap();
        assert!(task.description.as_ref().unwrap().contains("# Header"));
        assert!(task.description.as_ref().unwrap().contains("**Bold**"));
    }

    #[test]
    fn test_quick_add_inline() {
        let mut kanban = create_test_kanban();
        
        // Activate quick add in a column
        kanban.start_quick_add(0); // Add to first column
        
        assert!(kanban.is_quick_adding());
        assert_eq!(kanban.quick_add_column, Some(0));
        
        // Type new task title
        kanban.quick_add_buffer = "Quick Task".to_string();
        kanban.commit_quick_add();
        
        // Verify task added
        let new_task = kanban.tasks.iter()
            .find(|t| t.title == "Quick Task")
            .unwrap();
        assert_eq!(new_task.status, TaskStatus::Todo);
    }
}