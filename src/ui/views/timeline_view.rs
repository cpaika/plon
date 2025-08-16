use crate::domain::{task::Task, goal::Goal};
use eframe::egui::{self, Ui};
use chrono::{Utc, Duration};

pub struct TimelineView {
    days_to_show: i64,
}

impl TimelineView {
    pub fn new() -> Self {
        Self {
            days_to_show: 30,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, tasks: &[Task], goals: &[Goal]) {
        ui.heading("Timeline View");
        
        ui.horizontal(|ui| {
            ui.label("Days to show:");
            ui.add(egui::Slider::new(&mut self.days_to_show, 7..=90));
        });
        
        ui.separator();
        
        let today = Utc::now();
        let end_date = today + Duration::days(self.days_to_show);
        
        // Show scheduled tasks
        ui.label("Scheduled Tasks:");
        for task in tasks {
            if let Some(scheduled) = task.scheduled_date {
                if scheduled >= today && scheduled <= end_date {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: ", scheduled.format("%Y-%m-%d")));
                        ui.label(&task.title);
                    });
                }
            }
        }
        
        ui.separator();
        
        // Show goals with target dates
        ui.label("Goal Deadlines:");
        for goal in goals {
            if let Some(target) = goal.target_date {
                if target >= today && target <= end_date {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: ", target.format("%Y-%m-%d")));
                        ui.label(&goal.title);
                    });
                }
            }
        }
    }
}