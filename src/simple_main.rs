mod domain;
mod repository;  
mod services;
mod ui;
mod utils;

use domain::task::{Task, TaskStatus, Priority};
use domain::goal::Goal;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Plon - Project Management",
        options,
        Box::new(|cc| Box::new(PlonApp::new(cc))),
    )
}

struct PlonApp {
    tasks: Vec<Task>,
    goals: Vec<Goal>,
    show_task_editor: bool,
    current_view: ViewType,
    new_task_title: String,
    new_task_description: String,
    
    // Map view state
    camera_pos: egui::Vec2,
    zoom: f32,
}

#[derive(PartialEq)]
enum ViewType {
    Map,
    List,
    Kanban,
    Dashboard,
}

impl PlonApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create some sample tasks
        let mut tasks = Vec::new();
        
        let mut task1 = Task::new("Design UI".to_string(), "Create mockups for the user interface".to_string());
        task1.set_position(100.0, 100.0);
        task1.add_subtask("Create wireframes".to_string());
        task1.add_subtask("Design color scheme".to_string());
        tasks.push(task1);
        
        let mut task2 = Task::new("Implement backend".to_string(), "Build REST API".to_string());
        task2.set_position(300.0, 100.0);
        task2.update_status(TaskStatus::InProgress);
        tasks.push(task2);
        
        let mut task3 = Task::new("Write tests".to_string(), "Add unit and integration tests".to_string());
        task3.set_position(200.0, 250.0);
        tasks.push(task3);
        
        Self {
            tasks,
            goals: Vec::new(),
            show_task_editor: false,
            current_view: ViewType::Map,
            new_task_title: String::new(),
            new_task_description: String::new(),
            camera_pos: egui::Vec2::ZERO,
            zoom: 1.0,
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
                
                ui.selectable_value(&mut self.current_view, ViewType::Map, "🗺️ Map");
                ui.selectable_value(&mut self.current_view, ViewType::List, "📋 List");
                ui.selectable_value(&mut self.current_view, ViewType::Kanban, "📊 Kanban");
                ui.selectable_value(&mut self.current_view, ViewType::Dashboard, "📈 Dashboard");
                
                ui.separator();
                
                if ui.button("➕ New Task").clicked() {
                    self.show_task_editor = true;
                }
            });
        });
        
        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                ViewType::Map => self.show_map_view(ui),
                ViewType::List => self.show_list_view(ui),
                ViewType::Kanban => self.show_kanban_view(ui),
                ViewType::Dashboard => self.show_dashboard_view(ui),
            }
        });
        
        // Task editor window
        if self.show_task_editor {
            egui::Window::new("New Task")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut self.new_task_title);
                    
                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_task_description);
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() && !self.new_task_title.is_empty() {
                            let mut task = Task::new(
                                self.new_task_title.clone(),
                                self.new_task_description.clone()
                            );
                            task.set_position(
                                (self.tasks.len() as f64 * 100.0) % 500.0,
                                (self.tasks.len() as f64 * 50.0) % 300.0
                            );
                            self.tasks.push(task);
                            self.new_task_title.clear();
                            self.new_task_description.clear();
                            self.show_task_editor = false;
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

impl PlonApp {
    fn show_map_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("Zoom: {:.0}%", self.zoom * 100.0));
            if ui.button("🔍+").clicked() {
                self.zoom = (self.zoom * 1.2).min(3.0);
            }
            if ui.button("🔍-").clicked() {
                self.zoom = (self.zoom / 1.2).max(0.3);
            }
            if ui.button("🏠").clicked() {
                self.zoom = 1.0;
                self.camera_pos = egui::Vec2::ZERO;
            }
        });
        
        ui.separator();
        
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());
        
        // Handle panning
        if response.dragged() {
            self.camera_pos += response.drag_delta();
        }
        
        // Handle zoom with scroll
        if response.hovered() {
            let scroll = ui.input(|i| i.scroll_delta.y);
            if scroll != 0.0 {
                self.zoom = (self.zoom * (1.0 + scroll * 0.001)).clamp(0.3, 3.0);
            }
        }
        
        // Draw tasks
        let painter = ui.painter_at(available_rect);
        
        for task in &mut self.tasks {
            let pos = egui::Pos2::new(
                available_rect.center().x + (task.position.x as f32 + self.camera_pos.x) * self.zoom,
                available_rect.center().y + (task.position.y as f32 + self.camera_pos.y) * self.zoom
            );
            
            let size = egui::Vec2::new(150.0, 80.0) * self.zoom;
            let rect = egui::Rect::from_center_size(pos, size);
            
            let color = match task.status {
                TaskStatus::Todo => egui::Color32::from_rgb(200, 200, 200),
                TaskStatus::InProgress => egui::Color32::from_rgb(100, 150, 255),
                TaskStatus::Done => egui::Color32::from_rgb(100, 255, 100),
                TaskStatus::Blocked => egui::Color32::from_rgb(255, 100, 100),
                _ => egui::Color32::from_rgb(180, 180, 180),
            };
            
            painter.rect(
                rect,
                5.0,
                color,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100))
            );
            
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &task.title,
                egui::FontId::proportional(14.0 * self.zoom),
                egui::Color32::BLACK
            );
            
            if !task.subtasks.is_empty() {
                let (completed, total) = task.subtask_progress();
                painter.text(
                    rect.center() + egui::Vec2::new(0.0, 20.0 * self.zoom),
                    egui::Align2::CENTER_CENTER,
                    format!("{}/{}", completed, total),
                    egui::FontId::proportional(10.0 * self.zoom),
                    egui::Color32::from_rgb(60, 60, 60)
                );
            }
        }
    }
    
    fn show_list_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Task List");
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for task in &mut self.tasks {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let status_color = match task.status {
                            TaskStatus::Todo => egui::Color32::GRAY,
                            TaskStatus::InProgress => egui::Color32::from_rgb(100, 150, 255),
                            TaskStatus::Done => egui::Color32::from_rgb(100, 255, 100),
                            _ => egui::Color32::DARK_GRAY,
                        };
                        ui.colored_label(status_color, "●");
                        
                        ui.text_edit_singleline(&mut task.title);
                        
                        ui.label(format!("[{:?}]", task.priority));
                        
                        if !task.subtasks.is_empty() {
                            let (completed, total) = task.subtask_progress();
                            ui.label(format!("📝 {}/{}", completed, total));
                        }
                    });
                    
                    if !task.description.is_empty() {
                        ui.label(&task.description);
                    }
                });
            }
        });
    }
    
    fn show_kanban_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Kanban Board");
        
        ui.horizontal(|ui| {
            // Todo column
            ui.group(|ui| {
                ui.set_min_width(250.0);
                ui.label("📋 To Do");
                ui.separator();
                for task in &self.tasks {
                    if task.status == TaskStatus::Todo {
                        ui.group(|ui| {
                            ui.label(&task.title);
                            if !task.subtasks.is_empty() {
                                let (c, t) = task.subtask_progress();
                                ui.label(format!("Progress: {}/{}", c, t));
                            }
                        });
                    }
                }
            });
            
            // In Progress column
            ui.group(|ui| {
                ui.set_min_width(250.0);
                ui.label("🚀 In Progress");
                ui.separator();
                for task in &self.tasks {
                    if task.status == TaskStatus::InProgress {
                        ui.group(|ui| {
                            ui.label(&task.title);
                            if !task.subtasks.is_empty() {
                                let (c, t) = task.subtask_progress();
                                ui.label(format!("Progress: {}/{}", c, t));
                            }
                        });
                    }
                }
            });
            
            // Done column
            ui.group(|ui| {
                ui.set_min_width(250.0);
                ui.label("✅ Done");
                ui.separator();
                for task in &self.tasks {
                    if task.status == TaskStatus::Done {
                        ui.group(|ui| {
                            ui.label(&task.title);
                        });
                    }
                }
            });
        });
    }
    
    fn show_dashboard_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Dashboard");
        
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label("📊 Task Statistics");
                ui.separator();
                
                let total = self.tasks.len();
                let completed = self.tasks.iter()
                    .filter(|t| t.status == TaskStatus::Done)
                    .count();
                let in_progress = self.tasks.iter()
                    .filter(|t| t.status == TaskStatus::InProgress)
                    .count();
                let todo = self.tasks.iter()
                    .filter(|t| t.status == TaskStatus::Todo)
                    .count();
                
                ui.label(format!("Total Tasks: {}", total));
                ui.label(format!("✅ Completed: {}", completed));
                ui.label(format!("🚀 In Progress: {}", in_progress));
                ui.label(format!("📋 To Do: {}", todo));
                
                if total > 0 {
                    let percentage = (completed as f32 / total as f32) * 100.0;
                    ui.label(format!("Completion: {:.1}%", percentage));
                    
                    ui.add(egui::ProgressBar::new(percentage / 100.0)
                        .show_percentage());
                }
            });
            
            ui.group(|ui| {
                ui.label("🎯 Recent Activity");
                ui.separator();
                
                for (i, task) in self.tasks.iter().take(5).enumerate() {
                    ui.label(format!("{}. {}", i + 1, task.title));
                }
            });
        });
    }
}