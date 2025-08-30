use eframe::{NativeOptions, egui};
use plon::domain::goal::Goal;
use plon::domain::task::Task;
use plon::ui::views::timeline_view::TimelineView;

fn main() {
    // Simple test app to observe timeline behavior
    let options = NativeOptions::default();

    let _ = eframe::run_native(
        "Timeline Test",
        options,
        Box::new(|_cc| Box::new(TimelineTestApp::new())),
    );
}

struct TimelineTestApp {
    timeline_view: TimelineView,
    tasks: Vec<Task>,
    goals: Vec<Goal>,
    frame_count: usize,
}

impl TimelineTestApp {
    fn new() -> Self {
        let tasks: Vec<Task> = (0..20)
            .map(|i| {
                let mut task = Task::new(format!("Task {}", i), String::new());
                task.scheduled_date = Some(chrono::Utc::now());
                task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(i));
                task
            })
            .collect();

        Self {
            timeline_view: TimelineView::new(),
            tasks,
            goals: Vec::new(),
            frame_count: 0,
        }
    }
}

impl eframe::App for TimelineTestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("Frame: {}", self.frame_count));

            // This is the critical part - does this cause auto-scrolling?
            self.timeline_view.show(ui, &self.tasks, &self.goals);
        });

        // Only request repaint if something changed
        // If we see frame count continuously increasing without interaction,
        // something is requesting repaints
        if self.frame_count < 100 {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}
