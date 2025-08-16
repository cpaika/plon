use crate::domain::{task::Task, goal::Goal, dependency::{Dependency, DependencyType, DependencyGraph}};
use crate::services::summarization::{SummarizationService, SummaryCache};
use crate::services::DependencyService;
use crate::ui::widgets::task_detail_modal::{TaskDetailModal, TaskAction};
use crate::repository::comment_repository::CommentRepository;
use petgraph::visit::EdgeRef;
pub use crate::services::summarization::SummarizationLevel;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;
use tokio::runtime::Runtime;

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
    
    // Summarization
    summarization_service: Arc<SummarizationService>,
    summary_cache: SummaryCache,
    task_summaries: HashMap<Uuid, String>,
    goal_summaries: HashMap<Uuid, String>,
    runtime: Arc<Runtime>,
    last_summarization_zoom: f32,
    
    // Dependency creation state
    creating_dependency: bool,
    dependency_source: Option<Uuid>,
    dependency_preview_end: Option<Pos2>,
    
    // Dependencies
    dependency_graph: DependencyGraph,
    dependency_service: Option<Arc<DependencyService>>,
    
    // Task detail modal
    task_detail_modal: TaskDetailModal,
    comment_repository: Option<Arc<CommentRepository>>,
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
pub enum DetailLevel {
    Overview,    // Highly summarized
    Summary,     // Basic info
    Standard,    // Normal view
    Detailed,    // All details
}

impl DetailLevel {
    fn to_summarization_level(&self) -> SummarizationLevel {
        match self {
            DetailLevel::Overview => SummarizationLevel::HighLevel,
            DetailLevel::Summary => SummarizationLevel::MidLevel,
            DetailLevel::Standard => SummarizationLevel::LowLevel,
            DetailLevel::Detailed => SummarizationLevel::Detailed,
        }
    }
}

struct TaskCluster {
    center: Vec2,
    tasks: Vec<Uuid>,
    summary: String,
}

impl Default for MapView {
    fn default() -> Self {
        Self::new()
    }
}

