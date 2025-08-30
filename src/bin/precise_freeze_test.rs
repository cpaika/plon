use eframe::{NativeOptions, egui};
use parking_lot::Mutex;
use plon::domain::task::Task;
use plon::repository::Repository;
use plon::ui::PlonApp;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("=== Precise Freeze Detection ===");
    println!("Testing exact freeze conditions...");

    let freeze_detected = Arc::new(AtomicBool::new(false));
    let last_frame = Arc::new(Mutex::new(Instant::now()));
    let frame_counter = Arc::new(AtomicU64::new(0));

    let freeze_det = freeze_detected.clone();
    let last_frm = last_frame.clone();
    let frame_cnt = frame_counter.clone();

    // Monitor thread
    thread::spawn(move || {
        let mut last_count = 0;
        let mut stuck_count = 0;

        loop {
            thread::sleep(Duration::from_millis(100));
            let count = frame_cnt.load(Ordering::Relaxed);

            if count == last_count && count > 10 {
                stuck_count += 1;
                let elapsed = last_frm.lock().elapsed();
                println!("⚠️ Frame stuck at {} for {:?}", count, elapsed);

                if stuck_count > 3 {
                    println!("❌ FREEZE CONFIRMED at frame {}", count);
                    freeze_det.store(true, Ordering::Relaxed);

                    // Wait a bit then exit
                    thread::sleep(Duration::from_secs(2));
                    std::process::exit(1);
                }
            } else {
                if stuck_count > 0 {
                    println!("✓ Recovered after {} stuck checks", stuck_count);
                }
                stuck_count = 0;
                last_count = count;
            }
        }
    });

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    let frame_counter_app = frame_counter.clone();
    let last_frame_app = last_frame.clone();

    let _ = eframe::run_native(
        "Precise Freeze Test",
        options,
        Box::new(move |cc| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let repository = runtime.block_on(async {
                let pool = SqlitePoolOptions::new()
                    .connect("sqlite::memory:")
                    .await
                    .unwrap();

                sqlx::migrate!("./migrations").run(&pool).await.unwrap();

                let repo = Repository::new(pool);

                // Create different amounts of tasks to test
                for i in 0..100 {
                    let task = Task::new(format!("Task {}", i), format!("Description {}", i));
                    let _ = repo.tasks.create(&task).await;
                }

                println!("Created 100 test tasks");
                repo
            });

            let app = PlonApp::new(cc, repository);

            Box::new(PreciseTestApp {
                app,
                frame_count: frame_counter_app,
                last_frame_time: last_frame_app,
                test_start: Instant::now(),
                phase: TestPhase::Init,
                events_sent: 0,
            })
        }),
    );
}

#[derive(Debug, Clone, PartialEq)]
enum TestPhase {
    Init,
    NavigateToMap,
    WarmUp,
    SlowEvents,
    ModerateEvents,
    RapidEvents,
    Complete,
}

struct PreciseTestApp {
    app: PlonApp,
    frame_count: Arc<AtomicU64>,
    last_frame_time: Arc<Mutex<Instant>>,
    test_start: Instant,
    phase: TestPhase,
    events_sent: u64,
}

impl eframe::App for PreciseTestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let frame_start = Instant::now();
        self.frame_count.fetch_add(1, Ordering::Relaxed);
        *self.last_frame_time.lock() = frame_start;

        let elapsed = self.test_start.elapsed();
        let frame_num = self.frame_count.load(Ordering::Relaxed);

        // Print phase transitions
        let new_phase = match elapsed {
            d if d < Duration::from_millis(500) => TestPhase::Init,
            d if d < Duration::from_secs(1) => TestPhase::NavigateToMap,
            d if d < Duration::from_secs(3) => TestPhase::WarmUp,
            d if d < Duration::from_secs(6) => TestPhase::SlowEvents,
            d if d < Duration::from_secs(10) => TestPhase::ModerateEvents,
            d if d < Duration::from_secs(15) => TestPhase::RapidEvents,
            _ => TestPhase::Complete,
        };

        if new_phase != self.phase {
            println!(
                "\n>>> Phase: {:?} at frame {} (events sent: {})",
                new_phase, frame_num, self.events_sent
            );
            self.phase = new_phase.clone();
        }

        // Execute test actions
        match self.phase {
            TestPhase::Init => {
                // Wait
            }
            TestPhase::NavigateToMap => {
                // Click map button once
                if self.events_sent == 0 {
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
                    self.events_sent += 2;
                }
            }
            TestPhase::WarmUp => {
                // Just render without events
            }
            TestPhase::SlowEvents => {
                // One event every 30 frames
                if frame_num % 30 == 0 {
                    ctx.input_mut(|i| {
                        i.events.push(egui::Event::Scroll(egui::vec2(5.0, 10.0)));
                    });
                    self.events_sent += 1;
                }
            }
            TestPhase::ModerateEvents => {
                // One event every 10 frames
                if frame_num % 10 == 0 {
                    ctx.input_mut(|i| {
                        i.events.push(egui::Event::Scroll(egui::vec2(10.0, 15.0)));
                    });
                    self.events_sent += 1;
                }
            }
            TestPhase::RapidEvents => {
                // Two events every frame
                ctx.input_mut(|i| {
                    i.events.push(egui::Event::Scroll(egui::vec2(
                        (frame_num as f32 * 0.1).sin() * 20.0,
                        (frame_num as f32 * 0.1).cos() * 20.0,
                    )));
                    i.events.push(egui::Event::PointerMoved(egui::Pos2::new(
                        600.0 + (frame_num as f32 * 0.2).sin() * 100.0,
                        400.0 + (frame_num as f32 * 0.2).cos() * 100.0,
                    )));
                });
                self.events_sent += 2;

                // Log every 100 events
                if self.events_sent % 100 == 0 {
                    println!("   Sent {} events total", self.events_sent);
                }
            }
            TestPhase::Complete => {
                println!("✅ Test complete! No freeze after {} frames", frame_num);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // Update the app
        let before_update = Instant::now();
        self.app.update(ctx, frame);
        let update_time = before_update.elapsed();

        // Warn on slow updates
        if update_time > Duration::from_millis(50) {
            println!("⚠️ Slow update at frame {}: {:?}", frame_num, update_time);
        }

        ctx.request_repaint();
    }
}
