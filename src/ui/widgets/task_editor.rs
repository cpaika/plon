use crate::domain::{task::Task, resource::Resource};
use eframe::egui::{self, Ui};

pub fn show_task_editor(ui: &mut Ui, task: &mut Task, resources: &[Resource]) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut task.title);
        });
        
        ui.horizontal(|ui| {
            ui.label("Description:");
        });
        ui.text_edit_multiline(&mut task.description);
        
        ui.horizontal(|ui| {
            ui.label("Status:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", task.status))
                .show_ui(ui, |ui| {
                    use crate::domain::task::TaskStatus;
                    ui.selectable_value(&mut task.status, TaskStatus::Todo, "Todo");
                    ui.selectable_value(&mut task.status, TaskStatus::InProgress, "In Progress");
                    ui.selectable_value(&mut task.status, TaskStatus::Blocked, "Blocked");
                    ui.selectable_value(&mut task.status, TaskStatus::Review, "Review");
                    ui.selectable_value(&mut task.status, TaskStatus::Done, "Done");
                });
        });
        
        ui.horizontal(|ui| {
            ui.label("Priority:");
            egui::ComboBox::from_label("priority")
                .selected_text(format!("{:?}", task.priority))
                .show_ui(ui, |ui| {
                    use crate::domain::task::Priority;
                    ui.selectable_value(&mut task.priority, Priority::Low, "Low");
                    ui.selectable_value(&mut task.priority, Priority::Medium, "Medium");
                    ui.selectable_value(&mut task.priority, Priority::High, "High");
                    ui.selectable_value(&mut task.priority, Priority::Critical, "Critical");
                });
        });
        
        ui.horizontal(|ui| {
            ui.label("Estimated Hours:");
            if let Some(mut hours) = task.estimated_hours {
                ui.add(egui::DragValue::new(&mut hours).speed(0.1));
                task.estimated_hours = Some(hours);
            } else {
                if ui.button("Add estimate").clicked() {
                    task.estimated_hours = Some(1.0);
                }
            }
        });
        
        ui.separator();
        
        ui.label("Subtasks:");
        let mut subtasks_to_remove = Vec::new();
        for (i, subtask) in task.subtasks.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut subtask.completed, "");
                ui.label(&subtask.description);
                if ui.small_button("❌").clicked() {
                    subtasks_to_remove.push(i);
                }
            });
        }
        
        // Remove subtasks
        for i in subtasks_to_remove.iter().rev() {
            task.subtasks.remove(*i);
        }
        
        // Add new subtask
        ui.horizontal(|ui| {
            if ui.button("+ Add Subtask").clicked() {
                task.add_subtask("New subtask".to_string());
            }
        });
    });
}