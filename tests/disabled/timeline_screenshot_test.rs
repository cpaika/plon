use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Test that runs the app and captures screenshots to detect infinite scrolling
#[test]
#[ignore] // Run with: cargo test --test timeline_screenshot_test -- --ignored --nocapture
fn test_timeline_infinite_scroll_via_screenshots() {
    println!("Starting timeline infinite scroll detection test...");

    // Create temp directory for screenshots
    let screenshot_dir = "/tmp/plon_timeline_test";
    fs::create_dir_all(screenshot_dir).unwrap();

    // Start the application
    let mut app_process = Command::new("cargo")
        .args(&["run", "--release"])
        .spawn()
        .expect("Failed to start application");

    // Wait for app to start
    thread::sleep(Duration::from_secs(2));

    // Take screenshots over time
    let mut screenshots = Vec::new();
    for i in 0..10 {
        thread::sleep(Duration::from_millis(500));

        let screenshot_path = format!("{}/screenshot_{}.png", screenshot_dir, i);

        // Take screenshot using macOS screencapture
        let output = Command::new("screencapture")
            .args(&["-x", "-m", &screenshot_path])
            .output()
            .expect("Failed to take screenshot");

        if output.status.success() {
            println!("Captured screenshot {}", i);
            screenshots.push(screenshot_path);
        }
    }

    // Kill the app
    app_process.kill().ok();

    // Analyze screenshots for infinite scrolling
    let is_scrolling = analyze_screenshots_for_scrolling(&screenshots);

    // Clean up
    for screenshot in &screenshots {
        fs::remove_file(screenshot).ok();
    }

    assert!(
        !is_scrolling,
        "Infinite scrolling detected in timeline view!"
    );
}

fn analyze_screenshots_for_scrolling(screenshots: &[String]) -> bool {
    if screenshots.len() < 2 {
        return false;
    }

    // Compare consecutive screenshots
    let mut differences = Vec::new();

    for i in 1..screenshots.len() {
        let diff = compare_images(&screenshots[i - 1], &screenshots[i]);
        differences.push(diff);
        println!(
            "Difference between screenshot {} and {}: {:.2}%",
            i - 1,
            i,
            diff * 100.0
        );
    }

    // If all screenshots show significant differences, we have scrolling
    let high_diff_count = differences.iter().filter(|&&d| d > 0.1).count();

    if high_diff_count >= differences.len() - 1 {
        println!(
            "DETECTED: Continuous changes across all screenshots indicate infinite scrolling!"
        );
        return true;
    }

    false
}

fn compare_images(path1: &str, path2: &str) -> f32 {
    // Use ImageMagick compare to get difference metric
    let output = Command::new("compare")
        .args(&["-metric", "RMSE", path1, path2, "/dev/null"])
        .output();

    match output {
        Ok(result) => {
            // ImageMagick outputs to stderr
            let stderr = String::from_utf8_lossy(&result.stderr);
            // Parse the RMSE value
            if let Some(rmse_str) = stderr.split_whitespace().next() {
                if let Ok(rmse) = rmse_str.parse::<f32>() {
                    // Normalize to 0-1 range (assuming 16-bit color depth)
                    return rmse / 65535.0;
                }
            }
            0.0
        }
        Err(_) => {
            println!(
                "Warning: ImageMagick 'compare' command not found. Using file size comparison."
            );
            // Fallback: compare file sizes as a rough metric
            compare_file_sizes(path1, path2)
        }
    }
}

fn compare_file_sizes(path1: &str, path2: &str) -> f32 {
    let size1 = fs::metadata(path1).map(|m| m.len()).unwrap_or(0);
    let size2 = fs::metadata(path2).map(|m| m.len()).unwrap_or(0);

    if size1 == 0 || size2 == 0 {
        return 0.0;
    }

    let diff = (size1 as f32 - size2 as f32).abs();
    let avg = (size1 + size2) as f32 / 2.0;

    diff / avg
}

/// Alternative test using system events to detect rapid redraws
#[test]
fn test_timeline_redraw_frequency() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    println!("Testing timeline redraw frequency...");

    // This test would hook into the system's window server to detect rapid redraws
    // For now, we'll simulate by checking CPU usage of the app

    let is_running = Arc::new(AtomicBool::new(true));
    let cpu_samples = Arc::new(Mutex::new(Vec::new()));

    let is_running_clone = is_running.clone();
    let cpu_samples_clone = cpu_samples.clone();

    // Start monitoring thread
    let monitor_handle = thread::spawn(move || {
        while is_running_clone.load(Ordering::Relaxed) {
            // Get CPU usage of cargo/plon process
            let output = Command::new("ps")
                .args(&["aux"])
                .output()
                .expect("Failed to run ps");

            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("plon") && !line.contains("test") {
                    // Parse CPU usage (3rd column in ps aux output)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 2 {
                        if let Ok(cpu) = parts[2].parse::<f32>() {
                            cpu_samples_clone.lock().unwrap().push(cpu);
                            println!("CPU usage: {}%", cpu);
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    });

    // Start the app
    let mut app_process = Command::new("cargo")
        .args(&["run", "--release"])
        .spawn()
        .expect("Failed to start application");

    // Monitor for 5 seconds
    thread::sleep(Duration::from_secs(5));

    // Stop monitoring
    is_running.store(false, Ordering::Relaxed);
    monitor_handle.join().ok();

    // Kill the app
    app_process.kill().ok();

    // Analyze CPU usage
    let samples = cpu_samples.lock().unwrap();
    if !samples.is_empty() {
        let avg_cpu: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
        let max_cpu = samples.iter().fold(0.0f32, |a, &b| a.max(b));

        println!("Average CPU: {:.2}%, Max CPU: {:.2}%", avg_cpu, max_cpu);

        // If CPU usage is consistently high, we likely have infinite scrolling
        assert!(
            avg_cpu < 50.0,
            "High CPU usage detected ({:.2}%), likely due to infinite scrolling!",
            avg_cpu
        );
        assert!(
            max_cpu < 80.0,
            "Peak CPU usage too high ({:.2}%), likely due to infinite scrolling!",
            max_cpu
        );
    }
}
