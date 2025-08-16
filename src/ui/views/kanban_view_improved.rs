use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui::widgets::task_detail_modal::TaskDetailModal;
use crate::repository::comment_repository::CommentRepository;
use eframe::egui::{self, Ui, Rect, Pos2, Vec2, Color32, Rounding, Align2, Sense, CursorIcon, ScrollArea};
use std::collections::{HashMap, HashSet};
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;

pub struct KanbanView {
    pub columns: Vec<KanbanColumn>,
    pub tasks: Vec<Task>,
    pub drag_context: Option<DragContext>,
    pub selected_tasks: HashSet<Uuid>,
    pub search_filter: String,
    pub selected_task_id: Option<Uuid>,
    pub quick_add_states: HashMap<String, QuickAddState>,
    pub wip_limits: HashMap<String, usize>,
    pub column_collapse_state: HashMap<String, bool>,
    pub viewport_width: f32,
    pub task_detail_modal: TaskDetailModal,
    pub comment_repository: Option<Arc<CommentRepository>>,
}

#[derive(Clone)]
pub struct KanbanColumn {
    pub id: Uuid,
    pub title: String,
    pub status: TaskStatus,
    pub color: Color32,
    pub tasks: Vec<Uuid>,
    pub width: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub collapsed: bool,
    pub wip_limit: Option<usize>,
    pub bounds: Rect,
    pub visible: bool,
    pub position: usize,
}

pub struct DragContext {
    pub task_id: Uuid,
    pub start_position: Pos2,
    pub current_position: Pos2,
    pub offset: Vec2,
    pub original_column: usize,
    pub hover_column: Option<usize>,
    pub hover_position: Option<usize>,
}

#[derive(Clone)]
pub struct QuickAddState {
    pub visible: bool,
    pub text: String,
}

impl KanbanView {
    pub fn new() -> Self {
        // Initialize columns with proper bounds
        let spacing = 16.0;
        let column_width = 300.0;
        let x_offset = spacing;
        
        let columns = vec![
            KanbanColumn {
                id: Uuid::new_v4(),
                title: "To Do".to_string(),
                status: TaskStatus::Todo,
                color: Color32::from_rgb(200, 200, 200),
                tasks: Vec::new(),
                width: column_width,
                min_width: 250.0,
                max_width: 500.0,
                collapsed: false,
                wip_limit: None,
                bounds: Rect::from_min_size(
                    Pos2::new(x_offset, 100.0),
                    Vec2::new(column_width, 600.0)
                ),
                visible: true,
                position: 0,
            },
            KanbanColumn {
                id: Uuid::new_v4(),
                title: "In Progress".to_string(),
                status: TaskStatus::InProgress,
                color: Color32::from_rgb(100, 150, 255),
                tasks: Vec::new(),
                width: column_width,
                min_width: 250.0,
                max_width: 500.0,
                collapsed: false,
                wip_limit: Some(3),
                bounds: Rect::from_min_size(
                    Pos2::new(x_offset + column_width + spacing, 100.0),
                    Vec2::new(column_width, 600.0)
                ),
                visible: true,
                position: 1,
            },
            KanbanColumn {
                id: Uuid::new_v4(),
                title: "Review".to_string(),
                status: TaskStatus::Review,
                color: Color32::from_rgb(255, 200, 100),
                tasks: Vec::new(),
                width: column_width,
                min_width: 250.0,
                max_width: 500.0,
                collapsed: false,
                wip_limit: Some(2),
                bounds: Rect::from_min_size(
                    Pos2::new(x_offset + (column_width + spacing) * 2.0, 100.0),
                    Vec2::new(column_width, 600.0)
                ),
                visible: true,
                position: 2,
            },
            KanbanColumn {
                id: Uuid::new_v4(),
                title: "Done".to_string(),
                status: TaskStatus::Done,
                color: Color32::from_rgb(100, 200, 100),
                tasks: Vec::new(),
                width: column_width,
                min_width: 250.0,
                max_width: 500.0,
                collapsed: false,
                wip_limit: None,
                bounds: Rect::from_min_size(
                    Pos2::new(x_offset + (column_width + spacing) * 3.0, 100.0),
                    Vec2::new(column_width, 600.0)
                ),
                visible: true,
                position: 3,
            },
        ];

        let mut wip_limits = HashMap::new();
        wip_limits.insert("In Progress".to_string(), 3);
        wip_limits.insert("Review".to_string(), 2);

        let mut instance = Self {
            columns,
            tasks: Vec::new(),
            drag_context: None,
            selected_tasks: HashSet::new(),
            search_filter: String::new(),
            selected_task_id: None,
            quick_add_states: HashMap::new(),
            wip_limits,
            column_collapse_state: HashMap::new(),
            viewport_width: 1200.0,
            task_detail_modal: TaskDetailModal::new(),
            comment_repository: None,
        };
        
        // Initialize layout
        instance.update_layout(1200.0);
        instance
    }

