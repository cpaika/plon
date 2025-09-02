use image::{DynamicImage, GenericImageView, Rgba};
use std::fs;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

/// Visual test that detects infinite scrolling by analyzing actual screenshots
#[test]
fn test_timeline_visual_scroll_detection() {
    println!("Starting visual scroll detection test...");
    println!("This test will:");
    println!("1. Start the application");
    println!("2. Navigate to Timeline view");
    println!("3. Capture screenshots every 200ms");
    println!("4. Compare pixels to detect autonomous scrolling");

    // Start the application
    let mut app = start_application();

    // Wait for app to start
    thread::sleep(Duration::from_secs(3));

    // Click on Timeline view using AppleScript
    click_timeline_view();
    thread::sleep(Duration::from_millis(500));

    // Capture and analyze screenshots
    let result = capture_and_analyze_scrolling();

    // Kill the app
    app.kill().ok();

    // Assert based on results
    assert!(
        !result.is_scrolling,
        "INFINITE SCROLLING DETECTED!\n\
         Motion detected: {:.2}% of pixels changed on average\n\
         Consistent direction: {}\n\
         Frames analyzed: {}\n\
         The timeline view is scrolling autonomously!",
        result.average_motion * 100.0,
        result.consistent_direction,
        result.frames_analyzed
    );
}

struct ScrollAnalysisResult {
    is_scrolling: bool,
    average_motion: f32,
    consistent_direction: bool,
    frames_analyzed: usize,
}

fn start_application() -> Child {
    Command::new("cargo")
        .args(&["run", "--release"])
        .spawn()
        .expect("Failed to start application")
}

fn click_timeline_view() {
    // Use AppleScript to click on the Timeline button
    let script = r#"
    tell application "System Events"
        tell process "plon"
            set frontmost to true
            delay 0.5
            -- Look for and click the Timeline button
            click button "📅 Timeline" of window 1
        end tell
    end tell
    "#;

    Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok();
}

fn capture_and_analyze_scrolling() -> ScrollAnalysisResult {
    let screenshot_dir = "/tmp/plon_visual_test";
    fs::create_dir_all(screenshot_dir).unwrap();

    let mut screenshots = Vec::new();
    let start_time = Instant::now();

    // Capture screenshots for 3 seconds
    while start_time.elapsed() < Duration::from_secs(3) {
        let screenshot_path = format!("{}/frame_{}.png", screenshot_dir, screenshots.len());

        // Capture screenshot of the app window
        capture_app_window(&screenshot_path);

        if std::path::Path::new(&screenshot_path).exists() {
            screenshots.push(screenshot_path);
        }

        thread::sleep(Duration::from_millis(200));
    }

    println!("Captured {} screenshots", screenshots.len());

    // Analyze screenshots for motion
    let analysis = analyze_screenshot_sequence(&screenshots);

    // Clean up
    for path in &screenshots {
        fs::remove_file(path).ok();
    }

    analysis
}

