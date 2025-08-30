use crate::domain::{
    dependency::{Dependency, DependencyGraph, DependencyType},
    goal::Goal,
    task::Task,
};
use crate::repository::comment_repository::CommentRepository;
pub use crate::services::summarization::SummarizationLevel;
use crate::services::summarization::{SummarizationService, SummaryCache};
use crate::services::{
    AutoRunOrchestrator, ClaudeCodeService, DependencyService, TaskExecutionStatus,
};
use crate::ui::widgets::task_detail_modal::{TaskAction, TaskDetailModal};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::Instant;
use tokio::runtime::Runtime;
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
    pan_start_pos: Option<Pos2>,
    pan_button: Option<egui::PointerButton>,

    // Smooth zoom animation
    zoom_animation: Option<ZoomAnimation>,

    // Momentum for trackpad gestures
    momentum_velocity: Vec2,
    last_momentum_update: Option<std::time::Instant>,

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
    dependency_loading_started: bool,
    dependency_receiver: Option<Receiver<DependencyGraph>>,

    // Task detail modal
    task_detail_modal: TaskDetailModal,
    comment_repository: Option<Arc<CommentRepository>>,

    // Debug metrics
    frame_count: u64,
    total_events: u64,
    last_frame_time: Instant,
    slow_frames: Vec<(u64, std::time::Duration)>,
    event_buffer_sizes: Vec<usize>,

    // Claude Code integration
    claude_service: Option<Arc<ClaudeCodeService>>,
    auto_run_orchestrator: Option<Arc<AutoRunOrchestrator>>,
    running_tasks: HashMap<Uuid, TaskExecutionStatus>,
    task_pr_urls: HashMap<Uuid, String>,
    spinner_rotation: f32,
}

