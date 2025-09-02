use image::{DynamicImage, GenericImageView, Rgba};
use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Test that actually runs the app and detects infinite scroll in timeline view
/// This test will FAIL if infinite scroll is happening
#[test]
#[ignore] // Use --ignored to run this test since it launches the actual app
fn test_timeline_infinite_scroll_runtime_detection() {
    println!("=== Timeline Infinite Scroll Runtime Detection Test ===");
    println!("This test will:");
    println!("1. Launch the application");
    println!("2. Navigate to Timeline view");
    println!("3. Take screenshots for 5 seconds");
    println!("4. Analyze pixel movement to detect autonomous scrolling");
    println!();

    // Create directory for screenshots
    let screenshot_dir = "/tmp/plon_timeline_scroll_test";
    fs::create_dir_all(screenshot_dir).unwrap();

    // Start the application
    println!("Starting application...");
    let mut app_process = Command::new("cargo")
        .args(&["run", "--release"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start application");

    // Wait for app to fully launch
    thread::sleep(Duration::from_secs(3));

    // Navigate to Timeline view using AppleScript
    println!("Navigating to Timeline view...");
    navigate_to_timeline();
    thread::sleep(Duration::from_millis(1000));

    // Capture initial state
    println!("Starting screenshot capture...");
    let mut screenshots = Vec::new();
    let capture_duration = Duration::from_secs(5);
    let start_time = Instant::now();
    let mut frame_count = 0;

    while start_time.elapsed() < capture_duration {
        let screenshot_path = format!("{}/frame_{:04}.png", screenshot_dir, frame_count);

        // Take screenshot of the app window
        capture_screenshot(&screenshot_path);

        if std::path::Path::new(&screenshot_path).exists() {
            screenshots.push(screenshot_path);
            frame_count += 1;
        }

        // Capture every 200ms (5 fps)
        thread::sleep(Duration::from_millis(200));
    }

    println!("Captured {} screenshots", screenshots.len());

    // Kill the app
    println!("Stopping application...");
    app_process.kill().ok();
    thread::sleep(Duration::from_millis(500));

    // Analyze screenshots for movement
    println!("\nAnalyzing screenshots for autonomous scrolling...");
    let analysis = analyze_for_infinite_scroll(&screenshots);

    // Clean up screenshots
    for path in &screenshots {
        fs::remove_file(path).ok();
    }

    // Report results
    println!("\n=== Analysis Results ===");
    println!("Total frames analyzed: {}", analysis.frames_analyzed);
    println!(
        "Average pixel change: {:.2}%",
        analysis.average_pixel_change * 100.0
    );
    println!(
        "Maximum pixel change: {:.2}%",
        analysis.max_pixel_change * 100.0
    );
    println!(
        "Consistent movement detected: {}",
        analysis.consistent_movement
    );
    println!("Movement direction: {:?}", analysis.movement_direction);
    println!("Scrolling detected: {}", analysis.infinite_scroll_detected);

    // The test FAILS if infinite scroll is detected
    assert!(
        !analysis.infinite_scroll_detected,
        "\n❌ INFINITE SCROLL DETECTED IN TIMELINE VIEW!\n\
        The timeline view is scrolling autonomously without user input.\n\
        Average pixel change: {:.2}%\n\
        Max pixel change: {:.2}%\n\
        Movement pattern: {:?}\n\
        This indicates the UI is continuously re-rendering and moving.",
        analysis.average_pixel_change * 100.0,
        analysis.max_pixel_change * 100.0,
        analysis.movement_direction
    );

    println!("\n✅ No infinite scroll detected - Timeline view is stable");
}

fn navigate_to_timeline() {
    // Use AppleScript to click on Timeline button
    let script = r#"
    tell application "System Events"
        -- Wait for app window
        delay 0.5
        
        -- Find the app window
        set appWindows to (every window of every process whose name contains "plon")
        if (count of appWindows) > 0 then
            -- Click on Timeline button (try different possible labels)
            try
                click button "📅 Timeline" of window 1 of process "plon"
            on error
                try
                    click button "Timeline" of window 1 of process "plon"
                on error
                    -- Try to find it by position (usually in top toolbar)
                    click at {200, 100}
                end try
            end try
        end if
    end tell
    "#;

    Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok();
}

fn capture_screenshot(output_path: &str) {
    // Use screencapture to capture the screen
    Command::new("screencapture")
        .args(&[
            "-x", // No sound
            "-C", // Capture cursor
            "-S", // Capture screen with mouse selection
            output_path,
        ])
        .output()
        .ok();

    // Alternative: capture the whole screen if selection fails
    if !std::path::Path::new(output_path).exists() {
        Command::new("screencapture")
            .args(&["-x", "-m", output_path])
            .output()
            .ok();
    }
}

#[derive(Debug)]
struct ScrollAnalysis {
    frames_analyzed: usize,
    average_pixel_change: f32,
    max_pixel_change: f32,
    consistent_movement: bool,
    movement_direction: MovementDirection,
    infinite_scroll_detected: bool,
}

#[derive(Debug)]
enum MovementDirection {
    None,
    Horizontal,
    Vertical,
    Both,
}

fn analyze_for_infinite_scroll(screenshots: &[String]) -> ScrollAnalysis {
    if screenshots.len() < 3 {
        return ScrollAnalysis {
            frames_analyzed: screenshots.len(),
            average_pixel_change: 0.0,
            max_pixel_change: 0.0,
            consistent_movement: false,
            movement_direction: MovementDirection::None,
            infinite_scroll_detected: false,
        };
    }

    let mut pixel_changes = Vec::new();
    let mut horizontal_movements = Vec::new();
    let mut vertical_movements = Vec::new();

    // Compare consecutive frames
    for i in 1..screenshots.len() {
        if let Ok((change_ratio, h_move, v_move)) =
            compare_frames(&screenshots[i - 1], &screenshots[i])
        {
            pixel_changes.push(change_ratio);
            horizontal_movements.push(h_move);
            vertical_movements.push(v_move);

            println!(
                "Frame {}->{}: {:.2}% change, H:{}, V:{}",
                i - 1,
                i,
                change_ratio * 100.0,
                h_move,
                v_move
            );
        }
    }

    if pixel_changes.is_empty() {
        return ScrollAnalysis {
            frames_analyzed: screenshots.len(),
            average_pixel_change: 0.0,
            max_pixel_change: 0.0,
            consistent_movement: false,
            movement_direction: MovementDirection::None,
            infinite_scroll_detected: false,
        };
    }

    // Calculate statistics
    let average_change = pixel_changes.iter().sum::<f32>() / pixel_changes.len() as f32;
    let max_change = pixel_changes.iter().cloned().fold(0.0, f32::max);

    // Check for consistent movement
    let h_consistency = check_directional_consistency(&horizontal_movements);
    let v_consistency = check_directional_consistency(&vertical_movements);
    let consistent_movement = h_consistency || v_consistency;

    // Determine movement direction
    let movement_direction = match (h_consistency, v_consistency) {
        (true, true) => MovementDirection::Both,
        (true, false) => MovementDirection::Horizontal,
        (false, true) => MovementDirection::Vertical,
        (false, false) => MovementDirection::None,
    };

    // Detect infinite scroll:
    // - Significant pixel changes (>3% average)
    // - Consistent directional movement
    // - Multiple frames with movement (>50%)
    let frames_with_movement = pixel_changes.iter().filter(|&&c| c > 0.02).count();
    let infinite_scroll_detected = average_change > 0.03
        && consistent_movement
        && frames_with_movement > pixel_changes.len() / 2;

    ScrollAnalysis {
        frames_analyzed: screenshots.len(),
        average_pixel_change: average_change,
        max_pixel_change: max_change,
        consistent_movement,
        movement_direction,
        infinite_scroll_detected,
    }
}

fn compare_frames(path1: &str, path2: &str) -> Result<(f32, i32, i32), Box<dyn std::error::Error>> {
    let img1 = image::open(path1)?;
    let img2 = image::open(path2)?;

    // Ensure same dimensions
    if img1.dimensions() != img2.dimensions() {
        return Err("Images have different dimensions".into());
    }

    let (width, height) = img1.dimensions();
    let total_pixels = (width * height) as f32;

    // Focus on the center area where Timeline content would be
    // (avoiding toolbar and sidebar areas)
    let x_start = width / 5;
    let x_end = width * 4 / 5;
    let y_start = height / 5;
    let y_end = height * 4 / 5;

    let mut changed_pixels = 0u32;
    let mut x_shifts = Vec::new();
    let mut y_shifts = Vec::new();

    // Sample grid for movement detection
    let sample_step = 50;

    for y in (y_start..y_end).step_by(sample_step) {
        for x in (x_start..x_end).step_by(sample_step) {
            let pixel1 = img1.get_pixel(x, y);
            let pixel2 = img2.get_pixel(x, y);

            if pixels_significantly_different(&pixel1, &pixel2) {
                changed_pixels += 1;

                // Try to detect shift by finding similar pixel nearby
                if let Some((dx, dy)) = find_pixel_shift(&img1, &img2, x, y, 10) {
                    x_shifts.push(dx);
                    y_shifts.push(dy);
                }
            }
        }
    }

    let area_pixels = ((x_end - x_start) * (y_end - y_start)) as f32;
    let change_ratio = changed_pixels as f32 / (area_pixels / (sample_step * sample_step) as f32);

    // Calculate average shift
    let avg_x_shift = if !x_shifts.is_empty() {
        x_shifts.iter().sum::<i32>() / x_shifts.len() as i32
    } else {
        0
    };

    let avg_y_shift = if !y_shifts.is_empty() {
        y_shifts.iter().sum::<i32>() / y_shifts.len() as i32
    } else {
        0
    };

    Ok((change_ratio, avg_x_shift, avg_y_shift))
}

fn pixels_significantly_different(p1: &Rgba<u8>, p2: &Rgba<u8>) -> bool {
    const THRESHOLD: i32 = 20;

    let dr = (p1[0] as i32 - p2[0] as i32).abs();
    let dg = (p1[1] as i32 - p2[1] as i32).abs();
    let db = (p1[2] as i32 - p2[2] as i32).abs();

    dr > THRESHOLD || dg > THRESHOLD || db > THRESHOLD
}

fn find_pixel_shift(
    img1: &DynamicImage,
    img2: &DynamicImage,
    x: u32,
    y: u32,
    radius: u32,
) -> Option<(i32, i32)> {
    let pixel1 = img1.get_pixel(x, y);
    let (width, height) = img1.dimensions();

    // Search for matching pixel in nearby area
    for dy in -(radius as i32)..=(radius as i32) {
        for dx in -(radius as i32)..=(radius as i32) {
            if dx == 0 && dy == 0 {
                continue;
            }

            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;

            if new_x >= 0 && new_x < width as i32 && new_y >= 0 && new_y < height as i32 {
                let pixel2 = img2.get_pixel(new_x as u32, new_y as u32);
                if !pixels_significantly_different(&pixel1, &pixel2) {
                    return Some((dx, dy));
                }
            }
        }
    }
    None
}

fn check_directional_consistency(movements: &[i32]) -> bool {
    if movements.len() < 2 {
        return false;
    }

    let positive = movements.iter().filter(|&&m| m > 1).count();
    let negative = movements.iter().filter(|&&m| m < -1).count();

    // Consistent if >60% move in same direction
    let threshold = (movements.len() * 6) / 10;
    positive >= threshold || negative >= threshold
}

/// Quick test that can run without launching the app
#[test]
fn test_screenshot_analysis_logic() {
    // Test the analysis logic works correctly
    let empty_analysis = analyze_for_infinite_scroll(&[]);
    assert!(!empty_analysis.infinite_scroll_detected);
    assert_eq!(empty_analysis.frames_analyzed, 0);

    println!("✅ Screenshot analysis logic test passed");
}
