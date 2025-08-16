mod domain;
mod repository;
mod services;
mod ui;
mod utils;

use anyhow::Result;
use eframe::egui;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Run the native app without database for now
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Plon - Project Management",
        options,
        Box::new(|cc| Box::new(SimplePlonApp::new(cc))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))?;

    Ok(())
}

struct SimplePlonApp {
    current_view: ui::app::ViewType,
    tasks: Vec<domain::task::Task>,
    show_task_editor: bool,
    new_task_title: String,
    new_task_description: String,
}

impl SimplePlonApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_view: ui::app::ViewType::Map,
            tasks: Vec::new(),
            show_task_editor: false,
            new_task_title: String::new(),
            new_task_description: String::new(),
        }
    }
}

impl eframe::App for SimplePlonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎯 Plon - Project Management");
                ui.separator();
                
                // View selector
                ui.selectable_value(&mut self.current_view, ui::app::ViewType::List, "📋 List");
                ui.selectable_value(&mut self.current_view, ui::app::ViewType::Kanban, "📊 Kanban");
                ui.selectable_value(&mut self.current_view, ui::app::ViewType::Map, "🗺️ Map");
                ui.selectable_value(&mut self.current_view, ui::app::ViewType::Timeline, "📅 Timeline");
                ui.selectable_value(&mut self.current_view, ui::app::ViewType::Dashboard, "📈 Dashboard");
                
                ui.separator();
                
                if ui.button("➕ New Task").clicked() {
                    self.show_task_editor = true;
                }
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                ui::app::ViewType::Map => {
                    ui.heading("Map View");
                    ui.label("Double-click to create a task");
                    ui.separator();
                    
                    // Simple task display
                    for (i, task) in self.tasks.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Task {}: ", i + 1));
                            ui.text_edit_singleline(&mut task.title);
                            if ui.button("Delete").clicked() {
                                // Mark for deletion
                                task.title = String::new();
                            }
                        });
                    }
                    
                    // Remove empty tasks
                    self.tasks.retain(|t| !t.title.is_empty());
                }
                ui::app::ViewType::List => {
                    ui.heading("List View");
                    for task in &self.tasks {
                        ui.label(&task.title);
                    }
                }
                ui::app::ViewType::Kanban => {
                    ui.heading("Kanban View");
                    ui.horizontal(|ui| {
                        ui.group(|ui| {
                            ui.label("To Do");
                            for task in &self.tasks {
                                if task.status == domain::task::TaskStatus::Todo {
                                    ui.label(&task.title);
                                }
                            }
                        });
                        ui.group(|ui| {
                            ui.label("In Progress");
                            for task in &self.tasks {
                                if task.status == domain::task::TaskStatus::InProgress {
                                    ui.label(&task.title);
                                }
                            }
                        });
                        ui.group(|ui| {
                            ui.label("Done");
                            for task in &self.tasks {
                                if task.status == domain::task::TaskStatus::Done {
                                    ui.label(&task.title);
                                }
                            }
                        });
                    });
                }
                ui::app::ViewType::Timeline => {
                    ui.heading("Timeline View");
                    ui.label("Timeline visualization coming soon");
                }
                ui::app::ViewType::Dashboard => {
                    ui.heading("Dashboard");
                    ui.label(format!("Total tasks: {}", self.tasks.len()));
                    let completed = self.tasks.iter()
                        .filter(|t| t.status == domain::task::TaskStatus::Done)
                        .count();
                    ui.label(format!("Completed: {}", completed));
                    ui.label(format!("In Progress: {}", 
                        self.tasks.iter()
                            .filter(|t| t.status == domain::task::TaskStatus::InProgress)
                            .count()
                    ));
                }
                _ => {
                    ui.label("View not implemented");
                }
            }
        });

        // Task editor modal
        if self.show_task_editor {
            egui::Window::new("New Task")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut self.new_task_title);
                    
                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_task_description);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() && !self.new_task_title.is_empty() {
                            let task = domain::task::Task::new(
                                self.new_task_title.clone(),
                                self.new_task_description.clone()
                            );
                            self.tasks.push(task);
                            self.new_task_title.clear();
                            self.new_task_description.clear();
                            self.show_task_editor = false;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            self.show_task_editor = false;
                        }
                    });
                });
        }
    }
}