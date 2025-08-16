use super::kanban_view::{KanbanView, KanbanColumn, LayoutMode};
use crate::domain::task::{Task, TaskStatus};
use eframe::egui::{Pos2, Rect, Vec2};
use uuid::Uuid;

impl KanbanView {
    // Responsive column width calculation
    pub fn calculate_responsive_column_widths(&mut self, viewport_width: f32) {
        let gap = 16.0;
        let visible_columns: Vec<_> = self.columns.iter()
            .filter(|c| !c.is_collapsed && c.visible)
            .collect();
        
        if visible_columns.is_empty() {
            return;
        }
        
        let total_gaps = gap * (visible_columns.len() - 1) as f32;
        let available_width = viewport_width - total_gaps;
        let width_per_column = available_width / visible_columns.len() as f32;
        
        for column in &mut self.columns {
            if !column.is_collapsed && column.visible {
                column.width = width_per_column.clamp(column.min_width, column.max_width);
            }
        }
    }
    
    // Column resizing
    pub fn resize_column(&mut self, column_index: usize, new_width: f32) {
        if let Some(column) = self.columns.get_mut(column_index) {
            column.width = new_width.clamp(column.min_width, column.max_width);
        }
    }
    
    pub fn resize_column_with_neighbor(&mut self, column_index: usize, new_width: f32) {
        if !self.enable_neighbor_resize || column_index >= self.columns.len() - 1 {
            self.resize_column(column_index, new_width);
            return;
        }
        
        let old_width = self.columns[column_index].width;
        let diff = new_width - old_width;
        
        self.columns[column_index].width = new_width.clamp(
            self.columns[column_index].min_width,
            self.columns[column_index].max_width
        );
        
        // Adjust neighbor column
        let actual_diff = self.columns[column_index].width - old_width;
        self.columns[column_index + 1].width = (self.columns[column_index + 1].width - actual_diff)
            .clamp(
                self.columns[column_index + 1].min_width,
                self.columns[column_index + 1].max_width
            );
    }
    
    // Resize handle detection
    pub fn is_over_resize_handle(&self, pos: Pos2, column_index: usize) -> bool {
        if let Some(column) = self.columns.get(column_index) {
            let handle_width = 8.0;
            let right_edge = column.bounds.max.x;
            pos.x >= right_edge - handle_width && pos.x <= right_edge
                && pos.y >= column.bounds.min.y && pos.y <= column.bounds.max.y
        } else {
            false
        }
    }
    
    // Column collapse/expand
    pub fn toggle_column_collapse(&mut self, column_index: usize) {
        if let Some(column) = self.columns.get_mut(column_index) {
            if column.is_collapsed {
                column.is_collapsed = false;
                column.width = 300.0; // Restore default width
            } else {
                column.is_collapsed = true;
                column.width = 50.0; // Collapsed width
            }
        }
    }
    
    // Viewport resize handling
    pub fn handle_viewport_resize(&mut self, viewport_width: f32) {
        self.viewport_width = viewport_width;
        self.calculate_responsive_column_widths(viewport_width);
    }
    
    // Column reordering
    pub fn reorder_columns(&mut self, from: usize, to: usize) {
        if from < self.columns.len() && to < self.columns.len() {
            let column = self.columns.remove(from);
            self.columns.insert(to, column);
            
            // Update positions
            for (i, col) in self.columns.iter_mut().enumerate() {
                col.position = i;
            }
        }
    }
    
    // Column header positioning
    pub fn get_column_header_position(&self, column_id: Uuid) -> Pos2 {
        if let Some(column) = self.columns.iter().find(|c| c.id == column_id) {
            Pos2::new(column.bounds.min.x, 0.0) // Headers stay at top
        } else {
            Pos2::ZERO
        }
    }
    
    // Column visibility
    pub fn is_column_visible(&self, column_index: usize) -> bool {
        self.columns.get(column_index)
            .map(|c| c.visible && !c.is_collapsed)
            .unwrap_or(false)
    }
    
    pub fn scroll_to_column(&mut self, column_index: usize) {
        if let Some(column) = self.columns.get(column_index) {
            let column_center = column.bounds.center().x;
            let viewport_center = self.viewport_width / 2.0;
            self.scroll_offset.x = column_center - viewport_center;
        }
    }
    
    // Column gap adjustment
    pub fn set_column_gap(&mut self, gap: f32) {
        // Store gap preference and recalculate positions
        self.calculate_column_positions();
    }
    
    pub fn calculate_column_positions(&mut self) {
        let gap = 16.0; // Default gap
        let mut x = 0.0;
        
        for column in &mut self.columns {
            if column.visible {
                column.bounds = Rect::from_min_size(
                    Pos2::new(x, 0.0),
                    Vec2::new(column.width, 600.0) // Default height
                );
                x += column.width + gap;
            } else {
                column.bounds = Rect::NOTHING;
            }
        }
    }
    
