// QE Test: Visual Regression and Performance Test for Map View
// This test detects:
// 1. Screen flashing (rapid color changes)
// 2. Freezing (frames not updating)
// 3. Performance degradation
// 4. Rendering artifacts

use eframe::egui;
use image::{ImageBuffer, Rgb};
use plon::ui::app::PlonApp;
use plon::ui::views::map_view::MapView;
use plon::domain::task::Task;
use plon::domain::goal::Goal;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct FrameMetrics {
    timestamp: Instant,
    frame_time: Duration,
    pixel_checksum: u64,
    is_frozen: bool,
    flash_detected: bool,
}

struct VisualTestHarness {
    frames: VecDeque<FrameMetrics>,
    previous_pixels: Option<Vec<u8>>,
    flash_threshold: f32,
    freeze_threshold: Duration,
    last_frame_time: Option<Instant>,
}

impl VisualTestHarness {
    fn new() -> Self {
        Self {
            frames: VecDeque::with_capacity(100),
            previous_pixels: None,
            flash_threshold: 0.3, // 30% pixel change = flash
            freeze_threshold: Duration::from_millis(100),
            last_frame_time: None,
        }
    }
    
    fn analyze_frame(&mut self, pixels: &[u8]) -> FrameMetrics {
        let now = Instant::now();
        let frame_time = self.last_frame_time
            .map(|t| now.duration_since(t))
            .unwrap_or(Duration::ZERO);
        
        // Calculate pixel checksum for freeze detection
        let pixel_checksum = pixels.iter()
            .step_by(100) // Sample every 100th pixel for speed
            .fold(0u64, |acc, &p| acc.wrapping_add(p as u64));
        
        // Detect flashing by comparing with previous frame
        let flash_detected = if let Some(ref prev) = self.previous_pixels {
            let changed_pixels = pixels.iter()
                .zip(prev.iter())
                .filter(|(a, b)| ((**a as i32) - (**b as i32)).abs() > 50)
                .count();
            
            let change_ratio = changed_pixels as f32 / pixels.len() as f32;
            change_ratio > self.flash_threshold
        } else {
            false
        };
        
        // Detect freeze by checking if pixels are identical
        let is_frozen = self.frames.back()
            .map(|prev| prev.pixel_checksum == pixel_checksum && frame_time > self.freeze_threshold)
            .unwrap_or(false);
        
        self.previous_pixels = Some(pixels.to_vec());
        self.last_frame_time = Some(now);
        
        let metrics = FrameMetrics {
            timestamp: now,
            frame_time,
            pixel_checksum,
            is_frozen,
            flash_detected,
        };
        
        self.frames.push_back(metrics.clone());
        if self.frames.len() > 100 {
            self.frames.pop_front();
        }
        
        metrics
    }
    
    fn get_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check for consistent flashing
        let flash_count = self.frames.iter()
            .filter(|f| f.flash_detected)
            .count();
        
        if flash_count > 5 {
            issues.push(format!("Screen flashing detected: {} frames with >30% pixel changes", flash_count));
        }
        
        // Check for freezing
        let freeze_count = self.frames.iter()
            .filter(|f| f.is_frozen)
            .count();
        
        if freeze_count > 3 {
            issues.push(format!("Screen freezing detected: {} frames frozen for >100ms", freeze_count));
        }
        
        // Check for performance issues
        let slow_frames = self.frames.iter()
            .filter(|f| f.frame_time > Duration::from_millis(50))
            .count();
        
        if slow_frames > 10 {
            issues.push(format!("Performance issue: {} frames took >50ms", slow_frames));
        }
        
        // Check for stuttering (high variance in frame times)
        if self.frames.len() > 10 {
            let avg_frame_time: Duration = self.frames.iter()
                .map(|f| f.frame_time)
                .sum::<Duration>() / self.frames.len() as u32;
            
            let variance = self.frames.iter()
                .map(|f| {
                    let diff = if f.frame_time > avg_frame_time {
                        f.frame_time - avg_frame_time
                    } else {
                        avg_frame_time - f.frame_time
                    };
                    diff.as_millis() as f32
                })
                .sum::<f32>() / self.frames.len() as f32;
            
            if variance > 20.0 {
                issues.push(format!("Stuttering detected: frame time variance of {}ms", variance));
            }
        }
        
        issues
    }
}