    // Core drag and drop functionality
    pub fn is_dragging(&self) -> bool {
        self.drag_context.is_some()
    }

    pub fn get_dragging_task_id(&self) -> Option<Uuid> {
        self.drag_context.as_ref().map(|ctx| ctx.task_id)
    }

    pub fn start_drag(&mut self, task_id: Uuid, position: Pos2) {
        if let Some((col_idx, _)) = self.find_task_position(task_id) {
            self.drag_context = Some(DragContext {
                task_id,
                start_position: position,
                current_position: position,
                offset: Vec2::ZERO,
                original_column: col_idx,
                hover_column: None,
                hover_position: None,
            });
        }
    }

    pub fn update_drag_position(&mut self, position: Pos2) {
        // Calculate hover column first
        let hover_column = self.get_column_at_position(position);
        
        if let Some(ctx) = &mut self.drag_context {
            ctx.current_position = position;
            ctx.hover_column = hover_column;
        }
    }

    pub fn get_drag_position(&self) -> Option<Pos2> {
        self.drag_context.as_ref().map(|ctx| ctx.current_position)
    }

    pub fn complete_drag(&mut self, target_column: usize) {
        if let Some(ctx) = &self.drag_context {
            let task_id = ctx.task_id;
            
            // Remove from original column
            if let Some((orig_col, _)) = self.find_task_position(task_id) {
                self.columns[orig_col].tasks.retain(|&id| id != task_id);
            }
            
            // Add to target column
            if target_column < self.columns.len() {
                self.columns[target_column].tasks.push(task_id);
                
                // Update task status
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                    task.status = self.columns[target_column].status;
                }
            }
        }
        
