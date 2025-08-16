use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui::widgets::task_detail_modal::TaskDetailModal;
use crate::repository::comment_repository::CommentRepository;
use eframe::egui::{self, Ui, Rect, Pos2, Vec2, Color32, Rounding, Align2, Sense, CursorIcon, ScrollArea, lerp};
use std::collections::{HashMap, HashSet};
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct KanbanView {
    pub columns: Vec<KanbanColumn>,
    pub tasks: Vec<Task>,
    pub drag_state: Option<DragState>,
    pub animations: AnimationManager,
    pub selected_tasks: HashSet<Uuid>,
    pub search_filter: String,
    pub selected_task_id: Option<Uuid>,
    pub quick_add_states: HashMap<String, QuickAddState>,
    pub wip_limits: HashMap<String, usize>,
    pub column_collapse_state: HashMap<String, bool>,
    pub viewport_width: f32,
    pub task_detail_modal: TaskDetailModal,
    pub comment_repository: Option<Arc<CommentRepository>>,
    pub enable_smooth_animations: bool,
    pub animation_duration: Duration,
}

#[derive(Clone)]
pub struct KanbanColumn {
    pub id: Uuid,
    pub title: String,
    pub status: TaskStatus,
    pub color: Color32,
    pub task_order: Vec<Uuid>, // Ordered list of task IDs
    pub width: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub collapsed: bool,
    pub wip_limit: Option<usize>,
    pub bounds: Rect,
    pub visible: bool,
    pub position: usize,
}

pub struct DragState {
    pub task_id: Uuid,
    pub start_position: Pos2,
    pub current_position: Pos2,
    pub offset: Vec2,
    pub original_column: usize,
    pub original_position: usize,
    pub hover_column: Option<usize>,
    pub hover_position: Option<usize>,
    pub is_dragging_actual_card: bool,
    pub drag_start_time: Instant,
}

pub struct AnimationManager {
    pub card_animations: HashMap<Uuid, CardAnimation>,
    pub gap_animations: HashMap<(usize, usize), GapAnimation>,
    pub drop_animations: HashMap<Uuid, DropAnimation>,
    pub last_update: Instant,
}

#[derive(Clone)]
pub struct CardAnimation {
    pub start_pos: Vec2,
    pub target_pos: Vec2,
    pub current_pos: Vec2,
    pub start_time: Instant,
    pub duration: Duration,
    pub is_complete: bool,
}

#[derive(Clone)]
pub struct GapAnimation {
    pub start_height: f32,
    pub target_height: f32,
    pub current_height: f32,
    pub start_time: Instant,
    pub duration: Duration,
}

#[derive(Clone)]
pub struct DropAnimation {
    pub start_pos: Pos2,
    pub target_pos: Pos2,
    pub current_pos: Pos2,
    pub start_time: Instant,
    pub duration: Duration,
    pub is_animating: bool,
}

#[derive(Clone)]
pub struct QuickAddState {
    pub visible: bool,
    pub text: String,
}

#[derive(Clone)]
pub struct EmptyColumnDropIndicator {
    pub visible: bool,
    pub opacity: f32,
}

