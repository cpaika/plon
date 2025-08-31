use dioxus::prelude::*;
use dioxus::events::Key;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::state_simple::{TaskExecutionStatus, sample_tasks};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Utc;

#[component]
pub fn ListView() -> Element {
    // State
    let mut tasks = use_signal(|| sample_tasks());
    let mut filter_text = use_signal(|| String::new());
    let mut selected_status = use_signal(|| None::<TaskStatus>);
    let mut sort_by = use_signal(|| SortBy::Title);
    let mut selected_task = use_signal(|| None::<Uuid>);
    let mut editing_task = use_signal(|| None::<Uuid>);
    let running_tasks = use_signal(|| HashMap::<Uuid, TaskExecutionStatus>::new());
    
    // Filter and sort tasks
    let filtered_tasks = tasks.read().iter()
        .filter(|task| {
            // Text filter
            if !filter_text.read().is_empty() {
                let search = filter_text.read().to_lowercase();
                if !task.title.to_lowercase().contains(&search) &&
                   !task.description.to_lowercase().contains(&search) {
                    return false;
                }
            }
            
            // Status filter
            if let Some(status) = selected_status.read().as_ref() {
                if &task.status != status {
                    return false;
                }
            }
            
            true
        })
        .cloned()
        .collect::<Vec<_>>();
    
    let mut sorted_tasks = filtered_tasks;
    match *sort_by.read() {
        SortBy::Title => sorted_tasks.sort_by(|a, b| a.title.cmp(&b.title)),
        SortBy::Status => sorted_tasks.sort_by_key(|t| format!("{:?}", t.status)),
        SortBy::Priority => sorted_tasks.sort_by_key(|t| match t.priority {
            Priority::Critical => 0,
            Priority::High => 1,
            Priority::Medium => 2,
            Priority::Low => 3,
        }),
        SortBy::DueDate => sorted_tasks.sort_by_key(|t| t.due_date),
        SortBy::Created => sorted_tasks.sort_by_key(|t| t.created_at),
    }
    
    rsx! {
        div {
            style: "padding: 20px; max-width: 1200px; margin: 0 auto;",
            
            h2 { "Task List" }
            
            // Filters and controls
            div {
                style: "display: flex; gap: 10px; margin-bottom: 20px; padding: 15px; background: #f5f5f5; border-radius: 8px;",
                
                // Search box
                input {
                    r#type: "text",
                    placeholder: "Search tasks...",
                    value: "{filter_text}",
                    oninput: move |evt| filter_text.set(evt.value()),
                    style: "flex: 1; padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                }
                
                // Status filter
                select {
                    onchange: move |evt| {
                        selected_status.set(match evt.value().as_str() {
                            "Todo" => Some(TaskStatus::Todo),
                            "InProgress" => Some(TaskStatus::InProgress),
                            "Done" => Some(TaskStatus::Done),
                            "Blocked" => Some(TaskStatus::Blocked),
                            _ => None,
                        });
                    },
                    style: "padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                    
                    option { value: "All", "All Status" }
                    option { value: "Todo", "Todo" }
                    option { value: "InProgress", "In Progress" }
                    option { value: "Done", "Done" }
                    option { value: "Blocked", "Blocked" }
                }
                
                // Sort by
                select {
                    onchange: move |evt| {
                        sort_by.set(match evt.value().as_str() {
                            "Title" => SortBy::Title,
                            "Status" => SortBy::Status,
                            "Priority" => SortBy::Priority,
                            "DueDate" => SortBy::DueDate,
                            "Created" => SortBy::Created,
                            _ => SortBy::Title,
                        });
                    },
                    style: "padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                    
                    option { value: "Title", "Sort by Title" }
                    option { value: "Status", "Sort by Status" }
                    option { value: "Priority", "Sort by Priority" }
                    option { value: "DueDate", "Sort by Due Date" }
                    option { value: "Created", "Sort by Created" }
                }
                
                // Add task button
                button {
                    onclick: move |_| {
                        let new_task = Task::new("New Task".to_string(), String::new());
                        tasks.write().push(new_task);
                    },
                    style: "padding: 8px 16px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "➕ Add Task"
                }
                
                // Stats
                div {
                    style: "padding: 8px; background: white; border-radius: 4px; display: flex; gap: 15px; align-items: center;",
                    
                    span { "Total: {sorted_tasks.len()}" }
                    span {
                        style: "color: #666;",
                        "Todo: {sorted_tasks.iter().filter(|t| t.status == TaskStatus::Todo).count()}"
                    }
                    span {
                        style: "color: #4CAF50;",
                        "Done: {sorted_tasks.iter().filter(|t| t.status == TaskStatus::Done).count()}"
                    }
                }
            }
            
            // Task list
            div {
                style: "background: white; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                
                if sorted_tasks.is_empty() {
                    div {
                        style: "padding: 40px; text-align: center; color: #999;",
                        "No tasks found"
                    }
                } else {
                    for task in sorted_tasks {
                        TaskListItem {
                            key: "{task.id}",
                            task: task.clone(),
                            selected: selected_task.read().as_ref() == Some(&task.id),
                            editing: editing_task.read().as_ref() == Some(&task.id),
                            running: running_tasks.read().contains_key(&task.id),
                            tasks_signal: tasks.clone(),
                            selected_task_signal: selected_task.clone(),
                            editing_task_signal: editing_task.clone(),
                            running_tasks_signal: running_tasks.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TaskListItem(
    task: Task,
    selected: bool,
    editing: bool,
    running: bool,
    tasks_signal: Signal<Vec<Task>>,
    selected_task_signal: Signal<Option<Uuid>>,
    editing_task_signal: Signal<Option<Uuid>>,
    running_tasks_signal: Signal<HashMap<Uuid, TaskExecutionStatus>>,
) -> Element {
    let mut title_value = use_signal(|| task.title.clone());
    let mut description_value = use_signal(|| task.description.clone());
    
    let status_icon = match task.status {
        TaskStatus::Todo => "⭕",
        TaskStatus::InProgress => "🔄",
        TaskStatus::Done => "✅",
        TaskStatus::Blocked => "🚫",
        _ => "❓",
    };
    
    let status_color = match task.status {
        TaskStatus::Todo => "#808080",
        TaskStatus::InProgress => "#6495ff",
        TaskStatus::Done => "#64ff64",
        TaskStatus::Blocked => "#ff6464",
        _ => "#b4b4b4",
    };
    
    let priority_color = match task.priority {
        Priority::Critical => "#ff0000",
        Priority::High => "#ff8800",
        Priority::Medium => "#ffaa00",
        Priority::Low => "#888888",
    };
    
    let is_overdue = task.due_date.map_or(false, |due| due < Utc::now());
    
    rsx! {
        div {
            style: "padding: 15px; border-bottom: 1px solid #eee; cursor: pointer; 
                   background: {if selected { \"#f0f8ff\" } else { \"white\" }}; transition: background 0.2s;",
            onclick: move |_| selected_task_signal.set(Some(task.id)),
            
            div {
                style: "display: flex; align-items: center; gap: 15px;",
                
                // Status icon
                span {
                    style: "font-size: 24px; color: {status_color};",
                    "{status_icon}"
                }
                
                // Main content
                div {
                    style: "flex: 1;",
                    
                    // Title row
                    div {
                        style: "display: flex; align-items: center; gap: 10px; margin-bottom: 5px;",
                        
                        if editing {
                            input {
                                value: "{title_value}",
                                oninput: move |evt| title_value.set(evt.value()),
                                onkeydown: move |evt| {
                                    if evt.key() == Key::Enter {
                                        tasks_signal.with_mut(|tasks| {
                                            if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                                t.title = title_value.read().clone();
                                            }
                                        });
                                        editing_task_signal.set(None);
                                    }
                                },
                                style: "flex: 1; padding: 5px; font-size: 16px; font-weight: bold; 
                                       border: 1px solid #4CAF50; border-radius: 4px;",
                                autofocus: true,
                            }
                        } else {
                            h3 {
                                style: "margin: 0; font-size: 16px; flex: 1;",
                                ondoubleclick: move |_| editing_task_signal.set(Some(task.id)),
                                "{task.title}"
                            }
                        }
                        
                        // Priority badge
                        span {
                            style: "padding: 2px 8px; border-radius: 4px; font-size: 11px; 
                                  font-weight: bold; color: white; background: {priority_color};",
                            "{task.priority:?}"
                        }
                        
                        // Subtask progress
                        if !task.subtasks.is_empty() {
                            let completed = task.subtasks.iter().filter(|s| s.completed).count();
                            let total = task.subtasks.len();
                            span {
                                style: "padding: 2px 8px; background: #e3f2fd; border-radius: 4px; 
                                       font-size: 12px; color: #1976d2;",
                                "📝 {completed}/{total}"
                            }
                        }
                    }
                    
                    // Description row
                    if editing {
                        textarea {
                            value: "{description_value}",
                            oninput: move |evt| description_value.set(evt.value()),
                            onblur: move |_| {
                                tasks_signal.with_mut(|tasks| {
                                    if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                        t.description = description_value.read().clone();
                                    }
                                });
                            },
                            style: "width: 100%; padding: 5px; margin-top: 5px; border: 1px solid #ddd; 
                                   border-radius: 4px; resize: vertical; min-height: 60px;",
                        }
                    } else if !task.description.is_empty() {
                        p {
                            style: "margin: 5px 0; color: #666; font-size: 14px;",
                            "{task.description.chars().take(200).collect::<String>()}"
                            if task.description.len() > 200 { "..." }
                        }
                    }
                    
                    // Metadata row
                    div {
                        style: "display: flex; gap: 15px; margin-top: 8px; font-size: 12px; color: #888;",
                        
                        // Due date
                        if let Some(due) = task.due_date {
                            span {
                                style: if is_overdue { "color: #ff0000; font-weight: bold;" } else { "" },
                                if is_overdue { "⚠️ " } else { "📅 " }
                                "{due.format(\"%Y-%m-%d\")}"
                            }
                        }
                        
                        // Created date
                        span { "Created: {task.created_at.format(\"%Y-%m-%d\")}" }
                        
                        // Tags
                        if !task.tags.is_empty() {
                            div {
                                style: "display: flex; gap: 5px;",
                                for tag in task.tags.iter() {
                                    span {
                                        style: "padding: 2px 6px; background: #f0f0f0; border-radius: 3px;",
                                        "#{tag}"
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Action buttons
                div {
                    style: "display: flex; gap: 5px;",
                    
                    // Play button for Todo tasks
                    if task.status == TaskStatus::Todo && !running {
                        button {
                            onclick: move |evt| {
                                evt.stop_propagation();
                                running_tasks_signal.write().insert(task.id, TaskExecutionStatus::Running);
                            },
                            style: "padding: 8px; background: #4CAF50; color: white; border: none; 
                                   border-radius: 4px; cursor: pointer;",
                            title: "Start Claude Code",
                            "▶️"
                        }
                    }
                    
                    // Running indicator
                    if running {
                        div {
                            style: "padding: 8px; background: #FFA500; color: white; border-radius: 4px;",
                            "🔄"
                        }
                    }
                    
                    // Edit button
                    button {
                        onclick: move |evt| {
                            evt.stop_propagation();
                            if editing {
                                editing_task_signal.set(None);
                            } else {
                                editing_task_signal.set(Some(task.id));
                            }
                        },
                        style: "padding: 8px; background: #2196F3; color: white; border: none; 
                               border-radius: 4px; cursor: pointer;",
                        title: "Edit",
                        "✏️"
                    }
                    
                    // Status change button
                    if task.status != TaskStatus::Done {
                        button {
                            onclick: move |evt| {
                                evt.stop_propagation();
                                tasks_signal.with_mut(|tasks| {
                                    if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                        t.status = TaskStatus::Done;
                                        t.completed_at = Some(Utc::now());
                                    }
                                });
                            },
                            style: "padding: 8px; background: #4CAF50; color: white; border: none; 
                                   border-radius: 4px; cursor: pointer;",
                            title: "Mark as Done",
                            "✓"
                        }
                    }
                    
                    // Delete button
                    button {
                        onclick: move |evt| {
                            evt.stop_propagation();
                            tasks_signal.with_mut(|tasks| {
                                tasks.retain(|t| t.id != task.id);
                            });
                        },
                        style: "padding: 8px; background: #f44336; color: white; border: none; 
                               border-radius: 4px; cursor: pointer;",
                        title: "Delete",
                        "🗑"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum SortBy {
    Title,
    Status,
    Priority,
    DueDate,
    Created,
}