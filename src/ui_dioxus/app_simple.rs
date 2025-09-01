use dioxus::prelude::*;
use crate::ui_dioxus::views::*;
use crate::repository::Repository;
use std::sync::Arc;
use sqlx::SqlitePool;

#[component]
pub fn App() -> Element {
    // Initialize Repository and provide it as context
    let repository = use_resource(|| async {
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