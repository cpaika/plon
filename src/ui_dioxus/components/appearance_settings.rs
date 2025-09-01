use dioxus::prelude::*;
use crate::domain::app_settings::{AppSettings, Theme, FontSize, UiDensity, SidebarPosition};
use crate::repository::Repository;
use std::sync::Arc;

#[component]
pub fn AppearanceSettings() -> Element {
    let repository = use_context::<Arc<Repository>>();
    let mut settings = use_signal(|| None::<AppSettings>);
    let mut save_status = use_signal(String::new);
    
    // Form fields
    let mut theme = use_signal(|| Theme::Light);
    let mut accent_color = use_signal(|| "#3b82f6".to_string());
    let mut font_size = use_signal(|| FontSize::Medium);
    let mut ui_density = use_signal(|| UiDensity::Comfortable);
    let mut sidebar_position = use_signal(|| SidebarPosition::Left);
    let mut show_sidebar = use_signal(|| true);
    let mut show_toolbar = use_signal(|| true);
    let mut show_statusbar = use_signal(|| true);
    let mut enable_animations = use_signal(|| true);
    
    // Load settings on mount
    use_effect({
        let repo = repository.clone();
        move || {
            let repo = repo.clone();
            spawn(async move {
                match repo.app_settings.get_or_create_default().await {
                    Ok(loaded_settings) => {
                        theme.set(loaded_settings.theme);
                        accent_color.set(loaded_settings.accent_color.clone());
                        font_size.set(loaded_settings.font_size);
                        ui_density.set(loaded_settings.ui_density);
                        sidebar_position.set(loaded_settings.sidebar_position);
                        show_sidebar.set(loaded_settings.show_sidebar);
                        show_toolbar.set(loaded_settings.show_toolbar);
                        show_statusbar.set(loaded_settings.show_statusbar);
                        enable_animations.set(loaded_settings.enable_animations);
                        settings.set(Some(loaded_settings));
                    }
                    Err(e) => {
                        save_status.set(format!("Error loading settings: {}", e));
                    }
                }
            });
        }
    });
    
    let save_settings = move || {
        let repo = repository.clone();
        spawn(async move {
            if let Some(mut current_settings) = settings() {
                current_settings.theme = theme();
                current_settings.accent_color = accent_color();
                current_settings.font_size = font_size();
                current_settings.ui_density = ui_density();
                current_settings.sidebar_position = sidebar_position();
                current_settings.show_sidebar = show_sidebar();
                current_settings.show_toolbar = show_toolbar();
                current_settings.show_statusbar = show_statusbar();
                current_settings.enable_animations = enable_animations();
                current_settings.update_timestamp();
                
                match repo.app_settings.update(&current_settings).await {
                    Ok(_) => {
                        save_status.set("Appearance settings saved successfully!".to_string());
                        settings.set(Some(current_settings));
                    }
                    Err(e) => {
                        save_status.set(format!("Error saving settings: {}", e));
                    }
                }
            }
        });
    };
    
    let color_options = vec![
        "#3b82f6", // Blue
        "#8b5cf6", // Purple
        "#ec4899", // Pink
        "#10b981", // Green
        "#f59e0b", // Amber
        "#ef4444", // Red
        "#06b6d4", // Cyan
        "#6366f1", // Indigo
    ];
    
    rsx! {
        div { class: "settings-panel",
            style: "background: white; padding: 30px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);",
            
            h2 { 
                style: "font-size: 1.5rem; font-weight: 600; margin-bottom: 20px;",
                "Appearance Settings" 
            }
            
            div { style: "space-y: 20px;",
                // Theme Selection
                div { style: "margin-bottom: 24px;",
                    label { 
                        style: "display: block; font-weight: 500; margin-bottom: 12px;",
                        "Theme" 
                    }
                    div { style: "display: flex; gap: 12px;",
                        button {
                            style: format!(
                                "padding: 8px 16px; background: {}; border: 2px solid {}; border-radius: 6px; cursor: pointer; font-weight: {}; color: {};",
                                if theme() == Theme::Light { "#3b82f6" } else { "white" },
                                if theme() == Theme::Light { "#3b82f6" } else { "#e5e7eb" },
                                if theme() == Theme::Light { "500" } else { "400" },
                                if theme() == Theme::Light { "white" } else { "black" }
                            ),
                            onclick: move |_| theme.set(Theme::Light),
                            "☀️ Light"
                        }
                        button {
                            style: format!(
                                "padding: 8px 16px; background: {}; border: 2px solid {}; border-radius: 6px; cursor: pointer; font-weight: {}; color: {};",
                                if theme() == Theme::Dark { "#3b82f6" } else { "white" },
                                if theme() == Theme::Dark { "#3b82f6" } else { "#e5e7eb" },
                                if theme() == Theme::Dark { "500" } else { "400" },
                                if theme() == Theme::Dark { "white" } else { "black" }
                            ),
                            onclick: move |_| theme.set(Theme::Dark),
                            "🌙 Dark"
                        }
                        button {
                            style: format!(
                                "padding: 8px 16px; background: {}; border: 2px solid {}; border-radius: 6px; cursor: pointer; font-weight: {}; color: {};",
                                if theme() == Theme::Auto { "#3b82f6" } else { "white" },
                                if theme() == Theme::Auto { "#3b82f6" } else { "#e5e7eb" },
                                if theme() == Theme::Auto { "500" } else { "400" },
                                if theme() == Theme::Auto { "white" } else { "black" }
                            ),
                            onclick: move |_| theme.set(Theme::Auto),
                            "🔄 Auto"
                        }
                    }
                }
                
                // Accent Color
                div { style: "margin-bottom: 24px;",
                    label { 
                        style: "display: block; font-weight: 500; margin-bottom: 12px;",
                        "Accent Color" 
                    }
                    div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                        for color in color_options {
                            button {
                                style: format!(
                                    "width: 40px; height: 40px; background: {}; border: 3px solid {}; border-radius: 8px; cursor: pointer;",
                                    color,
                                    if accent_color() == color { "#1f2937" } else { "#e5e7eb" }
                                ),
                                onclick: move |_| accent_color.set(color.to_string()),
                                ""
                            }
                        }
                    }
                }
                
                // Font Size
                div { style: "margin-bottom: 24px;",
                    label { 
                        style: "display: block; font-weight: 500; margin-bottom: 12px;",
                        "Font Size" 
                    }
                    select {
                        style: "width: 200px; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                        value: "{font_size:?}",
                        onchange: move |e| {
                            match e.value().as_str() {
                                "Small" => font_size.set(FontSize::Small),
                                "Medium" => font_size.set(FontSize::Medium),
                                "Large" => font_size.set(FontSize::Large),
                                "ExtraLarge" => font_size.set(FontSize::ExtraLarge),
                                _ => {}
                            }
                        },
                        option { value: "Small", "Small" }
                        option { value: "Medium", "Medium" }
                        option { value: "Large", "Large" }
                        option { value: "ExtraLarge", "Extra Large" }
                    }
                }
                
                // UI Density
                div { style: "margin-bottom: 24px;",
                    label { 
                        style: "display: block; font-weight: 500; margin-bottom: 12px;",
                        "UI Density" 
                    }
                    div { style: "display: flex; gap: 12px;",
                        for density_option in vec![
                            (UiDensity::Compact, "Compact"),
                            (UiDensity::Comfortable, "Comfortable"),
                            (UiDensity::Spacious, "Spacious")
                        ] {
                            button {
                                style: format!(
                                    "padding: 8px 16px; background: {}; border: 2px solid {}; border-radius: 6px; cursor: pointer; font-weight: {};",
                                    if ui_density() == density_option.0 { "#3b82f6" } else { "white" },
                                    if ui_density() == density_option.0 { "#3b82f6" } else { "#e5e7eb" },
                                    if ui_density() == density_option.0 { "500" } else { "400" }
                                ),
                                onclick: move |_| ui_density.set(density_option.0),
                                "{density_option.1}"
                            }
                        }
                    }
                }
                
                // Layout Options
                div { style: "margin-bottom: 24px;",
                    h3 { 
                        style: "font-size: 1.1rem; font-weight: 600; margin-bottom: 16px; color: #374151;",
                        "Layout"
                    }
                    
                    div { style: "space-y: 12px;",
                        div { style: "display: flex; items-center; gap: 16px;",
                            label { style: "font-weight: 500; min-width: 120px;", "Sidebar Position" }
                            div { style: "display: flex; gap: 8px;",
                                button {
                                    style: format!(
                                        "padding: 6px 12px; background: {}; border: 1px solid {}; border-radius: 4px; cursor: pointer;",
                                        if sidebar_position() == SidebarPosition::Left { "#3b82f6" } else { "white" },
                                        if sidebar_position() == SidebarPosition::Left { "#3b82f6" } else { "#e5e7eb" }
                                    ),
                                    onclick: move |_| sidebar_position.set(SidebarPosition::Left),
                                    "Left"
                                }
                                button {
                                    style: format!(
                                        "padding: 6px 12px; background: {}; border: 1px solid {}; border-radius: 4px; cursor: pointer;",
                                        if sidebar_position() == SidebarPosition::Right { "#3b82f6" } else { "white" },
                                        if sidebar_position() == SidebarPosition::Right { "#3b82f6" } else { "#e5e7eb" }
                                    ),
                                    onclick: move |_| sidebar_position.set(SidebarPosition::Right),
                                    "Right"
                                }
                            }
                        }
                        
                        label { 
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: show_sidebar(),
                                onchange: move |_| show_sidebar.set(!show_sidebar())
                            }
                            "Show Sidebar"
                        }
                        
                        label { 
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: show_toolbar(),
                                onchange: move |_| show_toolbar.set(!show_toolbar())
                            }
                            "Show Toolbar"
                        }
                        
                        label { 
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: show_statusbar(),
                                onchange: move |_| show_statusbar.set(!show_statusbar())
                            }
                            "Show Status Bar"
                        }
                        
                        label { 
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: enable_animations(),
                                onchange: move |_| enable_animations.set(!enable_animations())
                            }
                            "Enable Animations"
                        }
                    }
                }
                
                // Save Button
                div { style: "display: flex; align-items: center; gap: 12px; margin-top: 24px; padding-top: 24px; border-top: 1px solid #e5e7eb;",
                    button {
                        style: "padding: 10px 20px; background: #3b82f6; color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: 500;",
                        onclick: move |_| save_settings(),
                        "Save Appearance Settings"
                    }
                    
                    if !save_status().is_empty() {
                        span {
                            style: format!(
                                "color: {}; font-size: 14px;",
                                if save_status().contains("Error") { "#ef4444" } else { "#10b981" }
                            ),
                            "{save_status}"
                        }
                    }
                }
            }
        }
    }
}