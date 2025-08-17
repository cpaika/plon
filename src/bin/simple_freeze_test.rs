use eframe::{egui, NativeOptions};
use plon::ui::PlonApp;
use plon::repository::Repository;
use plon::domain::task::Task;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use parking_lot::Mutex;

fn main() {
    println!("=== Simple Freeze Test ===");
    println!("Testing with minimal rapid actions");
    
    let freeze_detected = Arc::new(AtomicBool::new(false));
    let last_frame = Arc::new(Mutex::new(Instant::now()));
    let frame_counter = Arc::new(AtomicU64::new(0));
    
    let freeze_det_mon = freeze_detected.clone();
    let last_frame_mon = last_frame.clone();
    let frame_counter_mon = frame_counter.clone();
    
    thread::spawn(move || {
        let mut last_count = 0;
        loop {
            thread::sleep(Duration::from_millis(250));
            
            let count = frame_counter_mon.load(Ordering::Relaxed);
            if count == last_count && count > 10 {
                let elapsed = last_frame_mon.lock().elapsed();
                if elapsed > Duration::from_millis(500) {
                    println!("❌ FREEZE DETECTED after {} frames!", count);
                    freeze_det_mon.store(true, Ordering::Relaxed);
                    thread::sleep(Duration::from_secs(2));
                    std::process::exit(1);
                }
            }
            last_count = count;
        }
    });
    
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    
    let frame_cnt = frame_counter.clone();
    let last_frm = last_frame.clone();
    
    let _ = eframe::run_native(
        "Simple Freeze Test",
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
            
            Box::new(SimpleTestApp {
                app,
                frame_count: frame_cnt,
                last_frame_time: last_frm,
                test_start: Instant::now(),
                event_count: 0,
            })
        }),
    );
}

struct SimpleTestApp {
    app: PlonApp,
    frame_count: Arc<AtomicU64>,
    last_frame_time: Arc<Mutex<Instant>>,
    test_start: Instant,
    event_count: u32,
}

impl eframe::App for SimpleTestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.frame_count.fetch_add(1, Ordering::Relaxed);
        *self.last_frame_time.lock() = Instant::now();
        
        let elapsed = self.test_start.elapsed();
        let frame_num = self.frame_count.load(Ordering::Relaxed);
        
        if frame_num % 30 == 0 {
            println!("[Frame {}] Elapsed: {:?}, Events sent: {}", 
                     frame_num, elapsed, self.event_count);
        }
        
        // Navigate to map view initially
        if elapsed < Duration::from_millis(500) {
            // Do nothing
        } else if elapsed < Duration::from_secs(1) {
            // Click map button once
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
        } else if elapsed < Duration::from_secs(10) {
            // Simple scrolling every 10 frames
            if frame_num % 10 == 0 {
                ctx.input_mut(|i| {
                    i.events.push(egui::Event::Scroll(egui::vec2(0.0, 10.0)));
                    self.event_count += 1;
                });
            }
        } else if elapsed < Duration::from_secs(20) {
            // Increase frequency gradually
            if frame_num % 5 == 0 {
                ctx.input_mut(|i| {
                    i.events.push(egui::Event::Scroll(egui::vec2(5.0, 10.0)));
                    self.event_count += 1;
                });
            }
        } else if elapsed < Duration::from_secs(30) {
            // Test with ONE event per frame
            ctx.input_mut(|i| {
                i.events.push(egui::Event::Scroll(egui::vec2(
                    (frame_num as f32 * 0.1).sin() * 10.0,
                    (frame_num as f32 * 0.1).cos() * 10.0
                )));
                self.event_count += 1;
            });
        } else {
            println!("✅ Test completed successfully!");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        
        // Run the app
        self.app.update(ctx, frame);
        
        ctx.request_repaint();
    }
}