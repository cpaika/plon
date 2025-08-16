mod domain;
mod repository;
mod services;
mod ui;
mod utils;

use anyhow::Result;
use eframe::egui;
use repository::Repository;
use std::sync::Arc;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create a runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;
    
    // Initialize database synchronously in the runtime
    let repository = runtime.block_on(async {
        let pool = repository::database::init_database("plon.db").await?;
        Ok::<Repository, anyhow::Error>(Repository::new(pool))
    })?;

    // Run the native app
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Plon - Project Management",
        options,
        Box::new(move |cc| Box::new(PlonApp::new(cc, repository))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))?;

    Ok(())
}

use domain::{task::Task, goal::Goal, resource::Resource, dependency::DependencyType};
use services::{TaskService, GoalService, ResourceService, RecurringService, DependencyService};
use ui::views::recurring_view::RecurringView;
use ui::views::map_view::MapView;

pub struct PlonApp {
    repository: Arc<Repository>,
    task_service: Arc<TaskService>,
    goal_service: Arc<GoalService>,
    resource_service: Arc<ResourceService>,
    recurring_service: Arc<RecurringService>,
    dependency_service: Arc<DependencyService>,
    
    // UI State
    current_view: ViewType,
    selected_task_id: Option<uuid::Uuid>,
    show_task_editor: bool,
    show_goal_editor: bool,
    
    // Data
    tasks: Vec<Task>,
    goals: Vec<Goal>,
    resources: Vec<Resource>,
    
    // View instances
    recurring_view: RecurringView,
    map_view: MapView,
    
    // New task form
    new_task_title: String,
    new_task_description: String,
    
    // Map view state (kept for backward compatibility)
    camera_pos: egui::Vec2,
    zoom: f32,
    
    // Filter state
    filter_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewType {
    List,
    Kanban,
    Map,
    Timeline,
    Dashboard,
    Recurring,
}

impl PlonApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, repository: Repository) -> Self {
        let repository = Arc::new(repository);
        
        let task_service = Arc::new(TaskService::new(repository.clone()));
        let goal_service = Arc::new(GoalService::new(repository.clone()));
        let resource_service = Arc::new(ResourceService::new(repository.clone()));
        let recurring_service = Arc::new(RecurringService::new(repository.clone()));
        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        
        // Load initial data synchronously (for now - in production, this would be async)
        let tasks = std::thread::spawn({
            let service = task_service.clone();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(service.list_all())
                    .unwrap_or_default()
            }
        }).join().unwrap_or_default();
        
        let mut map_view = MapView::new();
        map_view.set_dependency_service(dependency_service.clone());
        
        Self {
            repository: repository.clone(),
            task_service,
            goal_service,
            resource_service,
            recurring_service,
            dependency_service,
            
            current_view: ViewType::Map,
            selected_task_id: None,
            show_task_editor: false,
            show_goal_editor: false,
            
            tasks,
            goals: Vec::new(),
            resources: Vec::new(),
            
            recurring_view: RecurringView::new(),
            map_view,
            
            new_task_title: String::new(),
            new_task_description: String::new(),
            
            camera_pos: egui::Vec2::ZERO,
            zoom: 1.0,
            filter_text: String::new(),
        }
    }
    
    fn create_task(&mut self) {
        if !self.new_task_title.is_empty() {
            let mut task = Task::new(
                self.new_task_title.clone(),
                self.new_task_description.clone()
            );
            
            // Position based on current view center
            task.set_position(
                (-self.camera_pos.x as f64 / self.zoom as f64),
                (-self.camera_pos.y as f64 / self.zoom as f64)
            );
            
            // Clone for async operation
            let task_clone = task.clone();
            let service = self.task_service.clone();
            
            // Save to database in background thread
            let handle = std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(async {
                    match service.create(task_clone).await {
                        Ok(_) => println!("Task saved successfully"),
                        Err(e) => eprintln!("Failed to save task: {}", e),
                    }
                })
            });
            
            // Add to local list immediately for UI responsiveness
            self.tasks.push(task);
            
            // Optionally wait for save to complete (or let it run in background)
            // handle.join().unwrap();
            
            // Clear form
            self.new_task_title.clear();
            self.new_task_description.clear();
            self.show_task_editor = false;
        }
    }
}