impl KanbanView {
    pub fn new() -> Self {
        let spacing = 16.0;
        let column_width = 300.0;
        let x_offset = spacing;
        
        let columns = vec![
            KanbanColumn {
                id: Uuid::new_v4(),
                title: "To Do".to_string(),
                status: TaskStatus::Todo,
                color: Color32::from_rgb(200, 200, 200),
                task_order: Vec::new(),
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
                task_order: Vec::new(),
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
                task_order: Vec::new(),
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
                task_order: Vec::new(),
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
            drag_state: None,
            animations: AnimationManager {
                card_animations: HashMap::new(),
                gap_animations: HashMap::new(),
                drop_animations: HashMap::new(),
                last_update: Instant::now(),
            },
            selected_tasks: HashSet::new(),
            search_filter: String::new(),
            selected_task_id: None,
            quick_add_states: HashMap::new(),
            wip_limits,
            column_collapse_state: HashMap::new(),
            viewport_width: 1200.0,
            task_detail_modal: TaskDetailModal::new(),
            comment_repository: None,
            enable_smooth_animations: true,
            animation_duration: Duration::from_millis(200),
        };
        
        instance.update_layout(1200.0);
        instance
    }

    // Enhanced drag and drop with animations
    pub fn start_drag_with_animation(&mut self, task_id: Uuid, position: Pos2) {
        if let Some((col_idx, task_pos)) = self.find_task_position(task_id) {
            self.drag_state = Some(DragState {
                task_id,
                start_position: position,
                current_position: position,
                offset: Vec2::ZERO,
                original_column: col_idx,
                original_position: task_pos,
                hover_column: None,
                hover_position: None,
                is_dragging_actual_card: true,
                drag_start_time: Instant::now(),
            });
        }
    }

    pub fn update_drag_with_animation(&mut self, position: Pos2) {
        let (old_hover, task_id, needs_animation) = if let Some(state) = &self.drag_state {
            let old = (state.hover_column, state.hover_position);
            (old, state.task_id, true)
        } else {
            return;
        };
        
        // Update drag state
        if let Some(state) = &mut self.drag_state {
            state.current_position = position;
        }
        
        // Calculate new positions
        let new_column = self.get_column_at_position(position);
        let new_position = if let Some(col_idx) = new_column {
            self.calculate_hover_position(col_idx, position.y)
        } else {
            None
        };
        
        // Update state with new positions
        if let Some(state) = &mut self.drag_state {
            state.hover_column = new_column;
            state.hover_position = new_position;
        }
        
        // Trigger animation if positions changed
        let new_hover = (new_column, new_position);
        if old_hover != new_hover && needs_animation {
            self.animate_gap_change(old_hover, new_hover, task_id);
        }
    }

    fn animate_gap_change(&mut self, old_hover: (Option<usize>, Option<usize>), new_hover: (Option<usize>, Option<usize>), task_id: Uuid) {
        // Close old gap
        if let (Some(col), Some(pos)) = old_hover {
            let key = (col, pos);
            self.animations.gap_animations.insert(key, GapAnimation {
                start_height: self.get_animated_gap_height(col, pos),
                target_height: 0.0,
                current_height: self.get_animated_gap_height(col, pos),
                start_time: Instant::now(),
                duration: self.animation_duration,
            });
        }
        
        // Open new gap
        if let (Some(col), Some(pos)) = new_hover {
            let key = (col, pos);
            let task_height = self.calculate_card_height_for_task(task_id);
            self.animations.gap_animations.insert(key, GapAnimation {
                start_height: 0.0,
                target_height: task_height + 8.0, // Include spacing
                current_height: 0.0,
                start_time: Instant::now(),
                duration: self.animation_duration,
            });
            
            // Animate cards shifting
            self.animate_cards_shift(col, pos, task_height + 8.0);
        }
    }

    fn animate_cards_shift(&mut self, column: usize, insert_pos: usize, gap_height: f32) {
        if column >= self.columns.len() {
            return;
        }
        
        let column_tasks = &self.columns[column].task_order;
        
        // Animate cards below the insertion point
        for (idx, &task_id) in column_tasks.iter().enumerate() {
            if idx >= insert_pos {
                let current_offset = self.animations.card_animations
                    .get(&task_id)
                    .map(|a| a.current_pos)
                    .unwrap_or(Vec2::ZERO);
                
                self.animations.card_animations.insert(task_id, CardAnimation {
                    start_pos: current_offset,
                    target_pos: Vec2::new(0.0, gap_height),
                    current_pos: current_offset,
                    start_time: Instant::now(),
                    duration: self.animation_duration,
                    is_complete: false,
                });
            }
        }
    }

    pub fn complete_drag_with_animation(&mut self, column_index: usize, position: usize) {
        if let Some(state) = &self.drag_state {
            let task_id = state.task_id;
            
            // Create drop animation
            self.animations.drop_animations.insert(task_id, DropAnimation {
                start_pos: state.current_position,
                target_pos: self.calculate_card_position(column_index, position),
                current_pos: state.current_position,
                start_time: Instant::now(),
                duration: Duration::from_millis(150),
                is_animating: true,
            });
            
            // Update task order
            self.reorder_task(task_id, column_index, position);
            
            // Clear animations
            self.clear_drag_animations();
        }
        
        self.drag_state = None;
    }

    fn reorder_task(&mut self, task_id: Uuid, target_column: usize, target_position: usize) {
        // Remove from original position
        for column in &mut self.columns {
            column.task_order.retain(|&id| id != task_id);
        }
        
        // Insert at new position
        if target_column < self.columns.len() {
            let column = &mut self.columns[target_column];
            let insert_pos = target_position.min(column.task_order.len());
            column.task_order.insert(insert_pos, task_id);
            
            // Update task status
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                task.status = column.status;
                // Store order in metadata
                task.metadata.insert("kanban_order".to_string(), insert_pos.to_string());
            }
        }
    }

