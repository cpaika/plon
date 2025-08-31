use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;
use chrono::Utc;

#[component]
pub fn ListView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut filter = use_signal(|| String::new());
    let mut status_filter = use_signal(|| None::<TaskStatus>);
    let mut editing_task = use_signal(|| None::<Uuid>);
    
    let filtered_tasks = tasks.read().iter()
        .filter(|t| {
            let text_match = filter.read().is_empty() || 
                t.title.to_lowercase().contains(&filter.read().to_lowercase()) ||
                t.description.to_lowercase().contains(&filter.read().to_lowercase());
            
            let status_match = status_filter.read().as_ref()
                .map_or(true, |s| &t.status == s);
            
            text_match && status_match
        })
        .cloned()
        .collect::<Vec<_>>();
    
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
                    onchange: move |evt| {
                        status_filter.set(match evt.value().as_str() {
                            "Todo" => Some(TaskStatus::Todo),
                            "InProgress" => Some(TaskStatus::InProgress),
                            "Done" => Some(TaskStatus::Done),
                            "Blocked" => Some(TaskStatus::Blocked),
                            _ => None,
                        });
                    },
                    style: "padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                    
                    option { value: "", "All Status" }
                    option { value: "Todo", "Todo" }
                    option { value: "InProgress", "In Progress" }
                    option { value: "Done", "Done" }
                    option { value: "Blocked", "Blocked" }
                }
                
                button {
                    onclick: move |_| {
                        tasks.write().push(Task::new("New Task".to_string(), String::new()));
                    },
                    style: "padding: 8px 16px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Add Task"
                }
            }
            
            // Stats
            div {
                style: "margin-bottom: 20px; padding: 10px; background: white; border-radius: 8px; display: flex; gap: 20px;",
                
                span { "Total: {filtered_tasks.len()}" }
                span { style: "color: #2196F3;", "In Progress: {filtered_tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count()}" }
                span { style: "color: #4CAF50;", "Done: {filtered_tasks.iter().filter(|t| t.status == TaskStatus::Done).count()}" }
            }
            
            // Task list
            div {
                style: "background: white; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                
                if filtered_tasks.is_empty() {
                    div {
                        style: "padding: 40px; text-align: center; color: #999;",
                        "No tasks found"
                    }
                }
                
                for task in filtered_tasks {
                    div {
                        key: "{task.id}",
                        style: "padding: 15px; border-bottom: 1px solid #eee; display: flex; align-items: center; gap: 15px;",
                        
                        // Status indicator
                        div {
                            style: "font-size: 24px;",
                            "{match task.status {
                                TaskStatus::Todo => \"⭕\",
                                TaskStatus::InProgress => \"🔄\",
                                TaskStatus::Done => \"✅\",
                                TaskStatus::Blocked => \"🚫\",
                                _ => \"❓\",
                            }}"
                        }
                        
                        // Task content
                        div {
                            style: "flex: 1;",
                            
                            if editing_task.read().as_ref() == Some(&task.id) {
                                input {
                                    value: "{task.title}",
                                    onkeydown: move |evt| {
                                        if evt.key() == dioxus::events::Key::Enter {
                                            editing_task.set(None);
                                        }
                                    },
                                    onblur: move |_| editing_task.set(None),
                                    style: "width: 100%; padding: 5px; font-size: 16px; font-weight: bold; border: 1px solid #4CAF50; border-radius: 4px;",
                                    autofocus: true,
                                }
                            } else {
                                h4 {
                                    style: "margin: 0; cursor: pointer;",
                                    ondoubleclick: move |_| editing_task.set(Some(task.id)),
                                    "{task.title}"
                                }
                            }
                            
                            if !task.description.is_empty() {
                                p {
                                    style: "margin: 5px 0; color: #666; font-size: 14px;",
                                    "{task.description.chars().take(150).collect::<String>()}"
                                }
                            }
                            
                            div {
                                style: "display: flex; gap: 10px; margin-top: 5px;",
                                
                                span {
                                    style: "font-size: 12px; padding: 2px 6px; background: {match task.priority {
                                        Priority::Critical => \"#ff0000\",
                                        Priority::High => \"#ff8800\",
                                        Priority::Medium => \"#ffaa00\",
                                        Priority::Low => \"#888888\",
                                    }}; color: white; border-radius: 3px;",
                                    "{task.priority:?}"
                                }
                                
                                if let Some(due) = task.due_date {
                                    span {
                                        style: "font-size: 12px; color: {if due < Utc::now() { \"#ff0000\" } else { \"#666\" }};",
                                        "📅 {due.format(\"%Y-%m-%d\")}"
                                    }
                                }
                                
                                if !task.subtasks.is_empty() {
                                    let completed = task.subtasks.iter().filter(|s| s.completed).count();
                                    span {
                                        style: "font-size: 12px; color: #2196F3;",
                                        "✓ {completed}/{task.subtasks.len()}"
                                    }
                                }
                            }
                        }
                        
                        // Actions
                        div {
                            style: "display: flex; gap: 5px;",
                            
                            if task.status != TaskStatus::Done {
                                button {
                                    onclick: move |_| {
                                        tasks.with_mut(|tasks| {
                                            if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                                t.status = TaskStatus::Done;
                                                t.completed_at = Some(Utc::now());
                                            }
                                        });
                                    },
                                    style: "padding: 6px 12px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                    "✓ Done"
                                }
                            }
                            
                            button {
                                onclick: move |_| {
                                    tasks.with_mut(|tasks| {
                                        tasks.retain(|t| t.id != task.id);
                                    });
                                },
                                style: "padding: 6px 12px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}