use eframe::{NativeOptions, egui};
use plon::domain::goal::Goal;
use plon::domain::task::Task;
use plon::ui::views::map_view::MapView;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Comprehensive headful test for map view panning, especially Mac trackpad support
struct MapPanningTest {
    map_view: MapView,
    tasks: Vec<Task>,
    goals: Vec<Goal>,

    // Test state
    test_phase: TestPhase,
    start_time: Instant,
    phase_start_time: Instant,

    // Tracking data
    camera_history: VecDeque<(f64, egui::Vec2, String)>, // (timestamp, position, event)
    zoom_history: VecDeque<(f64, f32, String)>,          // (timestamp, zoom, event)

    // Test results
    test_results: Vec<TestResult>,

    // Simulated input state
    simulated_scroll: egui::Vec2,
    simulated_zoom: f32,
    simulated_mouse_button: Option<egui::PointerButton>,
    simulated_modifiers: egui::Modifiers,
}

#[derive(Debug, Clone, PartialEq)]
enum TestPhase {
    Setup,
    TestTrackpadPan,
    TestPinchZoom,
    TestMiddleMousePan,
    TestShiftDrag,
    TestScrollWheel,
    TestPanOverTasks, // New phase to test panning over tasks
    Complete,
}

#[derive(Debug, Clone)]
struct TestResult {
    phase: TestPhase,
    passed: bool,
    message: String,
    camera_start: egui::Vec2,
    camera_end: egui::Vec2,
    zoom_start: f32,
    zoom_end: f32,
}

impl MapPanningTest {
    fn new() -> Self {
        // Create test tasks in a grid pattern for visual reference
        let mut tasks = Vec::new();
        for row in 0..10 {
            for col in 0..10 {
                let mut task = Task::new(
                    format!("Task {}:{}", row, col),
                    format!("Grid position ({}, {})", row, col),
                );
                task.set_position((col * 150) as f64, (row * 150) as f64);
                tasks.push(task);
            }
        }

        // Add some landmark tasks
        let mut center_task = Task::new("CENTER".to_string(), "Center reference point".to_string());
        center_task.set_position(750.0, 750.0);
        tasks.push(center_task);

        Self {
            map_view: MapView::new(),
            tasks,
            goals: Vec::new(),
            test_phase: TestPhase::Setup,
            start_time: Instant::now(),
            phase_start_time: Instant::now(),
            camera_history: VecDeque::with_capacity(1000),
            zoom_history: VecDeque::with_capacity(1000),
            test_results: Vec::new(),
            simulated_scroll: egui::Vec2::ZERO,
            simulated_zoom: 1.0,
            simulated_mouse_button: None,
            simulated_modifiers: egui::Modifiers::default(),
        }
    }

    fn advance_phase(&mut self) {
        // Record result for current phase
        if self.test_phase != TestPhase::Setup && self.test_phase != TestPhase::Complete {
            self.evaluate_current_phase();
        }

        // Move to next phase
        self.test_phase = match self.test_phase {
            TestPhase::Setup => TestPhase::TestTrackpadPan,
            TestPhase::TestTrackpadPan => TestPhase::TestPinchZoom,
            TestPhase::TestPinchZoom => TestPhase::TestMiddleMousePan,
            TestPhase::TestMiddleMousePan => TestPhase::TestShiftDrag,
            TestPhase::TestShiftDrag => TestPhase::TestScrollWheel,
            TestPhase::TestScrollWheel => TestPhase::TestPanOverTasks,
            TestPhase::TestPanOverTasks => TestPhase::Complete,
            TestPhase::Complete => TestPhase::Complete,
        };

        self.phase_start_time = Instant::now();

        // Reset simulated input
        self.simulated_scroll = egui::Vec2::ZERO;
        self.simulated_zoom = 1.0;
        self.simulated_mouse_button = None;
        self.simulated_modifiers = egui::Modifiers::default();
    }