        self.drag_context = None;
    }

    pub fn complete_drag_with_reorder(&mut self, column_index: usize, position: usize) {
        if let Some(ctx) = &self.drag_context {
            let task_id = ctx.task_id;
            
            // Remove from original position
            if let Some((orig_col, _)) = self.find_task_position(task_id) {
                self.columns[orig_col].tasks.retain(|&id| id != task_id);
            }
            
            // Insert at specific position
            if column_index < self.columns.len() {
                let column = &mut self.columns[column_index];
                let insert_pos = position.min(column.tasks.len());
                column.tasks.insert(insert_pos, task_id);
                
                // Update task status
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                    task.status = self.columns[column_index].status;
                }
            }
        }
        
        self.drag_context = None;
    }

    pub fn cancel_drag(&mut self) {
        self.drag_context = None;
    }

    // Column management
    pub fn is_over_column(&self, position: Pos2, column_index: usize) -> bool {
        if column_index >= self.columns.len() {
            return false;
        }
        
        let column = &self.columns[column_index];
        column.bounds.contains(position)
    }

    pub fn get_column_at_position(&self, position: Pos2) -> Option<usize> {
        self.columns.iter()
            .position(|col| col.bounds.contains(position))
    }

    pub fn get_tasks_for_column(&self, column_index: usize) -> Vec<&Task> {
        if column_index >= self.columns.len() {
            return Vec::new();
        }
        
        let column = &self.columns[column_index];
        
        // If the column has a specific task order, use that
        if !column.tasks.is_empty() {
            column.tasks.iter()
                .filter_map(|task_id| {
                    self.tasks.iter().find(|t| t.id == *task_id && self.matches_filter(t))
                })
                .collect()
        } else {
            // Otherwise, filter by status
            self.tasks.iter()
                .filter(|task| task.status == column.status && self.matches_filter(task))
                .collect()
        }
    }

    pub fn get_column_task_count(&self, column_index: usize) -> usize {
        self.get_tasks_for_column(column_index).len()
    }

    pub fn set_wip_limit(&mut self, column_index: usize, limit: usize) {
        if column_index < self.columns.len() {
            self.columns[column_index].wip_limit = Some(limit);
            self.wip_limits.insert(self.columns[column_index].title.clone(), limit);
        }
    }

    pub fn is_column_over_wip_limit(&self, column_index: usize) -> bool {
        if column_index >= self.columns.len() {
            return false;
        }
        
        let column = &self.columns[column_index];
        if let Some(limit) = column.wip_limit {
            self.get_column_task_count(column_index) > limit
        } else {
            false
        }
    }

    pub fn get_empty_column_message(&self, column_index: usize) -> String {
        if column_index >= self.columns.len() {
            return String::new();
        }
        
        match self.columns[column_index].status {
            TaskStatus::Todo => "No tasks to do yet".to_string(),
            TaskStatus::InProgress => "No tasks in progress".to_string(),
            TaskStatus::Review => "No tasks in review".to_string(),
            TaskStatus::Done => "No completed tasks".to_string(),
            TaskStatus::Blocked => "No blocked tasks".to_string(),
            TaskStatus::Cancelled => "No cancelled tasks".to_string(),
        }
    }

    pub fn toggle_column_collapse(&mut self, column_index: usize) {
        if column_index < self.columns.len() {
            let column = &mut self.columns[column_index];
            column.collapsed = !column.collapsed;
            self.column_collapse_state.insert(column.title.clone(), column.collapsed);
        }
    }

    pub fn is_column_collapsed(&self, column_index: usize) -> bool {
        if column_index >= self.columns.len() {
            return false;
        }
        self.columns[column_index].collapsed
    }

    // Task management
    pub fn add_task(&mut self, task: Task) {
        // Find the appropriate column for this task based on its status
        for column in self.columns.iter_mut() {
            if column.status == task.status {
                column.tasks.push(task.id);
                break;
            }
        }
        self.tasks.push(task);
    }
    
    pub fn select_task(&mut self, task_id: Uuid) {
        self.selected_task_id = Some(task_id);
        self.selected_tasks.clear();
        self.selected_tasks.insert(task_id);
    }

    pub fn add_to_selection(&mut self, task_id: Uuid) {
        self.selected_tasks.insert(task_id);
    }

    pub fn clear_selection(&mut self) {
        self.selected_task_id = None;
        self.selected_tasks.clear();
    }

    pub fn get_selected_task_id(&self) -> Option<Uuid> {
        self.selected_task_id
    }

    pub fn bulk_move_selected(&mut self, target_column: usize) {
        if target_column >= self.columns.len() {
            return;
        }
        
        let target_status = self.columns[target_column].status;
        
        for task_id in &self.selected_tasks {
            // Update task status
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *task_id) {
                task.status = target_status;
            }
        }
        
        self.clear_selection();
    }

    // Quick add functionality
    pub fn enable_quick_add(&mut self, column_index: usize) {
        if column_index < self.columns.len() {
            let column_title = self.columns[column_index].title.clone();
            self.quick_add_states.insert(column_title, QuickAddState {
                visible: true,
                text: String::new(),
            });
        }
    }

    pub fn is_quick_add_active(&self, column_index: usize) -> bool {
        if column_index >= self.columns.len() {
            return false;
        }
        
        self.quick_add_states
            .get(&self.columns[column_index].title)
            .map(|state| state.visible)
            .unwrap_or(false)
    }

    pub fn quick_add_task(&mut self, column_index: usize, title: String) {
        if column_index >= self.columns.len() || title.trim().is_empty() {
            return;
        }
        
        let mut task = Task::new(title, String::new());
        task.status = self.columns[column_index].status;
        
        self.columns[column_index].tasks.push(task.id);
        self.tasks.push(task);
        
        // Clear quick add state
        let column_title = self.columns[column_index].title.clone();
        self.quick_add_states.remove(&column_title);
    }

    // Search and filtering
    pub fn set_search_filter(&mut self, query: &str) {
        self.search_filter = query.to_lowercase();
    }

    pub fn get_visible_tasks(&self) -> Vec<&Task> {
        self.tasks.iter()
            .filter(|task| self.matches_filter(task))
            .collect()
    }

    fn matches_filter(&self, task: &Task) -> bool {
        if self.search_filter.is_empty() {
            return true;
        }
        
        task.title.to_lowercase().contains(&self.search_filter) ||
        task.description.to_lowercase().contains(&self.search_filter)
    }

    // Layout calculations
    pub fn calculate_column_width(&self, available_width: f32) -> f32 {
        let visible_columns = self.columns.iter()
            .filter(|col| col.visible && !col.collapsed)
            .count();
        
        if visible_columns == 0 {
            return 300.0;
        }
        
        // For standard desktop screens (>1000px), aim for comfortable column widths
        // For narrower screens, compress as needed
        let spacing = 16.0;
        let total_spacing = spacing * 2.0; // Left and right padding
        let column_spacing = spacing * (visible_columns - 1) as f32; // Between columns
        
        let available_for_columns = available_width - total_spacing - column_spacing;
        let calculated_width = available_for_columns / visible_columns as f32;
        
        // For wide screens with few columns, don't make them too wide
        // For narrow screens or many columns, ensure minimum usability
        if available_width >= 1000.0 && visible_columns <= 4 {
            // Desktop mode - prefer comfortable widths
            calculated_width.min(400.0).max(320.0)
        } else {
            // Mobile or many columns - allow more compression
            calculated_width.min(400.0).max(250.0)
        }
    }

    pub fn calculate_card_height(&self, task: &Task) -> f32 {
        let base_height = 80.0;
        let extra_per_subtask = 20.0;
        let extra_for_tags = if !task.tags.is_empty() { 25.0 } else { 0.0 };
        let extra_for_description = if !task.description.is_empty() { 20.0 } else { 0.0 };
        let max_height = 200.0;
        
        let subtask_height = task.subtasks.len() as f32 * extra_per_subtask;
        
        (base_height + subtask_height + extra_for_tags + extra_for_description).min(max_height)
    }

    pub fn get_card_spacing(&self) -> f32 {
        8.0
    }

    pub fn update_layout(&mut self, viewport_width: f32) {
        self.update_layout_with_height(viewport_width, 800.0)
    }
    
    pub fn update_layout_with_height(&mut self, viewport_width: f32, viewport_height: f32) {
        self.viewport_width = viewport_width;
        
        let column_width = self.calculate_column_width(viewport_width);
        let spacing = 16.0;
        let mut x_offset = spacing;
        
        for column in self.columns.iter_mut() {
            if column.visible && !column.collapsed {
                column.width = column_width;
                column.bounds = Rect::from_min_size(
                    Pos2::new(x_offset, 100.0),
                    Vec2::new(column_width, viewport_height.max(600.0))
                );
                x_offset += column_width + spacing;
            } else if column.collapsed {
                column.width = 50.0;
                column.bounds = Rect::from_min_size(
                    Pos2::new(x_offset, 100.0),
                    Vec2::new(50.0, viewport_height.max(600.0))
                );
                x_offset += 50.0 + spacing;
            }
        }
    }

    pub fn should_stack_columns(&self) -> bool {
        self.viewport_width < 768.0
    }

    // Visual helpers
    pub fn get_card_color(&self, task: &Task) -> Color32 {
        match task.priority {
            Priority::Critical => Color32::from_rgb(255, 100, 100),
            Priority::High => Color32::from_rgb(255, 150, 100),
            Priority::Medium => Color32::from_rgb(255, 255, 150),
            Priority::Low => Color32::from_rgb(200, 200, 200),
        }
    }

    pub fn should_highlight_as_overdue(&self, task: &Task) -> bool {
        task.is_overdue()
    }

    // Keyboard shortcuts
    pub fn handle_keyboard_shortcut(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
        if let Some(task_id) = self.selected_task_id {
            match key {
                egui::Key::ArrowRight if !modifiers.any() => {
                    // Move task to next column
                    if let Some((col_idx, _)) = self.find_task_position(task_id) {
                        if col_idx + 1 < self.columns.len() {
                            // Simulate a drag and drop operation
                            self.start_drag(task_id, Pos2::ZERO);
                            self.complete_drag(col_idx + 1);
                        }
                    }
                }
                egui::Key::ArrowLeft if !modifiers.any() => {
                    // Move task to previous column
                    if let Some((col_idx, _)) = self.find_task_position(task_id) {
                        if col_idx > 0 {
                            // Simulate a drag and drop operation
                            self.start_drag(task_id, Pos2::ZERO);
                            self.complete_drag(col_idx - 1);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Helper methods
    fn find_task_position(&self, task_id: Uuid) -> Option<(usize, usize)> {
        for (col_idx, column) in self.columns.iter().enumerate() {
            if let Some(task_idx) = column.tasks.iter().position(|&id| id == task_id) {
                return Some((col_idx, task_idx));
            }
        }
        
        // If not in column task lists, find by status
        if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
            for (col_idx, column) in self.columns.iter().enumerate() {
                if column.status == task.status {
                    return Some((col_idx, 0));
                }
            }
        }
        
        None
    }

    // Main render method
    pub fn show(&mut self, ui: &mut Ui, tasks: &mut Vec<Task>) {
        self.tasks = tasks.clone();
        
        // Sync task positions if columns are empty (initial load)
        for task in &self.tasks {
            let task_in_column = self.columns.iter()
                .any(|col| col.tasks.contains(&task.id));
            
            if !task_in_column {
                // Find the appropriate column for this task based on its status
                for (idx, column) in self.columns.iter_mut().enumerate() {
                    if column.status == task.status {
                        column.tasks.push(task.id);
                        break;
                    }
                }
            }
        }
        
        // Update layout based on available width and height
        let available_width = ui.available_width();
        let available_height = ui.available_height();
        self.update_layout_with_height(available_width, available_height);
        
        // Clone columns for iteration to avoid borrow issues
        let columns_for_render = self.columns.clone();
        
        // Header with search
        ui.horizontal(|ui| {
            ui.heading("📋 Kanban Board");
            
            ui.separator();
            
            ui.label("🔍");
            let search_response = ui.text_edit_singleline(&mut self.search_filter);
            if search_response.changed() {
                // Filter is applied automatically via matches_filter
            }
            
            ui.separator();
            
            if ui.button("➕ Add Column").clicked() {
                // TODO: Add custom column
            }
        });
        
        ui.separator();
        
        // Kanban board - use full available height
        let available_height = ui.available_height();
        ScrollArea::horizontal()
            .id_source("kanban_main_horizontal_scroll")
            .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.set_min_height(available_height);
                let columns_clone = self.columns.clone();
                for (col_idx, column) in columns_clone.iter().enumerate() {
                    if !column.visible {
                        continue;
                    }
                    
                    let column_rect = Rect::from_min_size(
                        ui.cursor().min,
                        Vec2::new(
                            if column.collapsed { 50.0 } else { column.width },
                            ui.available_height()
                        )
                    );
                    
                    // Update column bounds for drag detection
                    self.columns[col_idx].bounds = column_rect;
                    
                    // Draw column
                    ui.allocate_ui_at_rect(column_rect, |ui| {
                        self.render_column(ui, col_idx);
                    });
                    
                    ui.add_space(8.0);
                }
            });
        });
        
        // Handle drag visual
        if let Some(ctx) = &self.drag_context {
            if let Some(task) = self.tasks.iter().find(|t| t.id == ctx.task_id) {
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                
                // Draw dragged card at cursor
                let painter = ui.painter();
                let card_rect = Rect::from_min_size(
                    ctx.current_position - Vec2::new(100.0, 20.0),
                    Vec2::new(200.0, 80.0)
                );
                
                painter.rect_filled(
                    card_rect,
                    Rounding::same(4.0),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 200)
                );
                
                painter.text(
                    card_rect.center(),
                    Align2::CENTER_CENTER,
                    &task.title,
                    egui::FontId::default(),
                    Color32::BLACK
                );
            }
        }
        
        // Update tasks back
        *tasks = self.tasks.clone();
    }

    fn render_column(&mut self, ui: &mut Ui, column_index: usize) {
        // Get what we need upfront to avoid borrow issues
        let column_collapsed = self.columns[column_index].collapsed;
        let column_color = self.columns[column_index].color;
        let column_title = self.columns[column_index].title.clone();
        let column_wip_limit = self.columns[column_index].wip_limit;
        let column_width = self.columns[column_index].width;
        
        let tasks: Vec<Task> = self.get_tasks_for_column(column_index)
            .into_iter()
            .cloned()
            .collect();
        let is_over_limit = self.is_column_over_wip_limit(column_index);
        let task_count = tasks.len();
        
        ui.vertical(|ui| {
            // Column header
            ui.horizontal(|ui| {
                if ui.small_button(if column_collapsed { "▶" } else { "▼" }).clicked() {
                    self.toggle_column_collapse(column_index);
                }
                
                if !column_collapsed {
                    ui.colored_label(column_color, &column_title);
                    ui.label(format!("({})", task_count));
                    
                    if let Some(limit) = column_wip_limit {
                        let color = if is_over_limit {
                            Color32::RED
                        } else {
                            Color32::GRAY
                        };
                        ui.colored_label(color, format!("[WIP: {}]", limit));
                    }
                    
                    if ui.small_button("➕").clicked() {
                        self.enable_quick_add(column_index);
                    }
                }
            });
            
            if !column_collapsed {
                ui.separator();
                
                // Use most of the available height for the scroll area
                let scroll_height = ui.available_height() - 50.0; // Leave some space for header
                ScrollArea::vertical()
                    .id_source(format!("kanban_column_scroll_{}", column_index))
                    .max_height(scroll_height.max(400.0)) // Ensure minimum height
                    .show(ui, |ui| {
                        // Drop zone highlight
                        if self.is_dragging() {
                            let response = ui.allocate_response(
                                Vec2::new(column_width - 10.0, 10.0),
                                Sense::hover()
                            );
                            
                            if response.hovered() {
                                ui.painter().rect_filled(
                                    response.rect,
                                    Rounding::same(4.0),
                                    Color32::from_rgba_premultiplied(100, 150, 255, 50)
                                );
                            }
                        }
                        
                        // Render tasks
                        if tasks.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label(self.get_empty_column_message(column_index));
                            });
                        } else {
                            for task in tasks {
                                self.render_task_card(ui, &task, column_index);
                                ui.add_space(self.get_card_spacing());
                            }
                        }
                        
                        // Quick add form
                        if self.is_quick_add_active(column_index) {
                            self.render_quick_add_form(ui, column_index, &column_title);
                        }
                    });
            }
        });
    }

    fn render_quick_add_form(&mut self, ui: &mut Ui, column_index: usize, column_title: &str) {
        ui.horizontal(|ui| {
            let mut text = String::new();
            if let Some(state) = self.quick_add_states.get(column_title) {
                text = state.text.clone();
            }
            
            let response = ui.text_edit_singleline(&mut text);
            
            let mut should_add = false;
            let mut should_cancel = false;
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !text.is_empty() {
                should_add = true;
            }
            
            if response.changed() {
                // Update the text in the state
                if let Some(state) = self.quick_add_states.get_mut(column_title) {
                    state.text = text.clone();
                }
            }
            
            if ui.small_button("✓").clicked() && !text.is_empty() {
                should_add = true;
            }
            
            if ui.small_button("✗").clicked() {
                should_cancel = true;
            }
            
            // Apply actions after UI interaction
            if should_add {
                self.quick_add_task(column_index, text);
            }
            if should_cancel {
                self.quick_add_states.remove(column_title);
            }
        });
    }

    fn render_task_card(&mut self, ui: &mut Ui, task: &Task, column_index: usize) {
        let is_selected = self.selected_task_id == Some(task.id);
        let is_overdue = self.should_highlight_as_overdue(task);
        
        let card_color = if is_selected {
            Color32::from_rgb(200, 220, 255)
        } else if is_overdue {
            Color32::from_rgb(255, 200, 200)
        } else {
            Color32::from_rgb(250, 250, 250)
        };
        
        let response = ui.allocate_response(
            Vec2::new(self.columns[column_index].width - 20.0, self.calculate_card_height(task)),
            Sense::click_and_drag()
        );
        
        // Handle interactions
        if response.clicked() {
            self.select_task(task.id);
        }
        
        if response.drag_started() {
            self.start_drag(task.id, response.interact_pointer_pos().unwrap_or(Pos2::ZERO));
        }
        
        if self.is_dragging() {
            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                self.update_drag_position(pos);
            }
        }
        
        if response.drag_released() && self.is_dragging() {
            if let Some(target_col) = self.get_column_at_position(response.interact_pointer_pos().unwrap_or(Pos2::ZERO)) {
                self.complete_drag(target_col);
            } else {
                self.cancel_drag();
            }
        }
        
        // Draw card
        ui.painter().rect(
            response.rect,
            Rounding::same(4.0),
            card_color,
            egui::Stroke::new(1.0, Color32::GRAY)
        );
        
        // Card content
        ui.allocate_ui_at_rect(response.rect.shrink(8.0), |ui| {
            ui.vertical(|ui| {
                // Priority indicator
                let priority_color = match task.priority {
                    Priority::Critical => Color32::RED,
                    Priority::High => Color32::from_rgb(255, 150, 0),
                    Priority::Medium => Color32::from_rgb(255, 200, 0),
                    Priority::Low => Color32::GRAY,
                };
                
                ui.horizontal(|ui| {
                    ui.painter().circle_filled(
                        ui.cursor().min + Vec2::new(5.0, 10.0),
                        3.0,
                        priority_color
                    );
                    ui.add_space(10.0);
                    ui.label(&task.title);
                });
                
                if !task.description.is_empty() {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(&task.description)
                            .small()
                            .color(Color32::GRAY)
                    );
                }
                
                // Tags and metadata
                if !task.tags.is_empty() {
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        for tag in &task.tags {
                            ui.label(
                                egui::RichText::new(format!("#{}", tag))
                                    .small()
                                    .color(Color32::from_rgb(100, 150, 200))
                            );
                        }
                    });
                }
                
                // Due date
                if let Some(due) = task.due_date {
                    ui.add_space(4.0);
                    let days_until = (due - Utc::now()).num_days();
                    let date_color = if days_until < 0 {
                        Color32::RED
                    } else if days_until <= 3 {
                        Color32::from_rgb(255, 150, 0)
                    } else {
                        Color32::GRAY
                    };
                    
                    ui.colored_label(
                        date_color,
                        format!("📅 {}", due.format("%b %d"))
                    );
                }
                
                // Subtask progress
                if !task.subtasks.is_empty() {
                    ui.add_space(4.0);
                    let (completed, total) = task.subtask_progress();
                    ui.add(
                        egui::ProgressBar::new(completed as f32 / total as f32)
                            .text(format!("{}/{}", completed, total))
                    );
                }
            });
        });
    }
}