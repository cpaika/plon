use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::repository::Repository;
use std::sync::Arc;

#[component]
pub fn TaskCreateModal(
    on_create: EventHandler<Task>,
    on_cancel: EventHandler<()>,
) -> Element {
    let repository = use_context::<Arc<Repository>>();
    
    // Form state
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut priority = use_signal(|| Priority::Medium);
    let mut estimated_hours = use_signal(|| 0.0f32);
    let mut due_date = use_signal(|| None::<String>);
    let mut status = use_signal(|| TaskStatus::Todo);
    let mut assignee = use_signal(String::new);
    
    // UI state
    let mut saving = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    
    // Handle save
    let handle_save = move |_| {
        // Validate required fields
        if title.read().trim().is_empty() {
            error.set(Some("Title is required".to_string()));
            return;
        }
        
        spawn({
            let repository = repository.clone();
            
            async move {
                saving.set(true);
                error.set(None);
                
                // Create new task
                let mut task = Task::new(
                    title.read().clone(),
                    description.read().clone(),
                );
                
                task.priority = *priority.read();
                task.status = *status.read();
                task.estimated_hours = if *estimated_hours.read() > 0.0 {
                    Some(*estimated_hours.read())
                } else {
                    None
                };
                
                // Parse due date if provided
                if let Some(date_str) = due_date.read().as_ref() {
                    if !date_str.is_empty() {
                        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                            task.due_date = Some(date.and_hms_opt(23, 59, 59)
                                .unwrap()
                                .and_local_timezone(chrono::Local)
                                .unwrap()
                                .with_timezone(&chrono::Utc));
                        }
                    }
                }
                
                if !assignee.read().is_empty() {
                    task.assignee = Some(assignee.read().clone());
                }
                
                // Save to database
                match repository.tasks.create(&task).await {
                    Ok(_) => {
                        on_create.call(task);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to create task: {}", e)));
                    }
                }
                
                saving.set(false);
            }
        });
    };
    
    rsx! {
        // Modal backdrop
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
                   background: rgba(0, 0, 0, 0.5); z-index: 999;
                   display: flex; align-items: center; justify-content: center;",
            onclick: move |_| on_cancel.call(()),
            
            // Modal content
            div {
                style: "background: white; border-radius: 12px; padding: 24px;
                       width: 90%; max-width: 600px; max-height: 90vh; overflow-y: auto;
                       box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);",
                onclick: move |e| e.stop_propagation(),
                
                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                    h2 {
                        style: "margin: 0; font-size: 24px; font-weight: 600;",
                        "Create New Task"
                    }
                    button {
                        style: "background: none; border: none; font-size: 24px; cursor: pointer; padding: 0; width: 30px; height: 30px;",
                        onclick: move |_| on_cancel.call(()),
                        "×"
                    }
                }
                
                // Error message
                if let Some(err) = error.read().as_ref() {
                    div {
                        style: "background: #fee; color: #c00; padding: 10px; border-radius: 4px; margin-bottom: 15px;",
                        "{err}"
                    }
                }
                
                // Form
                div {
                    // Title field (required)
                    div {
                        style: "margin-bottom: 20px;",
                        label {
                            style: "display: block; margin-bottom: 5px; font-weight: 500;",
                            "Title ",
                            span { style: "color: #e00;", "*" }
                        }
                        input {
                            r#type: "text",
                            style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                   border-radius: 4px; font-size: 14px;",
                            value: "{title}",
                            oninput: move |e| title.set(e.value()),
                            placeholder: "Enter task title...",
                            required: true,
                        }
                    }
                    
                    // Description field
                    div {
                        style: "margin-bottom: 20px;",
                        label {
                            style: "display: block; margin-bottom: 5px; font-weight: 500;",
                            "Description"
                        }
                        textarea {
                            style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                   border-radius: 4px; font-size: 14px; min-height: 100px; 
                                   resize: vertical; font-family: inherit;",
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                            placeholder: "Enter task description...",
                        }
                    }
                    
                    // Two column layout for other fields
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 20px;",
                        
                        // Priority field
                        div {
                            label {
                                style: "display: block; margin-bottom: 5px; font-weight: 500;",
                                "Priority"
                            }
                            select {
                                style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                       border-radius: 4px; font-size: 14px;",
                                value: "{priority:?}",
                                onchange: move |e| {
                                    let p = match e.value().as_str() {
                                        "Low" => Priority::Low,
                                        "Medium" => Priority::Medium,
                                        "High" => Priority::High,
                                        "Critical" => Priority::Critical,
                                        _ => Priority::Medium,
                                    };
                                    priority.set(p);
                                },
                                option { value: "Low", "Low" }
                                option { value: "Medium", selected: true, "Medium" }
                                option { value: "High", "High" }
                                option { value: "Critical", "Critical" }
                            }
                        }
                        
                        // Status field
                        div {
                            label {
                                style: "display: block; margin-bottom: 5px; font-weight: 500;",
                                "Status"
                            }
                            select {
                                style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                       border-radius: 4px; font-size: 14px;",
                                value: "{status:?}",
                                onchange: move |e| {
                                    let s = match e.value().as_str() {
                                        "Todo" => TaskStatus::Todo,
                                        "InProgress" => TaskStatus::InProgress,
                                        "Review" => TaskStatus::Review,
                                        "Done" => TaskStatus::Done,
                                        "Blocked" => TaskStatus::Blocked,
                                        "Cancelled" => TaskStatus::Cancelled,
                                        _ => TaskStatus::Todo,
                                    };
                                    status.set(s);
                                },
                                option { value: "Todo", selected: true, "Todo" }
                                option { value: "InProgress", "In Progress" }
                                option { value: "Review", "Review" }
                                option { value: "Done", "Done" }
                                option { value: "Blocked", "Blocked" }
                                option { value: "Cancelled", "Cancelled" }
                            }
                        }
                    }
                    
                    // Second row
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 20px;",
                        
                        // Estimated hours field
                        div {
                            label {
                                style: "display: block; margin-bottom: 5px; font-weight: 500;",
                                "Estimated Hours"
                            }
                            input {
                                r#type: "number",
                                style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                       border-radius: 4px; font-size: 14px;",
                                value: "{estimated_hours}",
                                oninput: move |e| {
                                    if let Ok(hours) = e.value().parse::<f32>() {
                                        estimated_hours.set(hours);
                                    }
                                },
                                min: "0",
                                step: "0.5",
                                placeholder: "0.0",
                            }
                        }
                        
                        // Due date field
                        div {
                            label {
                                style: "display: block; margin-bottom: 5px; font-weight: 500;",
                                "Due Date"
                            }
                            input {
                                r#type: "date",
                                style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                       border-radius: 4px; font-size: 14px;",
                                value: "{due_date.read().as_ref().unwrap_or(&String::new())}",
                                oninput: move |e| due_date.set(Some(e.value())),
                            }
                        }
                    }
                    
                    // Assignee field
                    div {
                        style: "margin-bottom: 20px;",
                        label {
                            style: "display: block; margin-bottom: 5px; font-weight: 500;",
                            "Assignee"
                        }
                        input {
                            r#type: "text",
                            style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                   border-radius: 4px; font-size: 14px;",
                            value: "{assignee}",
                            oninput: move |e| assignee.set(e.value()),
                            placeholder: "Enter assignee name...",
                        }
                    }
                    
                    // Buttons
                    div {
                        style: "display: flex; justify-content: flex-end; gap: 10px; margin-top: 30px;",
                        
                        button {
                            style: "padding: 8px 20px; border: 1px solid #ddd; 
                                   background: white; color: #333; border-radius: 4px; 
                                   cursor: pointer; font-size: 14px;",
                            onclick: move |_| on_cancel.call(()),
                            disabled: *saving.read(),
                            "Cancel"
                        }
                        
                        button {
                            style: "padding: 8px 20px; border: none; 
                                   background: #4CAF50; color: white; border-radius: 4px; 
                                   cursor: pointer; font-size: 14px;",
                            onclick: handle_save,
                            disabled: *saving.read(),
                            if *saving.read() {
                                "Creating..."
                            } else {
                                "Create Task"
                            }
                        }
                    }
                }
            }
        }
    }
}