    pub fn update_animations(&mut self, elapsed: Duration) {
        let now = Instant::now();
        
        // Update gap animations
        let mut completed_gaps = Vec::new();
        for (key, animation) in &mut self.animations.gap_animations {
            let progress = animation.start_time.elapsed().as_secs_f32() / animation.duration.as_secs_f32();
            if progress >= 1.0 {
                animation.current_height = animation.target_height;
                if animation.target_height == 0.0 {
                    completed_gaps.push(*key);
                }
            } else {
                let eased_progress = ease_in_out_cubic(progress);
                animation.current_height = lerp(animation.start_height..=animation.target_height, eased_progress);
            }
        }
        
        // Remove completed gap closing animations
        for key in completed_gaps {
            self.animations.gap_animations.remove(&key);
        }
        
        // Update card animations
        let mut completed_cards = Vec::new();
        for (id, animation) in &mut self.animations.card_animations {
            let progress = animation.start_time.elapsed().as_secs_f32() / animation.duration.as_secs_f32();
            if progress >= 1.0 {
                animation.current_pos = animation.target_pos;
                animation.is_complete = true;
                completed_cards.push(*id);
            } else {
                let eased_progress = ease_in_out_cubic(progress);
                animation.current_pos = Vec2::new(
                    lerp(animation.start_pos.x..=animation.target_pos.x, eased_progress),
                    lerp(animation.start_pos.y..=animation.target_pos.y, eased_progress),
                );
            }
        }
        
        // Remove completed card animations
        for id in completed_cards {
            self.animations.card_animations.remove(&id);
        }
        
        // Update drop animations
        let mut completed_drops = Vec::new();
        for (id, animation) in &mut self.animations.drop_animations {
            let progress = animation.start_time.elapsed().as_secs_f32() / animation.duration.as_secs_f32();
            if progress >= 1.0 {
                animation.current_pos = animation.target_pos;
                animation.is_animating = false;
                completed_drops.push(*id);
            } else {
                let eased_progress = ease_in_out_cubic(progress);
                animation.current_pos = Pos2::new(
                    lerp(animation.start_pos.x..=animation.target_pos.x, eased_progress),
                    lerp(animation.start_pos.y..=animation.target_pos.y, eased_progress),
                );
            }
        }
        
        // Remove completed drop animations
        for id in completed_drops {
            self.animations.drop_animations.remove(&id);
        }
        
        self.animations.last_update = now;
    }

    // Helper methods for animations
    pub fn get_animated_gap_height(&self, column: usize, position: usize) -> f32 {
        self.animations.gap_animations
            .get(&(column, position))
            .map(|a| a.current_height)
            .unwrap_or(0.0)
    }

    pub fn get_card_animation_offset(&self, _column: usize, task_idx: usize) -> Vec2 {
        // Get task ID from column
        if let Some(column) = self.columns.get(_column) {
            if let Some(&task_id) = column.task_order.get(task_idx) {
                return self.animations.card_animations
                    .get(&task_id)
                    .map(|a| a.current_pos)
                    .unwrap_or(Vec2::ZERO);
            }
        }
        Vec2::ZERO
    }

    pub fn set_hover_insert_position(&mut self, column: usize, position: usize) {
        let (old_hover, task_id) = if let Some(state) = &self.drag_state {
            ((state.hover_column, state.hover_position), state.task_id)
        } else {
            return;
        };
        
        if let Some(state) = &mut self.drag_state {
            state.hover_column = Some(column);
            state.hover_position = Some(position);
        }
        
        let new_hover = (Some(column), Some(position));
        self.animate_gap_change(old_hover, new_hover, task_id);
    }

