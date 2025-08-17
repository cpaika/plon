use eframe::{egui, NativeOptions};
use plon::ui::PlonApp;
use plon::repository::Repository;
use plon::domain::task::Task;
use sqlx::sqlite::SqlitePoolOptions;
use std::time::{Duration, Instant};

fn main() {
    println!("=== Debug Freeze Test ===");
    
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    
    let _ = eframe::run_native(
        "Debug Test",
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
                
                // Add test tasks
                for i in 0..10 {
                    let task = Task::new(
                        format!("Task {}", i),
                        format!("Description {}", i)
                    );
                    let _ = repo.tasks.create(&task).await;
                }
                
                repo
            });
            
            let app = PlonApp::new(cc, repository);
            
            Box::new(DebugTestApp {
                app,
                test_start: Instant::now(),
                frame_count: 0,
                last_update_time: Instant::now(),
            })
        }),
    );
}

struct DebugTestApp {
    app: PlonApp,
    test_start: Instant,
    frame_count: u64,
    last_update_time: Instant,
}

impl eframe::App for DebugTestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let update_start = Instant::now();
        self.frame_count += 1;
        
        let elapsed = self.test_start.elapsed();
        
        // Navigate to map view
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
        } else if elapsed < Duration::from_secs(15) {
            // Add scroll events every frame after 5 seconds
            if elapsed > Duration::from_secs(5) {
                ctx.input_mut(|i| {
                    i.events.push(egui::Event::Scroll(egui::vec2(5.0, 10.0)));
                });
            }
        } else {
            println!("Test done");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        
        // Measure app update time
        let before_app = Instant::now();
        self.app.update(ctx, frame);
        let app_time = before_app.elapsed();
        
        // Print if slow
        if app_time > Duration::from_millis(50) {
            println!("SLOW FRAME {} at {:?}: app.update took {:?}", 
                     self.frame_count, elapsed, app_time);
        }
        
        // Print periodic status
        if self.frame_count % 60 == 0 {
            let frame_time = update_start.saturating_duration_since(self.last_update_time);
            println!("Frame {} at {:?}, frame time: {:?}", 
                     self.frame_count, elapsed, frame_time);
        }
        
        self.last_update_time = update_start;
        ctx.request_repaint();
    }
}