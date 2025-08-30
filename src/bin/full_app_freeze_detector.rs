// Full app test with comprehensive mouse/scroll simulation to trigger freezes
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
    println!("=== Full App Freeze Detector ===");
    println!("This test will simulate real user interactions to trigger freezes");

    // Setup monitoring
    let freeze_detected = Arc::new(AtomicBool::new(false));
    let last_frame = Arc::new(Mutex::new(Instant::now()));
    let frame_counter = Arc::new(AtomicU64::new(0));
    let current_action = Arc::new(Mutex::new(String::from("Starting")));

    // Clone for monitor thread
    let freeze_detected_mon = freeze_detected.clone();
    let last_frame_mon = last_frame.clone();
    let frame_counter_mon = frame_counter.clone();
    let current_action_mon = current_action.clone();

    // Spawn freeze detection thread
    let monitor = thread::spawn(move || {
        let mut last_count = 0;
        let mut warnings = 0;

        loop {
            thread::sleep(Duration::from_millis(250));

            let count = frame_counter_mon.load(Ordering::Relaxed);
            let action = current_action_mon.lock().clone();

            if count == last_count {
                warnings += 1;
                let elapsed = last_frame_mon.lock().elapsed();

                println!("⚠️  No frames for {:?} during: {}", elapsed, action);

                if warnings > 2 {
                    println!("❌ FREEZE DETECTED! App unresponsive during: {}", action);
                    freeze_detected_mon.store(true, Ordering::Relaxed);

                    // Print diagnostic info
                    println!("\n=== FREEZE DIAGNOSTIC ===");
                    println!("Last action: {}", action);
                    println!("Frames processed: {}", count);
                    println!("Time since last frame: {:?}", elapsed);
                    println!("========================\n");

                    // Force exit after 5 seconds of freeze
                    thread::sleep(Duration::from_secs(5));
                    println!("Force exiting frozen app");
                    std::process::exit(1);
                }
            } else {
                warnings = 0;
                last_count = count;
            }

            if freeze_detected_mon.load(Ordering::Relaxed) {
                break;
            }
        }
    });

    // Run the app
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Plon - Freeze Detection Test"),
        ..Default::default()
    };

    let frame_cnt = frame_counter.clone();
    let last_frm = last_frame.clone();
    let curr_action = current_action.clone();
    let freeze_det = freeze_detected.clone();

    let _ = eframe::run_native(
        "Plon Test",
        options,
        Box::new(move |cc| {
            // Create repository with test data
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let repository = runtime.block_on(async {
                println!("Creating test database...");
                let pool = SqlitePoolOptions::new()
                    .connect("sqlite::memory:")
                    .await
                    .unwrap();

                sqlx::migrate!("./migrations").run(&pool).await.unwrap();

                let repo = Repository::new(pool);

                // Add some test tasks
                for i in 0..50 {
                    let task = Task::new(
                        format!("Test Task {}", i),
                        format!("Description for task {}", i),
                    );
                    let _ = repo.tasks.create(&task).await;
                }

                println!("Created {} test tasks", 50);
                repo
            });

            let app = PlonApp::new(cc, repository);

            Box::new(InteractiveTestApp {
                app,
                frame_count: frame_cnt,
                last_frame_time: last_frm,
                current_action: curr_action,
                freeze_detected: freeze_det,
                test_start: Instant::now(),
                phase: TestPhase::Init,
                mouse_pos: egui::Pos2::new(600.0, 400.0),
                scroll_amount: 0.0,
                pan_start: None,
                action_timer: Instant::now(),
            })
        }),
    );

    let _ = monitor.join();
}

#[derive(Debug, Clone, PartialEq)]
enum TestPhase {
    Init,
    NavigateToMap,
    SlowPan,
    FastPan,
    SlowScroll,
    FastScroll,
    MixedScrollPan,
    RapidActions,
    Complete,
}

struct InteractiveTestApp {
    app: PlonApp,
    frame_count: Arc<AtomicU64>,
    last_frame_time: Arc<Mutex<Instant>>,
    current_action: Arc<Mutex<String>>,
    freeze_detected: Arc<AtomicBool>,
    test_start: Instant,
    phase: TestPhase,
    mouse_pos: egui::Pos2,
    scroll_amount: f32,
    pan_start: Option<egui::Pos2>,
    action_timer: Instant,
}

