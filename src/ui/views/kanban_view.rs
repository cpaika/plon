use crate::domain::task::Task;
use eframe::egui::{self, Ui};

pub struct KanbanView {
    columns: Vec<KanbanColumn>,
}

struct KanbanColumn {
    title: String,
    status: crate::domain::task::TaskStatus,
}

impl KanbanView {
    pub fn new() -> Self {
        Self {
            columns: vec![
                KanbanColumn {
                    title: "To Do".to_string(),
                    status: crate::domain::task::TaskStatus::Todo,
                },
                KanbanColumn {
                    title: "In Progress".to_string(),
                    status: crate::domain::task::TaskStatus::InProgress,
                },
                KanbanColumn {
                    title: "Review".to_string(),
                    status: crate::domain::task::TaskStatus::Review,
                },
                KanbanColumn {
                    title: "Done".to_string(),
                    status: crate::domain::task::TaskStatus::Done,
                },
            ],
        }
    }

    pub fn show(&mut self, ui: &mut Ui, tasks: &mut Vec<Task>) {
        ui.heading("Kanban Board");
        
        ui.horizontal(|ui| {
            for column in &self.columns {
                ui.vertical(|ui| {
                    ui.set_min_width(250.0);
                    ui.heading(&column.title);
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .max_height(600.0)
                        .show(ui, |ui| {
                            for task in tasks.iter_mut() {
                                if task.status == column.status {
                                    self.show_task_card(ui, task);
                                }
                            }
                        });
                });
                ui.separator();
            }
        });
    }

    fn show_task_card(&self, ui: &mut Ui, task: &mut Task) {
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