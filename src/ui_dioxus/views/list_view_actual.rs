use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;
use chrono::Utc;

#[component]
pub fn ListView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut filter = use_signal(|| String::new());
    let mut editing_task: Signal<Option<Uuid>> = use_signal(|| None);
    
    // Pre-filter tasks before the rsx! macro
    let filter_text = filter.read().clone();
    let all_tasks = tasks.read().clone();
    let filtered_tasks: Vec<Task> = all_tasks
        .into_iter()
        .filter(|t| filter_text.is_empty() || t.title.to_lowercase().contains(&filter_text.to_lowercase()))
        .collect();
    
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
                
                button {
                    onclick: move |_| {
                        let new_task = Task::new("New Task".to_string(), String::new());
                        tasks.write().push(new_task);
                    },
                    style: "padding: 8px 16px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Add Task"
                }
            }
            
            // Stats bar
            div {
                style: "margin-bottom: 20px; padding: 10px; background: white; border-radius: 8px; display: flex; gap: 20px;",
                
                span { "Total: {filtered_tasks.len()}" }
                span { 
                    style: "color: #2196F3;", 
                    "In Progress: {filtered_tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count()}" 
                }
                span { 
                    style: "color: #4CAF50;", 
                    "Completed: {filtered_tasks.iter().filter(|t| t.status == TaskStatus::Done).count()}" 
                }
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
                    TaskItem {
                        task: task.clone(),
                        is_editing: editing_task.read().as_ref() == Some(&task.id),
                        on_edit_toggle: move |_| {
                            if editing_task.read().as_ref() == Some(&task.id) {
                                editing_task.set(None);
                            } else {
                                editing_task.set(Some(task.id));
                            }
                        },
                        on_status_change: move |_| {
                            tasks.with_mut(|tasks| {
                                if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                    t.status = match t.status {
                                        TaskStatus::Todo => TaskStatus::InProgress,
                                        TaskStatus::InProgress => TaskStatus::Done,
                                        TaskStatus::Done => TaskStatus::Todo,
                                        _ => TaskStatus::Todo,
                                    };
                                    if t.status == TaskStatus::Done {
                                        t.completed_at = Some(Utc::now());
                                    }
                                }
                            });
                        },
                        on_delete: move |_| {
                            tasks.with_mut(|tasks| {
                                tasks.retain(|t| t.id != task.id);
                            });
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn TaskItem(
    task: Task,
    is_editing: bool,
    on_edit_toggle: EventHandler<()>,
    on_status_change: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "padding: 15px; border-bottom: 1px solid #eee; display: flex; align-items: center; gap: 15px;",
            
            // Status button
            button {
                onclick: move |_| on_status_change.call(()),
                style: "padding: 8px; background: none; border: none; font-size: 24px; cursor: pointer;",
                {match task.status {
                    TaskStatus::Todo => "⭕",
                    TaskStatus::InProgress => "🔄",
                    TaskStatus::Done => "✅",
                    TaskStatus::Blocked => "🚫",
                    _ => "❓",
                }}
            }
            
            // Task content
            div {
                style: "flex: 1;",
                
                if is_editing {
                    input {
                        value: "{task.title}",
                        onblur: move |_| on_edit_toggle.call(()),
                        style: "width: 100%; padding: 5px; font-size: 16px; font-weight: bold; border: 1px solid #4CAF50; border-radius: 4px;",
                        autofocus: true,
                    }
                } else {
                    h4 {
                        style: "margin: 0; cursor: pointer;",
                        onclick: move |_| on_edit_toggle.call(()),
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
                        style: "font-size: 12px; padding: 2px 6px; background: {match task.priority {
                            Priority::Critical => \"#ff0000\",
                            Priority::High => \"#ff8800\",
                            Priority::Medium => \"#ffaa00\",
                            Priority::Low => \"#888888\",
                        }}; color: white; border-radius: 3px;",
                        {match task.priority {
                            Priority::Critical => "Critical",
                            Priority::High => "High",
                            Priority::Medium => "Medium",
                            Priority::Low => "Low",
                        }}
                    }
                    
                    if let Some(due) = task.due_date {
                        span {
                            style: "font-size: 12px; color: {if due < Utc::now() { \"#ff0000\" } else { \"#666\" }};",
                            "📅 Due: {due.format(\"%Y-%m-%d\")}"
                        }
                    }
                    
                    if !task.subtasks.is_empty() {
                        let completed = task.subtasks.iter().filter(|s| s.completed).count();
                        let total = task.subtasks.len();
                        span {
                            style: "font-size: 12px; color: #2196F3;",
                            "✓ {completed}/{total} subtasks"
                        }
                    }
                }
            }
            
            // Actions
            div {
                style: "display: flex; gap: 5px;",
                
                button {
                    onclick: move |_| on_edit_toggle.call(()),
                    style: "padding: 6px 12px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Edit"
                }
                
                button {
                    onclick: move |_| on_delete.call(()),
                    style: "padding: 6px 12px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Delete"
                }
            }
        }
    }
}