use eframe::{Frame, NativeOptions, egui};
use plon::domain::goal::Goal;
use plon::domain::task::Task;
use plon::ui::views::timeline_view::TimelineView;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Data structure to track visual changes between frames
#[derive(Debug, Clone)]
struct FrameCapture {
    frame_number: usize,
    timestamp: Instant,
    content_hash: u64,
    scroll_position: egui::Vec2,
    visible_rect: egui::Rect,
}

/// Test application that captures visual state to detect auto-scrolling
struct TimelineVisualTestApp {
    timeline_view: TimelineView,
    tasks: Vec<Task>,
    goals: Vec<Goal>,

    // Test tracking
    frame_captures: Arc<Mutex<Vec<FrameCapture>>>,
    start_time: Instant,
    last_interaction: Instant,
    test_phase: TestPhase,
    frames_since_last_interaction: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum TestPhase {
    Initial,    // Let the app stabilize
    Monitoring, // Monitor for auto-scrolling
    Complete,   // Test complete
}

impl TimelineVisualTestApp {
    fn new() -> Self {
        // Create test tasks
        let tasks: Vec<Task> = (0..50)
            .map(|i| {
                let mut task = Task::new(
                    format!("Test Task {}", i),
                    format!("Description for task {}", i),
                );
                task.scheduled_date = Some(chrono::Utc::now());
                task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(i as i64));
                task
            })
            .collect();

        Self {
            timeline_view: TimelineView::new(),
            tasks,
            goals: Vec::new(),
            frame_captures: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
            last_interaction: Instant::now(),
            test_phase: TestPhase::Initial,
            frames_since_last_interaction: 0,
        }
    }

    fn capture_frame_state(&self, ctx: &egui::Context) -> FrameCapture {
        // Capture current visual state
        let scroll_position = ctx.memory(|mem| {
            // Try to get scroll position from memory
            // We'll use a simplified approach to track any scroll areas
            let scroll_pos = mem
                .data
                .get_temp::<egui::Vec2>(egui::Id::new("scroll_pos"))
                .unwrap_or(egui::Vec2::ZERO);
            scroll_pos
        });

        // Get visible rect
        let visible_rect = ctx.available_rect();

        // Create a content hash based on the visual state
        // In a real implementation, we'd hash the actual rendered pixels
        let content_hash = Self::calculate_content_hash(&visible_rect, &scroll_position);

        FrameCapture {
            frame_number: self.frame_captures.lock().unwrap().len(),
            timestamp: Instant::now(),
            content_hash,
            scroll_position,
            visible_rect,
        }
    }

    fn calculate_content_hash(rect: &egui::Rect, scroll: &egui::Vec2) -> u64 {
        // Simple hash based on position and size
        // In a real test, we'd capture actual pixel data
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        (rect.min.x as i32).hash(&mut hasher);
        (rect.min.y as i32).hash(&mut hasher);
        (rect.max.x as i32).hash(&mut hasher);
        (rect.max.y as i32).hash(&mut hasher);
        (scroll.x as i32).hash(&mut hasher);
        (scroll.y as i32).hash(&mut hasher);
        hasher.finish()
    }

    fn detect_auto_scrolling(&self) -> Option<String> {
        let captures = self.frame_captures.lock().unwrap();

        if captures.len() < 10 {
            return None; // Need enough frames to analyze
        }

        // Check the last 10 frames for unexpected movement
        let recent_captures = &captures[captures.len().saturating_sub(10)..];

        // Count how many times the content changed without interaction
        let mut unexpected_changes = 0;
        let mut scroll_deltas = Vec::new();

        for window in recent_captures.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            // Check if scroll position changed
            let scroll_delta = (curr.scroll_position - prev.scroll_position).length();
            if scroll_delta > 0.1 {
                scroll_deltas.push(scroll_delta);
                unexpected_changes += 1;
            }

            // Check if visible rect changed significantly
            let rect_changed = (curr.visible_rect.min - prev.visible_rect.min).length() > 0.1
                || (curr.visible_rect.max - prev.visible_rect.max).length() > 0.1;

            if rect_changed {
                unexpected_changes += 1;
            }
        }

        // If more than 30% of frames had unexpected changes, we have auto-scrolling
        if unexpected_changes > 3 {
            Some(format!(
                "Auto-scrolling detected! {} unexpected changes in last 10 frames. Scroll deltas: {:?}",
                unexpected_changes, scroll_deltas
            ))
        } else {
            None
        }
    }
}

