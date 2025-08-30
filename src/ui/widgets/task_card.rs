use crate::domain::task::Task;
use eframe::egui::{self, Ui};

pub fn show_task_card(ui: &mut Ui, task: &Task) {
    ui.group(|ui| {
        ui.label(&task.title);

        ui.horizontal(|ui| {
            ui.label(format!("Status: {:?}", task.status));
            ui.label(format!("Priority: {:?}", task.priority));
        });

        if !task.subtasks.is_empty() {
            let (completed, total) = task.subtask_progress();
            ui.label(format!("Progress: {}/{}", completed, total));
        }

        if let Some(due) = task.due_date {
            if task.is_overdue() {
                ui.colored_label(
                    egui::Color32::RED,
                    format!("Overdue: {}", due.format("%Y-%m-%d")),
                );
            } else {
                ui.label(format!("Due: {}", due.format("%Y-%m-%d")));
            }
        }
    });
}