    fn evaluate_current_phase(&mut self) {
        let camera_start = self
            .camera_history
            .iter()
            .find(|(_, _, event)| event.contains(&format!("{:?}", self.test_phase)))
            .map(|(_, pos, _)| *pos)
            .unwrap_or(egui::Vec2::ZERO);

        let camera_end = self.map_view.get_camera_position();
        let zoom_start = self.zoom_history.front().map(|(_, z, _)| *z).unwrap_or(1.0);
        let zoom_end = self.map_view.get_zoom_level();

        let (passed, message) = match self.test_phase {
            TestPhase::TestTrackpadPan => {
                let moved = (camera_end - camera_start).length() > 10.0;
                (
                    moved,
                    if moved {
                        "✅ Trackpad pan works! Camera moved as expected.".to_string()
                    } else {
                        "❌ Trackpad pan failed! Camera didn't move.".to_string()
                    },
                )
            }
            TestPhase::TestPinchZoom => {
                let zoomed = (zoom_end - zoom_start).abs() > 0.1;
                (
                    zoomed,
                    if zoomed {
                        format!(
                            "✅ Pinch zoom works! Zoom changed from {:.2} to {:.2}",
                            zoom_start, zoom_end
                        )
                    } else {
                        "❌ Pinch zoom failed! Zoom level didn't change.".to_string()
                    },
                )
            }
            TestPhase::TestMiddleMousePan => {
                let moved = (camera_end - camera_start).length() > 5.0;
                (
                    moved,
                    if moved {
                        "✅ Middle mouse pan works!".to_string()
                    } else {
                        "❌ Middle mouse pan failed!".to_string()
                    },
                )
            }
            TestPhase::TestShiftDrag => {
                let moved = (camera_end - camera_start).length() > 5.0;
                (
                    moved,
                    if moved {
                        "✅ Shift+drag pan works!".to_string()
                    } else {
                        "❌ Shift+drag pan failed!".to_string()
                    },
                )
            }
            TestPhase::TestScrollWheel => {
                let moved_or_zoomed = (camera_end - camera_start).length() > 5.0
                    || (zoom_end - zoom_start).abs() > 0.1;
                (
                    moved_or_zoomed,
                    if moved_or_zoomed {
                        "✅ Scroll wheel handling works!".to_string()
                    } else {
                        "❌ Scroll wheel handling failed!".to_string()
                    },
                )
            }
            TestPhase::TestPanOverTasks => {
                // This test passes if panning continues even when hovering over tasks
                // We'll check if camera moved during the test
                let moved = (camera_end - camera_start).length() > 10.0;
                (
                    moved,
                    if moved {
                        "✅ Panning continues when hovering over tasks!".to_string()
                    } else {
                        "❌ Panning STOPS when hovering over tasks - BUG DETECTED!".to_string()
                    },
                )
            }
            _ => (true, "N/A".to_string()),
        };

        self.test_results.push(TestResult {
            phase: self.test_phase.clone(),
            passed,
            message,
            camera_start,
            camera_end,
            zoom_start,
            zoom_end,
        });
    }

