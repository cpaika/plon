use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::ui_dioxus::router::Route;
use crate::ui_dioxus::views::{MapView, ListView, KanbanView, TimelineView, GanttView};

#[component]
pub fn App() -> Element {
    // Simple app without router for now
    
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn NavBar() -> Element {
    
    rsx! {
        nav {
            class: "navbar",
            
            div {
                class: "nav-brand",
                "Plon"
            }
            
            div {
                class: "nav-menu",
                
                NavItem { 
                    route: Route::Map,
                    label: "Map",
                    icon: "🗺️"
                }
                
                NavItem {
                    route: Route::List,
                    label: "List",
                    icon: "📝"
                }
                
                NavItem {
                    route: Route::Kanban,
                    label: "Kanban",
                    icon: "📋"
                }
                
                NavItem {
                    route: Route::Timeline,
                    label: "Timeline",
                    icon: "📅"
                }
                
                NavItem {
                    route: Route::Gantt,
                    label: "Gantt",
                    icon: "📊"
                }
            }
            
            div {
                class: "nav-actions",
                
                button {
                    class: "btn-primary",
                    onclick: move |_| {
                        // Create new task
                    },
                    "➕ New Task"
                }
            }
        }
    }
}

#[component]
fn NavItem(route: Route, label: &'static str, icon: &'static str) -> Element {
    let is_active = false;
    
    rsx! {
        button {
            class: if is_active { "nav-item active" } else { "nav-item" },
            onclick: move |_| {
                // Navigation will be handled by Link component
            },
            span { class: "nav-icon", "{icon}" }
            span { class: "nav-label", "{label}" }
        }
    }
}

#[component]
fn StatusBar() -> Element {
    // Simplified without state for now
    
    rsx! {
        div {
            class: "status-bar",
            
            div {
                class: "status-info",
                "Tasks: 0/0"
            }
        }
    }
}

#[component]
fn Dashboard() -> Element {
    
    rsx! {
        div {
            class: "dashboard",
            
            h1 { "Welcome to Plon" }
            
            div {
                class: "dashboard-stats",
                
                StatCard {
                    title: "Total Tasks",
                    value: "0".to_string(),
                    icon: "📋"
                }
                
                StatCard {
                    title: "In Progress",
                    value: "0".to_string(),
                    icon: "⏳"
                }
                
                StatCard {
                    title: "Completed",
                    value: "0".to_string(),
                    icon: "✅"
                }
            }
        }
    }
}

#[component]
fn StatCard(title: &'static str, value: String, icon: &'static str) -> Element {
    rsx! {
        div {
            class: "stat-card",
            
            div { class: "stat-icon", "{icon}" }
            div { class: "stat-content",
                div { class: "stat-title", "{title}" }
                div { class: "stat-value", "{value}" }
            }
        }
    }
}