impl MapView {
    pub fn new() -> Self {
        let runtime = Arc::new(Runtime::new().expect("Failed to create tokio runtime"));
        
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
            summarization_service: Arc::new(SummarizationService::new()),
            summary_cache: SummaryCache::new(200),
            task_summaries: HashMap::new(),
            goal_summaries: HashMap::new(),
            runtime,
            last_summarization_zoom: 1.0,
            creating_dependency: false,
            dependency_source: None,
            dependency_preview_end: None,
            dependency_graph: DependencyGraph::new(),
            dependency_service: None,
            task_detail_modal: TaskDetailModal::new(),
            comment_repository: None,
        }
    }

    pub fn set_dependency_service(&mut self, service: Arc<DependencyService>) {
        self.dependency_service = Some(service);
    }
    
    pub fn show(&mut self, ui: &mut Ui, tasks: &mut Vec<Task>, goals: &mut Vec<Goal>) {
        // Load dependencies if we have a service (do this once per frame)
        if let Some(service) = &self.dependency_service {
            // Load dependencies from database
            let service_clone = service.clone();
            let graph = std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(async {
                    service_clone.build_dependency_graph().await.ok()
                })
            }).join().unwrap();
            
            if let Some(graph) = graph {
                self.dependency_graph = graph;
            }
        }
        
        // Update detail level based on zoom
        let previous_detail = self.detail_level;
        self.detail_level = match self.zoom_level {
            z if z < 0.3 => DetailLevel::Overview,
            z if z < 0.6 => DetailLevel::Summary,
            z if z < 1.5 => DetailLevel::Standard,
            _ => DetailLevel::Detailed,
        };

        // Trigger re-summarization if zoom changed significantly
        if (self.zoom_level - self.last_summarization_zoom).abs() > 0.1 || previous_detail != self.detail_level {
            self.update_summaries(tasks, goals);
            self.last_summarization_zoom = self.zoom_level;
        }

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
            
            let summarization_level = self.detail_level.to_summarization_level();
            ui.label(format!("AI Level: {:?}", summarization_level));
            
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
            
            // Update dependency preview end position
            // Note: For actual implementation, this would need mutable self
            // if self.creating_dependency {
            //     if let Some(pointer_pos) = response.hover_pos() {
            //         self.dependency_preview_end = Some(pointer_pos);
            //     }
            // }
        }

        // Draw the map
        let painter = ui.painter_at(available_rect);
        let center = available_rect.center();
        
        // Cache values for closure
        let camera_pos = self.camera_pos;
        let zoom_level = self.zoom_level;
        
        // Transform function from world to screen coordinates
        let to_screen = |world_pos: Vec2| -> Pos2 {
            let scaled = (world_pos + camera_pos) * zoom_level;
            center + scaled
        };

        // Draw grid
        self.draw_grid(&painter, available_rect, &to_screen);

        // Draw dependencies
        self.draw_dependencies(&painter, tasks, to_screen);
        
        // Draw dependency preview if creating
        if self.creating_dependency
            && let (Some(source_id), Some(preview_end)) = (self.dependency_source, self.dependency_preview_end) {
                // Find source task position
                if let Some(source_task) = tasks.iter().find(|t| t.id == source_id) {
                    let source_pos = Vec2::new(source_task.position.x as f32, source_task.position.y as f32);
                    let source_screen = to_screen(source_pos);
                    
                    // Start from the right edge of the source task
                    let task_width = 150.0 * self.zoom_level / 2.0;
                    let arrow_start = source_screen + Vec2::new(task_width, 0.0);
                    
                    // Draw preview arrow
                    painter.arrow(
                        arrow_start,
                        preview_end - arrow_start,
                        Stroke::new(2.0, Color32::from_rgba_unmultiplied(100, 150, 255, 180))
                    );
                    
                    // Draw helper text
                    painter.text(
                        preview_end + Vec2::new(10.0, -10.0),
                        egui::Align2::LEFT_BOTTOM,
                        "Drop on left dot to create dependency\nRight-click or ESC to cancel",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(100, 100, 100),
                    );
                }
            }

        // Draw goals
        for goal in goals.iter_mut() {
            self.draw_goal(&painter, ui, goal, tasks, &to_screen, available_rect);
        }

        // Draw tasks or clusters based on zoom level
        if self.detail_level == DetailLevel::Overview {
            self.draw_clusters(&painter, ui, tasks, &to_screen, available_rect);
        } else {
            for task in tasks.iter_mut() {
                if task.goal_id.is_none() {  // Don't draw tasks inside goals separately
                    self.draw_task(&painter, ui, task, &to_screen, available_rect);
                }
            }
        }

        // Handle task creation on double-click
        if response.double_clicked() && !self.is_panning && !self.creating_dependency
            && let Some(pointer_pos) = response.interact_pointer_pos() {
                let world_pos = (pointer_pos - center) / self.zoom_level - self.camera_pos;
                let mut new_task = Task::new("New Task".to_string(), String::new());
                new_task.set_position(world_pos.x as f64, world_pos.y as f64);
                tasks.push(new_task);
            }
        
        // Cancel dependency creation on escape
        if self.creating_dependency
            && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.cancel_dependency_creation();
            }
        
        // Show help text
        if !self.creating_dependency {
            ui.ctx().debug_painter().text(
                egui::Pos2::new(10.0, available_rect.bottom() - 30.0),
                egui::Align2::LEFT_BOTTOM,
                "Drag from the right dot of a task to its left dot to create a dependency",
                egui::FontId::proportional(12.0),
                Color32::from_rgba_unmultiplied(100, 100, 100, 180),
            );
        }
        
        // Show task detail modal
        let resources = Vec::new(); // TODO: Get from context
        if let Some(action) = self.task_detail_modal.show(ui.ctx(), &resources) {
            match action {
                TaskAction::Update(updated_task) => {
                    // Update the task in the list
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == updated_task.id) {
                        *task = updated_task;
                    }
                }
                TaskAction::AddComment(comment) => {
                    // Save comment to database
                    if let Some(repo) = &self.comment_repository {
                        let repo_clone = repo.clone();
                        let runtime_clone = self.runtime.clone();
                        runtime_clone.spawn(async move {
                            if let Err(e) = repo_clone.create(&comment).await {
                                eprintln!("Failed to save comment: {}", e);
                            }
                        });
                    }
                }
                TaskAction::UpdateComment(comment) => {
                    // Update comment in database
                    if let Some(repo) = &self.comment_repository {
                        let repo_clone = repo.clone();
                        let runtime_clone = self.runtime.clone();
                        runtime_clone.spawn(async move {
                            if let Err(e) = repo_clone.update(&comment).await {
                                eprintln!("Failed to update comment: {}", e);
                            }
                        });
                    }
                }
                TaskAction::DeleteComment(id) => {
                    // Delete comment from database
                    if let Some(repo) = &self.comment_repository {
                        let repo_clone = repo.clone();
                        let runtime_clone = self.runtime.clone();
                        runtime_clone.spawn(async move {
                            if let Err(e) = repo_clone.delete(id).await {
                                eprintln!("Failed to delete comment: {}", e);
                            }
                        });
                    }
                }
            }
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect, to_screen: &impl Fn(Vec2) -> Pos2) {
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

    fn draw_task(&mut self, painter: &egui::Painter, ui: &mut Ui, task: &mut Task, to_screen: &impl Fn(Vec2) -> Pos2, clip_rect: Rect) {
        let world_pos = Vec2::new(task.position.x as f32, task.position.y as f32);
        let screen_pos = to_screen(world_pos);
        
        // Skip if outside view
        if !clip_rect.contains(screen_pos) {
            return;
        }

        let size = Vec2::new(150.0, 80.0) * self.zoom_level;
        let rect = Rect::from_center_size(screen_pos, size);
        
        // Connection points (dots on left and right)
        let dot_radius = 6.0 * self.zoom_level;
        let left_dot_pos = Pos2::new(rect.left(), rect.center().y);
        let right_dot_pos = Pos2::new(rect.right(), rect.center().y);
        
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
                let summary = self.task_summaries.get(&task.id)
                    .map(|s| s.as_str())
                    .unwrap_or(&task.title);
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    summary,
                    egui::FontId::proportional(12.0 * self.zoom_level),
                    Color32::BLACK,
                );
            }
            DetailLevel::Standard => {
                let summary = self.task_summaries.get(&task.id)
                    .map(|s| s.as_str())
                    .unwrap_or(&task.title);
                    
                painter.text(
                    rect.center() - Vec2::new(0.0, 20.0 * self.zoom_level),
                    egui::Align2::CENTER_CENTER,
                    summary,
                    egui::FontId::proportional(14.0 * self.zoom_level),
                    Color32::BLACK,
                );
                
                // Show progress if has subtasks
                if !task.subtasks.is_empty() {
                    let (completed, total) = task.subtask_progress();
                    painter.text(
                        rect.center() + Vec2::new(0.0, 10.0 * self.zoom_level),
                        egui::Align2::CENTER_CENTER,
                        format!("{}/{}", completed, total),
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
                        format!("Progress: {}/{}", completed, total),
                        egui::FontId::proportional(10.0 * self.zoom_level),
                        Color32::from_rgb(80, 80, 80),
                    );
                }
            }
        }
        
        // Draw connection dots
        let dot_color = if self.creating_dependency && self.dependency_source == Some(task.id) {
            Color32::from_rgb(100, 150, 255)  // Highlight when creating from this task
        } else {
            Color32::from_rgba_unmultiplied(100, 100, 100, 150)
        };
        
        // Left dot (for incoming connections)
        painter.circle_filled(left_dot_pos, dot_radius, dot_color);
        
        // Right dot (for outgoing connections)
        painter.circle_filled(right_dot_pos, dot_radius, dot_color);
        
        // Handle interaction with the task body
        let task_response = ui.allocate_rect(rect, Sense::click_and_drag());
        
        // Handle interaction with connection dots
        let left_dot_rect = Rect::from_center_size(left_dot_pos, Vec2::splat(dot_radius * 3.0));
        let right_dot_rect = Rect::from_center_size(right_dot_pos, Vec2::splat(dot_radius * 3.0));
        
        let left_dot_response = ui.allocate_rect(left_dot_rect, Sense::drag());
        let right_dot_response = ui.allocate_rect(right_dot_rect, Sense::drag());
        
        // Hover effects for dots
        if left_dot_response.hovered() || right_dot_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            
            // Draw hover highlight
            if left_dot_response.hovered() {
                painter.circle_stroke(left_dot_pos, dot_radius + 2.0, Stroke::new(2.0, Color32::from_rgb(100, 150, 255)));
            }
            if right_dot_response.hovered() {
                painter.circle_stroke(right_dot_pos, dot_radius + 2.0, Stroke::new(2.0, Color32::from_rgb(100, 150, 255)));
            }
        }
        
        // Start dependency creation from right dot (outgoing)
        if right_dot_response.drag_started() {
            self.start_dependency_creation(task.id);
            self.dependency_preview_end = Some(right_dot_pos);
        }
        
        // Complete dependency creation on left dot (incoming)
        if left_dot_response.hovered() && self.creating_dependency
            && let Some(source_id) = self.dependency_source
                && source_id != task.id {
                    // Highlight the target dot
                    painter.circle_stroke(left_dot_pos, dot_radius + 3.0, Stroke::new(3.0, Color32::from_rgb(50, 200, 50)));
                    
                    // Complete on mouse release
                    if ui.input(|i| i.pointer.any_released())
                        && let Some(dep) = self.complete_dependency_creation(task.id) {
                            // Save to database via dependency service
                            if let Some(service) = &self.dependency_service {
                                let service_clone = service.clone();
                                let dep_clone = dep.clone();
                                std::thread::spawn(move || {
                                    let runtime = tokio::runtime::Runtime::new().unwrap();
                                    runtime.block_on(async {
                                        if let Err(e) = service_clone.create_dependency(
                                            dep_clone.from_task_id,
                                            dep_clone.to_task_id,
                                            dep_clone.dependency_type,
                                        ).await {
                                            eprintln!("Failed to save dependency: {}", e);
                                        }
                                    });
                                });
                            }
                        }
                }
        
        // Update preview while dragging
        if self.creating_dependency {
            if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                self.dependency_preview_end = Some(pointer_pos);
            }
            
            // Cancel on right click or escape
            if ui.input(|i| i.pointer.secondary_clicked() || i.key_pressed(egui::Key::Escape)) {
                self.cancel_dependency_creation();
            }
        }
        
        // Handle regular task interactions
        if task_response.clicked() && !self.creating_dependency {
            self.selected_task_id = Some(task.id);
            self.selected_goal_id = None;
            
            // Open task detail modal
            let comments = if let Some(repo) = &self.comment_repository {
                // Load comments for this task
                let repo_clone = repo.clone();
                let task_id = task.id;
                let runtime_clone = self.runtime.clone();
                runtime_clone.block_on(async move {
                    repo_clone.list_for_entity(task_id).await.unwrap_or_default()
                })
            } else {
                Vec::new()
            };
            
            self.task_detail_modal.open(task.clone(), comments);
        }
        
        // Handle task dragging (only if not dragging from dots)
        if task_response.dragged() && !self.is_panning && !self.creating_dependency && !left_dot_response.dragged() && !right_dot_response.dragged() {
            let delta = task_response.drag_delta() / self.zoom_level;
            task.set_position(
                task.position.x + delta.x as f64,
                task.position.y + delta.y as f64,
            );
        }
    }

    fn draw_goal(&mut self, painter: &egui::Painter, ui: &mut Ui, goal: &mut Goal, tasks: &[Task], to_screen: &impl Fn(Vec2) -> Pos2, clip_rect: Rect) {
        let world_pos = Vec2::new(goal.position_x as f32, goal.position_y as f32);
        let screen_pos = to_screen(world_pos);
        
        // Skip if outside view
        if !clip_rect.contains(screen_pos) {
            return;
        }

        let size = Vec2::new(
            goal.position_width as f32 * self.zoom_level,
            goal.position_height as f32 * self.zoom_level,
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
        
        // Draw goal title or summary based on detail level
        let goal_text = if self.detail_level == DetailLevel::Overview || self.detail_level == DetailLevel::Summary {
            self.goal_summaries.get(&goal.id)
                .map(|s| s.as_str())
                .unwrap_or(&goal.title)
        } else {
            &goal.title
        };
        
        painter.text(
            rect.min + Vec2::new(10.0 * self.zoom_level, 10.0 * self.zoom_level),
            egui::Align2::LEFT_TOP,
            goal_text,
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
            format!("Progress: {:.0}%", progress),
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

    fn draw_dependencies(&self, painter: &egui::Painter, tasks: &[Task], to_screen: impl Fn(Vec2) -> Pos2 + Copy) {
        // Create a map of task IDs to positions for quick lookup
        let task_positions: HashMap<Uuid, Vec2> = tasks.iter()
            .map(|t| (t.id, Vec2::new(t.position.x as f32, t.position.y as f32)))
            .collect();
        
        // Draw all dependencies from the graph
        for dependency in self.dependency_graph.get_all_dependencies() {
            if let (Some(&from_pos), Some(&to_pos)) = (
                task_positions.get(&dependency.from_task_id),
                task_positions.get(&dependency.to_task_id),
            ) {
                // Calculate task rectangle bounds
                let task_size = Vec2::new(150.0, 80.0);
                
                // Calculate connection points based on dependency type
                let (start_offset, end_offset) = match dependency.dependency_type {
                    DependencyType::FinishToStart => {
                        // From right edge to left edge
                        (Vec2::new(task_size.x / 2.0, 0.0), Vec2::new(-task_size.x / 2.0, 0.0))
                    }
                    DependencyType::StartToStart => {
                        // From left edge to left edge
                        (Vec2::new(-task_size.x / 2.0, 0.0), Vec2::new(-task_size.x / 2.0, 0.0))
                    }
                    DependencyType::FinishToFinish => {
                        // From right edge to right edge
                        (Vec2::new(task_size.x / 2.0, 0.0), Vec2::new(task_size.x / 2.0, 0.0))
                    }
                    DependencyType::StartToFinish => {
                        // From left edge to right edge
                        (Vec2::new(-task_size.x / 2.0, 0.0), Vec2::new(task_size.x / 2.0, 0.0))
                    }
                };
                
                let start_screen = to_screen(from_pos + start_offset);
                let end_screen = to_screen(to_pos + end_offset);
                
                // Get the arrow path
                let path = calculate_arrow_path(start_screen, end_screen, dependency.dependency_type);
                
                // Draw the arrow as a bezier curve or lines
                let arrow_color = if self.dependency_graph.get_critical_path(&HashMap::new()).contains(&dependency.from_task_id) {
                    Color32::from_rgb(255, 100, 100) // Red for critical path
                } else {
                    Color32::from_rgb(100, 100, 200) // Blue for normal dependencies
                };
                
                let stroke = Stroke::new(2.0, arrow_color);
                
                if path.len() >= 4 {
                    // Draw as bezier curve
                    painter.add(egui::Shape::CubicBezier(
                        egui::epaint::CubicBezierShape {
                            points: [path[0], path[1], path[2], path[3]],
                            closed: false,
                            fill: Color32::TRANSPARENT,
                            stroke,
                        },
                    ));
                } else if path.len() >= 2 {
                    // Draw as line
                    painter.line_segment([path[0], path[path.len() - 1]], stroke);
                }
                
                // Draw arrowhead
                let arrow_size = 10.0;
                let end_point = path[path.len() - 1];
                let prev_point = if path.len() >= 2 { path[path.len() - 2] } else { path[0] };
                
                let direction = (end_point - prev_point).normalized();
                let perpendicular = Vec2::new(-direction.y, direction.x);
                
                let arrow_points = vec![
                    end_point,
                    end_point - direction * arrow_size + perpendicular * (arrow_size / 2.0),
                    end_point - direction * arrow_size - perpendicular * (arrow_size / 2.0),
                ];
                
                painter.add(egui::Shape::convex_polygon(
                    arrow_points,
                    arrow_color,
                    Stroke::NONE,
                ));
            }
        }
        
        // Draw dependency preview if creating
        if self.creating_dependency
            && let (Some(source_id), Some(preview_end)) = (self.dependency_source, self.dependency_preview_end)
                && let Some(&source_pos) = task_positions.get(&source_id) {
                    let task_size = Vec2::new(150.0, 80.0);
                    let start_screen = to_screen(source_pos + Vec2::new(task_size.x / 2.0, 0.0));
                    
                    // Draw preview line
                    painter.line_segment(
                        [start_screen, preview_end],
                        Stroke::new(2.0, Color32::from_rgba_unmultiplied(100, 100, 200, 128)),
                    );
                }
    }

    fn draw_clusters(&mut self, painter: &egui::Painter, ui: &mut Ui, tasks: &[Task], to_screen: &impl Fn(Vec2) -> Pos2, clip_rect: Rect) {
        // Group nearby tasks into clusters for overview mode
        if self.clusters.is_empty() {
            self.update_clusters(tasks);
        }
        
        for cluster in &self.clusters {
            let screen_pos = to_screen(cluster.center);
            if !clip_rect.contains(screen_pos) {
                continue;
            }
            
            let radius = 60.0 * self.zoom_level;
            painter.circle(
                screen_pos,
                radius,
                Color32::from_rgba_unmultiplied(100, 150, 255, 50),
                Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
            );
            
            painter.text(
                screen_pos,
                egui::Align2::CENTER_CENTER,
                &cluster.summary,
                egui::FontId::proportional(14.0 * self.zoom_level),
                Color32::BLACK,
            );
            
            painter.text(
                screen_pos + Vec2::new(0.0, 20.0 * self.zoom_level),
                egui::Align2::CENTER_CENTER,
                format!("{} tasks", cluster.tasks.len()),
                egui::FontId::proportional(10.0 * self.zoom_level),
                Color32::from_rgb(80, 80, 80),
            );
        }
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

    fn update_summaries(&mut self, tasks: &[Task], goals: &[Goal]) {
        let summarization_level = self.detail_level.to_summarization_level();
        let service = Arc::clone(&self.summarization_service);
        let runtime = Arc::clone(&self.runtime);
        
        // Clear old summaries
        self.task_summaries.clear();
        self.goal_summaries.clear();
        
        // Generate task summaries
        for task in tasks {
            let task_id = task.id;
            let content = format!("{}: {}", task.title, task.description);
            let svc = Arc::clone(&service);
            
            let summary = runtime.block_on(async move {
                svc.summarize(&content, summarization_level).await
            });
            
            self.task_summaries.insert(task_id, summary);
        }
        
        // Generate goal summaries
        for goal in goals {
            let goal_id = goal.id;
            let goal_tasks: Vec<_> = tasks
                .iter()
                .filter(|t| goal.task_ids.contains(&t.id))
                .cloned()
                .collect();
            
            let svc = Arc::clone(&service);
            let goal_clone = goal.clone();
            
            let summary = runtime.block_on(async move {
                svc.summarize_goal(&goal_clone, &goal_tasks, summarization_level).await
            });
            
            self.goal_summaries.insert(goal_id, summary);
        }
    }
    
    fn update_clusters(&mut self, tasks: &[Task]) {
        self.clusters.clear();
        
        // Simple grid-based clustering
        let cluster_size = 300.0f32;
        let mut cluster_map: HashMap<(i32, i32), Vec<Uuid>> = HashMap::new();
        
        for task in tasks {
            if task.goal_id.is_none() {
                let cluster_x = (task.position.x / cluster_size as f64).floor() as i32;
                let cluster_y = (task.position.y / cluster_size as f64).floor() as i32;
                
                cluster_map
                    .entry((cluster_x, cluster_y))
                    .or_default()
                    .push(task.id);
            }
        }
        
        // Create clusters with summaries
        let service = Arc::clone(&self.summarization_service);
        let runtime = Arc::clone(&self.runtime);
        
        for ((x, y), task_ids) in cluster_map {
            if task_ids.len() > 1 {
                let center_x = (x as f32 + 0.5) * cluster_size;
                let center_y = (y as f32 + 0.5) * cluster_size;
                
                let cluster_tasks: Vec<_> = tasks
                    .iter()
                    .filter(|t| task_ids.contains(&t.id))
                    .cloned()
                    .collect();
                
                let svc = Arc::clone(&service);
                let summary = runtime.block_on(async move {
                    svc.summarize_cluster(&cluster_tasks, SummarizationLevel::HighLevel).await
                });
                
                self.clusters.push(TaskCluster {
                    center: Vec2::new(center_x, center_y),
                    tasks: task_ids,
                    summary,
                });
            }
        }
    }
    
    pub fn set_zoom_level(&mut self, zoom: f32) {
        self.zoom_level = zoom.clamp(0.1, 5.0);
    }
    
    pub fn get_summarization_level(&self) -> SummarizationLevel {
        self.detail_level.to_summarization_level()
    }
    
    pub async fn get_task_summaries(&mut self, tasks: &[Task], service: &SummarizationService) -> Vec<String> {
        let level = self.get_summarization_level();
        let mut summaries = Vec::new();
        
        for task in tasks {
            let summary = service.summarize_with_cache(
                &mut self.summary_cache,
                task.id,
                &format!("{}: {}", task.title, task.description),
                level
            ).await;
            summaries.push(summary);
        }
        
        summaries
    }
    
    pub async fn get_visible_task_summaries(
        &mut self,
        tasks: &[Task],
        service: &SummarizationService,
        viewport: Rect
    ) -> Vec<String> {
        let level = self.get_summarization_level();
        let mut summaries = Vec::new();
        
        for task in tasks {
            let world_pos = Vec2::new(task.position.x as f32, task.position.y as f32);
            let screen_pos = viewport.center() + (world_pos + self.camera_pos) * self.zoom_level;
            
            if viewport.contains(screen_pos) {
                let summary = service.summarize_with_cache(
                    &mut self.summary_cache,
                    task.id,
                    &format!("{}: {}", task.title, task.description),
                    level
                ).await;
                summaries.push(summary);
            }
        }
        
        summaries
    }
    
    pub fn handle_scroll_delta(&mut self, delta: f32) {
        let zoom_factor = 1.0 + delta * 0.001;
        self.zoom_level = (self.zoom_level * zoom_factor).clamp(0.1, 5.0);
    }
    
    fn auto_arrange(&mut self, tasks: &mut [Task], goals: &mut [Goal]) {
        // Use the smart arrangement algorithm
        self.auto_arrange_smart(tasks, goals);
    }
    
    pub fn auto_arrange_smart(&mut self, tasks: &mut [Task], goals: &mut [Goal]) {
        use std::collections::{HashMap, VecDeque};
        use petgraph::algo::toposort;
        
        let spacing_x = 250.0;
        let spacing_y = 150.0;
        let group_spacing = 350.0;
        
        // Step 1: Build dependency graph and perform topological sort
        let mut task_levels: HashMap<Uuid, usize> = HashMap::new();
        let mut max_level = 0;
        
        // Build a dependency graph
        let mut dep_graph = petgraph::graph::DiGraph::new();
        let mut node_map: HashMap<Uuid, petgraph::graph::NodeIndex> = HashMap::new();
        
        // Add all tasks as nodes
        for task in tasks.iter() {
            let node = dep_graph.add_node(task.id);
            node_map.insert(task.id, node);
        }
        
        // Add dependency edges from the dependency graph
        if let Some(dependencies) = self.get_dependencies() {
            for dep in dependencies {
                if let (Some(&from_node), Some(&to_node)) = 
                    (node_map.get(&dep.from_task_id), node_map.get(&dep.to_task_id)) {
                    dep_graph.add_edge(from_node, to_node, ());
                }
            }
        }
        
        // Calculate levels using BFS for tasks with dependencies
        let mut visited = std::collections::HashSet::new();
        let mut queue = VecDeque::new();
        
        // Find root tasks (no incoming edges)
        for task in tasks.iter() {
            let node = node_map[&task.id];
            let has_incoming = dep_graph.edges_directed(node, petgraph::Direction::Incoming).count() > 0;
            if !has_incoming {
                queue.push_back((task.id, 0));
                visited.insert(task.id);
            }
        }
        
        // BFS to assign levels
        while let Some((task_id, level)) = queue.pop_front() {
            task_levels.insert(task_id, level);
            max_level = max_level.max(level);
            
            if let Some(&node) = node_map.get(&task_id) {
                for edge in dep_graph.edges_directed(node, petgraph::Direction::Outgoing) {
                    let target_id = dep_graph[edge.target()];
                    if !visited.contains(&target_id) {
                        visited.insert(target_id);
                        queue.push_back((target_id, level + 1));
                    }
                }
            }
        }
        
        // Step 2: Group tasks by similarity (status, tags, priority)
        let mut task_groups: HashMap<String, Vec<usize>> = HashMap::new();
        
        for (i, task) in tasks.iter().enumerate() {
            // Create a group key based on task properties
            let group_key = if task_levels.contains_key(&task.id) {
                // Tasks with dependencies get their own group based on level
                format!("dep_level_{}", task_levels.get(&task.id).unwrap_or(&0))
            } else {
                // Group by status and priority for non-dependent tasks
                let mut key = format!("{:?}_{:?}", task.status, task.priority);
                
                // Also consider tags for finer grouping
                if !task.tags.is_empty() {
                    let mut tags: Vec<_> = task.tags.iter().cloned().collect();
                    tags.sort();
                    key = format!("{}_{}", key, tags.join("_"));
                }
                key
            };
            
            task_groups.entry(group_key).or_insert_with(Vec::new).push(i);
        }
        
        // Step 3: Arrange tasks
        let mut current_x = 0.0;
        let mut group_positions: HashMap<String, (f64, f64)> = HashMap::new();
        
        // First, arrange dependency-based tasks (left to right by level)
        for level in 0..=max_level {
            let level_key = format!("dep_level_{}", level);
            if let Some(task_indices) = task_groups.get(&level_key) {
                let mut y = 0.0;
                for (j, &idx) in task_indices.iter().enumerate() {
                    tasks[idx].set_position(
                        level as f64 * spacing_x * 1.5,
                        y
                    );
                    y += spacing_y;
                }
                group_positions.insert(level_key.clone(), (level as f64 * spacing_x * 1.5, y / 2.0));
            }
        }
        
        // Calculate starting X position for non-dependency groups
        if max_level > 0 {
            current_x = (max_level + 1) as f64 * spacing_x * 1.5 + group_spacing;
        }
        
        // Then arrange other groups (grouped by similarity)
        let mut sorted_groups: Vec<_> = task_groups.iter()
            .filter(|(key, _)| !key.starts_with("dep_level_"))
            .collect();
        sorted_groups.sort_by_key(|(key, _)| key.as_str());
        
        for (group_key, task_indices) in sorted_groups {
            let mut y = 0.0;
            let items_per_column = 5;
            
            for (j, &idx) in task_indices.iter().enumerate() {
                let col = j / items_per_column;
                let row = j % items_per_column;
                
                tasks[idx].set_position(
                    current_x + col as f64 * spacing_x * 0.7,
                    y + row as f64 * spacing_y * 0.8
                );
            }
            
            let cols = (task_indices.len() + items_per_column - 1) / items_per_column;
            group_positions.insert(group_key.clone(), (current_x + (cols as f64 * spacing_x * 0.7) / 2.0, y + 2.5 * spacing_y));
            current_x += (cols as f64 * spacing_x * 0.7) + group_spacing * 0.5;
        }
        
        // Step 4: Arrange goals with their tasks
        if !goals.is_empty() {
            let mut goal_y: f64 = 0.0;
            
            // Find the maximum Y position of all tasks
            for task in tasks.iter() {
                goal_y = goal_y.max(task.position.y);
            }
            goal_y += spacing_y * 3.0;
            
            for (i, goal) in goals.iter_mut().enumerate() {
                let goal_x = (i % 3) as f64 * (group_spacing * 2.0);
                let goal_row = (i / 3) as f64;
                let goal_y_pos = goal_y + goal_row * 300.0;
                
                goal.set_position(goal_x, goal_y_pos, 500.0, 250.0);
                
                // Arrange tasks within goal
                let goal_tasks: Vec<_> = tasks
                    .iter_mut()
                    .filter(|t| t.goal_id == Some(goal.id))
                    .enumerate()
                    .map(|(idx, task)| (idx, task))
                    .collect();
                    
                for (j, (_, task)) in goal_tasks.into_iter().enumerate() {
                    let task_x = goal_x + 20.0 + (j % 3) as f64 * 150.0;
                    let task_y = goal_y_pos + 40.0 + (j / 3) as f64 * 70.0;
                    task.set_position(task_x, task_y);
                }
            }
        }
    }
    
    pub fn set_dependencies(&mut self, dependencies: Vec<Dependency>) {
        // Clear existing dependencies
        self.dependency_graph = DependencyGraph::new();
        
        // Add all dependencies to the graph
        for dep in dependencies {
            self.dependency_graph.add_dependency(&dep).ok();
        }
    }
    
    pub fn get_dependencies(&self) -> Option<Vec<Dependency>> {
        // Return a list of dependencies from the dependency graph
        // This is a simplified version - in reality, we'd need to store the dependencies
        None
    }
    
    // Dependency creation methods
    pub fn is_creating_dependency(&self) -> bool {
        self.creating_dependency
    }
    
    pub fn get_dependency_source(&self) -> Option<Uuid> {
        self.dependency_source
    }
    
    pub fn start_dependency_creation(&mut self, source_task_id: Uuid) {
        self.creating_dependency = true;
        self.dependency_source = Some(source_task_id);
        self.dependency_preview_end = None;
    }
    
    pub fn complete_dependency_creation(&mut self, target_task_id: Uuid) -> Option<Dependency> {
        if let Some(source_id) = self.dependency_source
            && source_id != target_task_id {
                let dependency = Dependency::new(source_id, target_task_id, DependencyType::FinishToStart);
                
                // Try to add to graph
                if self.dependency_graph.add_dependency(&dependency).is_ok() {
                    self.creating_dependency = false;
                    self.dependency_source = None;
                    self.dependency_preview_end = None;
                    return Some(dependency);
                }
            }
        self.cancel_dependency_creation();
        None
    }
    
    pub fn cancel_dependency_creation(&mut self) {
        self.creating_dependency = false;
        self.dependency_source = None;
        self.dependency_preview_end = None;
    }
    
    pub fn get_dependency_preview(&self, start_pos: Vec2, mouse_pos: Vec2) -> Option<(Vec2, Vec2)> {
        if self.creating_dependency {
            Some((start_pos, mouse_pos))
        } else {
            None
        }
    }
}

// Public utility functions for arrow drawing
pub fn calculate_arrow_path(start: Pos2, end: Pos2, dependency_type: DependencyType) -> Vec<Pos2> {
    let mut path = Vec::new();
    
    match dependency_type {
        DependencyType::FinishToStart => {
            // Straight arrow from right edge of source to left edge of target
            path.push(start);
            
            // Add control points for a smooth bezier curve
            let mid_x = (start.x + end.x) / 2.0;
            path.push(Pos2::new(mid_x, start.y));
            path.push(Pos2::new(mid_x, end.y));
            
            path.push(end);
        }
        DependencyType::StartToStart => {
            // Curved arrow from left to left
            path.push(start);
            
            let offset = 30.0;
            path.push(Pos2::new(start.x - offset, start.y));
            path.push(Pos2::new(end.x - offset, end.y));
            
            path.push(end);
        }
        DependencyType::FinishToFinish => {
            // Curved arrow from right to right
            path.push(start);
            
            let offset = 30.0;
            path.push(Pos2::new(start.x + offset, start.y));
            path.push(Pos2::new(end.x + offset, end.y));
            
            path.push(end);
        }
        DependencyType::StartToFinish => {
            // Diagonal arrow from left to right
            path.push(start);
            path.push(end);
        }
    }
    
    path
}

pub fn is_point_near_arrow(point: Pos2, start: Pos2, end: Pos2, tolerance: f32) -> bool {
    // Calculate distance from point to line segment
    let line_vec = end - start;
    let point_vec = point - start;
    
    let line_len_sq = line_vec.x * line_vec.x + line_vec.y * line_vec.y;
    if line_len_sq == 0.0 {
        // Start and end are the same point
        let dist = ((point.x - start.x).powi(2) + (point.y - start.y).powi(2)).sqrt();
        return dist <= tolerance;
    }
    
    // Calculate projection of point onto line
    let t = ((point_vec.x * line_vec.x + point_vec.y * line_vec.y) / line_len_sq).clamp(0.0, 1.0);
    
    // Find the closest point on the line segment
    let closest = start + line_vec * t;
    
    // Calculate distance from point to closest point on line
    let dist = ((point.x - closest.x).powi(2) + (point.y - closest.y).powi(2)).sqrt();
    
    dist <= tolerance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Task, TaskStatus, Priority, Position};
    use crate::domain::goal::Goal;
    use crate::domain::dependency::{Dependency, DependencyType};
    
    #[test]
    fn test_auto_arrange_groups_by_status() {
        let mut map_view = MapView::new();
        
        let mut tasks = vec![
            create_test_task("Task 1", TaskStatus::Todo),
            create_test_task("Task 2", TaskStatus::InProgress),
            create_test_task("Task 3", TaskStatus::Todo),
            create_test_task("Task 4", TaskStatus::Done),
        ];
        
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);
        
        // Tasks with the same status should be closer together
        let todo_tasks: Vec<_> = tasks.iter()
            .filter(|t| t.status == TaskStatus::Todo)
            .collect();
        let in_progress_tasks: Vec<_> = tasks.iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .collect();
            
        // Check that todo tasks are grouped
        if todo_tasks.len() > 1 {
            let dist = distance(&todo_tasks[0].position, &todo_tasks[1].position);
            assert!(dist < 200.0, "Todo tasks should be close together");
        }
    }
    
    #[test]
    fn test_auto_arrange_dependency_chain() {
        let mut map_view = MapView::new();
        
        let task1_id = uuid::Uuid::new_v4();
        let task2_id = uuid::Uuid::new_v4();
        let task3_id = uuid::Uuid::new_v4();
        
        let mut tasks = vec![
            create_test_task_with_id("Task 1", task1_id),
            create_test_task_with_id("Task 2", task2_id),
            create_test_task_with_id("Task 3", task3_id),
        ];
        
        // Create dependency chain: Task1 -> Task2 -> Task3
        let dependencies = vec![
            Dependency::new(task1_id, task2_id, DependencyType::FinishToStart),
            Dependency::new(task2_id, task3_id, DependencyType::FinishToStart),
        ];
        
        map_view.set_dependencies(dependencies);
        map_view.auto_arrange_smart(&mut tasks, &mut vec![]);
        
        // Tasks should be arranged left to right
        let task1 = tasks.iter().find(|t| t.id == task1_id).unwrap();
        let task2 = tasks.iter().find(|t| t.id == task2_id).unwrap();
        let task3 = tasks.iter().find(|t| t.id == task3_id).unwrap();
        
        assert!(task1.position.x < task2.position.x, "Task1 should be to the left of Task2");
        assert!(task2.position.x < task3.position.x, "Task2 should be to the left of Task3");
    }
    
    fn create_test_task(title: &str, status: TaskStatus) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.status = status;
        task
    }
    
    fn create_test_task_with_id(title: &str, id: uuid::Uuid) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.id = id;
        task
    }
    
    fn distance(p1: &Position, p2: &Position) -> f64 {
        ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
    }
}