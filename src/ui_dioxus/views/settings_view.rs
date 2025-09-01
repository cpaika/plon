use dioxus::prelude::*;
use crate::ui_dioxus::components::{
    AppearanceSettings, 
    ClaudeConfigAdmin, 
    GeneralSettings, 
    WorkspaceSettings
};

#[component]
pub fn SettingsView() -> Element {
    let mut active_tab = use_signal(|| "claude".to_string());
    
    rsx! {
        div { class: "settings-container",
            style: "padding: 20px; max-width: 1200px; margin: 0 auto;",
            
            // Settings Header
            div { class: "settings-header",
                style: "margin-bottom: 30px;",
                h1 { 
                    style: "font-size: 2rem; font-weight: bold; margin-bottom: 10px;",
                    "Settings" 
                }
                p { 
                    style: "color: #666;",
                    "Configure your Plon workspace and integrations"
                }
            }
            
            // Tab Navigation
            div { class: "settings-tabs",
                style: "display: flex; gap: 10px; border-bottom: 2px solid #e5e7eb; margin-bottom: 30px;",
                
                TabButton {
                    label: "Claude Code",
                    icon: "🤖",
                    active: active_tab() == "claude",
                    onclick: move |_| active_tab.set("claude".to_string())
                }
                
                TabButton {
                    label: "General",
                    icon: "⚙️",
                    active: active_tab() == "general",
                    onclick: move |_| active_tab.set("general".to_string())
                }
                
                TabButton {
                    label: "Workspace",
                    icon: "📁",
                    active: active_tab() == "workspace",
                    onclick: move |_| active_tab.set("workspace".to_string())
                }
                
                TabButton {
                    label: "Integrations",
                    icon: "🔗",
                    active: active_tab() == "integrations",
                    onclick: move |_| active_tab.set("integrations".to_string())
                }
                
                TabButton {
                    label: "Appearance",
                    icon: "🎨",
                    active: active_tab() == "appearance",
                    onclick: move |_| active_tab.set("appearance".to_string())
                }
            }
            
            // Tab Content
            div { class: "settings-content",
                if active_tab() == "claude" {
                    ClaudeConfigAdmin {}
                } else if active_tab() == "general" {
                    GeneralSettings {}
                } else if active_tab() == "workspace" {
                    WorkspaceSettings {}
                } else if active_tab() == "integrations" {
                    IntegrationsSettings {}
                } else if active_tab() == "appearance" {
                    AppearanceSettings {}
                }
            }
        }
    }
}

// Keep IntegrationsSettings component here since it's still a placeholder
#[component]
fn IntegrationsSettings() -> Element {
    rsx! {
        div { class: "settings-panel",
            style: "background: white; padding: 30px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);",
            
            h2 { 
                style: "font-size: 1.5rem; font-weight: 600; margin-bottom: 20px;",
                "Integrations" 
            }
            
            div { style: "space-y: 4px;",
                p { style: "color: #666; margin-bottom: 20px;",
                    "Connect with external services and tools"
                }
                
                // Integration status cards
                div { style: "display: grid; gap: 16px;",
                    IntegrationCard {
                        name: "GitHub",
                        icon: "🐙",
                        status: "Connected",
                        description: "Repository management and pull requests"
                    }
                    
                    IntegrationCard {
                        name: "Claude AI",
                        icon: "🤖",
                        status: "Configure in Claude Code tab",
                        description: "AI-powered code generation"
                    }
                    
                    IntegrationCard {
                        name: "Slack",
                        icon: "💬",
                        status: "Not connected",
                        description: "Team notifications and updates"
                    }
                }
            }
        }
    }
}

#[component]
fn IntegrationCard(name: &'static str, icon: &'static str, status: &'static str, description: &'static str) -> Element {
    let is_connected = status == "Connected";
    
    rsx! {
        div {
            style: "padding: 16px; background: #f9fafb; border-radius: 6px; display: flex; align-items: center; gap: 16px;",
            
            div { 
                style: "font-size: 2rem;",
                "{icon}" 
            }
            
            div { style: "flex: 1;",
                div { style: "font-weight: 600; margin-bottom: 4px;", "{name}" }
                div { style: "font-size: 0.875rem; color: #6b7280;", "{description}" }
            }
            
            div {
                style: format!(
                    "padding: 4px 12px; background: {}; color: {}; border-radius: 4px; font-size: 0.75rem; font-weight: 500;",
                    if is_connected { "#dcfce7" } else { "#f3f4f6" },
                    if is_connected { "#16a34a" } else { "#6b7280" }
                ),
                "{status}"
            }
        }
    }
}

#[component]
fn TabButton(label: &'static str, icon: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: if active { "settings-tab active" } else { "settings-tab" },
            style: format!(
                "padding: 12px 20px; background: {}; border: none; border-bottom: 3px solid {}; cursor: pointer; display: flex; align-items: center; gap: 8px; font-size: 14px; font-weight: {}; color: {}; transition: all 0.2s;",
                if active { "transparent" } else { "transparent" },
                if active { "#3b82f6" } else { "transparent" },
                if active { "600" } else { "400" },
                if active { "#3b82f6" } else { "#6b7280" }
            ),
            onclick: move |e| onclick.call(e),
            span { "{icon}" }
            span { "{label}" }
        }
    }
}

