use crate::domain::{resource::Resource, task::Task};
use eframe::egui::{self, Ui};

pub struct ListView {
    filter_text: String,
    selected_status: Option<String>,
}

impl Default for ListView {
    fn default() -> Self {
        Self::new()
    }
}

impl ListView {
    pub fn new() -> Self {
        Self {
            filter_text: String::new(),
            selected_status: None,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, tasks: &mut Vec<Task>, _resources: &[Resource]) {
        ui.heading("Task List");

        // Filters
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);

            ui.separator();

            if ui.button("All").clicked() {
                self.selected_status = None;
            }
            if ui.button("Todo").clicked() {
                self.selected_status = Some("Todo".to_string());
            }
            if ui.button("In Progress").clicked() {
                self.selected_status = Some("InProgress".to_string());
            }
            if ui.button("Done").clicked() {
                self.selected_status = Some("Done".to_string());
            }
        });

        ui.separator();

        // Task list
        egui::ScrollArea::vertical().show(ui, |ui| {
            for task in tasks.iter_mut() {
                // Apply filters
                if !self.filter_text.is_empty()
                    && !task
                        .title
                        .to_lowercase()
                        .contains(&self.filter_text.to_lowercase())
                {
                    continue;
                }

                ui.horizontal(|ui| {
                    // Status indicator
                    let status_color = match task.status {
                        crate::domain::task::TaskStatus::Todo => egui::Color32::GRAY,
                        crate::domain::task::TaskStatus::InProgress => {
                            egui::Color32::from_rgb(100, 150, 255)
                        }
                        crate::domain::task::TaskStatus::Done => {
                            egui::Color32::from_rgb(100, 255, 100)
                        }
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
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("⚠ {}", due.format("%Y-%m-%d")),
                            );
                        } else {
                            ui.label(format!("📅 {}", due.format("%Y-%m-%d")));
                        }
                    }
                });

                ui.separator();
            }
        });
    }
}
