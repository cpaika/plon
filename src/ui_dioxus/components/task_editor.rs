use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};

#[component]
pub fn TaskEditor(
    task: Option<Task>,
    onsave: EventHandler<Task>,
    oncancel: EventHandler<()>,
) -> Element {
    let is_new = task.is_none();
    let initial_task = task.unwrap_or_else(|| Task::new("".to_string(), "".to_string()));
    
    let mut title = use_signal(|| initial_task.title.clone());
    let mut description = use_signal(|| initial_task.description.clone());
    let mut status = use_signal(|| initial_task.status);
    let mut priority = use_signal(|| initial_task.priority);
    let mut assignee = use_signal(|| initial_task.assignee.clone().unwrap_or_default());
    let mut estimated_hours = use_signal(|| {
        initial_task.estimated_hours.map(|h| h.to_string()).unwrap_or_default()
    });
    
    let save_task = move |_| {
        let mut task = initial_task.clone();
        task.title = title.read().clone();
        task.description = description.read().clone();
        task.status = *status.read();
        task.priority = *priority.read();
        task.assignee = if assignee.read().is_empty() {
            None
        } else {
            Some(assignee.read().clone())
        };
        task.estimated_hours = estimated_hours.read()
            .parse::<f32>()
            .ok()
            .filter(|&h| h > 0.0);
        task.updated_at = chrono::Utc::now();
        
        onsave.call(task);
    };
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
                   background: rgba(0,0,0,0.5); z-index: 999;",
            onclick: move |_| oncancel.call(()),
            
            div {
                style: "position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
                       width: 500px; background: white; border-radius: 12px;
                       box-shadow: 0 10px 40px rgba(0,0,0,0.2); z-index: 1000; padding: 24px;",
                onclick: move |evt| evt.stop_propagation(),
                
                h2 {
                    style: "margin: 0 0 20px 0; font-size: 24px; font-weight: 600;",
                    if is_new { "Create New Task" } else { "Edit Task" }
                }
                
                // Title input
                div {
                    style: "margin-bottom: 16px;",
                    label {
                        style: "display: block; margin-bottom: 4px; font-weight: 500;",
                        "Title *"
                    }
                    input {
                        r#type: "text",
                        value: "{title}",
                        oninput: move |evt| title.set(evt.value()),
                        style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                               border-radius: 6px; font-size: 14px;",
                        placeholder: "Enter task title..."
                    }
                }
                
                // Description input
                div {
                    style: "margin-bottom: 16px;",
                    label {
                        style: "display: block; margin-bottom: 4px; font-weight: 500;",
                        "Description"
                    }
                    textarea {
                        value: "{description}",
                        oninput: move |evt| description.set(evt.value()),
                        style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                               border-radius: 6px; font-size: 14px; min-height: 80px; resize: vertical;",
                        placeholder: "Enter task description..."
                    }
                }
                
                // Status and Priority row
                div {
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;",
                    
                    div {
                        label {
                            style: "display: block; margin-bottom: 4px; font-weight: 500;",
                            "Status"
                        }
                        select {
                            value: "{status:?}",
                            onchange: move |evt| {
                                status.set(match evt.value().as_str() {
                                    "Todo" => TaskStatus::Todo,
                                    "InProgress" => TaskStatus::InProgress,
                                    "Blocked" => TaskStatus::Blocked,
                                    "Review" => TaskStatus::Review,
                                    "Done" => TaskStatus::Done,
                                    _ => TaskStatus::Todo,
                                });
                            },
                            style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                   border-radius: 6px; font-size: 14px;",
                            option { value: "Todo", "To Do" }
                            option { value: "InProgress", "In Progress" }
                            option { value: "Blocked", "Blocked" }
                            option { value: "Review", "Review" }
                            option { value: "Done", "Done" }
                        }
                    }
                    
                    div {
                        label {
                            style: "display: block; margin-bottom: 4px; font-weight: 500;",
                            "Priority"
                        }
                        select {
                            value: "{priority:?}",
                            onchange: move |evt| {
                                priority.set(match evt.value().as_str() {
                                    "Critical" => Priority::Critical,
                                    "High" => Priority::High,
                                    "Medium" => Priority::Medium,
                                    "Low" => Priority::Low,
                                    _ => Priority::Medium,
                                });
                            },
                            style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                   border-radius: 6px; font-size: 14px;",
                            option { value: "Critical", "Critical" }
                            option { value: "High", "High" }
                            option { value: "Medium", "Medium" }
                            option { value: "Low", "Low" }
                        }
                    }
                }
                
                // Assignee and Estimated Hours row
                div {
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 20px;",
                    
                    div {
                        label {
                            style: "display: block; margin-bottom: 4px; font-weight: 500;",
                            "Assignee"
                        }
                        input {
                            r#type: "text",
                            value: "{assignee}",
                            oninput: move |evt| assignee.set(evt.value()),
                            style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                   border-radius: 6px; font-size: 14px;",
                            placeholder: "Enter assignee email..."
                        }
                    }
                    
                    div {
                        label {
                            style: "display: block; margin-bottom: 4px; font-weight: 500;",
                            "Estimated Hours"
                        }
                        input {
                            r#type: "number",
                            value: "{estimated_hours}",
                            oninput: move |evt| estimated_hours.set(evt.value()),
                            style: "width: 100%; padding: 8px 12px; border: 1px solid #ddd; 
                                   border-radius: 6px; font-size: 14px;",
                            placeholder: "0.0",
                            step: "0.5",
                            min: "0"
                        }
                    }
                }
                
                // Action buttons
                div {
                    style: "display: flex; justify-content: flex-end; gap: 12px;",
                    
                    button {
                        onclick: move |_| oncancel.call(()),
                        style: "padding: 10px 20px; background: #f5f5f5; color: #333; 
                               border: 1px solid #ddd; border-radius: 6px; cursor: pointer; 
                               font-size: 14px; font-weight: 500;",
                        "Cancel"
                    }
                    
                    button {
                        onclick: save_task,
                        disabled: title.read().is_empty(),
                        style: if title.read().is_empty() {
                            "padding: 10px 20px; background: #2196F3; color: white; 
                             border: none; border-radius: 6px; 
                             font-size: 14px; font-weight: 500;
                             opacity: 0.5; cursor: not-allowed;"
                        } else {
                            "padding: 10px 20px; background: #2196F3; color: white; 
                             border: none; border-radius: 6px; cursor: pointer; 
                             font-size: 14px; font-weight: 500;"
                        },
                        if is_new { "Create Task" } else { "Save Changes" }
                    }
                }
            }
        }
    }
}