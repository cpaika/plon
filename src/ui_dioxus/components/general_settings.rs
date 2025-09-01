use dioxus::prelude::*;
use crate::domain::app_settings::AppSettings;
use crate::domain::task::TaskStatus;
use crate::repository::Repository;
use std::sync::Arc;

#[component]
pub fn GeneralSettings() -> Element {
    let repository = use_context::<Arc<Repository>>();
    let mut settings = use_signal(|| None::<AppSettings>);
    let mut save_status = use_signal(String::new);
    
    // Form fields
    let mut default_task_status = use_signal(|| TaskStatus::Todo);
    let mut auto_save_interval = use_signal(|| 30);
    let mut enable_notifications = use_signal(|| true);
    let mut notification_sound = use_signal(|| true);
    let mut date_format = use_signal(|| "MM/DD/YYYY".to_string());
    let mut time_format = use_signal(|| "12h".to_string());
    let mut week_starts_on = use_signal(|| "Sunday".to_string());
    let mut enable_time_tracking = use_signal(|| true);
    let mut show_task_numbers = use_signal(|| false);
    
    // Load settings on mount
    use_effect({
        let repo = repository.clone();
        move || {
            let repo = repo.clone();
            spawn(async move {
                match repo.app_settings.get_or_create_default().await {
                    Ok(loaded_settings) => {
                        default_task_status.set(loaded_settings.default_task_status);
                        auto_save_interval.set(loaded_settings.auto_save_interval_seconds);
                        enable_notifications.set(loaded_settings.enable_notifications);
                        notification_sound.set(loaded_settings.notification_sound);
                        date_format.set(loaded_settings.date_format.clone());
                        time_format.set(loaded_settings.time_format.clone());
                        week_starts_on.set(loaded_settings.week_starts_on.clone());
                        enable_time_tracking.set(loaded_settings.enable_time_tracking);
                        show_task_numbers.set(loaded_settings.show_task_numbers);
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
                current_settings.default_task_status = default_task_status();
                current_settings.auto_save_interval_seconds = auto_save_interval();
                current_settings.enable_notifications = enable_notifications();
                current_settings.notification_sound = notification_sound();
                current_settings.date_format = date_format();
                current_settings.time_format = time_format();
                current_settings.week_starts_on = week_starts_on();
                current_settings.enable_time_tracking = enable_time_tracking();
                current_settings.show_task_numbers = show_task_numbers();
                current_settings.update_timestamp();
                
                match repo.app_settings.update(&current_settings).await {
                    Ok(_) => {
                        save_status.set("Settings saved successfully!".to_string());
                        settings.set(Some(current_settings));
                    }
                    Err(e) => {
                        save_status.set(format!("Error saving settings: {}", e));
                    }
                }
            }
        });
    };
    
    rsx! {
        div { class: "settings-panel",
            style: "background: white; padding: 30px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);",
            
            h2 { 
                style: "font-size: 1.5rem; font-weight: 600; margin-bottom: 20px;",
                "General Settings" 
            }
            
            div { style: "space-y: 20px;",
                // Default Task Status
                div { style: "margin-bottom: 20px;",
                    label { 
                        style: "display: block; font-weight: 500; margin-bottom: 8px;",
                        "Default Task Status" 
                    }
                    select {
                        style: "width: 200px; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                        value: "{default_task_status:?}",
                        onchange: move |e| {
                            if let Ok(status) = e.value().parse::<String>() {
                                match status.as_str() {
                                    "Todo" => default_task_status.set(TaskStatus::Todo),
                                    "InProgress" => default_task_status.set(TaskStatus::InProgress),
                                    "Blocked" => default_task_status.set(TaskStatus::Blocked),
                                    "Review" => default_task_status.set(TaskStatus::Review),
                                    "Done" => default_task_status.set(TaskStatus::Done),
                                    _ => {}
                                }
                            }
                        },
                        option { value: "Todo", "Todo" }
                        option { value: "InProgress", "In Progress" }
                        option { value: "Blocked", "Blocked" }
                        option { value: "Review", "Review" }
                        option { value: "Done", "Done" }
                    }
                }
                
                // Auto-save Interval
                div { style: "margin-bottom: 20px;",
                    label { 
                        style: "display: block; font-weight: 500; margin-bottom: 8px;",
                        "Auto-save Interval (seconds)" 
                    }
                    input {
                        r#type: "number",
                        min: "10",
                        max: "300",
                        style: "width: 100px; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                        value: "{auto_save_interval}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<i32>() {
                                auto_save_interval.set(val);
                            }
                        }
                    }
                }
                
                // Notifications
                div { style: "margin-bottom: 20px;",
                    label { 
                        style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: enable_notifications(),
                            onchange: move |_| enable_notifications.set(!enable_notifications())
                        }
                        "Enable Notifications"
                    }
                    
                    if enable_notifications() {
                        label { 
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer; margin-left: 24px; margin-top: 8px;",
                            input {
                                r#type: "checkbox",
                                checked: notification_sound(),
                                onchange: move |_| notification_sound.set(!notification_sound())
                            }
                            "Play Sound"
                        }
                    }
                }
                
                // Date & Time Format
                div { style: "margin-bottom: 20px;",
                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                        div {
                            label { 
                                style: "display: block; font-weight: 500; margin-bottom: 8px;",
                                "Date Format" 
                            }
                            select {
                                style: "width: 100%; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                                value: "{date_format}",
                                onchange: move |e| date_format.set(e.value()),
                                option { value: "MM/DD/YYYY", "MM/DD/YYYY" }
                                option { value: "DD/MM/YYYY", "DD/MM/YYYY" }
                                option { value: "YYYY-MM-DD", "YYYY-MM-DD" }
                            }
                        }
                        
                        div {
                            label { 
                                style: "display: block; font-weight: 500; margin-bottom: 8px;",
                                "Time Format" 
                            }
                            select {
                                style: "width: 100%; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                                value: "{time_format}",
                                onchange: move |e| time_format.set(e.value()),
                                option { value: "12h", "12-hour" }
                                option { value: "24h", "24-hour" }
                            }
                        }
                    }
                }
                
                // Week Start
                div { style: "margin-bottom: 20px;",
                    label { 
                        style: "display: block; font-weight: 500; margin-bottom: 8px;",
                        "Week Starts On" 
                    }
                    select {
                        style: "width: 200px; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                        value: "{week_starts_on}",
                        onchange: move |e| week_starts_on.set(e.value()),
                        option { value: "Sunday", "Sunday" }
                        option { value: "Monday", "Monday" }
                    }
                }
                
                // Additional Options
                div { style: "margin-bottom: 20px;",
                    label { 
                        style: "display: flex; align-items: center; gap: 8px; cursor: pointer; margin-bottom: 12px;",
                        input {
                            r#type: "checkbox",
                            checked: enable_time_tracking(),
                            onchange: move |_| enable_time_tracking.set(!enable_time_tracking())
                        }
                        "Enable Time Tracking"
                    }
                    
                    label { 
                        style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: show_task_numbers(),
                            onchange: move |_| show_task_numbers.set(!show_task_numbers())
                        }
                        "Show Task Numbers"
                    }
                }
                
                // Save Button and Status
                div { style: "display: flex; align-items: center; gap: 12px; margin-top: 24px; padding-top: 24px; border-top: 1px solid #e5e7eb;",
                    button {
                        style: "padding: 10px 20px; background: #3b82f6; color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: 500;",
                        onclick: move |_| save_settings(),
                        "Save General Settings"
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