impl eframe::App for PlonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎯 Plon");
                ui.separator();
                
                // View selector
                ui.selectable_value(&mut self.current_view, ViewType::List, "📋 List");
                ui.selectable_value(&mut self.current_view, ViewType::Kanban, "📊 Kanban");
                ui.selectable_value(&mut self.current_view, ViewType::Map, "🗺️ Map");
                ui.selectable_value(&mut self.current_view, ViewType::Timeline, "📅 Timeline");
                ui.selectable_value(&mut self.current_view, ViewType::Dashboard, "📈 Dashboard");
                ui.selectable_value(&mut self.current_view, ViewType::Recurring, "🔄 Recurring");
                
                ui.separator();
                
                // Quick actions
                if ui.button("➕ New Task").clicked() {
                    self.show_task_editor = true;
                }
                
                if ui.button("🎯 New Goal").clicked() {
                    self.show_goal_editor = true;
                }
                
                ui.separator();
                
                // Search bar
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.filter_text);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄 Refresh").clicked() {
                        // Reload data
                        let service = self.task_service.clone();
                        if let Ok(tasks) = std::thread::spawn(move || {
                            tokio::runtime::Runtime::new()
                                .unwrap()
                                .block_on(service.list_all())
                        }).join() {
                            if let Ok(tasks) = tasks {
                                self.tasks = tasks;
                            }
                        }
                    }
                });
            });
        });
        
        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                ViewType::List => self.show_list_view(ui),
                ViewType::Kanban => self.show_kanban_view(ui),
                ViewType::Map => self.show_map_view(ui),
                ViewType::Timeline => self.show_timeline_view(ui),
                ViewType::Dashboard => self.show_dashboard_view(ui),
                ViewType::Recurring => self.show_recurring_view(ui),
            }
        });
        
        // Task editor modal
        if self.show_task_editor {
            egui::Window::new("Task Editor")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Title:");
                        ui.text_edit_singleline(&mut self.new_task_title);
                    });
                    
                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_task_description);
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            self.create_task();
                        }
                        
                        if ui.button("Cancel").clicked() {
                            self.show_task_editor = false;
                            self.new_task_title.clear();
                            self.new_task_description.clear();
                        }
                    });
                });
        }
    }
}

// View implementations
impl PlonApp {
    fn show_map_view(&mut self, ui: &mut egui::Ui) {
        // Use the MapView struct which has dependency support
        self.map_view.show(ui, &mut self.tasks, &mut self.goals);
    }
    
