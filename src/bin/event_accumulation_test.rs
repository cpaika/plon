use eframe::{NativeOptions, egui};
use std::time::Instant;

fn main() {
    println!("=== Event Accumulation Test ===");

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Event Test",
        options,
        Box::new(|_cc| {
            Box::new(EventTestApp {
                frame: 0,
                last_check: Instant::now(),
            })
        }),
    );
}

struct EventTestApp {
    frame: u64,
    last_check: Instant,
}

impl eframe::App for EventTestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame += 1;

        // Check how many events are in the buffer
        let event_count = ctx.input(|i| i.events.len());

        // Log every second
        if self.last_check.elapsed().as_secs() >= 1 {
            println!("Frame {}: {} events in buffer", self.frame, event_count);
            self.last_check = Instant::now();
        }

        // Add events continuously
        ctx.input_mut(|i| {
            // Add 2 events per frame
            i.events.push(egui::Event::Scroll(egui::vec2(1.0, 1.0)));
            i.events
                .push(egui::Event::PointerMoved(egui::Pos2::new(100.0, 100.0)));

            // Check if events are accumulating
            if i.events.len() > 1000 {
                println!("⚠️ WARNING: Event buffer has {} events!", i.events.len());
            }

            if i.events.len() > 5000 {
                println!(
                    "❌ CRITICAL: Event buffer has {} events! This will freeze!",
                    i.events.len()
                );
                std::process::exit(1);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("Frame: {}", self.frame));
            ui.label(format!("Events: {}", event_count));
        });

        ctx.request_repaint();
    }
}
