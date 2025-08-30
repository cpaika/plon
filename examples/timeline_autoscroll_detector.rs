use eframe::{NativeOptions, egui};
use plon::domain::goal::Goal;
use plon::domain::task::Task;
use plon::ui::views::timeline_view::TimelineView;
use std::time::{Duration, Instant};

/// Standalone app to detect auto-scrolling in timeline view
struct AutoScrollDetector {
    timeline_view: TimelineView,
    tasks: Vec<Task>,
    goals: Vec<Goal>,

    // Detection state
    frame_count: usize,
    start_time: Instant,
    last_paint_id: u64,
    paint_changes: Vec<(usize, String)>,
    last_scroll_offset: Option<egui::Vec2>,
    scroll_changes: Vec<(usize, egui::Vec2, egui::Vec2)>,
    last_rect: Option<egui::Rect>,
    rect_changes: Vec<(usize, egui::Rect, egui::Rect)>,
}

impl AutoScrollDetector {
    fn new() -> Self {
        // Create test tasks
        let tasks: Vec<Task> = (0..30)
            .map(|i| {
                let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
                task.scheduled_date = Some(chrono::Utc::now() + chrono::Duration::days(i));
                task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(i + 5));
                task
            })
            .collect();

        Self {
            timeline_view: TimelineView::new(),
            tasks,
            goals: Vec::new(),
            frame_count: 0,
            start_time: Instant::now(),
            last_paint_id: 0,
            paint_changes: Vec::new(),
            last_scroll_offset: None,
            scroll_changes: Vec::new(),
            last_rect: None,
            rect_changes: Vec::new(),
        }
    }

    fn detect_changes(&mut self, ctx: &egui::Context) {
        // Check if we're getting continuous repaints
        let paint_id = ctx.frame_nr();
        if paint_id != self.last_paint_id {
            self.paint_changes.push((
                self.frame_count,
                format!("Paint #{} -> #{}", self.last_paint_id, paint_id),
            ));
            self.last_paint_id = paint_id;
        }

        // Try to detect scroll position changes
        ctx.memory(|mem| {
            // Check for any scroll area state
            if let Some(scroll) = mem
                .data
                .get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll_area"))
            {
                if let Some(last) = self.last_scroll_offset {
                    if (scroll - last).length() > 0.01 {
                        self.scroll_changes.push((self.frame_count, last, scroll));
                    }
                }
                self.last_scroll_offset = Some(scroll);
            }
        });
    }

    fn report_findings(&self) {
        let elapsed = self.start_time.elapsed();

        println!("\n=== AUTO-SCROLL DETECTION REPORT ===");
        println!("Test duration: {:.2}s", elapsed.as_secs_f32());
        println!("Total frames: {}", self.frame_count);
        println!(
            "Frames per second: {:.1}",
            self.frame_count as f32 / elapsed.as_secs_f32()
        );

        if !self.paint_changes.is_empty() {
            println!("\n❌ CONTINUOUS REPAINTS DETECTED:");
            for (frame, change) in self.paint_changes.iter().take(10) {
                println!("  Frame {}: {}", frame, change);
            }
            if self.paint_changes.len() > 10 {
                println!("  ... and {} more", self.paint_changes.len() - 10);
            }
        }

        if !self.scroll_changes.is_empty() {
            println!("\n❌ SCROLL POSITION CHANGES DETECTED:");
            for (frame, from, to) in self.scroll_changes.iter().take(10) {
                println!(
                    "  Frame {}: ({:.2}, {:.2}) -> ({:.2}, {:.2})",
                    frame, from.x, from.y, to.x, to.y
                );
            }
            if self.scroll_changes.len() > 10 {
                println!("  ... and {} more", self.scroll_changes.len() - 10);
            }
        }

        if !self.rect_changes.is_empty() {
            println!("\n❌ LAYOUT CHANGES DETECTED:");
            for (frame, from, to) in self.rect_changes.iter().take(10) {
                println!(
                    "  Frame {}: size changed from {:?} to {:?}",
                    frame,
                    from.size(),
                    to.size()
                );
            }
        }

        // Final verdict
        let has_issues = !self.paint_changes.is_empty()
            || !self.scroll_changes.is_empty()
            || !self.rect_changes.is_empty();

        if has_issues {
            println!("\n❌ FAIL: Auto-scrolling/continuous repainting detected!");
            println!("The timeline view is unstable and causing performance issues.");
        } else {
            println!("\n✅ PASS: No auto-scrolling detected!");
            println!("The timeline view is stable.");
        }
    }
}

impl eframe::App for AutoScrollDetector {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // Detect changes before rendering
        self.detect_changes(ctx);

        // Only run for 5 seconds
        if self.start_time.elapsed() > Duration::from_secs(5) {
            self.report_findings();
            std::process::exit(if self.scroll_changes.is_empty() { 0 } else { 1 });
        }

        // Render the timeline view
        egui::CentralPanel::default().show(ctx, |ui| {
            // Track rect changes
            let rect_before = ui.max_rect();

            ui.heading("Timeline Auto-Scroll Detector");
            ui.label(format!(
                "Frame: {} | Elapsed: {:.1}s",
                self.frame_count,
                self.start_time.elapsed().as_secs_f32()
            ));
            ui.separator();

            // Show the timeline view
            self.timeline_view.show(ui, &self.tasks, &self.goals);

            // Check for rect changes
            let rect_after = ui.max_rect();
            if let Some(last) = self.last_rect {
                if (rect_after.size() - last.size()).length() > 0.01 {
                    self.rect_changes.push((self.frame_count, last, rect_after));
                }
            }
            self.last_rect = Some(rect_after);
        });

        // Request continuous updates to detect auto-scrolling
        ctx.request_repaint();
    }
}

fn main() {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("Timeline Auto-Scroll Detector"),
        ..Default::default()
    };

    let app = AutoScrollDetector::new();

    let _ = eframe::run_native(
        "Timeline Auto-Scroll Detector",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}