    fn simulate_input_for_phase(&mut self, ctx: &egui::Context) {
        let phase_time = self.phase_start_time.elapsed().as_secs_f32();

        match self.test_phase {
            TestPhase::TestTrackpadPan => {
                // Simulate two-finger trackpad pan
                if phase_time < 2.0 {
                    // Smooth circular motion
                    let angle = phase_time * 2.0;
                    self.simulated_scroll = egui::Vec2::new(angle.cos() * 30.0, angle.sin() * 30.0);
                }
            }
            TestPhase::TestPinchZoom => {
                // Simulate pinch zoom in and out
                if phase_time < 1.0 {
                    self.simulated_zoom = 1.0 + phase_time * 0.5; // Zoom in
                } else if phase_time < 2.0 {
                    self.simulated_zoom = 1.5 - (phase_time - 1.0) * 0.3; // Zoom out
                }
            }
            TestPhase::TestMiddleMousePan => {
                // Simulate middle mouse button drag
                if phase_time < 2.0 {
                    self.simulated_mouse_button = Some(egui::PointerButton::Middle);
                    // Mouse movement will be handled by response.dragged_by
                }
            }
            TestPhase::TestShiftDrag => {
                // Simulate shift + primary button drag
                if phase_time < 2.0 {
                    self.simulated_mouse_button = Some(egui::PointerButton::Primary);
                    self.simulated_modifiers.shift = true;
                }
            }
            TestPhase::TestScrollWheel => {
                // Simulate traditional scroll wheel
                if phase_time < 2.0 {
                    self.simulated_scroll.y = phase_time.sin() * 20.0;
                }
            }
            TestPhase::TestPanOverTasks => {
                // Simulate panning that goes over tasks
                // We'll use trackpad pan and move over where tasks are positioned
                if phase_time < 3.0 {
                    // Pan diagonally across the task grid
                    // Tasks are at positions like (0,0), (150,0), (300,0), etc.
                    // We want to pan across them to see if panning stops
                    self.simulated_scroll = egui::Vec2::new(
                        20.0, // Steady rightward pan
                        15.0, // Steady downward pan
                    );

                    // Also simulate middle mouse drag as backup
                    if phase_time > 1.5 {
                        self.simulated_mouse_button = Some(egui::PointerButton::Middle);
                    }
                }
            }
            _ => {}
        }

        // Apply simulated input to context
        ctx.input_mut(|input| {
            // Apply scroll delta (trackpad pan or scroll wheel)
            if self.simulated_scroll.length() > 0.01 {
                // Note: We can't directly set scroll_delta in tests, but we document what should happen
            }

            // Apply zoom delta (pinch gesture)
            if (self.simulated_zoom - 1.0).abs() > 0.01 {
                // Note: We can't directly set zoom_delta in tests, but we document what should happen
            }
        });
    }

    fn record_state(&mut self) {
        let timestamp = self.start_time.elapsed().as_secs_f64();
        let camera_pos = self.map_view.get_camera_position();
        let zoom = self.map_view.get_zoom_level();

        // Record camera position
        self.camera_history
            .push_back((timestamp, camera_pos, format!("{:?}", self.test_phase)));

        // Record zoom level
        self.zoom_history
            .push_back((timestamp, zoom, format!("{:?}", self.test_phase)));

        // Keep history size manageable
        if self.camera_history.len() > 1000 {
            self.camera_history.pop_front();
        }
        if self.zoom_history.len() > 1000 {
            self.zoom_history.pop_front();
        }
    }

    fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&"=".repeat(70));
        report.push_str("\n🗺️  MAP VIEW PANNING TEST REPORT\n");
        report.push_str(&"=".repeat(70));
        report.push_str("\n\n");

        // Overall status
        let all_passed = self.test_results.iter().all(|r| r.passed);
        if all_passed {
            report.push_str("✅ ALL TESTS PASSED!\n\n");
        } else {
            report.push_str("❌ SOME TESTS FAILED!\n\n");
        }

        // Individual test results
        report.push_str("Test Results:\n");
        report.push_str("-".repeat(50).as_str());
        report.push_str("\n");

        for result in &self.test_results {
            report.push_str(&format!("\n{:?}:\n", result.phase));
            report.push_str(&format!("  {}\n", result.message));
            report.push_str(&format!(
                "  Camera: ({:.1}, {:.1}) → ({:.1}, {:.1})\n",
                result.camera_start.x,
                result.camera_start.y,
                result.camera_end.x,
                result.camera_end.y
            ));
            report.push_str(&format!(
                "  Zoom: {:.2} → {:.2}\n",
                result.zoom_start, result.zoom_end
            ));
        }

        report.push_str("\n");
        report.push_str(&"=".repeat(70));
        report.push_str("\n");

        report
    }
}

