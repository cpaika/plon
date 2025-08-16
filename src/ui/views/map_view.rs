use crate::domain::{task::Task, goal::Goal};
use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use std::collections::HashMap;
use uuid::Uuid;

pub struct MapView {
    camera_pos: Vec2,
    zoom_level: f32,
    selected_task_id: Option<Uuid>,
    selected_goal_id: Option<Uuid>,
    dragging_item: Option<DragItem>,
    
    // Clustering
    clusters: Vec<TaskCluster>,
    
    // Interaction state
    is_panning: bool,
    last_mouse_pos: Option<Pos2>,
    
    // Semantic zoom
    detail_level: DetailLevel,
}

#[derive(Clone)]
struct DragItem {
    id: Uuid,
    item_type: DragItemType,
    offset: Vec2,
}

#[derive(Clone, Copy)]
enum DragItemType {
    Task,
    Goal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DetailLevel {
    Overview,    // Highly summarized
    Summary,     // Basic info
    Standard,    // Normal view
    Detailed,    // All details
}

struct TaskCluster {
    center: Vec2,
    tasks: Vec<Uuid>,
    summary: String,
}

impl MapView {
    pub fn new() -> Self {
        Self {
            camera_pos: Vec2::ZERO,
            zoom_level: 1.0,
            selected_task_id: None,
            selected_goal_id: None,
            dragging_item: None,
            clusters: Vec::new(),
            is_panning: false,
            last_mouse_pos: None,
            detail_level: DetailLevel::Standard,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, tasks: &mut Vec<Task>, goals: &mut Vec<Goal>) {
        // Update detail level based on zoom
        self.detail_level = match self.zoom_level {
            z if z < 0.3 => DetailLevel::Overview,
            z if z < 0.6 => DetailLevel::Summary,
            z if z < 1.5 => DetailLevel::Standard,
            _ => DetailLevel::Detailed,
        };

        // Show controls
        ui.horizontal(|ui| {
            ui.label(format!("Zoom: {:.0}%", self.zoom_level * 100.0));
            
            if ui.button("🔍+").clicked() {
                self.zoom_in();
            }
            if ui.button("🔍-").clicked() {
                self.zoom_out();
            }
            if ui.button("🏠 Reset").clicked() {
                self.reset_view();
            }
            
            ui.separator();
            
            ui.label(format!("Detail: {:?}", self.detail_level));
            
            ui.separator();
            
            if ui.button("📊 Auto-arrange").clicked() {
                self.auto_arrange(tasks, goals);
            }
        });

        ui.separator();

        // Main map area
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
        
        // Handle panning
        if response.dragged_by(egui::PointerButton::Middle) || 
           (response.dragged_by(egui::PointerButton::Primary) && ui.input(|i| i.modifiers.shift)) {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if let Some(last_pos) = self.last_mouse_pos {
                    let delta = pointer_pos - last_pos;
                    self.camera_pos += delta / self.zoom_level;
                }
                self.last_mouse_pos = Some(pointer_pos);
                self.is_panning = true;
            }
        } else {
            self.last_mouse_pos = None;
            self.is_panning = false;
        }

        // Handle zoom with scroll
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_factor = 1.0 + scroll_delta * 0.001;
                self.zoom_level = (self.zoom_level * zoom_factor).clamp(0.1, 5.0);
            }
        }

        // Draw the map
        let painter = ui.painter_at(available_rect);
        let center = available_rect.center();
        
        // Transform function from world to screen coordinates
        let to_screen = |world_pos: Vec2| -> Pos2 {
            let scaled = (world_pos + self.camera_pos) * self.zoom_level;
            center + scaled
        };

        // Draw grid
        self.draw_grid(&painter, available_rect, to_screen);

        // Draw dependencies
        self.draw_dependencies(&painter, tasks, to_screen);

        // Draw goals
        for goal in goals.iter_mut() {
            self.draw_goal(&painter, ui, goal, tasks, to_screen, available_rect);
        }

        // Draw tasks or clusters based on zoom level
        if self.detail_level == DetailLevel::Overview {
            self.draw_clusters(&painter, ui, tasks, to_screen, available_rect);
        } else {
            for task in tasks.iter_mut() {
                if task.goal_id.is_none() {  // Don't draw tasks inside goals separately
                    self.draw_task(&painter, ui, task, to_screen, available_rect);
                }
            }
        }

