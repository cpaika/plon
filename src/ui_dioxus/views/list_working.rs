use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;
use chrono::Utc;

#[component]
pub fn ListView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut filter = use_signal(|| String::new());
    let mut status_filter = use_signal(|| String::new());
    let mut editing_task = use_signal(|| None::<Uuid>);
    
    // Filter tasks based on search and status
    let filtered_tasks: Vec<Task> = tasks.read()
        .iter()
        .filter(|t| {
            let text_match = filter.read().is_empty() || 
                t.title.to_lowercase().contains(&filter.read().to_lowercase()) ||
                t.description.to_lowercase().contains(&filter.read().to_lowercase());
            
            let status_match = status_filter.read().is_empty() ||
                (status_filter.read() == "Todo" && t.status == TaskStatus::Todo) ||
                (status_filter.read() == "InProgress" && t.status == TaskStatus::InProgress) ||
                (status_filter.read() == "Done" && t.status == TaskStatus::Done) ||
                (status_filter.read() == "Blocked" && t.status == TaskStatus::Blocked);
            
            text_match && status_match
        })
        .cloned()
        .collect();
    
    // Count tasks by status
    let todo_count = tasks.read().iter().filter(|t| t.status == TaskStatus::Todo).count();
    let in_progress_count = tasks.read().iter().filter(|t| t.status == TaskStatus::InProgress).count();
    let done_count = tasks.read().iter().filter(|t| t.status == TaskStatus::Done).count();
    
    rsx! {
        div {
            style: "padding: 20px; max-width: 1000px; margin: 0 auto;",
            
            h2 { "Task List" }
            
            // Filters
            div {
                style: "margin-bottom: 20px; padding: 15px; background: #f5f5f5; border-radius: 8px; display: flex; gap: 10px;",
                
                input {
                    r#type: "text",
                    placeholder: "Search tasks...",
                    value: "{filter}",
                    oninput: move |evt| filter.set(evt.value()),
                    style: "flex: 1; padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                }
                
                select {
                    value: "{status_filter}",
                    onchange: move |evt| status_filter.set(evt.value()),
                    style: "padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                    
                    option { value: "", "All Status" }
                    option { value: "Todo", "Todo" }
                    option { value: "InProgress", "In Progress" }
                    option { value: "Done", "Done" }
                    option { value: "Blocked", "Blocked" }
                }
                
                button {
                    onclick: move |_| {
                        let new_task = Task::new("New Task".to_string(), String::new());
                        tasks.write().push(new_task);
                    },
                    style: "padding: 8px 16px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Add Task"
                }
            }
            
            // Stats
            div {
                style: "margin-bottom: 20px; padding: 10px; background: white; border-radius: 8px; display: flex; gap: 20px;",
                
                span { "Total: {filtered_tasks.len()}" }
                span { style: "color: #808080;", "Todo: {todo_count}" }
                span { style: "color: #2196F3;", "In Progress: {in_progress_count}" }
                span { style: "color: #4CAF50;", "Done: {done_count}" }
            }
            
            // Task list
            div {
                style: "background: white; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                
                if filtered_tasks.is_empty() {
                    div {
                        style: "padding: 40px; text-align: center; color: #999;",
                        "No tasks found"
                    }
                } else {
                    for task in filtered_tasks {
                        TaskRow {
                            task: task.clone(),
                            editing: editing_task.read().as_ref() == Some(&task.id),
                            onedit: move |_| {
                                if editing_task.read().as_ref() == Some(&task.id) {
                                    editing_task.set(None);
                                } else {
                                    editing_task.set(Some(task.id));
                                }
                            },
                            onstatuschange: move |new_status| {
                                tasks.with_mut(|tasks| {
                                    if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                        t.status = new_status;
                                        if new_status == TaskStatus::Done {
                                            t.completed_at = Some(Utc::now());
                                        }
                                    }
                                });
                            },
                            ondelete: move |_| {
                                tasks.with_mut(|tasks| {
                                    tasks.retain(|t| t.id != task.id);
                                });
                            },
                            onsave: move |new_title| {
                                tasks.with_mut(|tasks| {
                                    if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                        t.title = new_title;
                                    }
                                });
                                editing_task.set(None);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TaskRow(
    task: Task,
    editing: bool,
    onedit: EventHandler<MouseEvent>,
    onstatuschange: EventHandler<TaskStatus>,
    ondelete: EventHandler<MouseEvent>,
    onsave: EventHandler<String>
) -> Element {
    let mut edit_value = use_signal(|| task.title.clone());
    
    rsx! {
        div {
            style: "padding: 15px; border-bottom: 1px solid #eee; display: flex; align-items: center; gap: 15px;",
            
            // Status icon
            div {
                style: "font-size: 24px; cursor: pointer;",
                onclick: move |_| {
                    let new_status = match task.status {
                        TaskStatus::Todo => TaskStatus::InProgress,
                        TaskStatus::InProgress => TaskStatus::Done,
                        TaskStatus::Done => TaskStatus::Todo,
                        _ => TaskStatus::Todo,
                    };
                    onstatuschange.call(new_status);
                },
                "{status_icon}"
            }
            
            // Task content
            div {
                style: "flex: 1;",
                
                if editing {
                    div {
                        style: "display: flex; gap: 10px;",
                        
                        input {
                            value: "{edit_value}",
                            oninput: move |evt| edit_value.set(evt.value()),
                            style: "flex: 1; padding: 5px; font-size: 16px; font-weight: bold; border: 1px solid #4CAF50; border-radius: 4px;",
                            autofocus: true,
                        }
                        
                        button {
                            onclick: move |_| onsave.call(edit_value.read().clone()),
                            style: "padding: 5px 10px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            "Save"
                        }
                        
                        button {
                            onclick: move |evt| onedit.call(evt),
                            style: "padding: 5px 10px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            "Cancel"
                        }
                    }
                } else {
                    h4 {
                        style: "margin: 0; cursor: pointer;",
                        ondoubleclick: move |evt| onedit.call(evt),
                        "{task.title}"
                    }
                }
                
                if !task.description.is_empty() {
                    p {
                        style: "margin: 5px 0; color: #666; font-size: 14px;",
                        "{task.description}"
                    }
                }
                
                div {
                    style: "display: flex; gap: 10px; margin-top: 5px;",
                    
                    span {
                        style: "font-size: 12px; padding: 2px 6px; background: {priority_color}; color: white; border-radius: 3px;",
                        "{task.priority:?}"
                    }
                    
                    if let Some(due) = task.due_date {
                        span {
                            style: "font-size: 12px; color: {if is_overdue { \"#ff0000\" } else { \"#666\" }}; font-weight: {if is_overdue { \"bold\" } else { \"normal\" }};",
                            "📅 {due.format(\"%Y-%m-%d\")}"
                        }
                    }
                    
                    if !task.subtasks.is_empty() {
                        let completed = task.subtasks.iter().filter(|s| s.completed).count();
                        let total = task.subtasks.len();
                        span {
                            style: "font-size: 12px; color: #2196F3;",
                            "✓ {completed}/{total}"
                        }
                    }
                    
                    if !task.tags.is_empty() {
                        for tag in task.tags.iter() {
                            span {
                                style: "font-size: 12px; padding: 2px 6px; background: #f0f0f0; border-radius: 3px;",
                                "#{tag}"
                            }
                        }
                    }
                }
            }
            
            // Actions
            div {
                style: "display: flex; gap: 5px;",
                
                if !editing {
                    button {
                        onclick: move |evt| onedit.call(evt),
                        style: "padding: 6px 12px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "Edit"
                    }
                }
                
                button {
                    onclick: move |evt| ondelete.call(evt),
                    style: "padding: 6px 12px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Delete"
                }
            }
        }
    }
}