    // Equal height columns
    pub fn equalize_column_heights(&mut self) {
        let max_height = self.columns.iter()
            .map(|c| c.bounds.height())
            .fold(600.0f32, f32::max);
        
        for column in &mut self.columns {
            column.bounds.set_height(max_height);
        }
    }
    
    // Hide/show columns
    pub fn hide_column(&mut self, column_index: usize) {
        if let Some(column) = self.columns.get_mut(column_index) {
            column.visible = false;
            column.width = 0.0;
        }
    }
    
    pub fn show_column(&mut self, column_index: usize) {
        if let Some(column) = self.columns.get_mut(column_index) {
            column.visible = true;
            column.width = 300.0; // Default width
        }
    }
    
    // Fixed width columns
    pub fn set_column_fixed_width(&mut self, column_index: usize, width: f32) {
        if let Some(column) = self.columns.get_mut(column_index) {
            column.width = width;
            column.min_width = width;
            column.max_width = width;
        }
    }
    
    // Responsive breakpoints
    pub fn apply_responsive_layout(&mut self, viewport_width: f32) {
        if viewport_width < 768.0 {
            // Mobile: Stack columns
            for column in &mut self.columns {
                column.width = viewport_width - 32.0; // Full width minus padding
            }
        } else if viewport_width < 1024.0 {
            // Tablet: Compact columns
            for column in &mut self.columns {
                column.width = 250.0;
            }
        } else {
            // Desktop: Full columns
            self.calculate_responsive_column_widths(viewport_width);
        }
    }
    
    pub fn get_layout_mode(&self) -> LayoutMode {
        if self.viewport_width < 768.0 {
            LayoutMode::Stacked
        } else if self.viewport_width < 1024.0 {
            LayoutMode::Compact
        } else {
            LayoutMode::Full
        }
    }
    
    // Column content overflow
    pub fn column_needs_scrollbar(&self, column_index: usize) -> bool {
        if let Some(column) = self.columns.get(column_index) {
            let tasks_in_column = self.tasks.iter()
                .filter(|t| t.status == column.status)
                .count();
            
            let card_height = 80.0; // Approximate card height
            let content_height = tasks_in_column as f32 * card_height;
            
            content_height > column.bounds.height()
        } else {
            false
        }
    }
    
    pub fn get_column_content_height(&self, column_index: usize) -> f32 {
        if let Some(column) = self.columns.get(column_index) {
            let tasks_in_column = self.tasks.iter()
                .filter(|t| t.status == column.status)
                .count();
            
            let card_height = 80.0;
            tasks_in_column as f32 * card_height
        } else {
            0.0
        }
    }
    
    // Column drag to reorder
    pub fn start_column_drag(&mut self, column_index: usize, pos: Pos2) {
        // Store dragging state
    }
    
    pub fn update_column_drag(&mut self, pos: Pos2) {
        // Update drag position
    }
    
    pub fn drop_column(&mut self) {
        // Finalize column reordering
    }
    
    pub fn is_dragging_column(&self) -> bool {
        false // Simplified implementation
    }
    
    // Column width persistence
    pub fn save_column_widths(&self) -> Vec<f32> {
        self.columns.iter().map(|c| c.width).collect()
    }
    
    pub fn restore_column_widths(&mut self, widths: Vec<f32>) {
        for (column, &width) in self.columns.iter_mut().zip(widths.iter()) {
            column.width = width;
        }
    }
    
    pub fn reset_column_widths(&mut self) {
        for column in &mut self.columns {
            column.width = 300.0; // Default width
        }
    }
    
    // Auto-balance columns
    pub fn auto_balance_columns(&mut self) {
        let visible_columns = self.columns.iter()
            .filter(|c| c.visible && !c.is_collapsed)
            .count();
        
        if visible_columns == 0 {
            return;
        }
        
        let gap = 16.0;
        let total_gaps = gap * (visible_columns - 1) as f32;
        let available_width = self.viewport_width - total_gaps;
        let balanced_width = available_width / visible_columns as f32;
        
        for column in &mut self.columns {
            if column.visible && !column.is_collapsed {
                column.width = balanced_width;
            }
        }
    }
    
    // Custom column management
    pub fn add_custom_column(&mut self, title: String) {
        let column = KanbanColumn {
            id: Uuid::new_v4(),
            title,
            status: TaskStatus::Todo, // Default status
            color: eframe::egui::Color32::from_rgb(200, 200, 200),
            tasks: Vec::new(),
            width: 300.0,
            min_width: 250.0,
            max_width: 500.0,
            is_collapsed: false,
            wip_limit: None,
            bounds: Rect::NOTHING,
            is_resizing: false,
            resize_handle_hovered: false,
            collapsed: false,
            visible: true,
            position: self.columns.len(),
        };
        
        self.columns.push(column);
    }
}