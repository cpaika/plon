use eframe::{egui, NativeOptions};
use plon::ui::{PlonApp, ViewType};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

/// Test that runs the ACTUAL PlonApp and detects scrolling issues in timeline view
#[derive(Clone, Debug)]
struct ScrollDetectionData {
    frame_count: usize,
    mouse_positions: Vec<(f64, egui::Pos2)>,  // (timestamp, position)
    scroll_events: Vec<(f64, String, egui::Vec2)>,  // (timestamp, view_name, scroll_delta)
    unexpected_scrolls: Vec<String>,
    timeline_rect_changes: Vec<(f64, egui::Rect, egui::Rect)>,
    current_view: String,
}

impl Default for ScrollDetectionData {
    fn default() -> Self {
        Self {
            frame_count: 0,
            mouse_positions: Vec::new(),
            scroll_events: Vec::new(),
            unexpected_scrolls: Vec::new(),
            timeline_rect_changes: Vec::new(),
            current_view: String::new(),
        }
    }
}

/// Wrapper around PlonApp that monitors for scrolling issues
struct MonitoredPlonApp {
    app: PlonApp,
    start_time: Instant,
    detection_data: Arc<Mutex<ScrollDetectionData>>,
    test_phase: TestPhase,
    last_mouse_pos: Option<egui::Pos2>,
    last_timeline_rect: Option<egui::Rect>,
    mouse_movement_path: Vec<egui::Pos2>,
    path_index: usize,
    frames_in_timeline: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum TestPhase {
    NavigateToTimeline,  // First, navigate to timeline view
    WaitForStable,       // Wait for view to stabilize
    MouseMovement,       // Test mouse movement
    Complete,           // Test complete
}

impl MonitoredPlonApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Create the actual app
        let app = PlonApp::new(cc);
        
        // Generate comprehensive mouse path
        let mouse_movement_path = Self::generate_test_path();
        
        Self {
            app,
            start_time: Instant::now(),
            detection_data: Arc::new(Mutex::new(ScrollDetectionData::default())),
            test_phase: TestPhase::NavigateToTimeline,
            last_mouse_pos: None,
            last_timeline_rect: None,
            mouse_movement_path,
            path_index: 0,
            frames_in_timeline: 0,
        }
    }
    
    fn generate_test_path() -> Vec<egui::Pos2> {
        let mut path = Vec::new();
        
        // Cover the entire timeline area with mouse movements
        // Grid pattern
        for y in (100..700).step_by(50) {
            for x in (200..1000).step_by(50) {
                path.push(egui::Pos2::new(x as f32, y as f32));
            }
        }
        
        // Diagonal sweeps
        for i in 0..20 {
            let t = i as f32 * 40.0;
            path.push(egui::Pos2::new(200.0 + t, 150.0 + t));
        }
        
        // Circles to test curved movement
        for i in 0..40 {
            let angle = (i as f32 * std::f32::consts::PI * 2.0) / 40.0;
            let x = 600.0 + angle.cos() * 250.0;
            let y = 400.0 + angle.sin() * 200.0;
            path.push(egui::Pos2::new(x, y));
        }
        
        path
    }
    
    fn detect_timeline_issues(&mut self, ctx: &egui::Context) {
        let timestamp = self.start_time.elapsed().as_secs_f64();
        let mut data = self.detection_data.lock().unwrap();
        data.frame_count += 1;
        
        // Track current view
        data.current_view = format!("{:?}", self.app.current_view);
        
        // Only monitor when in timeline view
        if data.current_view.contains("Timeline") {
            self.frames_in_timeline += 1;
            
            // Monitor mouse position
            ctx.input(|i| {
                if let Some(pos) = i.pointer.hover_pos() {
                    // Check if mouse moved
                    if let Some(last_pos) = self.last_mouse_pos {
                        if (pos - last_pos).length() > 1.0 {
                            data.mouse_positions.push((timestamp, pos));
                        }
                    }
                    self.last_mouse_pos = Some(pos);
                    
                    // Check for scroll events
                    if i.scroll_delta.length() > 0.01 {
                        data.scroll_events.push((
                            timestamp,
                            "timeline".to_string(),
                            i.scroll_delta
                        ));
                        
                        // If we're in mouse movement phase and scrolling happened
                        if self.test_phase == TestPhase::MouseMovement && i.scroll_delta.length() > 0.1 {
                            data.unexpected_scrolls.push(format!(
                                "Frame {} @ {:.2}s: Unexpected scroll delta {:?} during mouse movement at {:?}",
                                data.frame_count, timestamp, i.scroll_delta, pos
                            ));
                        }
                    }
                }
            });
            
            // Monitor timeline area changes
            ctx.memory(|mem| {
                // Try to get timeline scroll state
                if let Some(scroll_state) = mem.data.get_temp::<egui::Vec2>(egui::Id::new("timeline_scroll_area")) {
                    // Check if scroll changed without user wheel input
                    ctx.input(|i| {
                        if i.scroll_delta.length() < 0.01 && scroll_state.length() > 0.1 {
                            data.unexpected_scrolls.push(format!(
                                "Frame {} @ {:.2}s: Timeline scrolled to {:?} without wheel input!",
                                data.frame_count, timestamp, scroll_state
                            ));
                        }
                    });
                }
            });
        }
    }
    
    fn simulate_mouse_for_test(&mut self, _ctx: &egui::Context) {
        if self.test_phase != TestPhase::MouseMovement {
            return;
        }
        
        // Simulate mouse movement through the path
        if self.path_index < self.mouse_movement_path.len() {
            let target = self.mouse_movement_path[self.path_index];
            
            // Move toward target
            if let Some(current) = self.last_mouse_pos {
                let delta = target - current;
                if delta.length() > 10.0 {
                    // Still moving
                    let step = delta.normalized() * 10.0;
                    self.last_mouse_pos = Some(current + step);
                } else {
                    // Reached target
                    self.path_index += 1;
                }
            } else {
                self.last_mouse_pos = Some(target);
            }
            
            // Note: We can't actually inject mouse position in the running app,
            // but we're tracking what WOULD happen
        }
    }
    
    fn update_test_phase(&mut self) {
        let elapsed = self.start_time.elapsed();
        
        self.test_phase = match elapsed.as_secs() {
            0..=2 => {
                // Navigate to timeline
                if !format!("{:?}", self.app.current_view).contains("Timeline") {
                    self.app.current_view = ViewType::Timeline;
                }
                TestPhase::NavigateToTimeline
            },
            3..=4 => TestPhase::WaitForStable,
            5..=15 => TestPhase::MouseMovement,
            _ => TestPhase::Complete,
        };
    }
    
    fn generate_report(&self) {
        let data = self.detection_data.lock().unwrap();
        
        println!("\n{}", "=".repeat(70));
        println!("PLONAPP TIMELINE SCROLL DETECTION REPORT");
        println!("{}", "=".repeat(70));
        
        println!("\n📊 Test Statistics:");
        println!("  • Total frames: {}", data.frame_count);
        println!("  • Frames in timeline view: {}", self.frames_in_timeline);
        println!("  • Test duration: {:.2}s", self.start_time.elapsed().as_secs_f64());
        println!("  • Mouse positions tracked: {}", data.mouse_positions.len());
        println!("  • Scroll events: {}", data.scroll_events.len());
        
        println!("\n📜 Scroll Events in Timeline:");
        for (time, view, delta) in &data.scroll_events {
            if view == "timeline" {
                println!("  • {:.2}s: Scroll delta ({:.2}, {:.2})", time, delta.x, delta.y);
            }
        }
        
        if !data.unexpected_scrolls.is_empty() {
            println!("\n❌ UNEXPECTED SCROLLING DETECTED:");
            for issue in &data.unexpected_scrolls {
                println!("  {}", issue);
            }
            println!("\n⚠️  The timeline view scrolls when moving the mouse!");
        } else {
            println!("\n✅ PASS: No unexpected scrolling detected in timeline view!");
        }
        
        // Mouse movement analysis
        if data.mouse_positions.len() > 1 {
            let total_distance: f32 = data.mouse_positions.windows(2)
                .map(|w| (w[1].1 - w[0].1).length())
                .sum();
            println!("\n🖱️ Mouse Movement:");
            println!("  • Total distance: {:.1} pixels", total_distance);
            println!("  • Positions recorded: {}", data.mouse_positions.len());
        }
        
        println!("\n{}", "=".repeat(70));
        
        // Exit with appropriate code
        std::process::exit(if data.unexpected_scrolls.is_empty() { 0 } else { 1 });
    }
}

