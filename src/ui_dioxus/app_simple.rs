use dioxus::prelude::*;
use crate::ui_dioxus::views::*;
use crate::repository::Repository;
use crate::services::TimeTrackingService;
use std::sync::Arc;
use sqlx::SqlitePool;
use std::path::Path;
use tokio::fs;

#[component]
pub fn App() -> Element {
    // Initialize Repository and provide it as context
    let repository = use_resource(|| async {
        // Create workspace directories if they don't exist
        create_workspace_directories().await;
        
        // Connect to the database
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:plon.db".to_string());
        
        let pool = SqlitePool::connect(&database_url)
            .await
            .expect("Failed to connect to database");
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
        
        Arc::new(Repository::new(pool))
    });
    
    let mut current_view = use_signal(|| "dashboard");
    
    // Wait for repository to be ready
    match &*repository.read_unchecked() {
        Some(repo) => {
            // Provide the repository as context for all child components
            use_context_provider(|| repo.clone());
            
            // Provide the TimeTrackingService as context
            let time_tracking_service = Arc::new(TimeTrackingService::new(repo.clone()));
            use_context_provider(|| time_tracking_service);
            
            rsx! {
                div {
                    class: "app-container",
            
            // Navigation bar
            nav {
                class: "navbar",
                
                div { class: "nav-brand", "Plon" }
                
                div {
                    class: "nav-menu",
                    
                    button {
                        class: if *current_view.read() == "goals" { "nav-item active" } else { "nav-item" },
                        onclick: move |_| current_view.set("goals"),
                        "🎯 Goals"
                    }
                    
                    button {
                        class: if *current_view.read() == "map" { "nav-item active" } else { "nav-item" },
                        onclick: move |_| current_view.set("map"),
                        "🗺️ Map"
                    }
                    
                    button {
                        class: if *current_view.read() == "list" { "nav-item active" } else { "nav-item" },
                        onclick: move |_| current_view.set("list"),
                        "📝 List"
                    }
                    
                    button {
                        class: if *current_view.read() == "kanban" { "nav-item active" } else { "nav-item" },
                        onclick: move |_| current_view.set("kanban"),
                        "📋 Kanban"
                    }
                    
                    button {
                        class: if *current_view.read() == "timeline" { "nav-item active" } else { "nav-item" },
                        onclick: move |_| current_view.set("timeline"),
                        "📅 Timeline"
                    }
                    
                    button {
                        class: if *current_view.read() == "gantt" { "nav-item active" } else { "nav-item" },
                        onclick: move |_| current_view.set("gantt"),
                        "📊 Gantt"
                    }
                    
                    button {
                        class: if *current_view.read() == "settings" { "nav-item active" } else { "nav-item" },
                        onclick: move |_| current_view.set("settings"),
                        "⚙️ Settings"
                    }
                }
            }
            
            // Main content area
            div {
                class: "main-content",
                
                match current_view.read().as_ref() {
                    "goals" => rsx! { GoalsView {} },
                    "map" => rsx! { MapView {} },
                    "list" => rsx! { ListView {} },
                    "kanban" => rsx! { KanbanView {} },
                    "timeline" => rsx! { TimelineView {} },
                    "gantt" => rsx! { GanttView {} },
                    "settings" => rsx! { SettingsView {} },
                    _ => rsx! { Dashboard {} },
                }
            }
            
            // Status bar
            div {
                class: "status-bar",
                div { class: "status-info", "Ready" }
                    }
                }
            }
        }
        None => {
            // Show loading state while repository is being initialized
            rsx! {
                div {
                    class: "loading-container",
                    style: "display: flex; justify-content: center; align-items: center; height: 100vh; font-size: 1.5rem;",
                    "Loading application..."
                }
            }
        }
    }
}

async fn create_workspace_directories() {
    // Get home directory
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    
    // Default workspace directories
    let directories = vec![
        format!("{}/plon-projects", home_dir),
        format!("{}/plon-backups", home_dir),
        format!("{}/plon-templates", home_dir),
        format!("{}/.plon", home_dir),  // Hidden config directory
        format!("{}/.plon/cache", home_dir),
        format!("{}/.plon/logs", home_dir),
    ];
    
    // Create each directory if it doesn't exist
    for dir_path in directories {
        let path = Path::new(&dir_path);
        if !path.exists() {
            match fs::create_dir_all(path).await {
                Ok(_) => {
                    println!("✅ Created workspace directory: {}", dir_path);
                }
                Err(e) => {
                    eprintln!("⚠️ Warning: Could not create directory {}: {}", dir_path, e);
                }
            }
        }
    }
    
    // Create a README in the main workspace directory
    let readme_path = format!("{}/plon-projects/README.md", home_dir);
    if !Path::new(&readme_path).exists() {
        let readme_content = r#"# Plon Workspace

This directory contains your Plon projects and tasks.

## Directory Structure

- `plon-projects/` - Your active projects and tasks
- `plon-backups/` - Automatic backups of your data
- `plon-templates/` - Reusable task templates
- `.plon/` - Configuration and cache files

## Getting Started

Create new tasks through the Plon desktop or web application.

"#;
        match fs::write(&readme_path, readme_content).await {
            Ok(_) => println!("📝 Created workspace README"),
            Err(e) => eprintln!("⚠️ Warning: Could not create README: {}", e),
        }
    }
}