    fn show_list_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Task List");
        
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical()
            .id_source("list_view_scroll")
            .show(ui, |ui| {
            for task in &mut self.tasks {
                // Apply filter
                if !self.filter_text.is_empty() && 
                   !task.title.to_lowercase().contains(&self.filter_text.to_lowercase()) {
                    continue;
                }
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Status indicator
                        let status_color = match task.status {
                            domain::task::TaskStatus::Todo => egui::Color32::GRAY,
                            domain::task::TaskStatus::InProgress => egui::Color32::from_rgb(100, 150, 255),
                            domain::task::TaskStatus::Done => egui::Color32::from_rgb(100, 255, 100),
                            _ => egui::Color32::DARK_GRAY,
                        };
                        
                        ui.colored_label(status_color, "●");
                        
                        // Task title (editable)
                        ui.text_edit_singleline(&mut task.title);
                        
                        // Priority
                        ui.label(format!("[{:?}]", task.priority));
                        
                        // Progress
                        if !task.subtasks.is_empty() {
                            let (completed, total) = task.subtask_progress();
                            ui.label(format!("{}/{}", completed, total));
                        }
                        
                        // Due date
                        if let Some(due) = task.due_date {
                            if task.is_overdue() {
                                ui.colored_label(egui::Color32::RED, format!("⚠ {}", due.format("%Y-%m-%d")));
                            } else {
                                ui.label(format!("📅 {}", due.format("%Y-%m-%d")));
                            }
                        }
                    });
                });
            }
        });
    }
    
    fn show_kanban_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Kanban Board");
        
        egui::ScrollArea::horizontal()
            .id_source("kanban_board_main")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
            let columns = [
                ("📋 To Do", domain::task::TaskStatus::Todo),
                ("🚀 In Progress", domain::task::TaskStatus::InProgress),
                ("👀 Review", domain::task::TaskStatus::Review),
                ("✅ Done", domain::task::TaskStatus::Done),
            ];
            
            for (title, status) in columns {
                ui.vertical(|ui| {
                    ui.set_min_width(250.0);
                    ui.heading(title);
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .id_source(format!("kanban_column_{:?}", status))
                        .max_height(600.0)
                        .show(ui, |ui| {
                            for task in &self.tasks {
                                if task.status == status {
                                    ui.group(|ui| {
                                        ui.label(&task.title);
                                        
                                        if !task.subtasks.is_empty() {
                                            let (completed, total) = task.subtask_progress();
                                            ui.label(format!("Progress: {}/{}", completed, total));
                                        }
                                        
                                        if let Some(due) = task.due_date {
                                            ui.label(format!("Due: {}", due.format("%Y-%m-%d")));
                                        }
                                    });
                                }
                            }
                        });
                });
                
                ui.separator();
            }
                });
            });
    }
    
    fn show_timeline_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Timeline View");
        
        let today = chrono::Utc::now();
        
        ui.label("Scheduled Tasks:");
        for task in &self.tasks {
            if let Some(scheduled) = task.scheduled_date {
                if scheduled >= today {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: ", scheduled.format("%Y-%m-%d")));
                        ui.label(&task.title);
                    });
                }
            }
        }
        
        ui.separator();
        
        ui.label("Due Tasks:");
        for task in &self.tasks {
            if let Some(due) = task.due_date {
                ui.horizontal(|ui| {
                    if task.is_overdue() {
                        ui.colored_label(egui::Color32::RED, format!("{}: ", due.format("%Y-%m-%d")));
                        ui.colored_label(egui::Color32::RED, &task.title);
                    } else {
                        ui.label(format!("{}: ", due.format("%Y-%m-%d")));
                        ui.label(&task.title);
                    }
                });
            }
        }
    }
    
    fn show_dashboard_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Project Dashboard");
        
        ui.horizontal(|ui| {
            // Task statistics
            ui.group(|ui| {
                ui.label("📊 Task Statistics");
                ui.separator();
                
                let total_tasks = self.tasks.len();
                let completed_tasks = self.tasks.iter()
                    .filter(|t| t.status == domain::task::TaskStatus::Done)
                    .count();
                let in_progress = self.tasks.iter()
                    .filter(|t| t.status == domain::task::TaskStatus::InProgress)
                    .count();
                let blocked = self.tasks.iter()
                    .filter(|t| t.status == domain::task::TaskStatus::Blocked)
                    .count();
                
                ui.label(format!("Total: {}", total_tasks));
                ui.label(format!("Completed: {}", completed_tasks));
                ui.label(format!("In Progress: {}", in_progress));
                ui.label(format!("Blocked: {}", blocked));
                
                if total_tasks > 0 {
                    let completion = (completed_tasks as f32 / total_tasks as f32) * 100.0;
                    ui.label(format!("Completion: {:.1}%", completion));
                    ui.add(egui::ProgressBar::new(completion / 100.0));
                }
            });
            
            ui.separator();
            
            // Goal statistics
            ui.group(|ui| {
                ui.label("🎯 Goal Statistics");
                ui.separator();
                ui.label(format!("Total Goals: {}", self.goals.len()));
            });
            
            ui.separator();
            
            // Overdue tasks
            ui.group(|ui| {
                ui.label("⚠️ Overdue Tasks");
                ui.separator();
                
                let overdue: Vec<_> = self.tasks.iter()
                    .filter(|t| t.is_overdue())
                    .collect();
                
                if overdue.is_empty() {
                    ui.label("No overdue tasks!");
                } else {
                    for task in overdue.iter().take(5) {
                        ui.label(&task.title);
                    }
                    if overdue.len() > 5 {
                        ui.label(format!("... and {} more", overdue.len() - 5));
                    }
                }
            });
        });
    }
    
    fn show_recurring_view(&mut self, ui: &mut egui::Ui) {
        self.recurring_view.show(ui, Some(&*self.recurring_service));
    }
}