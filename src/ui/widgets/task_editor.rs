use crate::domain::{claude_code::ClaudeCodeSession, resource::Resource, task::Task};
use eframe::egui::{self, Color32, RichText, Ui};
use uuid::Uuid;

pub fn show_task_editor(
    ui: &mut Ui,
    task: &mut Task,
    _resources: &[Resource],
    claude_sessions: &[ClaudeCodeSession],
    on_launch_claude: Option<&mut dyn FnMut(Uuid)>,
) {
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
            } else if ui.button("Add estimate").clicked() {
                task.estimated_hours = Some(1.0);
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

        ui.separator();

        // Claude Code Integration Section
        ui.heading("Claude Code Integration");

        // Show existing sessions for this task
        let task_sessions: Vec<_> = claude_sessions
            .iter()
            .filter(|s| s.task_id == task.id)
            .collect();

        if !task_sessions.is_empty() {
            ui.label("Previous Sessions:");
            for session in &task_sessions {
                ui.horizontal(|ui| {
                    let status_color = match session.status {
                        crate::domain::claude_code::SessionStatus::Completed => Color32::GREEN,
                        crate::domain::claude_code::SessionStatus::Failed => Color32::RED,
                        crate::domain::claude_code::SessionStatus::Cancelled => Color32::GRAY,
                        crate::domain::claude_code::SessionStatus::Working
                        | crate::domain::claude_code::SessionStatus::Initializing
                        | crate::domain::claude_code::SessionStatus::CreatingPR => Color32::YELLOW,
                        _ => Color32::WHITE,
                    };

                    ui.colored_label(status_color, format!("[{}]", session.status.as_str()));

                    if let Some(pr_url) = &session.pr_url {
                        ui.hyperlink_to("View PR", pr_url);
                    }

                    if let Some(branch) = &session.branch_name {
                        ui.label(format!("Branch: {}", branch));
                    }

                    ui.label(format!(
                        "Started: {}",
                        session.started_at.format("%Y-%m-%d %H:%M")
                    ));
                });
            }
            ui.add_space(5.0);
        }

        // Launch Claude Code button
        ui.horizontal(|ui| {
            let has_active_session = task_sessions.iter().any(|s| s.status.is_active());

            if has_active_session {
                ui.add_enabled(false, egui::Button::new("Claude Code Running..."));
            } else if ui
                .button(RichText::new("🤖 Launch Claude Code").size(16.0))
                .clicked()
                && let Some(callback) = on_launch_claude
            {
                callback(task.id);
            }

            ui.label("Launch an AI assistant to work on this task");
        });

        if task.description.is_empty() {
            ui.colored_label(
                Color32::from_rgb(255, 200, 0),
                "⚠️ Add a detailed description for better Claude Code results",
            );
        }
    });
}
