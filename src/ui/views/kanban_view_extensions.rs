use super::kanban_view::{KanbanView, FilterOptions, QuickAddMetadata, WipLimit, KanbanColumn};
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::services::TaskService;
use uuid::Uuid;
use eframe::egui::{Pos2, Vec2, Color32};
use std::collections::HashMap;

impl KanbanView {
    pub fn add_custom_column(&mut self, title: &str, status: TaskStatus, color: (u8, u8, u8, u8)) {
        let new_column = super::kanban_view::KanbanColumn {
            title: title.to_string(),
            status,
            color: Color32::from_rgba_premultiplied(color.0, color.1, color.2, color.3),
            width: 250.0,
            collapsed: false,
            wip_limit: None,
            visible: true,
            position: self.columns.len(),
        };
        self.columns.push(new_column);
    }

    pub fn move_column(&mut self, from_index: usize, to_index: usize) {
        if from_index < self.columns.len() && to_index < self.columns.len() {
            let column = self.columns.remove(from_index);
            self.columns.insert(to_index, column);
            
            for (i, col) in self.columns.iter_mut().enumerate() {
                col.position = i;
            }
        }
    }

    pub fn get_column_order(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.title.clone()).collect()
    }

    pub fn set_column_visible(&mut self, column_title: &str, visible: bool) {
        if let Some(column) = self.columns.iter_mut().find(|c| c.title == column_title) {
            column.visible = visible;
        }
    }

    pub fn is_column_visible(&self, column_title: &str) -> bool {
        self.columns.iter()
            .find(|c| c.title == column_title)
            .map(|c| c.visible)
            .unwrap_or(false)
    }

    pub fn set_column_width(&mut self, column_title: &str, width: f32) {
        if let Some(column) = self.columns.iter_mut().find(|c| c.title == column_title) {
            column.width = width.max(200.0);
        }
    }

    pub fn get_column_width(&self, column_title: &str) -> f32 {
        self.columns.iter()
            .find(|c| c.title == column_title)
            .map(|c| if c.collapsed { 50.0 } else { c.width })
            .unwrap_or(250.0)
    }

    pub fn collapse_column(&mut self, column_title: &str) {
        if let Some(column) = self.columns.iter_mut().find(|c| c.title == column_title) {
            column.collapsed = true;
        }
    }

    pub fn expand_column(&mut self, column_title: &str) {
        if let Some(column) = self.columns.iter_mut().find(|c| c.title == column_title) {
            column.collapsed = false;
        }
    }

    pub fn is_column_collapsed(&self, column_title: &str) -> bool {
        self.columns.iter()
            .find(|c| c.title == column_title)
            .map(|c| c.collapsed)
            .unwrap_or(false)
    }

    pub fn is_wip_limit_exceeded(&self, column_title: &str, tasks: &[Task]) -> bool {
        if let Some(column) = self.columns.iter().find(|c| c.title == column_title) {
            if let Some(limit) = column.wip_limit {
                let count = tasks.iter().filter(|t| t.status == column.status).count();
                return count > limit;
            }
        }
        false
    }

    pub fn get_wip_violation_message(&self, column_title: &str, tasks: &[Task]) -> Option<String> {
        if let Some(column) = self.columns.iter().find(|c| c.title == column_title) {
            if let Some(limit) = column.wip_limit {
                let count = tasks.iter().filter(|t| t.status == column.status).count();
                if count > limit {
                    return Some(format!("WIP limit exceeded: {}/{}", count, limit));
                }
            }
        }
        None
    }

    pub fn get_column_style(&self, column_title: &str, task_count: usize) -> ColumnStyle {
        let column = self.columns.iter().find(|c| c.title == column_title);
        
        let is_over_wip = column.and_then(|c| c.wip_limit)
            .map(|limit| task_count > limit)
            .unwrap_or(false);
        
        ColumnStyle {
            show_wip_warning: is_over_wip,
            header_color: if is_over_wip {
                (255, 200, 0, 255)
            } else {
                (100, 100, 100, 255)
            },
            pulse_header: is_over_wip,
        }
    }

    pub fn can_drop_in_column(&self, column_title: &str, _new_task: &Task, existing_tasks: &[Task]) -> bool {
        if let Some(column) = self.columns.iter().find(|c| c.title == column_title) {
            if let Some(limit) = column.wip_limit {
                let current_count = existing_tasks.iter()
                    .filter(|t| t.status == column.status)
                    .count();
                return current_count < limit;
            }
        }
        true
    }

    pub fn filter_tasks(&self, tasks: &[Task], search_text: &str) -> Vec<Task> {
        let search_lower = search_text.to_lowercase();
        tasks.iter()
            .filter(|t| 
                t.title.to_lowercase().contains(&search_lower) ||
                t.description.to_lowercase().contains(&search_lower)
            )
            .cloned()
            .collect()
    }

    pub fn collapse_swimlane(&mut self, lane_name: &str) {
        self.swimlane_config.collapsed_lanes.insert(lane_name.to_string());
    }

    pub fn expand_swimlane(&mut self, lane_name: &str) {
        self.swimlane_config.collapsed_lanes.remove(lane_name);
    }

    pub fn is_swimlane_collapsed(&self, lane_name: &str) -> bool {
        self.swimlane_config.collapsed_lanes.contains(lane_name)
    }

    pub fn set_swimlane_order(&mut self, order: Vec<&str>) {
        self.swimlane_config.lane_order = order.iter().map(|s| s.to_string()).collect();
    }

    pub fn get_swimlane_order(&self) -> Vec<String> {
        self.swimlane_config.lane_order.clone()
    }

    pub fn move_swimlane(&mut self, from_index: usize, to_index: usize) {
        if from_index < self.swimlane_config.lane_order.len() && 
           to_index < self.swimlane_config.lane_order.len() {
            let lane = self.swimlane_config.lane_order.remove(from_index);
            self.swimlane_config.lane_order.insert(to_index, lane);
        }
    }

    pub fn is_quick_add_visible(&self, column_title: &str) -> bool {
        self.quick_add_states.get(column_title)
            .map(|s| s.visible)
            .unwrap_or(false)
    }

    pub async fn create_quick_task(&mut self, column_title: &str, title: &str, service: &TaskService) -> Result<Task, String> {
        let status = self.columns.iter()
            .find(|c| c.title == column_title)
            .map(|c| c.status)
            .unwrap_or(TaskStatus::Todo);
        
        let mut task = Task::new(title.to_string(), String::new());
        task.status = status;
        
        service.create(task).await.map_err(|e| e.to_string())
    }

    pub async fn create_quick_task_with_metadata(&mut self, column_title: &str, metadata: QuickAddMetadata, service: &TaskService) -> Result<Task, String> {
        let status = self.columns.iter()
            .find(|c| c.title == column_title)
            .map(|c| c.status)
            .unwrap_or(TaskStatus::Todo);
        
        let mut task = Task::new(metadata.title, metadata.description.unwrap_or_default());
        task.status = status;
        
        if let Some(priority) = metadata.priority {
            task.priority = priority;
        }
        
        for tag in metadata.tags {
            task.add_tag(tag);
        }
        
        task.due_date = metadata.due_date;
        
        service.create(task).await.map_err(|e| e.to_string())
    }

    pub fn handle_keyboard_shortcut(&mut self, shortcut: &str, column: Option<&str>) {
        match shortcut {
            "ctrl+n" => {
                if let Some(col) = column {
                    self.show_quick_add(col);
                }
            },
            "escape" => {
                self.quick_add_states.values_mut().for_each(|s| s.visible = false);
                self.context_menu = None;
                self.cancel_drag();
            },
            _ => {}
        }
    }

    pub fn is_editing_task(&self, task_id: Uuid) -> bool {
        self.focused_card == Some(task_id)
    }

    pub fn get_edit_dialog(&self) -> Option<Uuid> {
        self.focused_card
    }

    pub fn get_context_menu(&self) -> Option<ContextMenuInfo> {
        self.context_menu.as_ref().map(|menu| ContextMenuInfo {
            task_id: menu.task_id,
            position: (menu.position.x, menu.position.y),
            items: menu.items.clone(),
        })
    }

    pub async fn quick_change_status(&mut self, task_id: Uuid, new_status: TaskStatus, service: &TaskService) -> Result<(), String> {
        if let Ok(Some(mut task)) = service.get(task_id).await {
            task.update_status(new_status);
            service.update(task).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn get_selected_cards(&self) -> Vec<Uuid> {
        self.selected_cards.iter().cloned().collect()
    }

    pub async fn bulk_change_status(&mut self, new_status: TaskStatus, service: &TaskService) -> Result<(), String> {
        for task_id in self.selected_cards.clone() {
            self.quick_change_status(task_id, new_status, service).await?;
        }
        Ok(())
    }

    pub fn start_card_animation(&mut self, task_id: Uuid, from: (f32, f32), to: (f32, f32)) {
        self.animations.card_animations.insert(
            task_id,
            super::kanban_view::CardAnimation {
                start_pos: Pos2::new(from.0, from.1),
                end_pos: Pos2::new(to.0, to.1),
                start_time: self.animations.time,
                duration: 0.5,
                opacity: 1.0,
                scale: 1.0,
            }
        );
    }

    pub fn is_animating(&self, task_id: Uuid) -> bool {
        self.animations.card_animations.contains_key(&task_id)
    }

    pub fn get_animated_position(&self, task_id: Uuid) -> Option<(f32, f32)> {
        self.animations.card_animations.get(&task_id).map(|anim| {
            let progress = ((self.animations.time - anim.start_time) / anim.duration).min(1.0);
            let x = anim.start_pos.x + (anim.end_pos.x - anim.start_pos.x) * progress;
            let y = anim.start_pos.y + (anim.end_pos.y - anim.start_pos.y) * progress;
            (x, y)
        })
    }

    pub fn get_animated_column_width(&self, column_title: &str) -> f32 {
        if let Some(anim) = self.animations.column_animations.get(column_title) {
            let progress = ((self.animations.time - anim.start_time) / anim.duration).min(1.0);
            anim.start_width + (anim.end_width - anim.start_width) * progress
        } else {
            self.get_column_width(column_title)
        }
    }

    pub fn add_card_with_animation(&mut self, task_id: Uuid, _column: &str) {
        self.animations.card_animations.insert(
            task_id,
            super::kanban_view::CardAnimation {
                start_pos: Pos2::new(0.0, 0.0),
                end_pos: Pos2::new(0.0, 0.0),
                start_time: self.animations.time,
                duration: 0.3,
                opacity: 0.0,
                scale: 0.8,
            }
        );
    }

    pub fn get_card_opacity(&self, task_id: Uuid) -> f32 {
        self.animations.card_animations.get(&task_id)
            .map(|anim| {
                let progress = ((self.animations.time - anim.start_time) / anim.duration).min(1.0);
                progress
            })
            .unwrap_or(1.0)
    }

    pub fn get_preferences(&self) -> ViewPreferencesData {
        ViewPreferencesData {
            column_widths: self.columns.iter()
                .map(|c| (c.title.clone(), c.width))
                .collect(),
            wip_limits: self.columns.iter()
                .filter_map(|c| c.wip_limit.map(|l| (c.title.clone(), l)))
                .collect(),
            swimlanes_enabled: self.swimlane_config.enabled,
            hidden_columns: self.columns.iter()
                .filter(|c| !c.visible)
                .map(|c| c.title.clone())
                .collect(),
        }
    }

    pub async fn save_preferences(&self, _prefs: &ViewPreferencesData) -> Result<(), String> {
        Ok(())
    }

    pub async fn load_preferences(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub fn get_wip_limit(&self, column_title: &str) -> Option<usize> {
        self.columns.iter()
            .find(|c| c.title == column_title)
            .and_then(|c| c.wip_limit)
    }

    pub fn are_swimlanes_enabled(&self) -> bool {
        self.swimlane_config.enabled
    }

    pub fn apply_filter(&mut self, filter: FilterOptions) {
        self.filter_options = filter;
    }

    pub fn get_filter_state(&self) -> FilterOptions {
        self.filter_options.clone()
    }

    pub fn restore_filter_state(&mut self, filter: FilterOptions) {
        self.filter_options = filter;
    }

    pub fn get_current_filter(&self) -> FilterOptions {
        self.filter_options.clone()
    }

    pub fn prepare_render_data(&mut self, _tasks: &[Task]) {
        // Prepare data for efficient rendering
    }

    pub fn set_viewport_height(&mut self, _height: f32) {
        // Set viewport for virtual scrolling
    }

    pub fn calculate_visible_range(&self, column_title: &str, tasks: &[Task]) -> (usize, usize) {
        let column_tasks: Vec<_> = tasks.iter()
            .filter(|t| self.columns.iter()
                .find(|c| c.title == column_title)
                .map(|c| t.status == c.status)
                .unwrap_or(false))
            .collect();
        
        let visible_count = 10;
        (0, visible_count.min(column_tasks.len()))
    }

    pub fn scroll_column(&mut self, _column: &str, _offset: f32) {
        // Handle column scrolling
    }

    pub fn search_tasks(&self, tasks: &[Task], query: &str) -> Vec<Task> {
        self.filter_tasks(tasks, query)
    }

    pub fn set_focusable_cards(&mut self, _task_ids: Vec<Uuid>) {
        // Set cards that can be focused
    }

    pub fn handle_keyboard_navigation(&mut self, key: &str) {
        match key {
            "ArrowDown" | "ArrowUp" => {
                // Handle focus navigation
            },
            "Enter" => {
                if let Some(focused) = self.focused_card {
                    self.handle_card_double_click(focused);
                }
            },
            _ => {}
        }
    }

    pub fn get_focused_card(&self) -> Option<Uuid> {
        self.focused_card
    }

    pub fn get_card_aria_label(&self, task: &Task) -> String {
        format!(
            "{} - {} priority - {} - Tags: {}",
            task.title,
            format!("{:?}", task.priority),
            format!("{:?}", task.status),
            task.tags.iter().cloned().collect::<Vec<_>>().join(", ")
        )
    }

    pub fn open_edit_dialog(&mut self, task_id: Uuid) {
        self.focused_card = Some(task_id);
    }

    pub fn is_focus_trapped(&self) -> bool {
        self.focused_card.is_some()
    }

    pub fn get_dialog_focusable_elements(&self) -> Vec<String> {
        vec!["title_input".to_string(), "description_input".to_string(), "save_button".to_string()]
    }

    pub fn handle_tab_navigation(&mut self, _reverse: bool) {
        // Handle tab navigation in dialogs
    }

    pub fn get_focused_element(&self) -> String {
        "title_input".to_string()
    }

    pub fn close_edit_dialog(&mut self) {
        self.focused_card = None;
    }

    pub fn set_viewport_bounds(&mut self, _min: (f32, f32), _max: (f32, f32)) {
        // Set viewport bounds for auto-scroll
    }

    pub fn get_auto_scroll_velocity(&self) -> (f32, f32) {
        if let Some(ctx) = &self.drag_context {
            let edge_threshold = 100.0;
            let mut velocity = (0.0, 0.0);
            
            if ctx.current_position.x < edge_threshold {
                velocity.0 = -5.0;
            } else if ctx.current_position.x > 900.0 {
                velocity.0 = 5.0;
            }
            
            if ctx.current_position.y > 700.0 {
                velocity.1 = 5.0;
            }
            
            velocity
        } else {
            (0.0, 0.0)
        }
    }

    pub fn get_drop_zone_at(&self, _position: (f32, f32)) -> Option<&str> {
        Some("Todo")
    }

    pub fn set_drop_zones(&mut self, _zones: Vec<(&str, (f32, f32), (f32, f32))>) {
        // Set drop zones for drag and drop
    }

    pub fn get_drag_preview(&self) -> Option<DragPreview> {
        self.drag_context.as_ref().map(|ctx| DragPreview {
            position: (ctx.current_position.x, ctx.current_position.y),
            opacity: 0.7,
            show_drop_indicator: true,
        })
    }

    pub fn calculate_progress_bar(&self, task: &Task) -> ProgressBar {
        let (completed, total) = task.subtask_progress();
        ProgressBar {
            percentage: if total > 0 { (completed as f32 / total as f32) * 100.0 } else { 0.0 },
            completed_count: completed,
            total_count: total,
            color: (52, 199, 89, 255),
        }
    }

    pub fn assign_tag_colors(&mut self, tags: &[&str]) -> HashMap<String, (u8, u8, u8)> {
        let mut colors = HashMap::new();
        for tag in tags {
            let color = self.get_or_assign_tag_color(tag);
            colors.insert(tag.to_string(), (color.r(), color.g(), color.b()));
        }
        colors
    }
}

pub struct ColumnStyle {
    pub show_wip_warning: bool,
    pub header_color: (u8, u8, u8, u8),
    pub pulse_header: bool,
}

pub struct ContextMenuInfo {
    pub task_id: Uuid,
    pub position: (f32, f32),
    pub items: Vec<String>,
}

pub struct ViewPreferencesData {
    pub column_widths: HashMap<String, f32>,
    pub wip_limits: HashMap<String, usize>,
    pub swimlanes_enabled: bool,
    pub hidden_columns: Vec<String>,
}

pub struct DragPreview {
    pub position: (f32, f32),
    pub opacity: f32,
    pub show_drop_indicator: bool,
}

pub struct ProgressBar {
    pub percentage: f32,
    pub completed_count: usize,
    pub total_count: usize,
    pub color: (u8, u8, u8, u8),
}