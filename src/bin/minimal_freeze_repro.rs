// Minimal reproduction of the freeze issue
use eframe::{NativeOptions, egui};
use std::time::Instant;

fn main() {
    println!("=== Minimal Freeze Reproduction ===");

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Minimal Freeze Test",
        options,
        Box::new(|_cc| {
            Box::new(MinimalApp {
                frame_count: 0,
                events_processed: 0,
                last_frame: Instant::now(),
            })
        }),
    );
}

struct MinimalApp {
    frame_count: u64,
    events_processed: u64,
    last_frame: Instant,
}

impl eframe::App for MinimalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        let frame_time = self.last_frame.elapsed();
        if frame_time.as_millis() > 100 {
            println!("Slow frame {} after {:?}", self.frame_count, frame_time);
        }
        self.last_frame = Instant::now();

        // Simple UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Frame: {}", self.frame_count));
            ui.label(format!("Events: {}", self.events_processed));

            // Process input events
            ui.input(|i| {
                self.events_processed += i.events.len() as u64;

                // Check scroll
                let scroll = i.smooth_scroll_delta;
                if scroll != egui::Vec2::ZERO {
                    if self.events_processed % 100 == 0 {
                        println!("Processed {} scroll events", self.events_processed);
                    }
                }
            });

            // Allocate a response area like map_view does
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

            // Draw something simple
            let painter = ui.painter_at(rect);
            painter.circle_filled(rect.center(), 50.0, egui::Color32::RED);

            // Handle dragging
            if response.dragged() {
                // Do nothing, just test the response
            }
        });

        // Always repaint to keep testing
        ctx.request_repaint();
    }
}
