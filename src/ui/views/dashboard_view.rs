use crate::domain::{goal::Goal, resource::Resource, task::Task};
use eframe::egui::{self, Ui};

pub struct DashboardView {}

impl Default for DashboardView {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui, tasks: &[Task], goals: &[Goal], resources: &[Resource]) {
        ui.heading("Project Dashboard");

        // Statistics
        ui.horizontal(|ui| {
            let total_tasks = tasks.len();
            let completed_tasks = tasks
                .iter()
                .filter(|t| t.status == crate::domain::task::TaskStatus::Done)
                .count();
            let in_progress = tasks
                .iter()
                .filter(|t| t.status == crate::domain::task::TaskStatus::InProgress)
                .count();
            let blocked = tasks
                .iter()
                .filter(|t| t.status == crate::domain::task::TaskStatus::Blocked)
                .count();

            ui.group(|ui| {
                ui.label("Task Statistics");
                ui.label(format!("Total: {}", total_tasks));
                ui.label(format!("Completed: {}", completed_tasks));
                ui.label(format!("In Progress: {}", in_progress));
                ui.label(format!("Blocked: {}", blocked));
                ui.label(format!(
                    "Completion: {:.1}%",
                    if total_tasks > 0 {
                        (completed_tasks as f32 / total_tasks as f32) * 100.0
                    } else {
                        0.0
                    }
                ));
            });

            ui.separator();

            ui.group(|ui| {
                ui.label("Goal Statistics");
                ui.label(format!("Total Goals: {}", goals.len()));
                let completed_goals = goals
                    .iter()
                    .filter(|g| g.status == crate::domain::goal::GoalStatus::Completed)
                    .count();
                ui.label(format!("Completed: {}", completed_goals));
                let at_risk = goals.iter().filter(|g| g.is_at_risk()).count();
                ui.label(format!("At Risk: {}", at_risk));
            });

            ui.separator();

            ui.group(|ui| {
                ui.label("Resource Utilization");
                for resource in resources {
                    ui.horizontal(|ui| {
                        ui.label(&resource.name);
                        ui.label(format!("{:.0}%", resource.utilization_percentage()));
                        if resource.is_overloaded() {
                            ui.colored_label(egui::Color32::RED, "⚠ Overloaded");
                        }
                    });
                }
            });
        });

        ui.separator();

        // Overdue tasks
        ui.label("Overdue Tasks:");
        for task in tasks {
            if task.is_overdue() {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::RED, "⚠");
                    ui.label(&task.title);
                    if let Some(due) = task.due_date {
                        ui.label(format!("Due: {}", due.format("%Y-%m-%d")));
                    }
                });
            }
        }

        ui.separator();

        // Goals at risk
        ui.label("Goals at Risk:");
        for goal in goals {
            if goal.is_at_risk() {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::YELLOW, "⚠");
                    ui.label(&goal.title);
                    if let Some(target) = goal.target_date {
                        ui.label(format!("Target: {}", target.format("%Y-%m-%d")));
                    }
                });
            }
        }
    }
}