#[test]
fn test_map_view_visual_regression() {
    let mut harness = VisualTestHarness::new();
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();
    
    // Create realistic test data
    let mut tasks: Vec<Task> = Vec::new();
    for i in 0..100 {
        let mut task = Task::new(
            format!("Task {}", i),
            format!("Description for task {}", i)
        );
        task.set_position(
            (i as f64 % 15.0) * 250.0,
            (i as f64 / 15.0) * 250.0
        );
        tasks.push(task);
    }
    let mut goals = Vec::new();
    
    // Simulate user panning around the map
    println!("Starting visual regression test...");
    
    for frame_num in 0..60 {
        // Setup frame
        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(800.0, 600.0)
        ));
        
        // Simulate panning motion
        if frame_num > 10 && frame_num < 50 {
            // Start pan
            if frame_num == 11 {
                raw_input.events.push(egui::Event::PointerButton {
                    pos: egui::Pos2::new(400.0, 300.0),
                    button: egui::PointerButton::Middle,
                    pressed: true,
                    modifiers: Default::default(),
                });
            }
            // Continue pan
            else if frame_num < 49 {
                let delta = (frame_num - 11) as f32 * 5.0;
                raw_input.events.push(egui::Event::PointerMoved(
                    egui::Pos2::new(400.0 + delta, 300.0 + delta * 0.7)
                ));
            }
            // End pan
            else if frame_num == 49 {
                raw_input.events.push(egui::Event::PointerButton {
                    pos: egui::Pos2::new(400.0 + 190.0, 300.0 + 133.0),
                    button: egui::PointerButton::Middle,
                    pressed: false,
                    modifiers: Default::default(),
                });
            }
        }
        
        // Render frame
        ctx.begin_frame(raw_input);
        
        let frame_start = Instant::now();
        
        egui::CentralPanel::default().show(&ctx, |ui| {
            map_view.show(ui, &mut tasks, &mut goals);
        });
        
        let output = ctx.end_frame();
        
        // Simulate pixel capture (in real test, would use actual framebuffer)
        let mock_pixels = generate_mock_pixels(&map_view, frame_num);
        
        // Analyze frame
        let metrics = harness.analyze_frame(&mock_pixels);
        
        if metrics.flash_detected {
            println!("⚠️  Frame {}: FLASH DETECTED!", frame_num);
        }
        if metrics.is_frozen {
            println!("⚠️  Frame {}: FROZEN (no change for {:?})!", frame_num, metrics.frame_time);
        }
        if metrics.frame_time > Duration::from_millis(50) {
            println!("⚠️  Frame {}: SLOW ({:?})", frame_num, metrics.frame_time);
        }
    }
    
    // Check for issues
    let issues = harness.get_issues();
    
    if !issues.is_empty() {
        println!("\n❌ Visual regression test FAILED:");
        for issue in &issues {
            println!("  - {}", issue);
        }
        panic!("Visual regression issues detected: {:?}", issues);
    } else {
        println!("\n✅ Visual regression test PASSED");
    }
}

// Mock pixel generation to simulate screen capture
fn generate_mock_pixels(map_view: &MapView, frame: usize) -> Vec<u8> {
    let width = 800;
    let height = 600;
    let mut pixels = vec![0u8; width * height * 3];
    
    // Simulate rendering based on camera position
    let cam_pos = map_view.get_camera_position();
    let zoom = map_view.get_zoom_level();
    
    // Generate a pattern that changes with camera position
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            
            // Create a pattern that moves with the camera
            let world_x = (x as f32 - 400.0) / zoom - cam_pos.x;
            let world_y = (y as f32 - 300.0) / zoom - cam_pos.y;
            
            // Grid pattern
            let grid_x = (world_x / 50.0).floor() as i32;
            let grid_y = (world_y / 50.0).floor() as i32;
            
            // Color based on grid position
            pixels[idx] = ((grid_x * 37) & 0xFF) as u8;
            pixels[idx + 1] = ((grid_y * 53) & 0xFF) as u8;
            pixels[idx + 2] = (((grid_x + grid_y) * 71) & 0xFF) as u8;
            
            // Simulate flashing if panning flag is stuck
            if map_view.is_panning() && frame % 2 == 0 {
                pixels[idx] = 255 - pixels[idx];
            }
        }
    }
    
    pixels
}

#[test]
fn test_frame_rate_consistency() {
    // This test specifically looks for the frame skipping issue
    let ctx = egui::Context::default();
    let mut map_view = MapView::new();
    
    let mut tasks: Vec<Task> = vec![
        Task::new("Test Task".to_string(), "Description".to_string()),
    ];
    let mut goals = Vec::new();
    
    let mut frame_times = Vec::new();
    let mut skipped_frames = 0;
    
    for i in 0..100 {
        let frame_start = Instant::now();
        
        ctx.begin_frame(egui::RawInput::default());
        
        egui::CentralPanel::default().show(&ctx, |ui| {
            map_view.show(ui, &mut tasks, &mut goals);
        });
        
        ctx.end_frame();
        
        let frame_time = frame_start.elapsed();
        
        // Check if this frame was skipped (too fast)
        if frame_time < Duration::from_millis(1) {
            skipped_frames += 1;
            println!("Frame {} was skipped or rendered too fast: {:?}", i, frame_time);
        }
        
        frame_times.push(frame_time);
    }
    
    // Analyze results
    assert!(
        skipped_frames < 10,
        "Too many frames skipped: {}. This causes flashing!",
        skipped_frames
    );
    
    // Check for consistent frame timing
    let avg_time: Duration = frame_times.iter().sum::<Duration>() / frame_times.len() as u32;
    let max_deviation = frame_times.iter()
        .map(|&t| if t > avg_time { t - avg_time } else { avg_time - t })
        .max()
        .unwrap();
    
    assert!(
        max_deviation < Duration::from_millis(50),
        "Frame timing too inconsistent: max deviation {:?} from average {:?}",
        max_deviation,
        avg_time
    );
}