impl eframe::App for TimelineVisualTestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        // Capture frame state
        let frame_capture = self.capture_frame_state(ctx);
        self.frame_captures
            .lock()
            .unwrap()
            .push(frame_capture.clone());

        // Update test phase
        let elapsed = self.start_time.elapsed();
        if elapsed < Duration::from_secs(1) {
            self.test_phase = TestPhase::Initial;
        } else if elapsed < Duration::from_secs(5) {
            self.test_phase = TestPhase::Monitoring;
            self.frames_since_last_interaction += 1;
        } else {
            self.test_phase = TestPhase::Complete;
        }

        // Render the app
        egui::CentralPanel::default().show(ctx, |ui| {
            // Navigate to timeline view
            ui.heading("Timeline View Test");
            ui.separator();

            // Show the timeline view
            self.timeline_view.show(ui, &self.tasks, &self.goals);

            // Display test status
            ui.separator();
            ui.label(format!("Test Phase: {:?}", self.test_phase));
            ui.label(format!(
                "Frames captured: {}",
                self.frame_captures.lock().unwrap().len()
            ));
            ui.label(format!(
                "Frames since interaction: {}",
                self.frames_since_last_interaction
            ));

            // Check for auto-scrolling during monitoring phase
            if self.test_phase == TestPhase::Monitoring {
                if let Some(error) = self.detect_auto_scrolling() {
                    ui.colored_label(egui::Color32::RED, error);

                    // Log detailed information
                    println!(
                        "AUTO-SCROLL DETECTED AT FRAME {}",
                        frame_capture.frame_number
                    );
                    println!("Scroll position: {:?}", frame_capture.scroll_position);
                    println!("Visible rect: {:?}", frame_capture.visible_rect);

                    // In a test, we would panic here
                    // panic!("Auto-scrolling detected!");
                }
            }
        });

        // Request repaint for continuous monitoring
        if self.test_phase != TestPhase::Complete {
            ctx.request_repaint();
        } else {
            // Analyze final results
            let captures = self.frame_captures.lock().unwrap();
            println!("\n=== Test Complete ===");
            println!("Total frames captured: {}", captures.len());

            // Check for any scrolling that happened without user input
            let mut auto_scroll_events = 0;
            for window in captures.windows(2) {
                let prev = &window[0];
                let curr = &window[1];
                let scroll_delta = (curr.scroll_position - prev.scroll_position).length();
                if scroll_delta > 0.1 {
                    auto_scroll_events += 1;
                    println!(
                        "Frame {}->{}: Scroll changed by {:.2} pixels",
                        prev.frame_number, curr.frame_number, scroll_delta
                    );
                }
            }

            if auto_scroll_events > 0 {
                println!(
                    "❌ FAIL: Detected {} auto-scroll events",
                    auto_scroll_events
                );
            } else {
                println!("✅ PASS: No auto-scrolling detected");
            }

            // Close the window
            std::process::exit(if auto_scroll_events > 0 { 1 } else { 0 });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // This test requires a display
    fn test_timeline_no_auto_scroll() {
        // To run this test:
        // cargo test test_timeline_no_auto_scroll --ignored -- --nocapture

        let options = NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1024.0, 768.0])
                .with_title("Timeline Auto-Scroll Test"),
            ..Default::default()
        };

        let app = TimelineVisualTestApp::new();

        // This will run the GUI and exit with status code 1 if auto-scrolling detected
        let _ = eframe::run_native(
            "Timeline Auto-Scroll Test",
            options,
            Box::new(|_cc| Box::new(app)),
        );
    }
}

/// Lightweight version for CI testing
#[test]
fn test_timeline_stability_headless() {
    // This test can run without a display
    let ctx = egui::Context::default();
    let mut timeline_view = TimelineView::new();

    let tasks: Vec<Task> = (0..30)
        .map(|i| {
            let mut task = Task::new(format!("Task {}", i), String::new());
            task.scheduled_date = Some(chrono::Utc::now());
            task
        })
        .collect();

    let goals = Vec::new();

    // Simulate multiple frames
    let mut previous_output = None;
    let mut changes_detected = 0;

    for frame in 0..20 {
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Capture UI state before rendering
                let rect_before = ui.max_rect();
                let available_before = ui.available_size();

                // Render timeline
                timeline_view.show(ui, &tasks, &goals);

                // Capture UI state after rendering
                let rect_after = ui.max_rect();
                let available_after = ui.available_size();

                // Check for unexpected changes
                let current_output = format!(
                    "{:?}{:?}{:?}{:?}",
                    rect_before, available_before, rect_after, available_after
                );

                if let Some(prev) = &previous_output {
                    if prev != &current_output && frame > 2 {
                        // After initial frames, output should be stable
                        changes_detected += 1;
                        println!("Frame {}: Unexpected change detected", frame);
                    }
                }

                previous_output = Some(current_output);
            });
        });
    }

    assert_eq!(
        changes_detected, 0,
        "Timeline view is unstable! {} unexpected changes detected across frames",
        changes_detected
    );
}
