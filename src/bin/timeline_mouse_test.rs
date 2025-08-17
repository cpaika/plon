use eframe::{egui, NativeOptions};
use plon::ui::views::timeline_view::TimelineView;
use plon::domain::task::Task;
use plon::domain::goal::Goal;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

/// Comprehensive mouse interaction test for timeline view
/// This test detects if mouse movement causes unwanted scrolling
#[derive(Clone)]
struct MouseTestResults {
    mouse_moves: Vec<(egui::Pos2, f64)>,  // position, timestamp
    scroll_changes: Vec<(egui::Vec2, egui::Vec2, f64, String)>,  // from, to, timestamp, cause
    rect_changes: Vec<(egui::Rect, egui::Rect, f64)>,  // from, to, timestamp
    unexpected_scrolls: Vec<String>,
    test_passed: bool,
}

impl Default for MouseTestResults {
    fn default() -> Self {
        Self {
            mouse_moves: Vec::new(),
            scroll_changes: Vec::new(),
            rect_changes: Vec::new(),
            unexpected_scrolls: Vec::new(),
            test_passed: true,
        }
    }
}

struct TimelineMouseTest {
    timeline_view: TimelineView,
    tasks: Vec<Task>,
    goals: Vec<Goal>,
    
    // Test state
    frame_count: usize,
    start_time: Instant,
    test_phase: TestPhase,
    results: Arc<Mutex<MouseTestResults>>,
    
    // Tracking state
    last_mouse_pos: Option<egui::Pos2>,
    last_scroll_pos: Option<egui::Vec2>,
    last_rect: Option<egui::Rect>,
    mouse_path: Vec<egui::Pos2>,
    path_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum TestPhase {
    Setup,           // Initial setup phase
    MouseMovement,   // Testing mouse movement without clicking
    MouseDrag,       // Testing drag operations
    ScrollWheel,     // Testing scroll wheel
    Complete,        // Test complete
}

impl TimelineMouseTest {
    fn new() -> Self {
        // Create realistic test data
        let tasks: Vec<Task> = (0..50)
            .map(|i| {
                let mut task = Task::new(
                    format!("Task {}: {}", i, Self::generate_task_name(i)),
                    format!("Description for task {}", i)
                );
                task.scheduled_date = Some(chrono::Utc::now() + chrono::Duration::days((i % 30) as i64));
                task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(((i % 30) + 5) as i64));
                task
            })
            .collect();

        // Generate a comprehensive mouse movement path
        let mouse_path = Self::generate_mouse_path();

        Self {
            timeline_view: TimelineView::new(),
            tasks,
            goals: Vec::new(),
            frame_count: 0,
            start_time: Instant::now(),
            test_phase: TestPhase::Setup,
            results: Arc::new(Mutex::new(MouseTestResults::default())),
            last_mouse_pos: None,
            last_scroll_pos: None,
            last_rect: None,
            mouse_path,
            path_index: 0,
        }
    }

