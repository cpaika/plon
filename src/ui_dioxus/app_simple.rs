use dioxus::prelude::*;
use crate::ui_dioxus::views::*;

#[component]
pub fn App() -> Element {
    let mut current_view = use_signal(|| "dashboard");
    
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