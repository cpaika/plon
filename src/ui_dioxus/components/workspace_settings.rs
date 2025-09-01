use dioxus::prelude::*;
use crate::domain::app_settings::AppSettings;
use crate::repository::Repository;
use std::sync::Arc;

#[component]
pub fn WorkspaceSettings() -> Element {
    let repository = use_context::<Arc<Repository>>();
    let mut settings = use_signal(|| None::<AppSettings>);
    let mut save_status = use_signal(String::new);
    
    // Form fields
    let mut default_project_directory = use_signal(String::new);
    let mut database_path = use_signal(String::new);
    let mut enable_auto_backup = use_signal(|| true);
    let mut backup_directory = use_signal(String::new);
    let mut backup_frequency_hours = use_signal(|| 24);
    let mut max_backups_to_keep = use_signal(|| 7);
    let mut enable_file_watching = use_signal(|| true);
    let mut git_auto_commit = use_signal(|| false);
    let mut task_template_directory = use_signal(String::new);
    
    // Load settings on mount
    use_effect({
        let repo = repository.clone();
        move || {
            let repo = repo.clone();
            spawn(async move {
                match repo.app_settings.get_or_create_default().await {
                    Ok(loaded_settings) => {
                        default_project_directory.set(loaded_settings.default_project_directory.clone());
                        database_path.set(loaded_settings.database_path.clone());
                        enable_auto_backup.set(loaded_settings.enable_auto_backup);
                        backup_directory.set(loaded_settings.backup_directory.clone());
                        backup_frequency_hours.set(loaded_settings.backup_frequency_hours);
                        max_backups_to_keep.set(loaded_settings.max_backups_to_keep);
                        enable_file_watching.set(loaded_settings.enable_file_watching);
                        git_auto_commit.set(loaded_settings.git_auto_commit);
                        task_template_directory.set(loaded_settings.task_template_directory.clone().unwrap_or_default());
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
                current_settings.default_project_directory = default_project_directory();
                current_settings.database_path = database_path();
                current_settings.enable_auto_backup = enable_auto_backup();
                current_settings.backup_directory = backup_directory();
                current_settings.backup_frequency_hours = backup_frequency_hours();
                current_settings.max_backups_to_keep = max_backups_to_keep();
                current_settings.enable_file_watching = enable_file_watching();
                current_settings.git_auto_commit = git_auto_commit();
                current_settings.task_template_directory = if task_template_directory().is_empty() {
                    None
                } else {
                    Some(task_template_directory())
                };
                current_settings.update_timestamp();
                
                match repo.app_settings.update(&current_settings).await {
                    Ok(_) => {
                        save_status.set("Workspace settings saved successfully!".to_string());
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
                "Workspace Settings" 
            }
            
            div { style: "space-y: 20px;",
                // Directory Settings
                div { style: "margin-bottom: 24px;",
                    h3 { 
                        style: "font-size: 1.1rem; font-weight: 600; margin-bottom: 16px; color: #374151;",
                        "📁 Directories"
                    }
                    
                    div { style: "space-y: 16px;",
                        div {
                            label { 
                                style: "display: block; font-weight: 500; margin-bottom: 8px;",
                                "Default Project Directory" 
                            }
                            input {
                                r#type: "text",
                                style: "width: 100%; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                                value: "{default_project_directory}",
                                placeholder: "~/plon-projects",
                                oninput: move |e| default_project_directory.set(e.value())
                            }
                            p { style: "text-xs text-gray-500 mt-2;",
                                "Where new projects and tasks will be created"
                            }
                        }
                        
                        div {
                            label { 
                                style: "display: block; font-weight: 500; margin-bottom: 8px;",
                                "Database Location" 
                            }
                            input {
                                r#type: "text",
                                style: "width: 100%; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                                value: "{database_path}",
                                placeholder: "plon.db",
                                oninput: move |e| database_path.set(e.value())
                            }
                            p { style: "text-xs text-gray-500 mt-2;",
                                "Path to the SQLite database file"
                            }
                        }
                        
                        div {
                            label { 
                                style: "display: block; font-weight: 500; margin-bottom: 8px;",
                                "Task Template Directory" 
                            }
                            input {
                                r#type: "text",
                                style: "width: 100%; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                                value: "{task_template_directory}",
                                placeholder: "~/plon-templates (optional)",
                                oninput: move |e| task_template_directory.set(e.value())
                            }
                            p { style: "text-xs text-gray-500 mt-2;",
                                "Directory containing task templates (optional)"
                            }
                        }
                    }
                }
                
                // Backup Settings
                div { style: "margin-bottom: 24px;",
                    h3 { 
                        style: "font-size: 1.1rem; font-weight: 600; margin-bottom: 16px; color: #374151;",
                        "💾 Backup"
                    }
                    
                    div { style: "space-y: 16px;",
                        label { 
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: enable_auto_backup(),
                                onchange: move |_| enable_auto_backup.set(!enable_auto_backup())
                            }
                            "Enable Automatic Backups"
                        }
                        
                        if enable_auto_backup() {
                            div {
                                label { 
                                    style: "display: block; font-weight: 500; margin-bottom: 8px;",
                                    "Backup Directory" 
                                }
                                input {
                                    r#type: "text",
                                    style: "width: 100%; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                                    value: "{backup_directory}",
                                    placeholder: "~/plon-backups",
                                    oninput: move |e| backup_directory.set(e.value())
                                }
                            }
                            
                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-top: 16px;",
                                div {
                                    label { 
                                        style: "display: block; font-weight: 500; margin-bottom: 8px;",
                                        "Backup Frequency (hours)" 
                                    }
                                    input {
                                        r#type: "number",
                                        min: "1",
                                        max: "168",
                                        style: "width: 100%; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                                        value: "{backup_frequency_hours}",
                                        oninput: move |e| {
                                            if let Ok(val) = e.value().parse::<i32>() {
                                                backup_frequency_hours.set(val);
                                            }
                                        }
                                    }
                                }
                                
                                div {
                                    label { 
                                        style: "display: block; font-weight: 500; margin-bottom: 8px;",
                                        "Backups to Keep" 
                                    }
                                    input {
                                        r#type: "number",
                                        min: "1",
                                        max: "30",
                                        style: "width: 100%; padding: 8px; border: 1px solid #e5e7eb; border-radius: 6px;",
                                        value: "{max_backups_to_keep}",
                                        oninput: move |e| {
                                            if let Ok(val) = e.value().parse::<i32>() {
                                                max_backups_to_keep.set(val);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Git Integration
                div { style: "margin-bottom: 24px;",
                    h3 { 
                        style: "font-size: 1.1rem; font-weight: 600; margin-bottom: 16px; color: #374151;",
                        "🔧 Advanced"
                    }
                    
                    div { style: "space-y: 12px;",
                        label { 
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: enable_file_watching(),
                                onchange: move |_| enable_file_watching.set(!enable_file_watching())
                            }
                            "Enable File Watching"
                        }
                        p { style: "text-xs text-gray-500 ml-6;",
                            "Automatically detect changes to project files"
                        }
                        
                        label { 
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: git_auto_commit(),
                                onchange: move |_| git_auto_commit.set(!git_auto_commit())
                            }
                            "Auto-commit Changes"
                        }
                        p { style: "text-xs text-gray-500 ml-6;",
                            "Automatically commit task changes to Git"
                        }
                    }
                }
                
                // Save Button
                div { style: "display: flex; align-items: center; gap: 12px; margin-top: 24px; padding-top: 24px; border-top: 1px solid #e5e7eb;",
                    button {
                        style: "padding: 10px 20px; background: #3b82f6; color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: 500;",
                        onclick: move |_| save_settings(),
                        "Save Workspace Settings"
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