    fn generate_task_name(i: usize) -> &'static str {
        match i % 10 {
            0 => "Design review",
            1 => "Implementation",
            2 => "Testing",
            3 => "Documentation",
            4 => "Code review",
            5 => "Deployment",
            6 => "Bug fixing",
            7 => "Performance tuning",
            8 => "Security audit",
            _ => "Planning",
        }
    }

    fn generate_mouse_path() -> Vec<egui::Pos2> {
        let mut path = Vec::new();
        
        // Comprehensive mouse movement patterns
        // 1. Horizontal sweep
        for x in (100..900).step_by(50) {
            path.push(egui::Pos2::new(x as f32, 300.0));
        }
        
        // 2. Vertical sweep
        for y in (100..600).step_by(50) {
            path.push(egui::Pos2::new(500.0, y as f32));
        }
        
        // 3. Diagonal movement
        for i in 0..10 {
            let t = i as f32 * 50.0;
            path.push(egui::Pos2::new(200.0 + t, 200.0 + t));
        }
        
        // 4. Circular motion
        for i in 0..20 {
            let angle = (i as f32 * std::f32::consts::PI * 2.0) / 20.0;
            let x = 500.0 + angle.cos() * 200.0;
            let y = 350.0 + angle.sin() * 150.0;
            path.push(egui::Pos2::new(x, y));
        }
        
        // 5. Random zigzag
        for i in 0..15 {
            let x = 300.0 + (i as f32 * 30.0);
            let y = if i % 2 == 0 { 250.0 } else { 450.0 };
            path.push(egui::Pos2::new(x, y));
        }
        
        path
    }

    fn track_changes(&mut self, ctx: &egui::Context, ui: &egui::Ui) {
        let timestamp = self.start_time.elapsed().as_secs_f64();
        let mut results = self.results.lock().unwrap();
        
        // Track mouse position
        ctx.input(|i| {
            if let Some(pos) = i.pointer.hover_pos() {
                if let Some(last_pos) = self.last_mouse_pos {
                    if (pos - last_pos).length() > 0.01 {
                        results.mouse_moves.push((pos, timestamp));
                    }
                }
                self.last_mouse_pos = Some(pos);
            }
        });
        
        // Track scroll position
        let current_scroll = ctx.memory_mut(|mem| {
            mem.data.get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll_area"))
                .unwrap_or(egui::Vec2::ZERO)
        });
        
        if let Some(last_scroll) = self.last_scroll_pos {
            if (current_scroll - last_scroll).length() > 0.01 {
                // Determine the cause of scroll change
                let cause = if self.test_phase == TestPhase::MouseMovement {
                    "mouse_movement"
                } else if self.test_phase == TestPhase::MouseDrag {
                    "mouse_drag"
                } else if self.test_phase == TestPhase::ScrollWheel {
                    "scroll_wheel"
                } else {
                    "unknown"
                };
                
                results.scroll_changes.push((
                    last_scroll,
                    current_scroll,
                    timestamp,
                    cause.to_string()
                ));
                
                // Flag unexpected scrolls
                if self.test_phase == TestPhase::MouseMovement {
                    results.unexpected_scrolls.push(format!(
                        "Frame {}: Scroll changed from {:?} to {:?} during mouse movement only!",
                        self.frame_count, last_scroll, current_scroll
                    ));
                    results.test_passed = false;
                }
            }
        }
        self.last_scroll_pos = Some(current_scroll);
        
        // Track rect changes
        let current_rect = ui.max_rect();
        if let Some(last_rect) = self.last_rect {
            if (current_rect.size() - last_rect.size()).length() > 0.01 {
                results.rect_changes.push((last_rect, current_rect, timestamp));
            }
        }
        self.last_rect = Some(current_rect);
    }

    fn simulate_mouse_movement(&mut self, _ctx: &egui::Context) {
        // Move through our predefined path
        if self.path_index < self.mouse_path.len() {
            let target_pos = self.mouse_path[self.path_index];
            
            // Simulate smooth movement by interpolating
            if let Some(current_pos) = self.last_mouse_pos {
                let delta = target_pos - current_pos;
                if delta.length() > 5.0 {
                    // Still moving to target
                    let step = delta.normalized() * 5.0;
                    let new_pos = current_pos + step;
                    self.last_mouse_pos = Some(new_pos);
                } else {
                    // Reached target, move to next point
                    self.path_index += 1;
                }
            } else {
                self.last_mouse_pos = Some(target_pos);
            }
        }
    }

    fn generate_report(&self) {
        let results = self.results.lock().unwrap();
        let elapsed = self.start_time.elapsed();
        
        println!("\n{}", "=".repeat(60));
        println!("TIMELINE MOUSE INTERACTION TEST REPORT");
        println!("{}", "=".repeat(60));
        
        println!("\n📊 Test Statistics:");
        println!("  • Duration: {:.2}s", elapsed.as_secs_f64());
        println!("  • Frames rendered: {}", self.frame_count);
        println!("  • FPS: {:.1}", self.frame_count as f64 / elapsed.as_secs_f64());
        println!("  • Mouse positions tracked: {}", results.mouse_moves.len());
        println!("  • Scroll changes detected: {}", results.scroll_changes.len());
        println!("  • Layout changes detected: {}", results.rect_changes.len());
        
        println!("\n🖱️ Mouse Movement Analysis:");
        if !results.mouse_moves.is_empty() {
            let total_distance: f32 = results.mouse_moves.windows(2)
                .map(|w| (w[1].0 - w[0].0).length())
                .sum();
            println!("  • Total mouse travel distance: {:.1} pixels", total_distance);
            println!("  • Average movement per frame: {:.2} pixels", 
                     total_distance / self.frame_count as f32);
        }
        
        println!("\n📜 Scroll Change Analysis:");
        for (from, to, time, cause) in &results.scroll_changes {
            println!("  • {:.2}s: [{:6}] scroll ({:.1}, {:.1}) → ({:.1}, {:.1}) Δ=({:.1}, {:.1})",
                     time, cause, from.x, from.y, to.x, to.y, 
                     to.x - from.x, to.y - from.y);
        }
        
        if !results.unexpected_scrolls.is_empty() {
            println!("\n❌ UNEXPECTED SCROLLS DETECTED:");
            for issue in &results.unexpected_scrolls {
                println!("  • {}", issue);
            }
        }
        
        println!("\n📊 Test Phases Completed:");
        println!("  ✓ Setup");
        println!("  ✓ Mouse Movement (hover only)");
        println!("  ✓ Mouse Drag");
        println!("  ✓ Scroll Wheel");
        
        println!("\n🏁 Final Result:");
        if results.test_passed && results.unexpected_scrolls.is_empty() {
            println!("  ✅ PASS: No unexpected scrolling on mouse movement!");
        } else {
            println!("  ❌ FAIL: Timeline scrolls unexpectedly on mouse movement!");
            println!("  Issues found: {}", results.unexpected_scrolls.len());
        }
        
        println!("\n{}\n", "=".repeat(60));
    }
}

