use super::kanban_view::{KanbanView, DragContext, CardAnimation, AnimationType, CardStyle, CardAction, EditMode, EasingType, LayoutMode};
use crate::domain::task::{Task, TaskStatus, Priority, SubTask};
use eframe::egui::{self, Pos2, Vec2, Rect, Color32, Key, Modifiers};
use uuid::Uuid;
use std::collections::HashSet;
use std::time::{Duration, Instant};

impl KanbanView {
    // Drag and Drop Core Methods
    pub fn start_drag(&mut self, task_id: Uuid, start_pos: Pos2) {
        let task = self.tasks.iter().find(|t| t.id == task_id);
        if let Some(task) = task {
            self.drag_context = Some(DragContext {
                task_id,
                start_position: start_pos,
                current_position: start_pos,
                offset: Vec2::ZERO,
                selected_tasks: HashSet::from([task_id]),
                is_multi_select: false,
                original_status: task.status.clone(),
                drag_velocity: Vec2::ZERO,
                last_update_time: Instant::now(),
            });
            
            // Start drag animation
            self.start_card_animation(task_id, AnimationType::Drag);
        }
    }
    
    pub fn start_multi_drag(&mut self, task_id: Uuid, start_pos: Pos2) {
        if !self.selected_tasks.is_empty() {
            let task = self.tasks.iter().find(|t| t.id == task_id);
            if let Some(task) = task {
                self.drag_context = Some(DragContext {
                    task_id,
                    start_position: start_pos,
                    current_position: start_pos,
                    offset: Vec2::ZERO,
                    selected_tasks: self.selected_tasks.clone(),
                    is_multi_select: true,
                    original_status: task.status.clone(),
                    drag_velocity: Vec2::ZERO,
                    last_update_time: Instant::now(),
                });
                
                // Start animations for all selected tasks
                for &id in &self.selected_tasks {
                    self.start_card_animation(id, AnimationType::Drag);
                }
            }
        }
    }
    
    pub fn update_drag_position(&mut self, new_pos: Pos2) {
        if let Some(ref mut ctx) = self.drag_context {
            let now = Instant::now();
            let dt = now.duration_since(ctx.last_update_time).as_secs_f32();
            
            if dt > 0.0 {
                ctx.drag_velocity = (new_pos - ctx.current_position) / dt;
            }
            
            ctx.current_position = new_pos;
            ctx.offset = new_pos - ctx.start_position;
            ctx.last_update_time = now;
        }
    }
    
    pub fn get_column_at_position(&self, pos: Pos2) -> Option<usize> {
        for (i, column) in self.columns.iter().enumerate() {
            if column.bounds.contains(pos) && !column.is_collapsed {
                return Some(i);
            }
        }
        None
    }
    
    pub fn drop_task_at_column(&mut self, column_index: usize) {
        if let Some(ctx) = &self.drag_context {
            let new_status = self.columns[column_index].status.clone();
            
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == ctx.task_id) {
                task.status = new_status;
                
                // End animation
                self.start_card_animation(ctx.task_id, AnimationType::Drop);
            }
        }
        