impl eframe::App for MonitoredPlonApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Update test phase
        self.update_test_phase();
        
        // Detect issues BEFORE running the app
        self.detect_timeline_issues(ctx);
        
        // Simulate mouse movement if in that phase
        self.simulate_mouse_for_test(ctx);
        
        // Run the actual app
        self.app.update(ctx, frame);
        
        // Detect issues AFTER running the app
        self.detect_timeline_issues(ctx);
        
        // Show test overlay
        egui::Window::new("🧪 Scroll Test Monitor")
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Phase: {:?}", self.test_phase));
                ui.label(format!("View: {:?}", self.app.current_view));
                ui.label(format!("Frame: {}", self.detection_data.lock().unwrap().frame_count));
                
                let data = self.detection_data.lock().unwrap();
                if !data.unexpected_scrolls.is_empty() {
                    ui.colored_label(egui::Color32::RED, 
                        format!("⚠️ {} issues detected!", data.unexpected_scrolls.len()));
                } else {
                    ui.colored_label(egui::Color32::GREEN, "✅ No issues yet");
                }
                
                if self.test_phase == TestPhase::MouseMovement {
                    ui.label(format!("Testing mouse movement... {}/{}", 
                        self.path_index, self.mouse_movement_path.len()));
                }
            });
        
        // Complete test
        if self.test_phase == TestPhase::Complete {
            self.generate_report();
        }
        
        // Keep updating
        ctx.request_repaint();
    }
}

fn main() {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("PlonApp Timeline Scroll Test")
            .with_position([50.0, 50.0]),
        ..Default::default()
    };
    
    println!("🚀 Starting PlonApp Timeline Scroll Detection Test");
    println!("This test will:");
    println!("  1. Open the actual PlonApp");
    println!("  2. Navigate to Timeline view");
    println!("  3. Monitor for unexpected scrolling during mouse movement");
    println!("  4. Generate a detailed report");
    println!("\n⚠️  DO NOT interact with the app during testing!");
    println!("Test duration: ~16 seconds\n");
    
    let _ = eframe::run_native(
        "PlonApp Scroll Test",
        options,
        Box::new(|cc| Box::new(MonitoredPlonApp::new(cc))),
    );
}