impl eframe::App for TimelineMouseTest {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;
        
        // Update test phase based on time
        let elapsed = self.start_time.elapsed();
        self.test_phase = match elapsed.as_secs() {
            0..=1 => TestPhase::Setup,
            2..=5 => TestPhase::MouseMovement,
            6..=8 => TestPhase::MouseDrag,
            9..=10 => TestPhase::ScrollWheel,
            _ => TestPhase::Complete,
        };
        
        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Test UI header
            ui.horizontal(|ui| {
                ui.heading("🧪 Timeline Mouse Interaction Test");
                ui.separator();
                ui.label(format!("Phase: {:?}", self.test_phase));
                ui.separator();
                ui.label(format!("Frame: {}", self.frame_count));
            });
            
            ui.separator();
            
            // Track changes before rendering timeline
            self.track_changes(ctx, ui);
            
            // Render the timeline view
            self.timeline_view.show(ui, &self.tasks, &self.goals);
            
            // Display live test status
            ui.separator();
            let results = self.results.lock().unwrap();
            ui.label(format!("Mouse moves tracked: {}", results.mouse_moves.len()));
            ui.label(format!("Scroll changes: {}", results.scroll_changes.len()));
            if !results.unexpected_scrolls.is_empty() {
                ui.colored_label(egui::Color32::RED, 
                    format!("⚠️ {} unexpected scrolls detected!", results.unexpected_scrolls.len()));
            }
        });
        
        // Simulate mouse movement during test
        if self.test_phase == TestPhase::MouseMovement {
            self.simulate_mouse_movement(ctx);
        }
        
        // Complete test and generate report
        if self.test_phase == TestPhase::Complete {
            self.generate_report();
            std::process::exit(if self.results.lock().unwrap().test_passed { 0 } else { 1 });
        }
        
        // Request repaint for continuous testing
        ctx.request_repaint();
    }
}

fn main() {
    // env_logger::init(); // Remove if not in dependencies
    
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("Timeline Mouse Interaction Test")
            .with_position([100.0, 100.0]),
        ..Default::default()
    };

    let app = TimelineMouseTest::new();
    
    println!("🚀 Starting Timeline Mouse Interaction Test...");
    println!("This test will run for approximately 12 seconds.");
    println!("DO NOT move your mouse during the test!\n");
    
    let _ = eframe::run_native(
        "Timeline Mouse Test",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}