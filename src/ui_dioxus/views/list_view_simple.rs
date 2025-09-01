use dioxus::prelude::*;
use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use crate::domain::task::{Task, TaskStatus, Priority};
use std::sync::Arc;
use chrono::Local;

#[component]
pub fn ListView() -> Element {
    // Get repository from context
    let repository = use_context::<Arc<Repository>>();
    let mut tasks = use_signal(|| Vec::<Task>::new());
    let mut loading = use_signal(|| true);
    let mut error_message = use_signal(String::new);
    let mut filter_status = use_signal(|| "all".to_string());
    let mut selected_task_id = use_signal(|| None::<uuid::Uuid>);
    
    // Load tasks on mount and when filter changes
    use_effect({
        let repo = repository.clone();
        let filter = filter_status.clone();
        move || {
            let repo = repo.clone();
            let filter_value = filter();
            spawn(async move {
                loading.set(true);
                let filters = TaskFilters {
                    status: if filter_value == "all" { 
                        None 
                    } else {
                        Some(match filter_value.as_str() {
                            "todo" => TaskStatus::Todo,
                            "in_progress" => TaskStatus::InProgress,
                            "done" => TaskStatus::Done,
                            "blocked" => TaskStatus::Blocked,
                            _ => TaskStatus::Todo,
                        })
                    },
                    assigned_resource_id: None,
                    goal_id: None,
                    overdue: false,
                    limit: None,
                };
                
                match repo.tasks.list(filters).await {
                    Ok(mut task_list) => {
                        // Sort by created date (newest first)
                        task_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                        
                        tasks.set(task_list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error_message.set(format!("Failed to load tasks: {}", e));
                        loading.set(false);
                    }
                }
            });
        }
    });
    
    // Handle task status update
    let update_task_status = move |(task_id, new_status): (uuid::Uuid, TaskStatus)| {
        let repo = repository.clone();
        let current_tasks = tasks.clone();
        spawn(async move {
            if let Some(task) = current_tasks().iter().find(|t| t.id == task_id) {
                let mut updated_task = task.clone();
                updated_task.status = new_status;
                updated_task.updated_at = chrono::Utc::now();
                
                match repo.tasks.update(&updated_task).await {
                    Ok(_) => {
                        // Update local state
                        let mut task_list = current_tasks();
                        if let Some(task) = task_list.iter_mut().find(|t| t.id == task_id) {
                            *task = updated_task;
                        }
                        tasks.set(task_list);
                    }
                    Err(e) => {
                        error_message.set(format!("Failed to update task: {}", e));
                    }
                }
            }
        });
    };
    
    rsx! {
        div {
            class: "list-view",
            style: "padding: 20px; max-width: 1200px; margin: 0 auto;",
            
            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; padding: 15px; background: white; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);",
                
                h1 { 
                    style: "font-size: 1.8rem; font-weight: bold;",
                    "📋 Task List" 
                }
                
                div {
                    style: "display: flex; gap: 15px; align-items: center;",
                    
                    // Status filter
                    select {
                        style: "padding: 8px 12px; border: 1px solid #e5e7eb; border-radius: 6px;",
                        value: "{filter_status}",
                        onchange: move |e| filter_status.set(e.value()),
                        option { value: "all", "All Tasks" }
                        option { value: "todo", "Todo" }
                        option { value: "in_progress", "In Progress" }
                        option { value: "done", "Done" }
                        option { value: "blocked", "Blocked" }
                    }
                    
                    // Task count
                    div {
                        style: "padding: 6px 12px; background: #f3f4f6; border-radius: 6px;",
                        "{tasks().len()} tasks"
                    }
                }
            }
            
            // Error message
            if !error_message().is_empty() {
                div {
                    style: "padding: 12px; background: #fee2e2; border: 1px solid #fca5a5; border-radius: 6px; color: #991b1b; margin-bottom: 20px;",
                    "{error_message}"
                }
            }
            
            // Loading state
            if loading() {
                div {
                    style: "text-align: center; padding: 40px; color: #6b7280;",
                    "Loading tasks..."
                }
            } else if tasks().is_empty() {
                // Empty state
                div {
                    style: "text-align: center; padding: 60px; background: white; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);",
                    div { style: "font-size: 3rem; margin-bottom: 16px;", "📝" }
                    h3 { style: "font-size: 1.2rem; font-weight: 600; margin-bottom: 8px;", "No tasks found" }
                    p { style: "color: #6b7280;", "Create your first task to get started" }
                }
            } else {
                // Task list
                div {
                    for task in tasks() {
                        TaskCard {
                            task: task.clone(),
                            selected: selected_task_id() == Some(task.id),
                            on_select: move |id| selected_task_id.set(Some(id)),
                            on_status_change: update_task_status.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TaskCard(
    task: Task,
    selected: bool,
    on_select: EventHandler<uuid::Uuid>,
    on_status_change: EventHandler<(uuid::Uuid, TaskStatus)>,
) -> Element {
    let status_color = match task.status {
        TaskStatus::Todo => ("#94a3b8", "#f1f5f9"),
        TaskStatus::InProgress => ("#3b82f6", "#dbeafe"),
        TaskStatus::Done => ("#10b981", "#d1fae5"),
        TaskStatus::Blocked => ("#ef4444", "#fee2e2"),
        TaskStatus::Review => ("#f59e0b", "#fef3c7"),
        TaskStatus::Cancelled => ("#6b7280", "#f3f4f6"),
    };
    
    rsx! {
        div {
            style: format!(
                "padding: 16px; background: {}; border: 2px solid {}; border-radius: 8px; margin-bottom: 12px; cursor: pointer; transition: all 0.2s;",
                if selected { "#f0f9ff" } else { "white" },
                if selected { "#3b82f6" } else { "#e5e7eb" }
            ),
            onclick: move |_| on_select.call(task.id),
            
            div {
                style: "display: flex; justify-content: space-between; align-items: start;",
                
                div {
                    style: "flex: 1;",
                    
                    // Header
                    div {
                        style: "display: flex; align-items: center; gap: 12px; margin-bottom: 8px;",
                        
                        // Status badge
                        div {
                            style: format!(
                                "padding: 4px 10px; background: {}; color: {}; border-radius: 4px; font-size: 12px; font-weight: 600;",
                                status_color.1, status_color.0
                            ),
                            "{task.status:?}"
                        }
                        
                        // Title
                        h3 {
                            style: "font-size: 1.1rem; font-weight: 600;",
                            "{task.title}"
                        }
                        
                        // Priority
                        div {
                            style: format!(
                                "padding: 2px 8px; background: {}; color: white; border-radius: 4px; font-size: 11px;",
                                match task.priority {
                                    Priority::Critical => "#dc2626",
                                    Priority::High => "#ef4444",
                                    Priority::Medium => "#f59e0b", 
                                    Priority::Low => "#6b7280",
                                }
                            ),
                            "{task.priority:?}"
                        }
                    }
                    
                    // Description
                    if !task.description.is_empty() {
                        p {
                            style: "color: #6b7280; margin-bottom: 8px;",
                            "{task.description}"
                        }
                    }
                    
                    // Metadata
                    div {
                        style: "display: flex; gap: 20px; font-size: 13px; color: #9ca3af;",
                        
                        div {
                            "Created: {task.created_at.with_timezone(&Local).format(\"%b %d, %Y\")}"
                        }
                        
                        if let Some(due) = task.due_date {
                            div {
                                style: if due < chrono::Utc::now() { "color: #ef4444;" } else { "" },
                                "Due: {due.with_timezone(&Local).format(\"%b %d, %Y\")}"
                            }
                        }
                    }
                }
                
                // Status selector
                select {
                    style: "padding: 6px; border: 1px solid #e5e7eb; border-radius: 4px; font-size: 14px;",
                    value: format!("{:?}", task.status),
                    onclick: move |e| e.stop_propagation(),
                    onchange: move |e| {
                        let new_status = match e.value().as_str() {
                            "Todo" => TaskStatus::Todo,
                            "InProgress" => TaskStatus::InProgress,
                            "Done" => TaskStatus::Done,
                            "Blocked" => TaskStatus::Blocked,
                            "Review" => TaskStatus::Review,
                            "Cancelled" => TaskStatus::Cancelled,
                            _ => return,
                        };
                        on_status_change.call((task.id, new_status));
                    },
                    option { value: "Todo", "Todo" }
                    option { value: "InProgress", "In Progress" }
                    option { value: "Done", "Done" }
                    option { value: "Blocked", "Blocked" }
                    option { value: "Review", "Review" }
                    option { value: "Cancelled", "Cancelled" }
                }
            }
        }
    }
}