    pub fn clear_hover_insert_position(&mut self) {
        let (old_hover, task_id) = if let Some(state) = &self.drag_state {
            ((state.hover_column, state.hover_position), state.task_id)
        } else {
            return;
        };
        
        if let Some(state) = &mut self.drag_state {
            state.hover_column = None;
            state.hover_position = None;
        }
        
        self.animate_gap_change(old_hover, (None, None), task_id);
    }

    pub fn get_drop_animation_state(&self, task_id: Uuid) -> DropAnimation {
        self.animations.drop_animations
            .get(&task_id)
            .cloned()
            .unwrap_or(DropAnimation {
                start_pos: Pos2::ZERO,
                target_pos: Pos2::ZERO,
                current_pos: Pos2::ZERO,
                start_time: Instant::now(),
                duration: Duration::from_millis(0),
                is_animating: false,
            })
    }

    pub fn get_card_opacity(&self, task_id: Uuid) -> f32 {
        if let Some(state) = &self.drag_state {
            if state.task_id == task_id && state.is_dragging_actual_card {
                return 0.3; // Make original position semi-transparent
            }
        }
        1.0
    }

    pub fn get_dragged_card_opacity(&self) -> f32 {
        0.9 // Slightly transparent dragged card
    }

    pub fn get_empty_column_drop_indicator(&self, column: usize) -> EmptyColumnDropIndicator {
        if let Some(state) = &self.drag_state {
            if state.hover_column == Some(column) && self.columns[column].task_order.is_empty() {
                return EmptyColumnDropIndicator {
                    visible: true,
                    opacity: 0.5,
                };
            }
        }
        EmptyColumnDropIndicator {
            visible: false,
            opacity: 0.0,
        }
    }

    pub fn handle_scroll_offset(&mut self, _offset: Vec2) {
        // Animations continue independently of scroll
    }

    pub fn get_card_position(&self, task_id: Uuid) -> Option<Pos2> {
        for (col_idx, column) in self.columns.iter().enumerate() {
            if let Some(pos_idx) = column.task_order.iter().position(|&id| id == task_id) {
                return Some(self.calculate_card_position(col_idx, pos_idx));
            }
        }
        None
    }

    pub fn calculate_insert_position(&self, column: usize, position: usize) -> Pos2 {
        if column >= self.columns.len() {
            return Pos2::ZERO;
        }
        
        let column_bounds = self.columns[column].bounds;
        let y_offset = 100.0 + (position as f32 * (80.0 + 8.0)); // Card height + spacing
        
        Pos2::new(column_bounds.min.x + 10.0, column_bounds.min.y + y_offset)
    }

    fn calculate_card_position(&self, column: usize, position: usize) -> Pos2 {
        self.calculate_insert_position(column, position)
    }

    fn calculate_hover_position(&self, column: usize, y: f32) -> Option<usize> {
        if column >= self.columns.len() {
            return None;
        }
        
        let column_bounds = self.columns[column].bounds;
        let relative_y = y - column_bounds.min.y - 100.0; // Subtract header height
        let card_height = 80.0 + 8.0; // Card height + spacing
        let position = (relative_y / card_height).floor().max(0.0) as usize;
        
        Some(position.min(self.columns[column].task_order.len()))
    }

    fn calculate_card_height_for_task(&self, task_id: Uuid) -> f32 {
        self.tasks.iter()
            .find(|t| t.id == task_id)
            .map(|t| self.calculate_card_height(t))
            .unwrap_or(80.0)
    }

    fn clear_drag_animations(&mut self) {
        // Clear all gap animations
        self.animations.gap_animations.clear();
        
        // Reset card animations
        self.animations.card_animations.clear();
    }

    // Core drag and drop functionality (compatibility)
    pub fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    pub fn get_dragging_task_id(&self) -> Option<Uuid> {
        self.drag_state.as_ref().map(|s| s.task_id)
    }

    pub fn start_drag(&mut self, task_id: Uuid, position: Pos2) {
        self.start_drag_with_animation(task_id, position);
    }

    pub fn update_drag_position(&mut self, position: Pos2) {
        self.update_drag_with_animation(position);
    }

