use super::kanban_view::{KanbanView, EditMode};
use crate::domain::task::{Task, TaskStatus, Priority};
use eframe::egui::{Key, Modifiers};
use uuid::Uuid;
use chrono::{DateTime, Utc};

impl KanbanView {
    // Inline editing activation
    pub fn handle_double_click(&mut self, task_id: Uuid) {
        self.start_inline_edit(task_id, EditMode::TaskTitle);
    }
    
    pub fn start_inline_edit(&mut self, task_id: Uuid, mode: EditMode) {
        if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
            self.editing_task_id = Some(task_id);
            self.edit_mode = Some(mode.clone());
            
            // Initialize edit buffer with current value
            self.edit_buffer = match mode {
                EditMode::TaskTitle => task.title.clone(),
                EditMode::TaskDescription => task.description.clone(),
                EditMode::TaskTags => task.tags.join(", "),
                EditMode::TaskPriority => format!("{:?}", task.priority),
                EditMode::TaskAssignee => task.assignee.clone().unwrap_or_default(),
                EditMode::TaskDueDate => task.due_date.map(|d| d.to_string()).unwrap_or_default(),
            };
        }
    }
    
    pub fn is_editing_task(&self, task_id: Uuid) -> bool {
        self.editing_task_id == Some(task_id)
    }
    
    // Commit and cancel
    pub fn commit_inline_edit(&mut self) {
        if let Some(task_id) = self.editing_task_id {
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                if let Some(mode) = &self.edit_mode {
                    match mode {
                        EditMode::TaskTitle => {
                            if !self.edit_buffer.is_empty() {
                                task.title = self.edit_buffer.clone();
                            } else {
                                self.validation_error_message = "Title cannot be empty".to_string();
                                return;
                            }
                        }
                        EditMode::TaskDescription => {
                            task.description = self.edit_buffer.clone();
                        }
                        EditMode::TaskTags => {
                            task.tags = self.edit_buffer.split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                        _ => {}
                    }
                }
            }
        }
        
        self.editing_task_id = None;
        self.edit_mode = None;
        self.edit_buffer.clear();
        self.validation_error_message.clear();
    }
    
    pub fn cancel_inline_edit(&mut self) {
        self.editing_task_id = None;
        self.edit_mode = None;
        self.edit_buffer.clear();
        self.validation_error_message.clear();
    }
    
    // Column title editing
    pub fn start_column_title_edit(&mut self, column_id: Uuid) {
        if let Some(column) = self.columns.iter().find(|c| c.id == column_id) {
            self.editing_column_id = Some(column_id);
            self.edit_buffer = column.title.clone();
        }
    }
    
    pub fn is_editing_column(&self, column_id: Uuid) -> bool {
        self.editing_column_id == Some(column_id)
    }
    
    pub fn commit_column_edit(&mut self) {
        if let Some(column_id) = self.editing_column_id {
            if let Some(column) = self.columns.iter_mut().find(|c| c.id == column_id) {
                if !self.edit_buffer.is_empty() {
                    column.title = self.edit_buffer.clone();
                }
            }
        }
        
        self.editing_column_id = None;
        self.edit_buffer.clear();
    }
    
    // Validation
    pub fn has_validation_error(&self) -> bool {
        !self.validation_error_message.is_empty()
    }
    
    // Tab navigation
    pub fn handle_tab_key(&mut self) {
        if let Some(current_mode) = &self.edit_mode {
            let next_mode = match current_mode {
                EditMode::TaskTitle => EditMode::TaskDescription,
                EditMode::TaskDescription => EditMode::TaskTags,
                EditMode::TaskTags => EditMode::TaskPriority,
                EditMode::TaskPriority => EditMode::TaskAssignee,
                EditMode::TaskAssignee => EditMode::TaskDueDate,
                EditMode::TaskDueDate => EditMode::TaskTitle,
            };
            
            self.commit_inline_edit();
            if let Some(task_id) = self.editing_task_id {
                self.start_inline_edit(task_id, next_mode);
            }
        }
    }
    
    // Auto-save
    pub fn trigger_auto_save(&mut self) {
        if self.enable_auto_save && self.editing_task_id.is_some() {
            self.commit_inline_edit();
        }
    }
    
    // Tag management
    pub fn add_tag_inline(&mut self, tag: &str) {
        if !self.edit_buffer.is_empty() {
            self.edit_buffer.push_str(", ");
        }
        self.edit_buffer.push_str(tag);
    }
    
    // Priority setting
    pub fn set_priority_inline(&mut self, priority: Priority) {
        self.edit_buffer = format!("{:?}", priority);
    }
    
    // Assignee suggestions
    pub fn get_assignee_suggestions(&self, query: &str) -> Vec<String> {
        // In a real implementation, this would query available users
        vec![
            "John Doe".to_string(),
            "Jane Smith".to_string(),
            "Bob Wilson".to_string(),
        ].into_iter()
            .filter(|name| name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }
    
    pub fn select_assignee_suggestion(&mut self, assignee: &str) {
        self.edit_buffer = assignee.to_string();
    }
    
    // Due date
    pub fn set_due_date_inline(&mut self, due_date: DateTime<Utc>) {
        self.edit_buffer = due_date.to_string();
    }
    
    // Multi-line support
    pub fn is_multiline_edit_mode(&self) -> bool {
        matches!(self.edit_mode, Some(EditMode::TaskDescription))
    }
    
    pub fn get_edit_lines_count(&self) -> usize {
        self.edit_buffer.lines().count()
    }
    
    // Mode switching
    pub fn switch_edit_mode(&mut self, new_mode: EditMode) {
        self.commit_inline_edit();
        if let Some(task_id) = self.editing_task_id {
            self.start_inline_edit(task_id, new_mode);
        }
    }
    
    // Keyboard shortcuts
    pub fn handle_key_shortcut(&mut self, key: Key, modifiers: Modifiers, task_id: Uuid) {
        match key {
            Key::F2 if modifiers.is_none() => {
                self.start_inline_edit(task_id, EditMode::TaskTitle);
            }
            Key::Enter if modifiers.is_none() && self.editing_task_id.is_some() => {
                self.commit_inline_edit();
            }
            Key::Escape if modifiers.is_none() && self.editing_task_id.is_some() => {
                self.cancel_inline_edit();
            }
            _ => {}
        }
    }
    
    // Focus management
    pub fn has_edit_focus(&self) -> bool {
        self.editing_task_id.is_some() || self.editing_column_id.is_some()
    }
    
    pub fn handle_click_outside(&mut self) {
        if self.has_edit_focus() {
            self.commit_inline_edit();
            self.commit_column_edit();
        }
    }
    
    // Undo/redo
    pub fn undo_last_edit(&mut self) {
        // In a real implementation, this would use an undo stack
    }
    
    pub fn redo_last_edit(&mut self) {
        // In a real implementation, this would use a redo stack
    }
    
    // Bulk editing
    pub fn start_bulk_inline_edit(&mut self, mode: EditMode) {
        if !self.selected_tasks.is_empty() {
            self.edit_mode = Some(mode);
            self.edit_buffer.clear();
        }
    }
    
    pub fn set_bulk_priority(&mut self, priority: Priority) {
        for task_id in &self.selected_tasks {
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *task_id) {
                task.priority = priority.clone();
            }
        }
    }
    
    pub fn commit_bulk_edit(&mut self) {
        // Apply changes to all selected tasks
        self.selected_tasks.clear();
        self.edit_mode = None;
        self.edit_buffer.clear();
    }
    
    // Quick add
    pub fn start_quick_add(&mut self, column_index: usize) {
        self.quick_add_column = Some(column_index);
        self.quick_add_buffer.clear();
    }
    
    pub fn is_quick_adding(&self) -> bool {
        self.quick_add_column.is_some()
    }
    
    pub fn commit_quick_add(&mut self) {
        if let Some(column_index) = self.quick_add_column {
            if !self.quick_add_buffer.is_empty() {
                let mut task = Task::new_simple(self.quick_add_buffer.clone());
                task.status = self.columns[column_index].status.clone();
                self.tasks.push(task);
            }
        }
        
        self.quick_add_column = None;
        self.quick_add_buffer.clear();
    }
}