fn capture_app_window(output_path: &str) {
    // Use screencapture to capture the main display
    // -x: no sounds
    // -m: capture main monitor only
    let result = Command::new("screencapture")
        .args(&["-x", "-m", output_path])
        .output()
        .expect("Failed to run screencapture");

    if !result.status.success() {
        println!(
            "Screenshot failed: {:?}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

fn analyze_screenshot_sequence(screenshots: &[String]) -> ScrollAnalysisResult {
    if screenshots.len() < 3 {
        return ScrollAnalysisResult {
            is_scrolling: false,
            average_motion: 0.0,
            consistent_direction: false,
            frames_analyzed: screenshots.len(),
        };
    }

    let mut motion_scores = Vec::new();
    let mut vertical_movements = Vec::new();
    let mut horizontal_movements = Vec::new();

    // Compare consecutive frames
    for i in 1..screenshots.len() {
        if let Ok(motion) = detect_motion_between_frames(&screenshots[i - 1], &screenshots[i]) {
            motion_scores.push(motion.pixel_change_ratio);
            vertical_movements.push(motion.vertical_shift);
            horizontal_movements.push(motion.horizontal_shift);

            println!(
                "Frame {}->{}: {:.2}% pixels changed, v_shift={}, h_shift={}",
                i - 1,
                i,
                motion.pixel_change_ratio * 100.0,
                motion.vertical_shift,
                motion.horizontal_shift
            );
        }
    }

    if motion_scores.is_empty() {
        return ScrollAnalysisResult {
            is_scrolling: false,
            average_motion: 0.0,
            consistent_direction: false,
            frames_analyzed: screenshots.len(),
        };
    }

    // Calculate average motion
    let average_motion = motion_scores.iter().sum::<f32>() / motion_scores.len() as f32;

    // Check for consistent directional movement (indicating scrolling)
    let vertical_consistency = check_movement_consistency(&vertical_movements);
    let horizontal_consistency = check_movement_consistency(&horizontal_movements);
    let consistent_direction = vertical_consistency || horizontal_consistency;

    // Detect infinite scrolling:
    // 1. Significant motion in most frames (>5% pixel change)
    // 2. Consistent direction of movement
    // 3. Continuous motion (not just initial render)
    let significant_motion_frames = motion_scores.iter().filter(|&&m| m > 0.05).count();
    let is_scrolling = significant_motion_frames >= motion_scores.len() / 2
        && consistent_direction
        && average_motion > 0.03;

    ScrollAnalysisResult {
        is_scrolling,
        average_motion,
        consistent_direction,
        frames_analyzed: screenshots.len(),
    }
}

struct MotionData {
    pixel_change_ratio: f32,
    vertical_shift: i32,
    horizontal_shift: i32,
}

fn detect_motion_between_frames(
    path1: &str,
    path2: &str,
) -> Result<MotionData, Box<dyn std::error::Error>> {
    // Load images
    let img1 = image::open(path1)?;
    let img2 = image::open(path2)?;

    // Ensure same dimensions
    if img1.dimensions() != img2.dimensions() {
        return Err("Images have different dimensions".into());
    }

    let (width, height) = img1.dimensions();
    let total_pixels = (width * height) as f32;

    // Convert to RGBA8
    let img1_rgba = img1.to_rgba8();
    let img2_rgba = img2.to_rgba8();

    // Count changed pixels and detect shift patterns
    let mut changed_pixels = 0u32;
    let mut shift_samples = Vec::new();

    // Sample points for shift detection (grid of 10x10)
    let sample_step_x = width / 10;
    let sample_step_y = height / 10;

    for y in (0..height).step_by(sample_step_y as usize) {
        for x in (0..width).step_by(sample_step_x as usize) {
            let pixel1 = img1_rgba.get_pixel(x, y);

            // Try to find this pixel in nearby locations in img2
            if let Some((dx, dy)) = find_pixel_shift(&img1_rgba, &img2_rgba, x, y, 20) {
                shift_samples.push((dx, dy));
            }
        }
    }

    // Count all changed pixels
    for y in 0..height {
        for x in 0..width {
            let pixel1 = img1_rgba.get_pixel(x, y);
            let pixel2 = img2_rgba.get_pixel(x, y);

            if pixels_differ(pixel1, pixel2) {
                changed_pixels += 1;
            }
        }
    }

    // Calculate average shift
    let (avg_h_shift, avg_v_shift) = if !shift_samples.is_empty() {
        let sum_h: i32 = shift_samples.iter().map(|(h, _)| h).sum();
        let sum_v: i32 = shift_samples.iter().map(|(_, v)| v).sum();
        (
            sum_h / shift_samples.len() as i32,
            sum_v / shift_samples.len() as i32,
        )
    } else {
        (0, 0)
    };

    Ok(MotionData {
        pixel_change_ratio: changed_pixels as f32 / total_pixels,
        vertical_shift: avg_v_shift,
        horizontal_shift: avg_h_shift,
    })
}

fn find_pixel_shift(
    img1: &image::RgbaImage,
    img2: &image::RgbaImage,
    x: u32,
    y: u32,
    search_radius: u32,
) -> Option<(i32, i32)> {
    let pixel1 = img1.get_pixel(x, y);
    let (width, height) = img1.dimensions();

    // Search in a spiral pattern for matching pixel
    for radius in 1..=search_radius {
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                // Only check perimeter of current radius
                if dx.abs() != radius as i32 && dy.abs() != radius as i32 {
                    continue;
                }

                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;

                if new_x >= 0 && new_x < width as i32 && new_y >= 0 && new_y < height as i32 {
                    let pixel2 = img2.get_pixel(new_x as u32, new_y as u32);
                    if !pixels_differ(pixel1, pixel2) {
                        return Some((dx, dy));
                    }
                }
            }
        }
    }
    None
}

fn pixels_differ(p1: &Rgba<u8>, p2: &Rgba<u8>) -> bool {
    // Allow small differences for compression artifacts
    const THRESHOLD: i32 = 10;

    let dr = (p1[0] as i32 - p2[0] as i32).abs();
    let dg = (p1[1] as i32 - p2[1] as i32).abs();
    let db = (p1[2] as i32 - p2[2] as i32).abs();

    dr > THRESHOLD || dg > THRESHOLD || db > THRESHOLD
}

fn check_movement_consistency(movements: &[i32]) -> bool {
    if movements.len() < 2 {
        return false;
    }

    // Check if most movements are in the same direction
    let positive = movements.iter().filter(|&&m| m > 0).count();
    let negative = movements.iter().filter(|&&m| m < 0).count();

    let majority = movements.len() * 2 / 3;
    positive >= majority || negative >= majority
}
