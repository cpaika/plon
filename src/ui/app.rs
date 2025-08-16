use crate::domain::{task::Task, goal::Goal, resource::Resource};
use crate::repository::Repository;
use crate::services::{TaskService, GoalService, ResourceService, TaskConfigService};
use eframe::egui::{self, Context, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct PlonApp {
    repository: Arc<Repository>,
    task_service: Arc<TaskService>,
    goal_service: Arc<GoalService>,
    resource_service: Arc<ResourceService>,
    task_config_service: Arc<TaskConfigService>,
    
    // UI State
    current_view: ViewType,
    selected_task_id: Option<Uuid>,
    selected_goal_id: Option<Uuid>,
    show_task_editor: bool,
    show_goal_editor: bool,
    
    // View components
    list_view: super::views::list_view::ListView,
    kanban_view: super::views::kanban_view::KanbanView,
    map_view: super::views::map_view::MapView,
    timeline_view: super::views::timeline_view::TimelineView,
    dashboard_view: super::views::dashboard_view::DashboardView,
    recurring_view: super::views::recurring_view::RecurringView,
    metadata_config_view: super::views::metadata_config_view::MetadataConfigView,
    
    // Cache
    tasks: Vec<Task>,
    goals: Vec<Goal>,
    resources: Vec<Resource>,
    
    // Runtime
    runtime: tokio::runtime::Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewType {
    List,
    Kanban,
    Map,
    Timeline,
    Dashboard,
    Recurring,
    MetadataConfig,
}

impl PlonApp {
    pub fn new(cc: &eframe::CreationContext<'_>, repository: Repository) -> Self {
        // Setup custom fonts and styles
        setup_custom_fonts(&cc.egui_ctx);
        
        let repository = Arc::new(repository);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        
        let task_service = Arc::new(TaskService::new(repository.clone()));
        let goal_service = Arc::new(GoalService::new(repository.clone()));
        let resource_service = Arc::new(ResourceService::new(repository.clone()));
        let task_config_service = Arc::new(TaskConfigService::new(repository.clone()));
        
        let mut app = Self {
            repository: repository.clone(),
            task_service: task_service.clone(),
            goal_service: goal_service.clone(),
            resource_service: resource_service.clone(),
            task_config_service: task_config_service.clone(),
            
            current_view: ViewType::Map,
            selected_task_id: None,
            selected_goal_id: None,
            show_task_editor: false,
            show_goal_editor: false,
            
            list_view: super::views::list_view::ListView::new(),
            kanban_view: super::views::kanban_view::KanbanView::new(),
            map_view: super::views::map_view::MapView::new(),
            timeline_view: super::views::timeline_view::TimelineView::new(),
            dashboard_view: super::views::dashboard_view::DashboardView::new(),
            recurring_view: super::views::recurring_view::RecurringView::new(),
            metadata_config_view: super::views::metadata_config_view::MetadataConfigView::new(),
            
            tasks: Vec::new(),
            goals: Vec::new(),
            resources: Vec::new(),
            
            runtime,
        };
        
        // Load initial data
        app.load_data();
        
        app
    }
    
    fn load_data(&mut self) {
        let task_service = self.task_service.clone();
        let goal_service = self.goal_service.clone();
        let resource_service = self.resource_service.clone();
        
        // Load tasks
        self.tasks = self.runtime.block_on(async {
            task_service.list_all().await.unwrap_or_default()
        });
        
        // Load goals
        self.goals = self.runtime.block_on(async {
            goal_service.list_all().await.unwrap_or_default()
        });
        
        // Load resources
        self.resources = self.runtime.block_on(async {
            resource_service.list_all().await.unwrap_or_default()
        });
    }
    
    fn show_top_panel(&mut self, ctx: &Context) {
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
                ui.selectable_value(&mut self.current_view, ViewType::MetadataConfig, "⚙️ Metadata");
                
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
                let search = ui.text_edit_singleline(&mut String::new());
                if search.changed() {
                    // TODO: Implement search
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙️ Settings").clicked() {
                        // TODO: Open settings
                    }
                    
                    if ui.button("🔄 Refresh").clicked() {
                        self.load_data();
                    }
                });
            });
        });
    }
    
    fn show_main_content(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                ViewType::List => {
                    self.list_view.show(ui, &mut self.tasks, &self.resources);
                }
                ViewType::Kanban => {
                    self.kanban_view.show(ui, &mut self.tasks);
                }
                ViewType::Map => {
                    self.map_view.show(ui, &mut self.tasks, &mut self.goals);
                }
                ViewType::Timeline => {
                    self.timeline_view.show(ui, &self.tasks, &self.goals);
                }
                ViewType::Dashboard => {
                    self.dashboard_view.show(ui, &self.tasks, &self.goals, &self.resources);
                }
                ViewType::Recurring => {
                    self.recurring_view.show(ui, None);
                }
                ViewType::MetadataConfig => {
                    self.metadata_config_view.show(ui, Some(self.task_config_service.clone()));
                }
            }
        });
    }
    
    fn show_modals(&mut self, ctx: &Context) {
        // Task editor modal
        if self.show_task_editor {
            egui::Window::new("Task Editor")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    if let Some(task_id) = self.selected_task_id {
                        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                            super::widgets::task_editor::show_task_editor(ui, task, &self.resources);
                        }
                    } else {
                        // New task
                        let mut new_task = Task::new("New Task".to_string(), String::new());
                        super::widgets::task_editor::show_task_editor(ui, &mut new_task, &self.resources);
                        
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Create").clicked() {
                                let task_service = self.task_service.clone();
                                let task = new_task.clone();
                                self.runtime.spawn(async move {
                                    task_service.create(task).await.ok();
                                });
                                self.tasks.push(new_task);
                                self.show_task_editor = false;
                            }
                            
                            if ui.button("Cancel").clicked() {
                                self.show_task_editor = false;
                            }
                        });
                    }
                });
        }
        
        // Goal editor modal
        if self.show_goal_editor {
            egui::Window::new("Goal Editor")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    // TODO: Implement goal editor
                    ui.label("Goal editor coming soon...");
                    
                    if ui.button("Close").clicked() {
                        self.show_goal_editor = false;
                    }
                });
        }
    }
}

impl eframe::App for PlonApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.show_top_panel(ctx);
        self.show_main_content(ctx);
        self.show_modals(ctx);
        
        // Request repaint for animations
        ctx.request_repaint();
    }
}

fn setup_custom_fonts(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    
    // Increase default text size
    style.text_styles = [
        (egui::TextStyle::Small, egui::FontId::new(12.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Button, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Heading, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(13.0, egui::FontFamily::Monospace)),
    ].into();
    
    ctx.set_style(style);
}