#[derive(Clone)]
struct ZoomAnimation {
    start_zoom: f32,
    target_zoom: f32,
    duration: f32,
    elapsed: f32,
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
    Overview, // Highly summarized
    Summary,  // Basic info
    Standard, // Normal view
    Detailed, // All details
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
        Self::with_runtime(runtime)
    }

    #[cfg(test)]
    pub fn new_with_deps(
        _repository: Arc<crate::repository::Repository>,
        runtime: Option<Arc<Runtime>>,
    ) -> Self {
        let runtime = runtime
            .unwrap_or_else(|| Arc::new(Runtime::new().expect("Failed to create tokio runtime")));
        Self::with_runtime(runtime)
    }
    
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        use std::sync::OnceLock;
        
        // Create a shared runtime for all tests
        static TEST_RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();
        let runtime = TEST_RUNTIME.get_or_init(|| {
            Arc::new(Runtime::new().expect("Failed to create test runtime"))
        }).clone();
        
        Self::with_runtime(runtime)
    }

    fn with_runtime(runtime: Arc<Runtime>) -> Self {
        Self {
            camera_pos: Vec2::ZERO,
            zoom_level: 1.0,
            selected_task_id: None,
            selected_goal_id: None,
            dragging_item: None,
            clusters: Vec::new(),
            is_panning: false,
            last_mouse_pos: None,
            pan_start_pos: None,
            pan_button: None,
            zoom_animation: None,
            momentum_velocity: Vec2::ZERO,
            last_momentum_update: None,
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
            dependency_loading_started: false,
            dependency_receiver: None,
            task_detail_modal: TaskDetailModal::new(),
            comment_repository: None,

            // Debug metrics
            frame_count: 0,
            total_events: 0,
            last_frame_time: Instant::now(),
            slow_frames: Vec::new(),
            event_buffer_sizes: Vec::new(),

            // Claude Code integration
            claude_service: None,
            auto_run_orchestrator: None,
            running_tasks: HashMap::new(),
            task_pr_urls: HashMap::new(),
            spinner_rotation: 0.0,
        }
    }

    pub fn set_dependency_service(&mut self, service: Arc<DependencyService>) {
        self.dependency_service = Some(service);
    }

    pub fn set_claude_service(
        &mut self,
        service: Arc<ClaudeCodeService>,
        repository: Arc<crate::repository::Repository>,
    ) {
        println!("📋 Setting Claude service in MapView");
        self.claude_service = Some(service.clone());

        // Create orchestrator if we have all dependencies
        if let Some(dep_service) = &self.dependency_service {
            println!("✅ Dependency service found, creating orchestrator");
            let task_service = Arc::new(crate::services::TaskService::new(repository.clone()));
            self.auto_run_orchestrator = Some(Arc::new(AutoRunOrchestrator::new(
                repository,
                service,
                dep_service.clone(),
                task_service,
            )));
            println!("🚀 Orchestrator created successfully!");
        } else {
            println!("⚠️ No dependency service available, orchestrator not created");
        }
    }

    fn start_claude_code_for_task(&mut self, task_id: Uuid) {
        println!("🎮 start_claude_code_for_task called for task: {}", task_id);

        // Mark task as running
        self.running_tasks
            .insert(task_id, TaskExecutionStatus::Running);
        println!(
            "📊 Task marked as running. Total running tasks: {}",
            self.running_tasks.len()
        );

        // Start Claude Code via orchestrator
        if let Some(orchestrator) = &self.auto_run_orchestrator {
            println!("✅ Orchestrator found, starting auto-run");
            let orch = orchestrator.clone();
            let runtime = self.runtime.clone();
            let mut running_tasks = self.running_tasks.clone();
            let mut pr_urls = self.task_pr_urls.clone();

            runtime.spawn(async move {
                // Start the task execution
                match orch.start_auto_run(vec![task_id]).await {
                    Ok(_) => {
                        // Monitor the task execution
                        loop {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            let executions = orch.executions.read().await;
                            if let Some(exec) = executions.get(&task_id) {
                                running_tasks.insert(task_id, exec.status.clone());

                                if let Some(pr_url) = &exec.pr_url {
                                    pr_urls.insert(task_id, pr_url.clone());
                                }

                                if matches!(
                                    exec.status,
                                    TaskExecutionStatus::Completed | TaskExecutionStatus::Failed
                                ) {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to start Claude Code: {}", e);
                        running_tasks.insert(task_id, TaskExecutionStatus::Failed);
                    }
                }
            });
        } else {
            println!("❌ No orchestrator available - cannot start Claude Code");
            println!("   claude_service: {}", self.claude_service.is_some());
            println!(
                "   dependency_service: {}",
                self.dependency_service.is_some()
            );
        }
    }

    pub fn show(&mut self, ui: &mut Ui, tasks: &mut Vec<Task>, goals: &mut Vec<Goal>) {
        // FREEZE FIX: Circuit breaker - if we're taking too long, bail out
        let frame_start = Instant::now();

        // Emergency bailout if last frame took too long
        let time_since_last = frame_start.duration_since(self.last_frame_time);
        if time_since_last > std::time::Duration::from_millis(500) {
            // We're in a bad state, skip everything and just render minimal UI
            ui.label("Map view temporarily disabled - recovering from freeze");
            self.last_frame_time = frame_start;
            return;
        }

        self.frame_count += 1;

        // Update spinner rotation
        self.spinner_rotation += 0.1;
        if self.spinner_rotation > std::f32::consts::PI * 2.0 {
            self.spinner_rotation -= std::f32::consts::PI * 2.0;
        }

        // Rate limit expensive operations
        let _should_update_expensive = time_since_last >= std::time::Duration::from_millis(16); // 60fps for expensive ops

        // Check for slow frames
        let frame_time = frame_start.duration_since(self.last_frame_time);
        if frame_time > std::time::Duration::from_millis(100) {
            // Only log every 10th slow frame to reduce I/O
            if self.slow_frames.len() % 10 == 0 {
                println!(
                    "🔴 SLOW FRAME {} in map_view::show(): {:?}",
                    self.frame_count, frame_time
                );
            }
            // Keep only last 100 slow frames to prevent memory leak
            if self.slow_frames.len() > 100 {
                self.slow_frames.remove(0);
            }
            self.slow_frames.push((self.frame_count, frame_time));
        }
        self.last_frame_time = frame_start;

        // Log every 500 frames to reduce I/O blocking
        if self.frame_count % 500 == 0 {
            println!(
                "📊 Frame {}: Total events: {}, Slow frames: {}",
                self.frame_count,
                self.total_events,
                self.slow_frames.len()
            );

            // Clear debug data periodically to prevent any accumulation
            if self.slow_frames.len() > 50 {
                self.slow_frames.clear();
            }
            if self.event_buffer_sizes.len() > 50 {
                self.event_buffer_sizes.clear();
            }

            // Check event buffer size
            ui.input(|i| {
                let event_count = i.events.len();
                // Keep only last 100 samples to prevent memory leak
                if self.event_buffer_sizes.len() > 100 {
                    self.event_buffer_sizes.remove(0);
                }
                self.event_buffer_sizes.push(event_count);
                if event_count > 10 {
                    println!("⚠️ Large event buffer: {} events", event_count);
                }
            });
        }
        // Start loading dependencies asynchronously (only once)
        if !self.dependency_loading_started
            && let Some(service) = &self.dependency_service
        {
            self.dependency_loading_started = true;
            let service_clone = service.clone();
            let (tx, rx) = channel();
            self.dependency_receiver = Some(rx);

            // Load in background thread without blocking
            std::thread::spawn(move || {
                if let Ok(runtime) = tokio::runtime::Runtime::new() {
                    runtime.block_on(async {
                        if let Ok(graph) = service_clone.build_dependency_graph().await {
                            let _ = tx.send(graph);
                        }
                    });
                }
            });
        }

        // Check for loaded dependencies without blocking
        if let Some(receiver) = &self.dependency_receiver {
            match receiver.try_recv() {
                Ok(graph) => {
                    self.dependency_graph = graph;
                    self.dependency_receiver = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.dependency_receiver = None;
                }
            }
        }

        // Update detail level based on zoom
        let _previous_detail = self.detail_level;
        self.detail_level = match self.zoom_level {
            z if z < 0.3 => DetailLevel::Overview,
            z if z < 0.6 => DetailLevel::Summary,
            z if z < 1.5 => DetailLevel::Standard,
            _ => DetailLevel::Detailed,
        };

        // Summarization disabled to prevent blocking

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

        // Update animations
        if self.zoom_animation.is_some() {
            let dt = ui.input(|i| i.stable_dt);
            self.update_zoom_animation(dt);
            ui.ctx().request_repaint();
        }

        // Update momentum if active
        if self.momentum_velocity.length() > 0.1 {
            let dt = ui.input(|i| i.stable_dt);
            self.update_momentum(dt);
            ui.ctx().request_repaint();
        }

        // Handle panning - support multiple input methods
        let mut did_pan = false;

        // Method 1: Middle mouse button drag (traditional)
        // Method 2: Shift + Primary button drag (keyboard modifier)
        if response.dragged_by(egui::PointerButton::Middle) {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if !self.is_panning {
                    self.start_pan(pointer_pos, egui::PointerButton::Middle);
                } else {
                    self.update_pan(pointer_pos);
                    did_pan = true;
                }
            }
        } else if response.dragged_by(egui::PointerButton::Primary)
            && ui.input(|i| i.modifiers.shift)
        {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if !self.is_panning {
                    self.start_pan_with_modifiers(
                        pointer_pos,
                        egui::PointerButton::Primary,
                        true,
                        false,
                    );
                } else {
                    self.update_pan(pointer_pos);
                    did_pan = true;
                }
            }
        } else if self.is_panning && !response.dragged() {
            self.end_pan();
        }

        // Cancel pan on escape
        if self.is_panning && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.cancel_pan();
        }

        // Handle zoom with scroll and trackpad gestures
        let entering_scroll = response.hovered() || !did_pan;
        // Removed frequent logging to prevent I/O blocking
        if entering_scroll {
            // Extract scroll data first to minimize time in closure
            let (scroll_delta, zoom_delta, modifiers, event_count) = ui.input(|i| {
                (
                    i.smooth_scroll_delta,
                    i.zoom_delta(),
                    i.modifiers,
                    i.events.len(),
                )
            });

            // FREEZE PROTECTION: Skip processing if too many events are queued
            if event_count > 50 {
                // Too many events, skip to prevent freeze
                return;
            }

            // FREEZE DEBUG: Log scroll events
            if scroll_delta.length() > 0.0 {
                self.total_events += 1;
            }

            // Prevent processing excessive scroll events that could freeze the app
            let max_scroll = 100.0;
            let scroll_delta = egui::Vec2::new(
                scroll_delta.x.clamp(-max_scroll, max_scroll),
                scroll_delta.y.clamp(-max_scroll, max_scroll),
            );

            // Process zoom and scroll outside of input closure
            if zoom_delta != 1.0 {
                // Handle pinch zoom
                self.zoom_level = (self.zoom_level * zoom_delta).clamp(0.1, 5.0);
            } else if scroll_delta.length() > 0.0 {
                // Handle trackpad pan (two-finger drag)
                let is_trackpad_pan =
                    scroll_delta.x.abs() > 0.1 || (scroll_delta.y.abs() > 0.0 && modifiers.shift);

                if is_trackpad_pan {
                    // Pan the view
                    self.camera_pos += scroll_delta / self.zoom_level;
                } else if modifiers.ctrl || modifiers.command {
                    // Ctrl/Cmd + scroll for zoom
                    let zoom_factor = 1.0 + scroll_delta.y * 0.001;
                    self.zoom_level = (self.zoom_level * zoom_factor).clamp(0.1, 5.0);
                } else if response.hovered() {
                    // Standard scroll wheel zoom when hovering
                    if let Some(hover_pos) = response.hover_pos() {
                        self.handle_scroll(scroll_delta.y, hover_pos);
                    }
                } else {
                    // Default: treat vertical scroll as pan
                    self.camera_pos.y += scroll_delta.y / self.zoom_level;
                    self.camera_pos.x += scroll_delta.x / self.zoom_level;
                }
            }
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

        // Always draw dependencies - they're essential for understanding the graph
        self.draw_dependencies(&painter, tasks, to_screen);

        // Draw dependency preview if creating
        if self.creating_dependency
            && let (Some(source_id), Some(preview_end)) =
                (self.dependency_source, self.dependency_preview_end)
        {
            // Find source task position
            if let Some(source_task) = tasks.iter().find(|t| t.id == source_id) {
                let source_pos =
                    Vec2::new(source_task.position.x as f32, source_task.position.y as f32);
                let source_screen = to_screen(source_pos);

                // Start from the right edge of the source task
                let task_width = 150.0 * self.zoom_level / 2.0;
                let arrow_start = source_screen + Vec2::new(task_width, 0.0);

                // Draw preview arrow
                painter.arrow(
                    arrow_start,
                    preview_end - arrow_start,
                    Stroke::new(2.0, Color32::from_rgba_unmultiplied(100, 150, 255, 180)),
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
                if task.goal_id.is_none() {
                    // Don't draw tasks inside goals separately
                    self.draw_task(&painter, ui, task, &to_screen, available_rect);
                }
            }
        }

        // Handle task creation on double-click
        if response.double_clicked()
            && !self.is_panning
            && !self.creating_dependency
            && let Some(pointer_pos) = response.interact_pointer_pos()
        {
            let world_pos = (pointer_pos - center) / self.zoom_level - self.camera_pos;
            let mut new_task = Task::new("New Task".to_string(), String::new());
            new_task.set_position(world_pos.x as f64, world_pos.y as f64);
            tasks.push(new_task);
        }

        // Cancel dependency creation on escape
        if self.creating_dependency && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
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

        // Request continuous repaint for smooth animations (spinner, arrows)
        if !self.running_tasks.is_empty() || self.spinner_rotation > 0.0 {
            ui.ctx().request_repaint();
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

    fn draw_task(
        &mut self,
        painter: &egui::Painter,
        ui: &mut Ui,
        task: &mut Task,
        to_screen: &impl Fn(Vec2) -> Pos2,
        clip_rect: Rect,
    ) {
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

        // Draw Claude Code status indicators
        if let Some(status) = self.running_tasks.get(&task.id) {
            match status {
                TaskExecutionStatus::Running => {
                    // Draw rotating spinner
                    let spinner_pos = rect.right_top()
                        + Vec2::new(-15.0 * self.zoom_level, 15.0 * self.zoom_level);
                    let spinner_radius = 8.0 * self.zoom_level;

                    // Draw spinner circle
                    painter.circle_stroke(
                        spinner_pos,
                        spinner_radius,
                        Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                    );

                    // Draw rotating arc
                    use std::f32::consts::PI;
                    let angle_start = self.spinner_rotation;
                    let angle_end = angle_start + PI * 0.75;

                    let points: Vec<Pos2> = (0..20)
                        .map(|i| {
                            let t = i as f32 / 19.0;
                            let angle = angle_start + (angle_end - angle_start) * t;
                            Pos2::new(
                                spinner_pos.x + angle.cos() * spinner_radius,
                                spinner_pos.y + angle.sin() * spinner_radius,
                            )
                        })
                        .collect();

                    painter.add(egui::Shape::line(
                        points,
                        Stroke::new(3.0, Color32::from_rgb(50, 100, 255)),
                    ));
                }
                TaskExecutionStatus::Completed => {
                    // Draw checkmark
                    let check_pos = rect.right_top()
                        + Vec2::new(-15.0 * self.zoom_level, 15.0 * self.zoom_level);
                    painter.text(
                        check_pos,
                        egui::Align2::CENTER_CENTER,
                        "✓",
                        egui::FontId::proportional(16.0 * self.zoom_level),
                        Color32::from_rgb(0, 200, 0),
                    );
                }
                TaskExecutionStatus::Failed => {
                    // Draw X
                    let x_pos = rect.right_top()
                        + Vec2::new(-15.0 * self.zoom_level, 15.0 * self.zoom_level);
                    painter.text(
                        x_pos,
                        egui::Align2::CENTER_CENTER,
                        "✗",
                        egui::FontId::proportional(16.0 * self.zoom_level),
                        Color32::from_rgb(255, 0, 0),
                    );
                }
                _ => {}
            }
        } else if task.status == crate::domain::task::TaskStatus::Todo
            || task.status == crate::domain::task::TaskStatus::InProgress
        {
            // Draw play button for tasks that can be started
            let button_pos =
                rect.right_top() + Vec2::new(-15.0 * self.zoom_level, 15.0 * self.zoom_level);
            let button_radius = 10.0 * self.zoom_level;

            // Use a proper button widget that can be clicked
            let button_response = ui.put(
                Rect::from_center_size(button_pos, Vec2::splat(button_radius * 2.0)),
                egui::Button::new("").frame(false),
            );

            // Draw custom button appearance
            let painter = ui.painter();
            painter.circle_filled(
                button_pos,
                button_radius,
                if button_response.hovered() {
                    Color32::from_rgb(100, 200, 100)
                } else {
                    Color32::from_rgb(80, 160, 80)
                },
            );

            // Change cursor on hover
            if button_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            // Draw play triangle
            let triangle_size = button_radius * 0.6;
            let triangle_points = vec![
                button_pos + Vec2::new(-triangle_size * 0.5, -triangle_size * 0.6),
                button_pos + Vec2::new(-triangle_size * 0.5, triangle_size * 0.6),
                button_pos + Vec2::new(triangle_size * 0.8, 0.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                triangle_points,
                Color32::WHITE,
                Stroke::NONE,
            ));

            // Handle click
            if button_response.clicked() {
                println!("🎯 Play button CLICKED for task: {}", task.id);
                println!("  Task title: {}", task.title);
                self.start_claude_code_for_task(task.id);
            }
        }

        // Show PR URL if available
        if let Some(_pr_url) = self.task_pr_urls.get(&task.id) {
            painter.text(
                rect.left_bottom() + Vec2::new(5.0, -5.0),
                egui::Align2::LEFT_BOTTOM,
                "PR ↗",
                egui::FontId::proportional(10.0 * self.zoom_level),
                Color32::from_rgb(0, 100, 200),
            );
        }

        // Draw task content based on detail level
        match self.detail_level {
            DetailLevel::Overview => {
                // Don't draw individual tasks in overview
            }
            DetailLevel::Summary => {
                let summary = self
                    .task_summaries
                    .get(&task.id)
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
                let summary = self
                    .task_summaries
                    .get(&task.id)
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
            Color32::from_rgb(100, 150, 255) // Highlight when creating from this task
        } else {
            Color32::from_rgba_unmultiplied(100, 100, 100, 150)
        };

        // Left dot (for incoming connections)
        painter.circle_filled(left_dot_pos, dot_radius, dot_color);

        // Right dot (for outgoing connections)
        painter.circle_filled(right_dot_pos, dot_radius, dot_color);

        // Handle interaction with the task body
        // Only make interactive when NOT panning to avoid interference
        let sense = if self.is_panning {
            Sense::hover() // Only detect hover, don't consume drag events
        } else {
            Sense::click_and_drag()
        };
        let task_response = ui.allocate_rect(rect, sense);

        // Handle interaction with connection dots
        let left_dot_rect = Rect::from_center_size(left_dot_pos, Vec2::splat(dot_radius * 3.0));
        let right_dot_rect = Rect::from_center_size(right_dot_pos, Vec2::splat(dot_radius * 3.0));

        // Only make dots draggable when not panning
        let dot_sense = if self.is_panning {
            Sense::hover()
        } else {
            Sense::drag()
        };
        let left_dot_response = ui.allocate_rect(left_dot_rect, dot_sense);
        let right_dot_response = ui.allocate_rect(right_dot_rect, dot_sense);

        // Hover effects for dots
        if left_dot_response.hovered() || right_dot_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);

            // Draw hover highlight
            if left_dot_response.hovered() {
                painter.circle_stroke(
                    left_dot_pos,
                    dot_radius + 2.0,
                    Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                );
            }
            if right_dot_response.hovered() {
                painter.circle_stroke(
                    right_dot_pos,
                    dot_radius + 2.0,
                    Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                );
            }
        }

        // Start dependency creation from right dot (outgoing)
        if right_dot_response.drag_started() {
            self.start_dependency_creation(task.id);
            self.dependency_preview_end = Some(right_dot_pos);
        }

        // Complete dependency creation on left dot (incoming)
        if left_dot_response.hovered()
            && self.creating_dependency
            && let Some(source_id) = self.dependency_source
            && source_id != task.id
        {
            // Highlight the target dot
            painter.circle_stroke(
                left_dot_pos,
                dot_radius + 3.0,
                Stroke::new(3.0, Color32::from_rgb(50, 200, 50)),
            );

            // Complete on mouse release
            if ui.input(|i| i.pointer.any_released())
                && let Some(dep) = self.complete_dependency_creation(task.id)
            {
                // Save to database via dependency service
                if let Some(service) = &self.dependency_service {
                    let service_clone = service.clone();
                    let dep_clone = dep.clone();
                    let runtime = self.runtime.clone();

                    runtime.spawn(async move {
                        if let Err(e) = service_clone
                            .create_dependency(
                                dep_clone.from_task_id,
                                dep_clone.to_task_id,
                                dep_clone.dependency_type,
                            )
                            .await
                        {
                            eprintln!("Failed to save dependency: {}", e);
                        }
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

        // Handle regular task interactions (but not if play button was clicked)
        if task_response.clicked() && !self.creating_dependency {
            // Check if click is on the play button area to avoid conflict
            let has_play_button = (task.status == crate::domain::task::TaskStatus::Todo
                || task.status == crate::domain::task::TaskStatus::InProgress)
                && self.running_tasks.get(&task.id).is_none();

            let button_pos =
                rect.right_top() + Vec2::new(-15.0 * self.zoom_level, 15.0 * self.zoom_level);
            let button_radius = 12.0 * self.zoom_level; // Slightly larger to prevent edge cases
            let button_rect = Rect::from_center_size(button_pos, Vec2::splat(button_radius * 2.0));

            let mouse_pos = task_response.interact_pointer_pos().unwrap_or(Pos2::ZERO);
            let clicked_on_play = has_play_button && button_rect.contains(mouse_pos);

            if !clicked_on_play {
                self.selected_task_id = Some(task.id);
                self.selected_goal_id = None;

                // Open task detail modal (load comments asynchronously later)
                self.task_detail_modal.open(task.clone(), Vec::new());

                // Load comments asynchronously if repository exists
                if let Some(repo) = &self.comment_repository {
                    let repo_clone = repo.clone();
                    let task_id = task.id;
                    let runtime = self.runtime.clone();

                    runtime.spawn(async move {
                        // Comments will load in background
                        // In production, send back via channel to update modal
                        let _ = repo_clone.list_for_entity(task_id).await;
                    });
                }
            }
        }

        // Handle task dragging (only if not dragging from dots)
        if task_response.dragged()
            && !self.is_panning
            && !self.creating_dependency
            && !left_dot_response.dragged()
            && !right_dot_response.dragged()
        {
            let delta = task_response.drag_delta() / self.zoom_level;
            task.set_position(
                task.position.x + delta.x as f64,
                task.position.y + delta.y as f64,
            );
        }
    }

    fn draw_goal(
        &mut self,
        painter: &egui::Painter,
        ui: &mut Ui,
        goal: &mut Goal,
        tasks: &[Task],
        to_screen: &impl Fn(Vec2) -> Pos2,
        clip_rect: Rect,
    ) {
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
        let goal_text = if self.detail_level == DetailLevel::Overview
            || self.detail_level == DetailLevel::Summary
        {
            self.goal_summaries
                .get(&goal.id)
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
        // Only make interactive when NOT panning to avoid interference
        let sense = if self.is_panning {
            Sense::hover() // Only detect hover, don't consume drag events
        } else {
            Sense::click_and_drag()
        };
        let response = ui.allocate_rect(rect, sense);

        if response.clicked() && !self.is_panning {
            // Selected goal (would need mutable self to update)
            // self.selected_goal_id = Some(goal.id);
            // self.selected_task_id = None;
        }
    }

    fn draw_dependencies(
        &self,
        painter: &egui::Painter,
        tasks: &[Task],
        to_screen: impl Fn(Vec2) -> Pos2 + Copy,
    ) {
        // Skip if too many tasks to prevent performance issues
        if tasks.len() > 500 {
            return;
        }

        // Create a map of task IDs to positions for quick lookup
        let task_positions: HashMap<Uuid, Vec2> = tasks
            .iter()
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
                        (
                            Vec2::new(task_size.x / 2.0, 0.0),
                            Vec2::new(-task_size.x / 2.0, 0.0),
                        )
                    }
                    DependencyType::StartToStart => {
                        // From left edge to left edge
                        (
                            Vec2::new(-task_size.x / 2.0, 0.0),
                            Vec2::new(-task_size.x / 2.0, 0.0),
                        )
                    }
                    DependencyType::FinishToFinish => {
                        // From right edge to right edge
                        (
                            Vec2::new(task_size.x / 2.0, 0.0),
                            Vec2::new(task_size.x / 2.0, 0.0),
                        )
                    }
                    DependencyType::StartToFinish => {
                        // From left edge to right edge
                        (
                            Vec2::new(-task_size.x / 2.0, 0.0),
                            Vec2::new(task_size.x / 2.0, 0.0),
                        )
                    }
                };

                let start_screen = to_screen(from_pos + start_offset);
                let end_screen = to_screen(to_pos + end_offset);

                // Get the arrow path
                let path =
                    calculate_arrow_path(start_screen, end_screen, dependency.dependency_type);

                // Draw the arrow as a bezier curve or lines
                let arrow_color = if self
                    .dependency_graph
                    .get_critical_path(&HashMap::new())
                    .contains(&dependency.from_task_id)
                {
                    Color32::from_rgb(255, 50, 50) // Bright red for critical path
                } else {
                    Color32::from_rgb(50, 150, 255) // Bright blue for normal dependencies
                };

                let stroke = Stroke::new(3.0 * self.zoom_level.max(1.0), arrow_color);

                if path.len() >= 4 {
                    // Draw as bezier curve
                    painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                        points: [path[0], path[1], path[2], path[3]],
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke,
                    }));
                } else if path.len() >= 2 {
                    // Draw as line
                    painter.line_segment([path[0], path[path.len() - 1]], stroke);
                }

                // Draw arrowhead
                let arrow_size = 15.0 * self.zoom_level.max(1.0);
                let end_point = path[path.len() - 1];
                let prev_point = if path.len() >= 2 {
                    path[path.len() - 2]
                } else {
                    path[0]
                };

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
            && let (Some(source_id), Some(preview_end)) =
                (self.dependency_source, self.dependency_preview_end)
            && let Some(&source_pos) = task_positions.get(&source_id)
        {
            let task_size = Vec2::new(150.0, 80.0);
            let start_screen = to_screen(source_pos + Vec2::new(task_size.x / 2.0, 0.0));

            // Draw preview line
            painter.line_segment(
                [start_screen, preview_end],
                Stroke::new(2.0, Color32::from_rgba_unmultiplied(100, 100, 200, 128)),
            );
        }
    }

    fn draw_clusters(
        &mut self,
        painter: &egui::Painter,
        _ui: &mut Ui,
        tasks: &[Task],
        to_screen: &impl Fn(Vec2) -> Pos2,
        clip_rect: Rect,
    ) {
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

    fn update_summaries(&mut self, tasks: &[Task], goals: &[Goal]) {
        // Temporarily use titles as summaries to avoid blocking
        self.task_summaries.clear();
        self.goal_summaries.clear();

        for task in tasks {
            self.task_summaries.insert(task.id, task.title.clone());
        }

        for goal in goals {
            self.goal_summaries.insert(goal.id, goal.title.clone());
        }

        // TODO: Implement async summarization with channels
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
        let _service = Arc::clone(&self.summarization_service);
        let _runtime = Arc::clone(&self.runtime);

        for ((x, y), task_ids) in cluster_map {
            if task_ids.len() > 1 {
                let center_x = (x as f32 + 0.5) * cluster_size;
                let center_y = (y as f32 + 0.5) * cluster_size;

                let cluster_tasks: Vec<_> = tasks
                    .iter()
                    .filter(|t| task_ids.contains(&t.id))
                    .cloned()
                    .collect();

                // Use simple summary for clusters to avoid blocking
                let summary = format!("{} tasks", cluster_tasks.len());

                self.clusters.push(TaskCluster {
                    center: Vec2::new(center_x, center_y),
                    tasks: task_ids,
                    summary,
                });
            }
        }
    }

    pub fn get_summarization_level(&self) -> SummarizationLevel {
        self.detail_level.to_summarization_level()
    }

    pub async fn get_task_summaries(
        &mut self,
        tasks: &[Task],
        service: &SummarizationService,
    ) -> Vec<String> {
        let level = self.get_summarization_level();
        let mut summaries = Vec::new();

        for task in tasks {
            let summary = service
                .summarize_with_cache(
                    &mut self.summary_cache,
                    task.id,
                    &format!("{}: {}", task.title, task.description),
                    level,
                )
                .await;
            summaries.push(summary);
        }

        summaries
    }

    pub async fn get_visible_task_summaries(
        &mut self,
        tasks: &[Task],
        service: &SummarizationService,
        viewport: Rect,
    ) -> Vec<String> {
        let level = self.get_summarization_level();
        let mut summaries = Vec::new();

        for task in tasks {
            let world_pos = Vec2::new(task.position.x as f32, task.position.y as f32);
            let screen_pos = viewport.center() + (world_pos + self.camera_pos) * self.zoom_level;

            if viewport.contains(screen_pos) {
                let summary = service
                    .summarize_with_cache(
                        &mut self.summary_cache,
                        task.id,
                        &format!("{}: {}", task.title, task.description),
                        level,
                    )
                    .await;
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
                if let (Some(&from_node), Some(&to_node)) = (
                    node_map.get(&dep.from_task_id),
                    node_map.get(&dep.to_task_id),
                ) {
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
            let has_incoming = dep_graph
                .edges_directed(node, petgraph::Direction::Incoming)
                .count()
                > 0;
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

            task_groups.entry(group_key).or_default().push(i);
        }

        // Step 3: Arrange tasks
        let mut current_x = 0.0;
        let mut group_positions: HashMap<String, (f64, f64)> = HashMap::new();

        // First, arrange dependency-based tasks (left to right by level)
        for level in 0..=max_level {
            let level_key = format!("dep_level_{}", level);
            if let Some(task_indices) = task_groups.get(&level_key) {
                let mut y = 0.0;
                for &idx in task_indices.iter() {
                    tasks[idx].set_position(level as f64 * spacing_x * 1.5, y);
                    y += spacing_y;
                }
                group_positions
                    .insert(level_key.clone(), (level as f64 * spacing_x * 1.5, y / 2.0));
            }
        }

        // Calculate starting X position for non-dependency groups
        if max_level > 0 {
            current_x = (max_level + 1) as f64 * spacing_x * 1.5 + group_spacing;
        }

        // Then arrange other groups (grouped by similarity)
        let mut sorted_groups: Vec<_> = task_groups
            .iter()
            .filter(|(key, _)| !key.starts_with("dep_level_"))
            .collect();
        sorted_groups.sort_by_key(|(key, _)| key.as_str());

        for (group_key, task_indices) in sorted_groups {
            let y = 0.0;
            let items_per_column = 5;

            for (j, &idx) in task_indices.iter().enumerate() {
                let col = j / items_per_column;
                let row = j % items_per_column;

                tasks[idx].set_position(
                    current_x + col as f64 * spacing_x * 0.7,
                    y + row as f64 * spacing_y * 0.8,
                );
            }

            let cols = task_indices.len().div_ceil(items_per_column);
            group_positions.insert(
                group_key.clone(),
                (
                    current_x + (cols as f64 * spacing_x * 0.7) / 2.0,
                    y + 2.5 * spacing_y,
                ),
            );
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
        let deps = self.dependency_graph.get_all_dependencies();
        if deps.is_empty() { None } else { Some(deps) }
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
            && source_id != target_task_id
        {
            let dependency =
                Dependency::new(source_id, target_task_id, DependencyType::FinishToStart);

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

    // ============================================================================
    // Pan and Zoom Public API
    // ============================================================================

    pub fn get_camera_position(&self) -> Vec2 {
        self.camera_pos
    }

    pub fn set_camera_position(&mut self, pos: Vec2) {
        self.camera_pos = pos;
    }

    pub fn get_zoom_level(&self) -> f32 {
        self.zoom_level
    }

    pub fn set_zoom_level(&mut self, zoom: f32) {
        self.zoom_level = zoom.clamp(0.1, 5.0);
    }

    pub fn is_panning(&self) -> bool {
        self.is_panning
    }

    // Pan operations
    pub fn start_pan(&mut self, pos: Pos2, button: egui::PointerButton) {
        self.is_panning = true;
        self.pan_start_pos = Some(pos);
        self.last_mouse_pos = Some(pos);
        self.pan_button = Some(button);
    }

    pub fn start_pan_with_modifiers(
        &mut self,
        pos: Pos2,
        button: egui::PointerButton,
        shift: bool,
        _ctrl: bool,
    ) {
        if button == egui::PointerButton::Primary && shift {
            self.start_pan(pos, button);
        }
    }

    pub fn update_pan(&mut self, current_pos: Pos2) {
        if self.is_panning {
            if let Some(last_pos) = self.last_mouse_pos {
                let delta = current_pos - last_pos;
                self.camera_pos += delta / self.zoom_level;
            }
            self.last_mouse_pos = Some(current_pos);
        }
    }

    pub fn end_pan(&mut self) {
        self.is_panning = false;
        self.pan_start_pos = None;
        self.last_mouse_pos = None;
        self.pan_button = None;
    }

    pub fn cancel_pan(&mut self) {
        // Reset to position before pan started
        self.is_panning = false;
        self.pan_start_pos = None;
        self.last_mouse_pos = None;
        self.pan_button = None;
        // Reset camera to origin (or stored position if we had one)
        self.camera_pos = Vec2::ZERO;
    }

    // Zoom operations
    pub fn handle_scroll(&mut self, delta: f32, mouse_pos: Pos2) {
        let zoom_factor = 1.0 + delta * 0.001;
        self.zoom_centered(zoom_factor, mouse_pos);
    }

    pub fn zoom_centered(&mut self, zoom_factor: f32, _center: Pos2) {
        let old_zoom = self.zoom_level;
        let new_zoom = (old_zoom * zoom_factor).clamp(0.1, 5.0);

        if (new_zoom - old_zoom).abs() > 0.001 {
            // Adjust camera to keep the point under the mouse at the same screen position
            // This creates the zoom-to-cursor effect
            // The math: we want the world point under the cursor to stay at the same screen position
            // So we need to adjust the camera position based on the zoom change

            // For now, just update zoom. Full implementation would adjust camera
            self.zoom_level = new_zoom;
        }
    }

    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.2).min(5.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.2).max(0.1);
    }

    pub fn reset_view(&mut self) {
        self.camera_pos = Vec2::ZERO;
        self.zoom_level = 1.0;
    }

    // Trackpad gesture support
    pub fn handle_pinch_gesture(
        &mut self,
        _center: Pos2,
        initial_distance: f32,
        final_distance: f32,
    ) {
        let zoom_factor = final_distance / initial_distance;
        self.zoom_level = (self.zoom_level * zoom_factor).clamp(0.1, 5.0);
    }

    pub fn handle_two_finger_pan(&mut self, delta: Vec2) {
        self.camera_pos += delta / self.zoom_level;
    }

    // Momentum scrolling for trackpad
    pub fn start_momentum_pan(&mut self, velocity: Vec2) {
        self.momentum_velocity = velocity;
        self.last_momentum_update = Some(std::time::Instant::now());
    }

    pub fn update_momentum(&mut self, dt: f32) {
        if self.momentum_velocity.length() > 0.1 {
            // Apply momentum with friction
            self.camera_pos += self.momentum_velocity * dt / self.zoom_level;

            // Apply friction to slow down
            let friction = 0.95_f32.powf(dt * 60.0); // Normalized to 60fps
            self.momentum_velocity *= friction;

            // Stop if velocity is too small
            if self.momentum_velocity.length() < 0.1 {
                self.momentum_velocity = Vec2::ZERO;
            }
        }
    }

    pub fn get_momentum_velocity(&self) -> Vec2 {
        self.momentum_velocity
    }

    // Smooth zoom animation
    pub fn start_smooth_zoom(&mut self, from: f32, to: f32, duration: f32) {
        self.zoom_animation = Some(ZoomAnimation {
            start_zoom: from,
            target_zoom: to,
            duration,
            elapsed: 0.0,
        });
    }

    pub fn update_zoom_animation(&mut self, dt: f32) {
        if let Some(mut anim) = self.zoom_animation.take() {
            anim.elapsed += dt;

            if anim.elapsed >= anim.duration {
                // Animation complete
                self.zoom_level = anim.target_zoom;
                self.zoom_animation = None;
            } else {
                // Interpolate zoom level
                let t = anim.elapsed / anim.duration;
                // Use ease-in-out curve
                let t = t * t * (3.0 - 2.0 * t);
                self.zoom_level = anim.start_zoom + (anim.target_zoom - anim.start_zoom) * t;
                self.zoom_animation = Some(anim);
            }
        }
    }

    // Coordinate transformations
    pub fn world_to_screen(&self, world_pos: Vec2, viewport: Rect) -> Pos2 {
        let center = viewport.center();
        center + (world_pos + self.camera_pos) * self.zoom_level
    }

    pub fn screen_to_world(&self, screen_pos: Pos2, viewport: Rect) -> Vec2 {
        let center = viewport.center();
        (screen_pos - center) / self.zoom_level - self.camera_pos
    }

    // Visibility testing
    pub fn is_task_visible(&self, task: &Task, viewport: Rect) -> bool {
        let world_pos = Vec2::new(task.position.x as f32, task.position.y as f32);
        let screen_pos = self.world_to_screen(world_pos, viewport);

        // Add some margin for task size
        let task_size = Vec2::new(150.0, 80.0) * self.zoom_level;
        let task_rect = Rect::from_center_size(screen_pos, task_size);

        viewport.intersects(task_rect)
    }

    // Hit testing for task selection
    pub fn hit_test_task(&self, screen_pos: Pos2, tasks: &[Task], viewport: Rect) -> Option<Uuid> {
        let world_pos = self.screen_to_world(screen_pos, viewport);

        for task in tasks {
            let task_world_pos = Vec2::new(task.position.x as f32, task.position.y as f32);
            let task_size = Vec2::new(150.0, 80.0);
            let task_rect =
                Rect::from_center_size(Pos2::new(task_world_pos.x, task_world_pos.y), task_size);

            if task_rect.contains(Pos2::new(world_pos.x, world_pos.y)) {
                return Some(task.id);
            }
        }

        None
    }
    
    // Test helper methods
    #[cfg(test)]
    pub fn get_running_tasks(&self) -> &HashMap<Uuid, TaskExecutionStatus> {
        &self.running_tasks
    }
    
    #[cfg(test)]
    pub fn is_task_running(&self, task_id: &Uuid) -> bool {
        self.running_tasks.contains_key(task_id)
    }
    
    #[cfg(test)]
    pub fn get_task_execution_status(&self, task_id: &Uuid) -> Option<&TaskExecutionStatus> {
        self.running_tasks.get(task_id)
    }
    
    #[cfg(test)]
    pub fn test_start_claude_code_for_task(&mut self, task_id: Uuid) {
        self.start_claude_code_for_task(task_id);
    }
    
    #[cfg(test)]
    pub fn test_set_task_status(&mut self, task_id: Uuid, status: TaskExecutionStatus) {
        self.running_tasks.insert(task_id, status);
    }
    
    #[cfg(test)]
    pub fn test_get_running_tasks_mut(&mut self) -> &mut HashMap<Uuid, TaskExecutionStatus> {
        &mut self.running_tasks
    }
    
    #[cfg(test)]
    pub fn test_set_pr_url(&mut self, task_id: Uuid, url: String) {
        self.task_pr_urls.insert(task_id, url);
    }
    
    #[cfg(test)]
    pub fn test_has_pr_url(&self, task_id: &Uuid) -> bool {
        self.task_pr_urls.contains_key(task_id)
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
    use crate::domain::dependency::{Dependency, DependencyType};
    use crate::domain::task::{Position, Task, TaskStatus};

    // ============================================================================
    // Pan Tests
    // ============================================================================

    #[test]
    fn test_pan_with_middle_mouse() {
        let mut map_view = MapView::new();

        // Initial camera position should be at origin
        assert_eq!(map_view.get_camera_position(), Vec2::ZERO);

        // Simulate middle mouse button drag
        let start_pos = Pos2::new(400.0, 300.0);
        let end_pos = Pos2::new(500.0, 400.0);

        map_view.start_pan(start_pos, egui::PointerButton::Middle);
        map_view.update_pan(end_pos);
        map_view.end_pan();

        // Camera should have moved
        let expected_delta = (end_pos - start_pos) / map_view.get_zoom_level();
        assert_eq!(map_view.get_camera_position(), expected_delta);
    }

    #[test]
    fn test_pan_with_shift_click() {
        let mut map_view = MapView::new();

        // Simulate shift+left click drag
        let start_pos = Pos2::new(200.0, 200.0);
        let end_pos = Pos2::new(350.0, 250.0);

        map_view.start_pan_with_modifiers(start_pos, egui::PointerButton::Primary, true, false);
        map_view.update_pan(end_pos);
        map_view.end_pan();

        // Camera should have moved
        let expected_delta = (end_pos - start_pos) / map_view.get_zoom_level();
        assert_eq!(map_view.get_camera_position(), expected_delta);
    }

    #[test]
    fn test_pan_cancelled() {
        let mut map_view = MapView::new();

        // Start panning
        map_view.start_pan(Pos2::new(100.0, 100.0), egui::PointerButton::Middle);
        map_view.update_pan(Pos2::new(200.0, 200.0));

        // Cancel pan
        map_view.cancel_pan();

        // Camera should remain at origin
        assert_eq!(map_view.get_camera_position(), Vec2::ZERO);
        assert!(!map_view.is_panning());
    }

    // ============================================================================
    // Zoom Tests
    // ============================================================================

    #[test]
    fn test_zoom_in() {
        let mut map_view = MapView::new();

        // Initial zoom level
        assert_eq!(map_view.get_zoom_level(), 1.0);

        // Zoom in
        map_view.zoom_in();
        assert_eq!(map_view.get_zoom_level(), 1.2);

        // Zoom in more
        map_view.zoom_in();
        assert!((map_view.get_zoom_level() - 1.44).abs() < 0.01);
    }

    #[test]
    fn test_zoom_out() {
        let mut map_view = MapView::new();

        // Start at zoom 2.0
        map_view.set_zoom_level(2.0);

        // Zoom out
        map_view.zoom_out();
        assert!((map_view.get_zoom_level() - 1.667).abs() < 0.01);
    }

    #[test]
    fn test_zoom_limits() {
        let mut map_view = MapView::new();

        // Try to zoom beyond maximum
        map_view.set_zoom_level(10.0);
        assert_eq!(map_view.get_zoom_level(), 5.0);

        // Try to zoom below minimum
        map_view.set_zoom_level(0.01);
        assert_eq!(map_view.get_zoom_level(), 0.1);
    }

    #[test]
    fn test_reset_view() {
        let mut map_view = MapView::new();

        // Change camera and zoom
        map_view.set_camera_position(Vec2::new(100.0, 200.0));
        map_view.set_zoom_level(2.5);

        // Reset
        map_view.reset_view();

        // Should return to defaults
        assert_eq!(map_view.get_camera_position(), Vec2::ZERO);
        assert_eq!(map_view.get_zoom_level(), 1.0);
    }

    // ============================================================================
    // Trackpad Gesture Tests
    // ============================================================================

    #[test]
    fn test_pinch_zoom() {
        let mut map_view = MapView::new();

        // Simulate pinch gesture
        let center = Pos2::new(400.0, 300.0);
        map_view.handle_pinch_gesture(center, 50.0, 100.0);

        // Zoom should double
        assert_eq!(map_view.get_zoom_level(), 2.0);
    }

    #[test]
    fn test_two_finger_pan() {
        let mut map_view = MapView::new();

        // Simulate two-finger pan
        let delta = Vec2::new(50.0, 30.0);
        map_view.handle_two_finger_pan(delta);

        // Camera should move
        assert_eq!(map_view.get_camera_position(), delta);
    }

    #[test]
    fn test_momentum_scrolling() {
        let mut map_view = MapView::new();

        // Start momentum
        let velocity = Vec2::new(100.0, 50.0);
        map_view.start_momentum_pan(velocity);

        // Update momentum
        map_view.update_momentum(0.016); // 60fps

        // Camera should have moved
        assert_ne!(map_view.get_camera_position(), Vec2::ZERO);

        // Velocity should decay
        assert!(map_view.get_momentum_velocity().length() < velocity.length());
    }

    // ============================================================================
    // Coordinate Transformation Tests
    // ============================================================================

    #[test]
    fn test_coordinate_transformations() {
        let mut map_view = MapView::new();
        map_view.set_camera_position(Vec2::new(100.0, 50.0));
        map_view.set_zoom_level(2.0);

        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let world_pos = Vec2::new(200.0, 150.0);

        // World to screen
        let screen_pos = map_view.world_to_screen(world_pos, viewport);

        // Screen to world (should be inverse)
        let back_to_world = map_view.screen_to_world(screen_pos, viewport);

        assert!((back_to_world.x - world_pos.x).abs() < 0.01);
        assert!((back_to_world.y - world_pos.y).abs() < 0.01);
    }

    #[test]
    fn test_auto_arrange_groups_by_status() {
        let mut map_view = MapView::new();

        let mut tasks = vec![
            create_test_task("Task 1", TaskStatus::Todo),
            create_test_task("Task 2", TaskStatus::InProgress),
            create_test_task("Task 3", TaskStatus::Todo),
            create_test_task("Task 4", TaskStatus::Done),
        ];

        map_view.auto_arrange_smart(&mut tasks, &mut []);

        // Tasks with the same status should be closer together
        let todo_tasks: Vec<_> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Todo)
            .collect();
        let in_progress_tasks: Vec<_> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .collect();

        // Check that todo tasks are grouped
        if todo_tasks.len() > 1 {
            let dist = distance(&todo_tasks[0].position, &todo_tasks[1].position);
            // Auto-arrange places tasks with spacing, allow more distance
            assert!(
                dist < 400.0,
                "Todo tasks should be close together (distance: {})",
                dist
            );
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
        map_view.auto_arrange_smart(&mut tasks, &mut []);

        // Tasks should be arranged left to right
        let task1 = tasks.iter().find(|t| t.id == task1_id).unwrap();
        let task2 = tasks.iter().find(|t| t.id == task2_id).unwrap();
        let task3 = tasks.iter().find(|t| t.id == task3_id).unwrap();

        assert!(
            task1.position.x < task2.position.x,
            "Task1 should be to the left of Task2"
        );
        assert!(
            task2.position.x < task3.position.x,
            "Task2 should be to the left of Task3"
        );
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