impl eframe::App for InteractiveTestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Update monitoring
        self.frame_count.fetch_add(1, Ordering::Relaxed);
        *self.last_frame_time.lock() = Instant::now();

        let elapsed = self.test_start.elapsed();
        let frame_num = self.frame_count.load(Ordering::Relaxed);

        // Log every 30 frames
        if frame_num % 30 == 0 {
            println!(
                "[Frame {}] Phase: {:?}, Elapsed: {:?}",
                frame_num, self.phase, elapsed
            );
        }

        // Execute test phases
        match self.phase {
            TestPhase::Init => {
                if elapsed > Duration::from_secs(1) {
                    self.phase = TestPhase::NavigateToMap;
                    self.set_action("Navigating to Map View");
                    println!("\n>>> Phase: Navigate to Map View");
                }
            }

            TestPhase::NavigateToMap => {
                // Click on Map view button (usually around x=300)
                if self.action_timer.elapsed() > Duration::from_millis(100) {
                    self.simulate_click(ctx, egui::Pos2::new(300.0, 30.0));
                    self.phase = TestPhase::SlowPan;
                    self.action_timer = Instant::now();
                    println!("\n>>> Phase: Slow Pan");
                }
            }

            TestPhase::SlowPan => {
                self.set_action("Slow panning with middle mouse");

                // Simulate slow middle-mouse drag
                if frame_num % 5 == 0 {
                    if self.pan_start.is_none() {
                        // Start pan
                        self.pan_start = Some(self.mouse_pos);
                        ctx.input_mut(|i| {
                            i.events.push(egui::Event::PointerButton {
                                pos: self.mouse_pos,
                                button: egui::PointerButton::Middle,
                                pressed: true,
                                modifiers: Default::default(),
                            });
                        });
                    } else {
                        // Continue pan
                        self.mouse_pos.x += 2.0;
                        self.mouse_pos.y += 1.5;
                        ctx.input_mut(|i| {
                            i.events.push(egui::Event::PointerMoved(self.mouse_pos));
                        });
                    }
                }

                if elapsed > Duration::from_secs(4) {
                    // End pan
                    if self.pan_start.is_some() {
                        ctx.input_mut(|i| {
                            i.events.push(egui::Event::PointerButton {
                                pos: self.mouse_pos,
                                button: egui::PointerButton::Middle,
                                pressed: false,
                                modifiers: Default::default(),
                            });
                        });
                        self.pan_start = None;
                    }
                    self.phase = TestPhase::FastPan;
                    println!("\n>>> Phase: Fast Pan");
                }
            }

            TestPhase::FastPan => {
                self.set_action("Fast panning");

                // Rapid panning
                if frame_num % 2 == 0 {
                    let delta = (elapsed.as_secs_f32() * 10.0).sin() * 20.0;
                    self.simulate_middle_drag(
                        ctx,
                        self.mouse_pos,
                        self.mouse_pos + egui::vec2(delta, delta * 0.7),
                    );
                    self.mouse_pos.x += delta;
                    self.mouse_pos.y += delta * 0.7;
                }

                if elapsed > Duration::from_secs(6) {
                    self.phase = TestPhase::SlowScroll;
                    println!("\n>>> Phase: Slow Scroll");
                }
            }

            TestPhase::SlowScroll => {
                self.set_action("Slow trackpad scrolling");

                // Simulate trackpad scroll
                if frame_num % 10 == 0 {
                    ctx.input_mut(|i| {
                        i.events.push(egui::Event::Scroll(egui::vec2(0.0, 5.0)));
                    });
                    self.scroll_amount += 5.0;
                }

                if elapsed > Duration::from_secs(8) {
                    self.phase = TestPhase::FastScroll;
                    println!("\n>>> Phase: Fast Scroll (this often triggers freeze)");
                }
            }

            TestPhase::FastScroll => {
                self.set_action("Fast continuous scrolling");

                // Rapid scrolling - this often triggers the freeze
                ctx.input_mut(|i| {
                    let time = elapsed.as_secs_f32();
                    let scroll_x = (time * 5.0).sin() * 15.0;
                    let scroll_y = (time * 3.0).cos() * 20.0;

                    i.events
                        .push(egui::Event::Scroll(egui::vec2(scroll_x, scroll_y)));

                    // Also add some horizontal scrolling
                    if frame_num % 3 == 0 {
                        i.events.push(egui::Event::Scroll(egui::vec2(25.0, 0.0)));
                    }
                });

                if elapsed > Duration::from_secs(11) {
                    self.phase = TestPhase::MixedScrollPan;
                    println!("\n>>> Phase: Mixed Scroll and Pan");
                }
            }

            TestPhase::MixedScrollPan => {
                self.set_action("Mixed scrolling and panning");

                // Alternate between scroll and pan rapidly
                if frame_num % 4 == 0 {
                    // Scroll
                    ctx.input_mut(|i| {
                        i.events.push(egui::Event::Scroll(egui::vec2(10.0, 15.0)));
                    });
                } else if frame_num % 4 == 2 {
                    // Quick pan
                    self.simulate_middle_drag(
                        ctx,
                        self.mouse_pos,
                        self.mouse_pos + egui::vec2(10.0, 10.0),
                    );
                }

                if elapsed > Duration::from_secs(14) {
                    self.phase = TestPhase::RapidActions;
                    println!("\n>>> Phase: Rapid Actions (stress test)");
                }
            }

            TestPhase::RapidActions => {
                self.set_action("Rapid mixed actions - stress test");

                // Throw everything at it
                ctx.input_mut(|i| {
                    // Scroll
                    i.events.push(egui::Event::Scroll(egui::vec2(
                        (frame_num as f32 * 0.1).sin() * 30.0,
                        (frame_num as f32 * 0.15).cos() * 40.0,
                    )));

                    // Mouse movement
                    i.events.push(egui::Event::PointerMoved(egui::Pos2::new(
                        600.0 + (frame_num as f32 * 0.2).sin() * 200.0,
                        400.0 + (frame_num as f32 * 0.3).cos() * 150.0,
                    )));

                    // Occasional clicks
                    if frame_num % 20 == 0 {
                        i.events.push(egui::Event::PointerButton {
                            pos: self.mouse_pos,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: Default::default(),
                        });
                        i.events.push(egui::Event::PointerButton {
                            pos: self.mouse_pos,
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: Default::default(),
                        });
                    }
                });

                if elapsed > Duration::from_secs(18) {
                    self.phase = TestPhase::Complete;
                    println!("\n>>> Test Complete - No freeze detected!");
                    // Don't close immediately - let it run a bit more to see if freeze happens
                    // ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }

            TestPhase::Complete => {
                self.set_action("Test complete - rendering normally");
                // Just render normally without events
                if elapsed > Duration::from_secs(25) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // Check for freeze
        if self.freeze_detected.load(Ordering::Relaxed) {
            println!("❌ Freeze detected - exiting");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            std::process::exit(1);
        }

        // Run the actual app
        self.app.update(ctx, frame);

        // Always request repaint to keep test running
        ctx.request_repaint();
    }
}

impl InteractiveTestApp {
    fn set_action(&mut self, action: &str) {
        *self.current_action.lock() = action.to_string();
    }

    fn simulate_click(&mut self, ctx: &egui::Context, pos: egui::Pos2) {
        ctx.input_mut(|i| {
            i.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            });
            i.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            });
        });
    }

    fn simulate_middle_drag(&mut self, ctx: &egui::Context, from: egui::Pos2, to: egui::Pos2) {
        ctx.input_mut(|i| {
            // Press
            i.events.push(egui::Event::PointerButton {
                pos: from,
                button: egui::PointerButton::Middle,
                pressed: true,
                modifiers: Default::default(),
            });
            // Move
            i.events.push(egui::Event::PointerMoved(to));
            // Release
            i.events.push(egui::Event::PointerButton {
                pos: to,
                button: egui::PointerButton::Middle,
                pressed: false,
                modifiers: Default::default(),
            });
        });
    }
}