impl eframe::App for MapPanningTest {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update test phase based on time
        let phase_duration = Duration::from_secs(3);
        if self.phase_start_time.elapsed() > phase_duration {
            self.advance_phase();
        }

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            // Test header
            ui.heading("🗺️ Map View Panning Test");
            ui.separator();

            // Current test status
            ui.horizontal(|ui| {
                ui.label("Current Phase:");
                ui.colored_label(
                    egui::Color32::from_rgb(100, 200, 255),
                    format!("{:?}", self.test_phase),
                );
                ui.separator();
                ui.label(format!(
                    "Time: {:.1}s",
                    self.phase_start_time.elapsed().as_secs_f32()
                ));
            });

            // Instructions for current phase
            let instruction = match self.test_phase {
                TestPhase::Setup => "Initializing test environment...",
                TestPhase::TestTrackpadPan => "Testing two-finger trackpad pan (Mac gesture)...",
                TestPhase::TestPinchZoom => "Testing pinch-to-zoom gesture...",
                TestPhase::TestMiddleMousePan => "Testing middle mouse button pan...",
                TestPhase::TestShiftDrag => "Testing Shift+drag pan...",
                TestPhase::TestScrollWheel => "Testing scroll wheel behavior...",
                TestPhase::TestPanOverTasks => {
                    "Testing panning when hovering over tasks (checking for bug)..."
                }
                TestPhase::Complete => "Test complete! See results below.",
            };
            ui.label(instruction);
            ui.separator();

            // Map view with overlay info
            ui.group(|ui| {
                // Record state before rendering
                self.record_state();

                // Simulate input for current phase
                self.simulate_input_for_phase(ctx);

                // Render the map view
                self.map_view.show(ui, &mut self.tasks, &mut self.goals);
            });

            // Real-time stats
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Camera:");
                let pos = self.map_view.get_camera_position();
                ui.monospace(format!("({:.1}, {:.1})", pos.x, pos.y));
                ui.separator();
                ui.label("Zoom:");
                ui.monospace(format!("{:.2}x", self.map_view.get_zoom_level()));
                ui.separator();
                ui.label("Panning:");
                ui.colored_label(
                    if self.map_view.is_panning() {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::GRAY
                    },
                    if self.map_view.is_panning() {
                        "Yes"
                    } else {
                        "No"
                    },
                );
            });

            // Test results (if complete)
            if self.test_phase == TestPhase::Complete {
                ui.separator();
                ui.heading("Test Results:");

                for result in &self.test_results {
                    ui.horizontal(|ui| {
                        let color = if result.passed {
                            egui::Color32::from_rgb(0, 200, 0)
                        } else {
                            egui::Color32::from_rgb(200, 0, 0)
                        };
                        ui.colored_label(color, &result.message);
                    });
                }

                // Print report to console
                if self.test_results.len() == 6 {
                    // Only print once (now 6 tests)
                    println!("\n{}", self.generate_report());

                    // Exit with appropriate code
                    let all_passed = self.test_results.iter().all(|r| r.passed);
                    std::process::exit(if all_passed { 0 } else { 1 });
                }
            }
        });

        // Keep the test running
        ctx.request_repaint();
    }
}

fn main() {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Map Panning Test - Mac Trackpad Support")
            .with_position([100.0, 100.0]),
        ..Default::default()
    };

    println!("\n🚀 Starting Map View Panning Test");
    println!("This test validates all panning methods, especially Mac trackpad support.");
    println!("The test will run automatically through different input methods.");
    println!("Duration: ~21 seconds\n");
    println!("⚠️  Testing for BUG: Panning stops when hovering over tasks/goals");

    let app = MapPanningTest::new();

    let _ = eframe::run_native("Map Panning Test", options, Box::new(|_cc| Box::new(app)));
}