        // Handle task creation on double-click
        if response.double_clicked() && !self.is_panning {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let world_pos = (pointer_pos - center) / self.zoom_level - self.camera_pos;
                let mut new_task = Task::new("New Task".to_string(), String::new());
                new_task.set_position(world_pos.x as f64, world_pos.y as f64);
                tasks.push(new_task);
            }
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect, to_screen: impl Fn(Vec2) -> Pos2) {
        let grid_size = 50.0 * self.zoom_level;
        let grid_color = Color32::from_rgba_unmultiplied(128, 128, 128, 20);
        
        // Calculate visible grid range
        let start_x = ((rect.min.x / grid_size).floor() * grid_size) as i32;
        let end_x = ((rect.max.x / grid_size).ceil() * grid_size) as i32;
        let start_y = ((rect.min.y / grid_size).floor() * grid_size) as i32;
        let end_y = ((rect.max.y / grid_size).ceil() * grid_size) as i32;
        
        // Draw vertical lines
        for x in (start_x..=end_x).step_by(grid_size as usize) {
            let world_x = x as f32 / self.zoom_level - self.camera_pos.x;
            let p1 = to_screen(Vec2::new(world_x, start_y as f32));
            let p2 = to_screen(Vec2::new(world_x, end_y as f32));
            painter.line_segment([p1, p2], Stroke::new(1.0, grid_color));
        }
        
        // Draw horizontal lines
        for y in (start_y..=end_y).step_by(grid_size as usize) {
            let world_y = y as f32 / self.zoom_level - self.camera_pos.y;
            let p1 = to_screen(Vec2::new(start_x as f32, world_y));
            let p2 = to_screen(Vec2::new(end_x as f32, world_y));
            painter.line_segment([p1, p2], Stroke::new(1.0, grid_color));
        }
    }

    fn draw_task(&self, painter: &egui::Painter, ui: &mut Ui, task: &mut Task, to_screen: impl Fn(Vec2) -> Pos2, clip_rect: Rect) {
        let world_pos = Vec2::new(task.position.x as f32, task.position.y as f32);
        let screen_pos = to_screen(world_pos);
        
        // Skip if outside view
        if !clip_rect.contains(screen_pos) {
            return;
        }

        let size = Vec2::new(150.0, 80.0) * self.zoom_level;
        let rect = Rect::from_center_size(screen_pos, size);
        
        // Determine color based on status
        let fill_color = match task.status {
            crate::domain::task::TaskStatus::Todo => Color32::from_rgb(200, 200, 200),
            crate::domain::task::TaskStatus::InProgress => Color32::from_rgb(100, 150, 255),
            crate::domain::task::TaskStatus::Done => Color32::from_rgb(100, 255, 100),
            crate::domain::task::TaskStatus::Blocked => Color32::from_rgb(255, 100, 100),
            _ => Color32::from_rgb(180, 180, 180),
        };
        
        let selected = self.selected_task_id == Some(task.id);
        let stroke_color = if selected {
            Color32::from_rgb(255, 200, 0)
        } else {
            Color32::from_rgb(100, 100, 100)
        };
        
        // Draw task rectangle
        painter.rect(
            rect,
            5.0,
            fill_color,
            Stroke::new(if selected { 3.0 } else { 1.0 }, stroke_color),
        );
        
        // Draw task content based on detail level
        match self.detail_level {
            DetailLevel::Overview => {
                // Don't draw individual tasks in overview
            }
            DetailLevel::Summary => {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &task.title,
                    egui::FontId::proportional(12.0 * self.zoom_level),
                    Color32::BLACK,
                );
            }
            DetailLevel::Standard => {
                painter.text(
                    rect.center() - Vec2::new(0.0, 20.0 * self.zoom_level),
                    egui::Align2::CENTER_CENTER,
                    &task.title,
                    egui::FontId::proportional(14.0 * self.zoom_level),
                    Color32::BLACK,
                );
                
                // Show progress if has subtasks
                if !task.subtasks.is_empty() {
                    let (completed, total) = task.subtask_progress();
                    painter.text(
                        rect.center() + Vec2::new(0.0, 10.0 * self.zoom_level),
                        egui::Align2::CENTER_CENTER,
                        &format!("{}/{}", completed, total),
                        egui::FontId::proportional(10.0 * self.zoom_level),
                        Color32::from_rgb(80, 80, 80),
                    );
                }
            }
            DetailLevel::Detailed => {
                painter.text(
                    rect.center() - Vec2::new(0.0, 25.0 * self.zoom_level),
                    egui::Align2::CENTER_CENTER,
                    &task.title,
                    egui::FontId::proportional(16.0 * self.zoom_level),
                    Color32::BLACK,
                );
                
                // Show metadata
                if let Some(category) = task.metadata.get("category") {
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        category,
                        egui::FontId::proportional(12.0 * self.zoom_level),
                        Color32::from_rgb(100, 100, 100),
                    );
                }
                
                // Show progress
                if !task.subtasks.is_empty() {
                    let (completed, total) = task.subtask_progress();
                    painter.text(
                        rect.center() + Vec2::new(0.0, 20.0 * self.zoom_level),
                        egui::Align2::CENTER_CENTER,
                        &format!("Progress: {}/{}", completed, total),
                        egui::FontId::proportional(10.0 * self.zoom_level),
                        Color32::from_rgb(80, 80, 80),
                    );
                }
            }
        }
        
        // Handle interaction
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        
        if response.clicked() {
            // Selected task (would need mutable self to update)
            // self.selected_task_id = Some(task.id);
            // self.selected_goal_id = None;
        }
        
        if response.dragged() && !self.is_panning {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let world_pos = (pointer_pos - rect.center()) / self.zoom_level;
                task.set_position(
                    (task.position.x + world_pos.x as f64),
                    (task.position.y + world_pos.y as f64),
                );
            }
        }
    }

    fn draw_goal(&self, painter: &egui::Painter, ui: &mut Ui, goal: &mut Goal, tasks: &[Task], to_screen: impl Fn(Vec2) -> Pos2, clip_rect: Rect) {
        let world_pos = Vec2::new(goal.position.x as f32, goal.position.y as f32);
        let screen_pos = to_screen(world_pos);
        
        // Skip if outside view
        if !clip_rect.contains(screen_pos) {
            return;
        }

        let size = Vec2::new(
            goal.position.width as f32 * self.zoom_level,
            goal.position.height as f32 * self.zoom_level,
        );
        let rect = Rect::from_min_size(screen_pos, size);
        
        // Parse color
        let color = Color32::from_rgba_unmultiplied(74, 144, 226, 30);
        let selected = self.selected_goal_id == Some(goal.id);
        
        // Draw goal rectangle
        painter.rect(
            rect,
            10.0,
            color,
            Stroke::new(
                if selected { 3.0 } else { 2.0 },
                if selected {
                    Color32::from_rgb(255, 200, 0)
                } else {
                    Color32::from_rgb(74, 144, 226)
                },
            ),
        );
        
        // Draw goal title
        painter.text(
            rect.min + Vec2::new(10.0 * self.zoom_level, 10.0 * self.zoom_level),
            egui::Align2::LEFT_TOP,
            &goal.title,
            egui::FontId::proportional(16.0 * self.zoom_level),
            Color32::from_rgb(50, 50, 50),
        );
        
        // Calculate and show progress
        let task_statuses: Vec<_> = tasks
            .iter()
            .filter(|t| goal.task_ids.contains(&t.id))
            .map(|t| (t.id, t.status == crate::domain::task::TaskStatus::Done))
            .collect();
        
        let progress = goal.calculate_progress(&task_statuses);
        
        painter.text(
            rect.min + Vec2::new(10.0 * self.zoom_level, 30.0 * self.zoom_level),
            egui::Align2::LEFT_TOP,
            &format!("Progress: {:.0}%", progress),
            egui::FontId::proportional(12.0 * self.zoom_level),
            Color32::from_rgb(100, 100, 100),
        );
        
        // Handle interaction
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        
        if response.clicked() {
            // Selected goal (would need mutable self to update)
            // self.selected_goal_id = Some(goal.id);
            // self.selected_task_id = None;
        }
    }

    fn draw_dependencies(&self, painter: &egui::Painter, tasks: &[Task], to_screen: impl Fn(Vec2) -> Pos2) {
        // TODO: Draw dependency arrows between tasks
    }

    fn draw_clusters(&self, painter: &egui::Painter, ui: &mut Ui, tasks: &[Task], to_screen: impl Fn(Vec2) -> Pos2, clip_rect: Rect) {
        // TODO: Implement clustering and drawing of task clusters
    }

    fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.2).min(5.0);
    }

    fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.2).max(0.1);
    }

    fn reset_view(&mut self) {
        self.camera_pos = Vec2::ZERO;
        self.zoom_level = 1.0;
    }

    fn auto_arrange(&mut self, tasks: &mut [Task], goals: &mut [Goal]) {
        // Simple grid arrangement for now
        let mut x = 0.0;
        let mut y = 0.0;
        let spacing = 200.0;
        let items_per_row = 5;
        
        for (i, task) in tasks.iter_mut().enumerate() {
            if task.goal_id.is_none() {
                x = (i % items_per_row) as f64 * spacing;
                y = (i / items_per_row) as f64 * spacing;
                task.set_position(x, y);
            }
        }
        
        // Arrange goals
        y += spacing * 2.0;
        for (i, goal) in goals.iter_mut().enumerate() {
            x = (i % 3) as f64 * (spacing * 3.0);
            y = (i / 3) as f64 * (spacing * 2.0);
            goal.set_position(x, y, 250.0, 180.0);
            
            // Arrange tasks within goal
            let goal_tasks: Vec<_> = tasks
                .iter_mut()
                .filter(|t| t.goal_id == Some(goal.id))
                .collect();
                
            for (j, task) in goal_tasks.into_iter().enumerate() {
                let task_x = x + 20.0 + (j % 2) as f64 * 120.0;
                let task_y = y + 40.0 + (j / 2) as f64 * 60.0;
                task.set_position(task_x, task_y);
            }
        }
    }
}