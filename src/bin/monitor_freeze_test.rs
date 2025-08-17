use eframe::{egui, NativeOptions};
use plon::ui::PlonApp;
use plon::repository::Repository;
use plon::domain::task::Task;
use sqlx::sqlite::SqlitePoolOptions;
use std::time::{Duration, Instant};

fn main() {
    println!("=== Monitor Freeze Test ===");
    println!("Monitoring what happens around the freeze point...");
    
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    
    let _ = eframe::run_native(
        "Monitor Test",
        options,
        Box::new(move |cc| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let repository = runtime.block_on(async {
                let pool = SqlitePoolOptions::new()
                    .connect("sqlite::memory:")
                    .await
                    .unwrap();
                
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .unwrap();
                
                let repo = Repository::new(pool);
                
                for i in 0..30 {
                    let task = Task::new(
                        format!("Task {}", i),
                        format!("Description {}", i)
                    );
                    let _ = repo.tasks.create(&task).await;
                }
                
                repo
            });
            
            let app = PlonApp::new(cc, repository);
            
            Box::new(MonitorTestApp {
                app,
                test_start: Instant::now(),
                frame_count: 0,
                events_sent: 0,
                update_times: Vec::new(),
            })
        }),
    );
}

struct MonitorTestApp {
    app: PlonApp,
    test_start: Instant,
    frame_count: u64,
    events_sent: u64,
    update_times: Vec<Duration>,
}

impl eframe::App for MonitorTestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let update_start = Instant::now();
        self.frame_count += 1;
        
        let elapsed = self.test_start.elapsed();
        
        // Navigate to map
        if elapsed < Duration::from_millis(500) {
            // Wait
        } else if elapsed < Duration::from_secs(1) {
            ctx.input_mut(|i| {
                i.events.push(egui::Event::PointerButton {
                    pos: egui::Pos2::new(300.0, 30.0),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                });
                i.events.push(egui::Event::PointerButton {
                    pos: egui::Pos2::new(300.0, 30.0),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                });
            });
        } else if elapsed < Duration::from_secs(20) {
            // Start simulating rapid actions after 10 seconds
            if elapsed > Duration::from_secs(10) {
                // Simulate the same rapid actions as the freeze test
                ctx.input_mut(|i| {
                    // Add multiple events like the freeze test does
                    i.events.push(egui::Event::Scroll(egui::vec2(
                        (self.frame_count as f32 * 0.1).sin() * 30.0,
                        (self.frame_count as f32 * 0.15).cos() * 40.0
                    )));
                    
                    i.events.push(egui::Event::PointerMoved(
                        egui::Pos2::new(
                            600.0 + (self.frame_count as f32 * 0.2).sin() * 200.0,
                            400.0 + (self.frame_count as f32 * 0.3).cos() * 150.0
                        )
                    ));
                    
                    self.events_sent += 2;
                    
                    if self.frame_count % 20 == 0 {
                        i.events.push(egui::Event::PointerButton {
                            pos: egui::Pos2::new(600.0, 400.0),
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: Default::default(),
                        });
                        i.events.push(egui::Event::PointerButton {
                            pos: egui::Pos2::new(600.0, 400.0),
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: Default::default(),
                        });
                        self.events_sent += 2;
                    }
                });
                
                // Monitor around frame 2100-2200
                if self.frame_count >= 2000 && self.frame_count <= 2200 {
                    if self.frame_count % 10 == 0 {
                        println!("Frame {}: Events sent: {}, Elapsed: {:?}", 
                                 self.frame_count, self.events_sent, elapsed);
                    }
                }
            }
        } else {
            // Calculate average update time
            if !self.update_times.is_empty() {
                let avg = self.update_times.iter().sum::<Duration>() / self.update_times.len() as u32;
                let max = self.update_times.iter().max().unwrap();
                println!("Test complete. Avg frame time: {:?}, Max: {:?}", avg, max);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        
        // Run the app
        self.app.update(ctx, frame);
        
        let update_time = update_start.elapsed();
        self.update_times.push(update_time);
        
        // Alert on slow frames
        if update_time > Duration::from_millis(100) {
            println!("⚠️ SLOW FRAME {} at {:?}: {:?}", 
                     self.frame_count, elapsed, update_time);
        }
        
        ctx.request_repaint();
    }
}