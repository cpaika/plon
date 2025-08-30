#!/usr/bin/env rust-script
//! Test script to detect timeline scrolling on mouse movement
//! Run with: cargo run --bin run_timeline_scroll_test

use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    println!("🧪 Timeline Scroll Detection Test");
    println!("{}", "=".repeat(50));
    println!();
    println!("This test will:");
    println!("1. Launch the PlonApp");
    println!("2. You manually navigate to Timeline view");
    println!("3. Move your mouse around WITHOUT clicking or scrolling");
    println!("4. Observe if the timeline scrolls on its own");
    println!();
    println!("⚠️  IMPORTANT: After navigating to Timeline:");
    println!("   - DO NOT click anything");
    println!("   - DO NOT use scroll wheel");
    println!("   - ONLY move your mouse cursor around");
    println!();
    println!("Starting app in 3 seconds...");

    thread::sleep(Duration::from_secs(3));

    // Launch the app
    println!("Launching PlonApp...");
    let mut child = Command::new("cargo")
        .args(&["run", "--release"])
        .spawn()
        .expect("Failed to launch app");

    println!();
    println!("{}", "=".repeat(50));
    println!("TEST INSTRUCTIONS:");
    println!("1. Click on '📅 Timeline' in the top bar");
    println!("2. Once in Timeline view, ONLY move your mouse");
    println!("3. Watch if the view scrolls without any clicks/wheel");
    println!("4. Close the app when done testing");
    println!("{}", "=".repeat(50));
    println!();
    println!("Waiting for app to close...");

    // Wait for the app to close
    let status = child.wait().expect("Failed to wait for app");

    println!();
    println!("App closed with status: {:?}", status);
    println!();
    println!("TEST QUESTIONS:");
    println!("1. Did the timeline scroll when you only moved the mouse? (Y/N)");
    println!("2. Did scrolling happen without clicking or using scroll wheel? (Y/N)");
    println!();
    println!("If you answered YES to either question, the bug is confirmed!");
}