    pub fn get_drag_position(&self) -> Option<Pos2> {
        self.drag_state.as_ref().map(|s| s.current_position)
    }

    pub fn complete_drag(&mut self, target_column: usize) {
        if let Some(state) = &self.drag_state {
            let position = self.columns[target_column].task_order.len();
            self.complete_drag_with_animation(target_column, position);
        }
    }

    pub fn complete_drag_with_reorder(&mut self, column_index: usize, position: usize) {
        self.complete_drag_with_animation(column_index, position);
    }

    pub fn cancel_drag(&mut self) {
        self.clear_drag_animations();
        self.drag_state = None;
    }

    // Column management
    pub fn is_over_column(&self, position: Pos2, column_index: usize) -> bool {
        if column_index >= self.columns.len() {
            return false;
        }
        self.columns[column_index].bounds.contains(position)
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
        
        // Use the ordered task list
        column.task_order.iter()
            .filter_map(|task_id| {
                self.tasks.iter().find(|t| t.id == *task_id && self.matches_filter(t))
            })
            .collect()
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
    pub fn add_task(&mut self, mut task: Task) {
        // Find the appropriate column for this task based on its status
        for column in self.columns.iter_mut() {
            if column.status == task.status {
                // Add order metadata
                let order = column.task_order.len();
                task.metadata.insert("kanban_order".to_string(), order.to_string());
                column.task_order.push(task.id);
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
        
        for &task_id in &self.selected_tasks.clone() {
            // Remove from current column
            for column in &mut self.columns {
                column.task_order.retain(|&id| id != task_id);
            }
            
            // Add to target column
            self.columns[target_column].task_order.push(task_id);
            
            // Update task status
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                task.status = target_status;
                let order = self.columns[target_column].task_order.len() - 1;
                task.metadata.insert("kanban_order".to_string(), order.to_string());
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
        
        let order = self.columns[column_index].task_order.len();
        task.metadata.insert("kanban_order".to_string(), order.to_string());
        
        self.columns[column_index].task_order.push(task.id);
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
        
        let spacing = 16.0;
        let total_spacing = spacing * 2.0;
        let column_spacing = spacing * (visible_columns - 1) as f32;
        
        let available_for_columns = available_width - total_spacing - column_spacing;
        let calculated_width = available_for_columns / visible_columns as f32;
        
        if available_width >= 1000.0 && visible_columns <= 4 {
            calculated_width.min(400.0).max(320.0)
        } else {
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
                    if let Some((col_idx, pos_idx)) = self.find_task_position(task_id) {
                        if col_idx + 1 < self.columns.len() {
                            self.reorder_task(task_id, col_idx + 1, self.columns[col_idx + 1].task_order.len());
                        }
                    }
                }
                egui::Key::ArrowLeft if !modifiers.any() => {
                    if let Some((col_idx, pos_idx)) = self.find_task_position(task_id) {
                        if col_idx > 0 {
                            self.reorder_task(task_id, col_idx - 1, self.columns[col_idx - 1].task_order.len());
                        }
                    }
                }
                egui::Key::ArrowUp if !modifiers.any() => {
                    if let Some((col_idx, pos_idx)) = self.find_task_position(task_id) {
                        if pos_idx > 0 {
                            self.reorder_task(task_id, col_idx, pos_idx - 1);
                        }
                    }
                }
                egui::Key::ArrowDown if !modifiers.any() => {
                    if let Some((col_idx, pos_idx)) = self.find_task_position(task_id) {
                        if pos_idx < self.columns[col_idx].task_order.len() - 1 {
                            self.reorder_task(task_id, col_idx, pos_idx + 1);
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
            if let Some(task_idx) = column.task_order.iter().position(|&id| id == task_id) {
                return Some((col_idx, task_idx));
            }
        }
        None
    }

    // Main render method
    pub fn show(&mut self, ui: &mut Ui, tasks: &mut Vec<Task>) {
        self.tasks = tasks.clone();
        
        // Sync task positions if needed
        let task_ids: Vec<Uuid> = self.tasks.iter().map(|t| t.id).collect();
        for task_id in task_ids {
            let task_in_column = self.columns.iter()
                .any(|col| col.task_order.contains(&task_id));
            
            if !task_in_column {
                // Find the task's status
                let task_status = self.tasks.iter()
                    .find(|t| t.id == task_id)
                    .map(|t| t.status);
                
                if let Some(status) = task_status {
                    for column in self.columns.iter_mut() {
                        if column.status == status {
                            let order = column.task_order.len();
                            column.task_order.push(task_id);
                            if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                                t.metadata.insert("kanban_order".to_string(), order.to_string());
                            }
                            break;
                        }
                    }
                }
            }
        }
        
        // Update animations
        self.update_animations(self.animations.last_update.elapsed());
        
        // Update layout
        let available_width = ui.available_width();
        let available_height = ui.available_height();
        self.update_layout_with_height(available_width, available_height);
        
        // Header
        ui.horizontal(|ui| {
            ui.heading("📋 Kanban Board");
            
            ui.separator();
            
            ui.label("🔍");
            let search_response = ui.text_edit_singleline(&mut self.search_filter);
            if search_response.changed() {
                // Filter is applied automatically
            }
        });
        
        ui.separator();
        
        // Kanban board
        ScrollArea::horizontal()
            .id_source("kanban_main_horizontal_scroll")
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.set_min_height(available_height);
                    
                    for col_idx in 0..self.columns.len() {
                        if !self.columns[col_idx].visible {
                            continue;
                        }
                        
                        let column_rect = Rect::from_min_size(
                            ui.cursor().min,
                            Vec2::new(
                                if self.columns[col_idx].collapsed { 50.0 } else { self.columns[col_idx].width },
                                ui.available_height()
                            )
                        );
                        
                        self.columns[col_idx].bounds = column_rect;
                        
                        ui.allocate_ui_at_rect(column_rect, |ui| {
                            self.render_column(ui, col_idx);
                        });
                        
                        ui.add_space(8.0);
                    }
                });
            });
        
        // Render dragged card on top
        if let Some(state) = &self.drag_state {
            if state.is_dragging_actual_card {
                if let Some(task) = self.tasks.iter().find(|t| t.id == state.task_id) {
                    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                    
                    let painter = ui.painter();
                    let card_size = Vec2::new(
                        self.columns.get(state.original_column)
                            .map(|c| c.width - 20.0)
                            .unwrap_or(280.0),
                        self.calculate_card_height(task)
                    );
                    
                    let card_rect = Rect::from_min_size(
                        state.current_position - card_size / 2.0,
                        card_size
                    );
                    
                    // Shadow
                    painter.rect(
                        card_rect.translate(Vec2::new(2.0, 2.0)),
                        Rounding::same(4.0),
                        Color32::from_rgba_unmultiplied(0, 0, 0, 50),
                        egui::Stroke::NONE
                    );
                    
                    // Card
                    painter.rect_filled(
                        card_rect,
                        Rounding::same(4.0),
                        Color32::from_rgba_unmultiplied(250, 250, 250, 230)
                    );
                    
                    // Title
                    painter.text(
                        card_rect.min + Vec2::new(10.0, 10.0),
                        Align2::LEFT_TOP,
                        &task.title,
                        egui::FontId::default(),
                        Color32::BLACK
                    );
                }
            }
        }
        
        // Update tasks back
        *tasks = self.tasks.clone();
    }

    fn render_column(&mut self, ui: &mut Ui, column_index: usize) {
        let column_collapsed = self.columns[column_index].collapsed;
        let column_color = self.columns[column_index].color;
        let column_title = self.columns[column_index].title.clone();
        let column_wip_limit = self.columns[column_index].wip_limit;
        
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
                
                let scroll_height = ui.available_height() - 50.0;
                ScrollArea::vertical()
                    .id_source(format!("kanban_column_scroll_{}", column_index))
                    .max_height(scroll_height.max(400.0))
                    .show(ui, |ui| {
                        // Empty column drop indicator
                        if tasks.is_empty() {
                            let indicator = self.get_empty_column_drop_indicator(column_index);
                            if indicator.visible {
                                let rect = Rect::from_min_size(
                                    ui.cursor().min,
                                    Vec2::new(self.columns[column_index].width - 20.0, 100.0)
                                );
                                ui.painter().rect(
                                    rect,
                                    Rounding::same(4.0),
                                    Color32::from_rgba_unmultiplied(100, 150, 255, (indicator.opacity * 255.0) as u8),
                                    egui::Stroke::new(2.0, Color32::from_rgb(100, 150, 255))
                                );
                            } else {
                                ui.centered_and_justified(|ui| {
                                    ui.label(self.get_empty_column_message(column_index));
                                });
                            }
                        } else {
                            let mut cumulative_offset = 0.0;
                            
                            for (idx, task) in tasks.iter().enumerate() {
                                // Check for gap animation before this card
                                let gap_height = self.get_animated_gap_height(column_index, idx);
                                if gap_height > 0.0 {
                                    ui.add_space(gap_height);
                                    cumulative_offset += gap_height;
                                }
                                
                                // Apply animation offset
                                let offset = self.get_card_animation_offset(column_index, idx);
                                if offset.y != 0.0 {
                                    ui.add_space(offset.y - cumulative_offset);
                                    cumulative_offset = offset.y;
                                }
                                
                                self.render_task_card(ui, task, column_index);
                                ui.add_space(self.get_card_spacing());
                            }
                            
                            // Check for gap at the end
                            let end_gap = self.get_animated_gap_height(column_index, tasks.len());
                            if end_gap > 0.0 {
                                ui.add_space(end_gap);
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
        let opacity = self.get_card_opacity(task.id);
        
        let base_color = if is_selected {
            Color32::from_rgb(200, 220, 255)
        } else if is_overdue {
            Color32::from_rgb(255, 200, 200)
        } else {
            Color32::from_rgb(250, 250, 250)
        };
        
        let card_color = Color32::from_rgba_unmultiplied(
            base_color.r(),
            base_color.g(),
            base_color.b(),
            (base_color.a() as f32 * opacity) as u8
        );
        
        let response = ui.allocate_response(
            Vec2::new(self.columns[column_index].width - 20.0, self.calculate_card_height(task)),
            Sense::click_and_drag()
        );
        
        // Handle interactions
        if response.clicked() {
            self.select_task(task.id);
        }
        
        if response.drag_started() {
            self.start_drag_with_animation(task.id, response.interact_pointer_pos().unwrap_or(Pos2::ZERO));
        }
        
        if self.is_dragging() {
            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                self.update_drag_with_animation(pos);
            }
        }
        
        if response.drag_released() && self.is_dragging() {
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some(target_col) = self.get_column_at_position(pos) {
                    if let Some(hover_pos) = self.calculate_hover_position(target_col, pos.y) {
                        self.complete_drag_with_animation(target_col, hover_pos);
                    }
                } else {
                    self.cancel_drag();
                }
            }
        }
        
        // Draw card
        ui.painter().rect(
            response.rect,
            Rounding::same(4.0),
            card_color,
            egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(128, 128, 128, (opacity * 255.0) as u8))
        );
        
        // Card content with opacity
        ui.allocate_ui_at_rect(response.rect.shrink(8.0), |ui| {
            ui.vertical(|ui| {
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
                        Color32::from_rgba_unmultiplied(
                            priority_color.r(),
                            priority_color.g(),
                            priority_color.b(),
                            (priority_color.a() as f32 * opacity) as u8
                        )
                    );
                    ui.add_space(10.0);
                    ui.label(&task.title);
                });
                
                if !task.description.is_empty() {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(&task.description)
                            .small()
                            .color(Color32::from_rgba_unmultiplied(128, 128, 128, (opacity * 200.0) as u8))
                    );
                }
                
                if !task.tags.is_empty() {
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        for tag in &task.tags {
                            ui.label(
                                egui::RichText::new(format!("#{}", tag))
                                    .small()
                                    .color(Color32::from_rgba_unmultiplied(100, 150, 200, (opacity * 255.0) as u8))
                            );
                        }
                    });
                }
                
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
                        Color32::from_rgba_unmultiplied(
                            date_color.r(),
                            date_color.g(),
                            date_color.b(),
                            (date_color.a() as f32 * opacity) as u8
                        ),
                        format!("📅 {}", due.format("%b %d"))
                    );
                }
                
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

// Easing function for smooth animations
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}