mod domain;
mod repository;
mod services;
mod ui;
mod utils;

use anyhow::Result;
use eframe::egui;
use repository::Repository;
use ui::PlonApp;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize database
    let pool = repository::database::init_database("plon.db").await?;
    let repository = Repository::new(pool);

    // Run the native app
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Plon - Project Management",
        options,
        Box::new(|cc| Box::new(PlonApp::new(cc, repository))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))?;

    Ok(())
}