use plon::ui::app::PlonApp;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    println!("Starting map view hang test...");
    
    // Set up a watchdog timer
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    thread::spawn(move || {
        let start = Instant::now();
        while running_clone.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
            
            if start.elapsed() > Duration::from_secs(30) {
                eprintln!("Test has been running for 30 seconds - likely hung!");
                std::process::exit(1);
            }
        }
    });
    
    // Run the app for a short time
    println!("Creating app...");
    let native_options = eframe::NativeOptions::default();
    
    // Use a custom runner that exits after a short time
    let result = eframe::run_native(
        "Plon Hang Test",
        native_options,
        Box::new(|cc| {
            // Create the app
            let app = PlonApp::new(cc);
            
            // Schedule app to close after 5 seconds
            let ctx = cc.egui_ctx.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(5));
                println!("Requesting app close...");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            });
            
            Box::new(app)
        }),
    );
    
    running.store(false, Ordering::Relaxed);
    
    match result {
        Ok(_) => println!("App ran successfully without hanging"),
        Err(e) => eprintln!("App error: {}", e),
    }
}