        self.drag_context = None;
    }
    
    pub fn drop_tasks_at_column(&mut self, column_index: usize) {
        if let Some(ctx) = &self.drag_context {
            let new_status = self.columns[column_index].status.clone();
            
            for task_id in &ctx.selected_tasks {
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *task_id) {
                    task.status = new_status.clone();
                    self.start_card_animation(*task_id, AnimationType::Drop);
                }
            }
        }
        
        self.drag_context = None;
        self.selected_tasks.clear();
    }
    
    pub fn drop_task_at_position(&mut self, column_index: usize, position: usize) {
        if let Some(ctx) = &self.drag_context {
            let new_status = self.columns[column_index].status.clone();
            
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == ctx.task_id) {
                task.status = new_status;
                // In a real implementation, we'd also update the task's position within the column
            }
        }
        
        self.drag_context = None;
    }
    
    pub fn drop_task_between(&mut self, column_index: usize, before_index: usize, after_index: usize) {
        if let Some(ctx) = &self.drag_context {
            let new_status = self.columns[column_index].status.clone();
            
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == ctx.task_id) {
                task.status = new_status;
                // In a real implementation, position the task between before_index and after_index
            }
        }
        
        self.drag_context = None;
    }
    
    // Auto-scroll methods
    pub fn should_auto_scroll(&self) -> bool {
        if let Some(ctx) = &self.drag_context {
            let edge_threshold = 50.0;
            let pos = ctx.current_position;
            let bounds = self.view_bounds;
            
            pos.x < bounds.min.x + edge_threshold ||
            pos.x > bounds.max.x - edge_threshold ||
            pos.y < bounds.min.y + edge_threshold ||
            pos.y > bounds.max.y - edge_threshold
        } else {
            false
        }
    }
    
    pub fn get_scroll_direction(&self) -> Vec2 {
        if let Some(ctx) = &self.drag_context {
            let edge_threshold = 50.0;
            let pos = ctx.current_position;
            let bounds = self.view_bounds;
            let mut dir = Vec2::ZERO;
            
            if pos.x < bounds.min.x + edge_threshold {
                dir.x = -1.0;
            } else if pos.x > bounds.max.x - edge_threshold {
                dir.x = 1.0;
            }
            
            if pos.y < bounds.min.y + edge_threshold {
                dir.y = -1.0;
            } else if pos.y > bounds.max.y - edge_threshold {
                dir.y = 1.0;
            }
            
            dir
        } else {
            Vec2::ZERO
        }
    }
    
    // Drag preview methods
    pub fn should_show_drag_preview(&self) -> bool {
        self.drag_context.is_some()
    }
    
    pub fn get_drag_preview_opacity(&self) -> f32 {
        0.7
    }
    
    pub fn get_drag_preview_bounds(&self) -> Option<Rect> {
        if let Some(ctx) = &self.drag_context {
            Some(Rect::from_center_size(
                ctx.current_position,
                Vec2::new(200.0, 80.0)
            ))
        } else {
            None
        }
    }
    
    // Keyboard handling
    pub fn handle_escape_key(&mut self) {
        self.drag_context = None;
        self.selected_tasks.clear();
    }
    
    pub fn handle_key_press(&mut self, key: Key) {
        match key {
            Key::Escape => self.handle_escape_key(),
            Key::ArrowLeft => self.move_to_previous_column(),
            Key::ArrowRight => self.move_to_next_column(),
            Key::ArrowUp => self.move_up_in_column(),
            Key::ArrowDown => self.move_down_in_column(),
            Key::Enter => self.confirm_drop(),
            _ => {}
        }
    }
    
    fn move_to_previous_column(&mut self) {
        // Implementation for keyboard navigation
    }
    
    fn move_to_next_column(&mut self) {
        // Implementation for keyboard navigation
    }
    
    fn move_up_in_column(&mut self) {
        // Implementation for keyboard navigation
    }
    
    fn move_down_in_column(&mut self) {
        // Implementation for keyboard navigation
    }
    
    fn confirm_drop(&mut self) {
        if let Some(target_col) = self.get_target_column() {
            self.drop_task_at_column(target_col);
        }
    }
    
    pub fn get_target_column(&self) -> Option<usize> {
        if let Some(ctx) = &self.drag_context {
            self.get_column_at_position(ctx.current_position)
        } else {
            None
        }
    }
    
    pub fn get_target_position_in_column(&self) -> Option<usize> {
        // Return the position within the column based on current drag position
        Some(0) // Simplified implementation
    }
    
    // Drop indicator methods
    pub fn should_show_drop_indicator(&self) -> bool {
        if let Some(ctx) = &self.drag_context {
            self.get_column_at_position(ctx.current_position).is_some()
        } else {
            false
        }
    }
    
    pub fn get_drop_indicator_column(&self) -> Option<usize> {
        if let Some(ctx) = &self.drag_context {
            self.get_column_at_position(ctx.current_position)
        } else {
            None
        }
    }
    
    // WIP limit checking
    pub fn can_drop_in_column(&self, column_index: usize) -> bool {
        let column = &self.columns[column_index];
        
        if let Some(limit) = column.wip_limit {
            let current_count = self.tasks.iter()
                .filter(|t| t.status == column.status)
                .count();
            
            if let Some(ctx) = &self.drag_context {
                let moving_out = self.tasks.iter()
                    .filter(|t| ctx.selected_tasks.contains(&t.id))
                    .any(|t| t.status == column.status);
                    
                if moving_out {
                    return true; // Moving within same column is allowed
                }
                
                let moving_in_count = ctx.selected_tasks.len();
                return current_count + moving_in_count <= limit;
            }
            
            current_count < limit
        } else {
            true
        }
    }
    
    pub fn should_show_wip_warning(&self, column_index: usize) -> bool {
        !self.can_drop_in_column(column_index)
    }
    
    // Animation methods
    pub fn get_animated_drag_position(&self, t: f32) -> Pos2 {
        if let Some(ctx) = &self.drag_context {
            ctx.start_position.lerp(ctx.current_position, t)
        } else {
            Pos2::ZERO
        }
    }
    
    pub fn start_card_animation(&mut self, task_id: Uuid, animation_type: AnimationType) {
        let animation = CardAnimation {
            start_pos: Pos2::ZERO,
            end_pos: Pos2::ZERO,
            start_time: 0.0,
            duration: 0.3,
            opacity: 1.0,
            scale: 1.0,
            animation_type,
        };
        
        self.card_animations.insert(task_id, animation);
    }
    
    pub fn get_animation_progress(&self, task_id: Uuid, normalized_time: f32) -> f32 {
        normalized_time.clamp(0.0, 1.0)
    }
    
    pub fn apply_easing(&self, t: f32, easing_type: EasingType) -> f32 {
        match easing_type {
            EasingType::Linear => t,
            EasingType::EaseIn => t * t,
            EasingType::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingType::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
    
    // Touch support
    pub fn handle_touch_start(&mut self, task_id: Uuid, pos: Pos2) {
        self.start_drag(task_id, pos);
    }
    
    pub fn handle_touch_move(&mut self, pos: Pos2) {
        self.update_drag_position(pos);
    }
    
    pub fn handle_touch_end(&mut self, pos: Pos2) {
        if let Some(column_index) = self.get_column_at_position(pos) {
            self.drop_task_at_column(column_index);
        } else {
            self.drag_context = None;
        }
    }
    
    // Velocity tracking
    pub fn record_drag_velocity(&mut self) {
        // Velocity is already recorded in update_drag_position
    }
    
    pub fn get_drag_velocity(&self) -> Vec2 {
        if let Some(ctx) = &self.drag_context {
            ctx.drag_velocity
        } else {
            Vec2::ZERO
        }
    }
    
    // Accessibility
    pub fn get_accessibility_announcement(&self) -> String {
        if let Some(ctx) = &self.drag_context {
            if let Some(col_idx) = self.get_column_at_position(ctx.current_position) {
                format!("Dragging task. Over column {}", self.columns[col_idx].title)
            } else {
                "Dragging task".to_string()
            }
        } else {
            "".to_string()
        }
    }
}