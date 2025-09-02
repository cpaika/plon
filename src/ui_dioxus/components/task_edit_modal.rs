use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::repository::Repository;
use std::sync::Arc;

#[component]
pub fn TaskEditModal(
    task: Task,
    on_save: EventHandler<Task>,
    on_cancel: EventHandler<()>,
) -> Element {
    let repository = use_context::<Arc<Repository>>();
    
    // Local state for form fields
    let mut title = use_signal(|| task.title.clone());
    let mut description = use_signal(|| task.description.clone());
    let mut estimated_hours = use_signal(|| task.estimated_hours.unwrap_or(0.0));
    let mut priority = use_signal(|| task.priority);
    let mut status = use_signal(|| task.status);
    let mut due_date = use_signal(|| task.due_date.map(|d| d.format("%Y-%m-%d").to_string()));
    let mut assignee = use_signal(|| task.assignee.clone().unwrap_or_default());
    let mut saving = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    
    // Handle save
    let handle_save = move |_| {
        spawn({
            let repository = repository.clone();
            let mut task = task.clone();
            
            async move {
                saving.set(true);
                error.set(None);
                
                // Update task with new values
                task.title = title.read().clone();
                task.description = description.read().clone();
                task.estimated_hours = if *estimated_hours.read() > 0.0 {
                    Some(*estimated_hours.read())
                } else {
                    None
                };
                task.priority = *priority.read();
                task.status = *status.read();
                
                // Parse and set due date
                if let Some(date_str) = due_date.read().as_ref() {
                    if !date_str.is_empty() {
                        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                            task.due_date = Some(date.and_hms_opt(23, 59, 59)
                                .unwrap()
                                .and_local_timezone(chrono::Local)
                                .unwrap()
                                .with_timezone(&chrono::Utc));
                        }
                    } else {
                        task.due_date = None;
                    }
                } else {
                    task.due_date = None;
                }
                
                task.assignee = if assignee.read().is_empty() {
                    None
                } else {
                    Some(assignee.read().clone())
                };
                
                task.updated_at = chrono::Utc::now();
                
                // Save to database
                match repository.tasks.update(&task).await {
                    Ok(_) => {
                        on_save.call(task);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to save: {}", e)));
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
                       width: 90%; max-width: 600px; max-height: 80vh; overflow-y: auto;
                       box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);",
                onclick: move |e| e.stop_propagation(),
                
                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                    h2 {
                        style: "margin: 0; font-size: 24px; font-weight: 600;",
                        "Edit Task"
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
                    
                    // Title field
                    div {
                        style: "margin-bottom: 20px;",
                        label {
                            style: "display: block; margin-bottom: 5px; font-weight: 500;",
                            "Title"
                        }
                        input {
                            r#type: "text",
                            style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                   border-radius: 4px; font-size: 14px;",
                            value: "{title}",
                            oninput: move |e| title.set(e.value()),
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
                                   border-radius: 4px; font-size: 14px; min-height: 120px; 
                                   resize: vertical; font-family: inherit;",
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                            placeholder: "Enter task description...",
                        }
                    }
                    
                    // Two column layout for additional fields
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
                                option { value: "Low", selected: *priority.read() == Priority::Low, "Low" }
                                option { value: "Medium", selected: *priority.read() == Priority::Medium, "Medium" }
                                option { value: "High", selected: *priority.read() == Priority::High, "High" }
                                option { value: "Critical", selected: *priority.read() == Priority::Critical, "Critical" }
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
                                option { value: "Todo", selected: *status.read() == TaskStatus::Todo, "Todo" }
                                option { value: "InProgress", selected: *status.read() == TaskStatus::InProgress, "In Progress" }
                                option { value: "Review", selected: *status.read() == TaskStatus::Review, "Review" }
                                option { value: "Done", selected: *status.read() == TaskStatus::Done, "Done" }
                                option { value: "Blocked", selected: *status.read() == TaskStatus::Blocked, "Blocked" }
                                option { value: "Cancelled", selected: *status.read() == TaskStatus::Cancelled, "Cancelled" }
                            }
                        }
                    }
                    
                    // Second row with estimated hours and due date
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
                            r#type: "button",
                            style: "padding: 8px 20px; border: 1px solid #ddd; 
                                   background: white; color: #333; border-radius: 4px; 
                                   cursor: pointer; font-size: 14px;",
                            onclick: move |_| on_cancel.call(()),
                            disabled: *saving.read(),
                            "Cancel"
                        }
                        
                        button {
                            style: "padding: 8px 20px; border: none; 
                                   background: #007bff; color: white; border-radius: 4px; 
                                   cursor: pointer; font-size: 14px;",
                            onclick: handle_save,
                            disabled: *saving.read(),
                            if *saving.read() {
                                "Saving..."
                            } else {
                                "Save"
                            }
                        }
                    }
                }
            }
        }
    }
}