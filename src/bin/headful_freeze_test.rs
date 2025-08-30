// Headful test that runs the actual app and detects freezes during scrolling
use eframe::{NativeOptions, egui};
use plon::repository::Repository;
use plon::ui::PlonApp;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("Starting headful freeze detection test...");

    // Shared state for monitoring
    let app_responsive = Arc::new(AtomicBool::new(true));
    let frame_count = Arc::new(AtomicU64::new(0));
    let last_frame_time = Arc::new(parking_lot::Mutex::new(Instant::now()));
    let freeze_detected = Arc::new(AtomicBool::new(false));
    let test_phase = Arc::new(parking_lot::Mutex::new(TestPhase::Startup));

    // Clone for monitoring thread
    let app_responsive_monitor = app_responsive.clone();
    let frame_count_monitor = frame_count.clone();
    let last_frame_time_monitor = last_frame_time.clone();
    let freeze_detected_monitor = freeze_detected.clone();
    let test_phase_monitor = test_phase.clone();

    // Spawn monitoring thread
    let monitor_handle = thread::spawn(move || {
        let mut last_count = 0u64;
        let mut freeze_start: Option<Instant> = None;

        loop {
            thread::sleep(Duration::from_millis(100));

            let current_count = frame_count_monitor.load(Ordering::Relaxed);
            let current_phase = test_phase_monitor.lock().clone();

            // Check if frames are advancing
            if current_count == last_count && current_phase != TestPhase::Complete {
                // No new frames in 100ms
                if freeze_start.is_none() {
                    freeze_start = Some(Instant::now());
                    println!(
                        "⚠️  Warning: No frames for 100ms during {:?}",
                        current_phase
                    );
                }

                let freeze_duration = freeze_start.unwrap().elapsed();

                if freeze_duration > Duration::from_millis(500) {
                    println!(
                        "❌ FREEZE DETECTED: No frames for {:?} during {:?}!",
                        freeze_duration, current_phase
                    );
                    freeze_detected_monitor.store(true, Ordering::Relaxed);
                    app_responsive_monitor.store(false, Ordering::Relaxed);

                    // Get last frame time for debugging
                    let last_time = last_frame_time_monitor.lock();
                    println!("   Last frame was at: {:?} ago", last_time.elapsed());

                    // Force exit after detecting freeze
                    if freeze_duration > Duration::from_secs(2) {
                        println!("❌ App frozen for >2 seconds. Terminating test.");
                        std::process::exit(1);
                    }
                }
            } else {
                // Frames are advancing
                if freeze_start.is_some() {
                    println!("✓ App responsive again after freeze");
                }
                freeze_start = None;
                last_count = current_count;
            }

            // Exit after test completes
            if current_phase == TestPhase::Complete {
                break;
            }
        }
    });

    // Create native options
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    // Run the app with instrumentation
    let frame_counter = frame_count.clone();
    let last_frame_timer = last_frame_time.clone();
    let test_phase_runner = test_phase.clone();
    let freeze_detector = freeze_detected.clone();

    let result = eframe::run_native(
        "Plon Freeze Test",
        options,
        Box::new(move |cc| {
            // Create a test repository
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let repository = runtime.block_on(async {
                let pool = SqlitePoolOptions::new()
                    .connect("sqlite::memory:")
                    .await
                    .unwrap();

                // Run migrations
                sqlx::migrate!("./migrations").run(&pool).await.unwrap();

                Repository::new(pool)
            });

            let app = PlonApp::new(cc, repository);

            // Wrap the app with our test harness
            Box::new(TestHarness {
                app,
                frame_count: frame_counter,
                last_frame_time: last_frame_timer,
                test_phase: test_phase_runner,
                freeze_detected: freeze_detector,
                test_start: Instant::now(),
                last_input_time: Instant::now(),
                scroll_position: 0.0,
                mouse_position: egui::Pos2::new(512.0, 384.0),
            })
        }),
    );

    // Wait for monitor thread
    let _ = monitor_handle.join();

    // Check results
    if freeze_detected.load(Ordering::Relaxed) {
        eprintln!("❌ TEST FAILED: Freeze detected during scrolling!");
        std::process::exit(1);
    } else {
        println!("✅ TEST PASSED: No freezes detected");
    }

    if let Err(e) = result {
        eprintln!("App error: {}", e);
        std::process::exit(1);
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TestPhase {
    Startup,
    NavigatingToMap,
    InitialPanning,
    ScrollingSlowly,
    ScrollingQuickly,
    RapidScrolling,
    ContinuousScroll,
    Complete,
}

struct TestHarness {
    app: PlonApp,
    frame_count: Arc<AtomicU64>,
    last_frame_time: Arc<parking_lot::Mutex<Instant>>,
    test_phase: Arc<parking_lot::Mutex<TestPhase>>,
    freeze_detected: Arc<AtomicBool>,
    test_start: Instant,
    last_input_time: Instant,
    scroll_position: f32,
    mouse_position: egui::Pos2,
}

impl eframe::App for TestHarness {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Update frame counter
        self.frame_count.fetch_add(1, Ordering::Relaxed);
        *self.last_frame_time.lock() = Instant::now();

        let elapsed = self.test_start.elapsed();
        let frame_num = self.frame_count.load(Ordering::Relaxed);

        // Simulate user interactions based on test phase
        let current_phase = self.test_phase.lock().clone();

        match current_phase {
            TestPhase::Startup => {
                if elapsed > Duration::from_millis(500) {
                    println!("Phase: Navigating to map view...");
                    *self.test_phase.lock() = TestPhase::NavigatingToMap;

                    // Simulate clicking on Map View button
                    // We'll simulate clicking in the area where the Map button likely is
                    // (this is a bit hacky but works for testing)
                    self.mouse_position = egui::Pos2::new(300.0, 50.0); // Approximate Map button location
                    ctx.input_mut(|i| {
                        i.events.push(egui::Event::PointerButton {
                            pos: self.mouse_position,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: Default::default(),
                        });
                        i.events.push(egui::Event::PointerButton {
                            pos: self.mouse_position,
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: Default::default(),
                        });
                    });
                }
            }

            TestPhase::NavigatingToMap => {
                if elapsed > Duration::from_secs(1) {
                    println!("Phase: Initial panning...");
                    *self.test_phase.lock() = TestPhase::InitialPanning;
                }
            }

            TestPhase::InitialPanning => {
                // Simulate middle mouse drag
                if frame_num % 2 == 0 {
                    self.simulate_pan(ctx, 5.0, 3.0);
                }

                if elapsed > Duration::from_secs(3) {
                    println!("Phase: Slow scrolling...");
                    *self.test_phase.lock() = TestPhase::ScrollingSlowly;
                }
            }

            TestPhase::ScrollingSlowly => {
                // Simulate slow trackpad scrolling
                if frame_num % 5 == 0 {
                    self.simulate_scroll(ctx, 0.0, 10.0);
                }

                if elapsed > Duration::from_secs(5) {
                    println!("Phase: Quick scrolling...");
                    *self.test_phase.lock() = TestPhase::ScrollingQuickly;
                }
            }

            TestPhase::ScrollingQuickly => {
                // Simulate faster scrolling
                if frame_num % 2 == 0 {
                    self.simulate_scroll(ctx, 5.0, 15.0);
                }

                if elapsed > Duration::from_secs(7) {
                    println!("Phase: Rapid scrolling (this often triggers the freeze)...");
                    *self.test_phase.lock() = TestPhase::RapidScrolling;
                }
            }

            TestPhase::RapidScrolling => {
                // Simulate very rapid scrolling - this often triggers freezes
                self.simulate_scroll(ctx, 20.0, 30.0);

                // Also simulate some horizontal scrolling
                if frame_num % 3 == 0 {
                    self.simulate_scroll(ctx, 15.0, 0.0);
                }

                if elapsed > Duration::from_secs(9) {
                    println!("Phase: Continuous scroll test...");
                    *self.test_phase.lock() = TestPhase::ContinuousScroll;
                }
            }

            TestPhase::ContinuousScroll => {
                // Continuous scrolling without pause
                let time = elapsed.as_secs_f32();
                let scroll_x = (time * 3.0).sin() * 20.0;
                let scroll_y = (time * 2.0).cos() * 25.0;
                self.simulate_scroll(ctx, scroll_x, scroll_y);

                if elapsed > Duration::from_secs(12) {
                    println!("Test complete.");
                    *self.test_phase.lock() = TestPhase::Complete;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }

            TestPhase::Complete => {
                // Test done
            }
        }

        // Check for freeze
        if self.freeze_detected.load(Ordering::Relaxed) {
            println!(
                "Freeze detected at frame {} during {:?}",
                frame_num, current_phase
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Update the actual app
        self.app.update(ctx, frame);

        // Request repaint to keep the test running
        ctx.request_repaint();
    }
}

impl TestHarness {
    fn simulate_scroll(&mut self, ctx: &egui::Context, dx: f32, dy: f32) {
        ctx.input_mut(|i| {
            i.events.push(egui::Event::Scroll(egui::vec2(dx, dy)));
        });
        self.scroll_position += dy;
    }

    fn simulate_pan(&mut self, ctx: &egui::Context, dx: f32, dy: f32) {
        // Simulate middle mouse button drag
        ctx.input_mut(|i| {
            // Mouse down
            i.events.push(egui::Event::PointerButton {
                pos: self.mouse_position,
                button: egui::PointerButton::Middle,
                pressed: true,
                modifiers: Default::default(),
            });

            // Move
            self.mouse_position += egui::vec2(dx, dy);
            i.events
                .push(egui::Event::PointerMoved(self.mouse_position));

            // Mouse up
            i.events.push(egui::Event::PointerButton {
                pos: self.mouse_position,
                button: egui::PointerButton::Middle,
                pressed: false,
                modifiers: Default::default(),
            });
        });
    }
}
