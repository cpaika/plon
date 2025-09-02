use dioxus::prelude::*;
use dioxus::events::Key;
use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::components::{TaskEditModal, TaskCreateModal, ExportButton};
use crate::services::TimeTrackingService;
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
    let mut editing_task = use_signal(|| None::<Task>);
    let mut creating_task = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let mut sort_by = use_signal(|| "created_desc".to_string());
    let mut selected_tasks = use_signal(|| std::collections::HashSet::<uuid::Uuid>::new());
    let mut bulk_mode = use_signal(|| false);
    
    // Keyboard shortcuts
    let handle_keydown = move |e: KeyboardEvent| {
        let key = e.key();
        let ctrl_or_cmd = if cfg!(target_os = "macos") { e.modifiers().meta() } else { e.modifiers().ctrl() };
        
        match key {
            // Cmd/Ctrl + N - New task
            Key::Character(c) if (c == "n" || c == "N") && ctrl_or_cmd => {
                e.stop_propagation();
                creating_task.set(true);
            },
            // Cmd/Ctrl + F - Focus search
            Key::Character(c) if (c == "f" || c == "F") && ctrl_or_cmd => {
                e.stop_propagation();
                // Would need a ref to focus the search input
            },
            // Escape - Clear selection or close modals
            Key::Escape => {
                if editing_task().is_some() {
                    editing_task.set(None);
                } else if *creating_task.read() {
                    creating_task.set(false);
                } else if *bulk_mode.read() {
                    bulk_mode.set(false);
                    selected_tasks.set(std::collections::HashSet::new());
                } else {
                    selected_task_id.set(None);
                }
            },
            // Cmd/Ctrl + A - Select all
            Key::Character(c) if (c == "a" || c == "A") && ctrl_or_cmd => {
                e.stop_propagation();
                if !*bulk_mode.read() {
                    bulk_mode.set(true);
                }
                let all_task_ids: std::collections::HashSet<_> = tasks().iter().map(|t| t.id).collect();
                selected_tasks.set(all_task_ids);
            },
            // Delete - Delete selected task (with confirmation)
            Key::Delete if selected_task_id().is_some() => {
                // Would implement delete with confirmation
            },
            // Arrow keys - Navigate tasks
            Key::ArrowDown => {
                let current_tasks = tasks();
                if !current_tasks.is_empty() {
                    if let Some(current_id) = selected_task_id() {
                        if let Some(current_idx) = current_tasks.iter().position(|t| t.id == current_id) {
                            if current_idx < current_tasks.len() - 1 {
                                selected_task_id.set(Some(current_tasks[current_idx + 1].id));
                            }
                        }
                    } else {
                        selected_task_id.set(Some(current_tasks[0].id));
                    }
                }
            },
            Key::ArrowUp => {
                let current_tasks = tasks();
                if !current_tasks.is_empty() {
                    if let Some(current_id) = selected_task_id() {
                        if let Some(current_idx) = current_tasks.iter().position(|t| t.id == current_id) {
                            if current_idx > 0 {
                                selected_task_id.set(Some(current_tasks[current_idx - 1].id));
                            }
                        }
                    } else {
                        selected_task_id.set(Some(current_tasks[current_tasks.len() - 1].id));
                    }
                }
            },
            // Enter - Edit selected task
            Key::Enter if selected_task_id().is_some() => {
                if let Some(task_id) = selected_task_id() {
                    if let Some(task) = tasks().into_iter().find(|t| t.id == task_id) {
                        editing_task.set(Some(task));
                    }
                }
            },
            _ => {}
        }
    };
    
    // Load tasks on mount and when filter changes
    use_effect({
        let repo = repository.clone();
        move || {
            let repo = repo.clone();
            let filter_val = filter_status();
            spawn(async move {
            loading.set(true);
            let filters = TaskFilters {
                status: if filter_val == "all" { 
                    None 
                } else {
                    Some(match filter_val.as_str() {
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
                Ok(task_list) => {
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
    let update_task_status = {
        let repo = repository.clone();
        move |(task_id, new_status): (uuid::Uuid, TaskStatus)| {
            let repo = repo.clone();
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
        }
    };
    
    rsx! {
        div {
            class: "list-view",
            style: "padding: 20px; max-width: 1200px; margin: 0 auto;",
            onkeydown: handle_keydown,
            tabindex: "0",
            
            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; padding: 15px; background: white; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);",
                
                h1 { 
                    style: "font-size: 1.8rem; font-weight: bold;",
                    "📋 Task List" 
                }
                
                div {
                    style: "display: flex; gap: 15px; align-items: center;",
                    
                    // Export button
                    ExportButton {}
                    
                    // Bulk mode toggle
                    button {
                        style: format!(
                            "padding: 8px 16px; background: {}; color: {}; 
                             border: 1px solid {}; border-radius: 6px; cursor: pointer; 
                             font-size: 14px; font-weight: 500;",
                            if *bulk_mode.read() { "#3b82f6" } else { "white" },
                            if *bulk_mode.read() { "white" } else { "#333" },
                            if *bulk_mode.read() { "#3b82f6" } else { "#e5e7eb" }
                        ),
                        onclick: move |_| {
                            let new_mode = !*bulk_mode.read();
                            bulk_mode.set(new_mode);
                            if !new_mode {
                                selected_tasks.set(std::collections::HashSet::new());
                            }
                        },
                        "✓ Select Multiple"
                    }
                    
                    // Bulk actions (visible when in bulk mode)
                    if *bulk_mode.read() && !selected_tasks.read().is_empty() {
                        div {
                            style: "display: flex; gap: 10px;",
                            
                            span {
                                style: "padding: 8px 12px; background: #f3f4f6; border-radius: 6px; font-size: 14px;",
                                "{selected_tasks.read().len()} selected"
                            }
                            
                            button {
                                style: "padding: 8px 12px; background: #10b981; color: white; 
                                       border: none; border-radius: 6px; cursor: pointer; font-size: 14px;",
                                onclick: {
                                    let repo = repository.clone();
                                    let selected = selected_tasks.read().clone();
                                    let current_tasks = tasks().clone();
                                    move |_| {
                                        let repo = repo.clone();
                                        let selected = selected.clone();
                                        let current_tasks = current_tasks.clone();
                                        spawn(async move {
                                            for task_id in selected {
                                                if let Some(task) = current_tasks.iter().find(|t| t.id == task_id) {
                                                    let mut updated_task = task.clone();
                                                    updated_task.status = TaskStatus::Done;
                                                    updated_task.updated_at = chrono::Utc::now();
                                                    let _ = repo.tasks.update(&updated_task).await;
                                                }
                                            }
                                        });
                                        // Clear selections after spawning
                                        selected_tasks.set(std::collections::HashSet::new());
                                        bulk_mode.set(false);
                                    }
                                },
                                "Mark as Done"
                            }
                            
                            button {
                                style: "padding: 8px 12px; background: #ef4444; color: white; 
                                       border: none; border-radius: 6px; cursor: pointer; font-size: 14px;",
                                onclick: move |_| {
                                    // Would show confirmation dialog
                                    selected_tasks.set(std::collections::HashSet::new());
                                },
                                "Delete"
                            }
                        }
                    }
                    
                    // Create button with tooltip
                    div {
                        style: "position: relative;",
                        button {
                            style: "padding: 8px 16px; background: #4CAF50; color: white; 
                                   border: none; border-radius: 6px; cursor: pointer; 
                                   font-size: 14px; font-weight: 500; display: flex; 
                                   align-items: center; gap: 6px;",
                            onclick: move |_| creating_task.set(true),
                            title: if cfg!(target_os = "macos") { "⌘N" } else { "Ctrl+N" },
                            "➕ New Task"
                        }
                    }
                    
                    // Search input
                    input {
                        r#type: "text",
                        style: "padding: 8px 12px; border: 1px solid #e5e7eb; border-radius: 6px; 
                               width: 250px; font-size: 14px;",
                        placeholder: "Search tasks...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                    
                    // Sort dropdown
                    select {
                        style: "padding: 8px 12px; border: 1px solid #e5e7eb; border-radius: 6px;",
                        value: "{sort_by}",
                        onchange: move |e| sort_by.set(e.value()),
                        option { value: "created_desc", "Newest First" }
                        option { value: "created_asc", "Oldest First" }
                        option { value: "due_asc", "Due Date (Earliest)" }
                        option { value: "due_desc", "Due Date (Latest)" }
                        option { value: "priority_desc", "Priority (Highest)" }
                        option { value: "priority_asc", "Priority (Lowest)" }
                        option { value: "title_asc", "Title (A-Z)" }
                        option { value: "title_desc", "Title (Z-A)" }
                        option { value: "status", "Status" }
                    }
                    
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
                    
                    // Task count - use memoized count
                    div {
                        style: "padding: 6px 12px; background: #f3f4f6; border-radius: 6px;",
                        {
                            // Use memoized task count to avoid recalculating
                            let count = use_memo(move || {
                                let query = search_query.read().to_lowercase();
                                if query.is_empty() {
                                    tasks().len()
                                } else {
                                    tasks().into_iter().filter(|task| {
                                        task.title.to_lowercase().contains(&query) ||
                                        task.description.to_lowercase().contains(&query) ||
                                        task.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
                                    }).count()
                                }
                            });
                            format!("{} tasks", count())
                        }
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
                // Task list with memoized filtering and sorting
                div {
                    {
                        // Use memo to compute filtered and sorted tasks only when dependencies change
                        let filtered_sorted_tasks = use_memo(move || {
                            let query = search_query.read().to_lowercase();
                            let sort_value = sort_by.read().clone();
                            let all_tasks = tasks();
                            
                            // Filter tasks
                            let mut filtered_tasks = if query.is_empty() {
                                all_tasks.clone()
                            } else {
                                all_tasks.into_iter().filter(|task| {
                                    task.title.to_lowercase().contains(&query) ||
                                    task.description.to_lowercase().contains(&query) ||
                                    task.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
                                }).collect::<Vec<_>>()
                            };
                            
                            // Apply sorting
                            match sort_value.as_str() {
                                "created_desc" => filtered_tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
                                "created_asc" => filtered_tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
                                "due_asc" => filtered_tasks.sort_by(|a, b| {
                                    match (a.due_date, b.due_date) {
                                        (None, None) => std::cmp::Ordering::Equal,
                                        (None, Some(_)) => std::cmp::Ordering::Greater,
                                        (Some(_), None) => std::cmp::Ordering::Less,
                                        (Some(a_due), Some(b_due)) => a_due.cmp(&b_due),
                                    }
                                }),
                                "due_desc" => filtered_tasks.sort_by(|a, b| {
                                    match (a.due_date, b.due_date) {
                                        (None, None) => std::cmp::Ordering::Equal,
                                        (None, Some(_)) => std::cmp::Ordering::Less,
                                        (Some(_), None) => std::cmp::Ordering::Greater,
                                        (Some(a_due), Some(b_due)) => b_due.cmp(&a_due),
                                    }
                                }),
                                "priority_desc" => filtered_tasks.sort_by(|a, b| {
                                    let a_priority = match a.priority {
                                        Priority::Critical => 4,
                                        Priority::High => 3,
                                        Priority::Medium => 2,
                                        Priority::Low => 1,
                                    };
                                    let b_priority = match b.priority {
                                        Priority::Critical => 4,
                                        Priority::High => 3,
                                        Priority::Medium => 2,
                                        Priority::Low => 1,
                                    };
                                    b_priority.cmp(&a_priority)
                                }),
                                "priority_asc" => filtered_tasks.sort_by(|a, b| {
                                    let a_priority = match a.priority {
                                        Priority::Critical => 4,
                                        Priority::High => 3,
                                        Priority::Medium => 2,
                                        Priority::Low => 1,
                                    };
                                    let b_priority = match b.priority {
                                        Priority::Critical => 4,
                                        Priority::High => 3,
                                        Priority::Medium => 2,
                                        Priority::Low => 1,
                                    };
                                    a_priority.cmp(&b_priority)
                                }),
                                "title_asc" => filtered_tasks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
                                "title_desc" => filtered_tasks.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase())),
                                "status" => filtered_tasks.sort_by(|a, b| {
                                    let a_status = match a.status {
                                        TaskStatus::Todo => 1,
                                        TaskStatus::InProgress => 2,
                                        TaskStatus::Review => 3,
                                        TaskStatus::Blocked => 4,
                                        TaskStatus::Done => 5,
                                        TaskStatus::Cancelled => 6,
                                    };
                                    let b_status = match b.status {
                                        TaskStatus::Todo => 1,
                                        TaskStatus::InProgress => 2,
                                        TaskStatus::Review => 3,
                                        TaskStatus::Blocked => 4,
                                        TaskStatus::Done => 5,
                                        TaskStatus::Cancelled => 6,
                                    };
                                    a_status.cmp(&b_status)
                                }),
                                _ => {}
                            }
                            
                            filtered_tasks
                        });
                        
                        rsx! {
                            for task in filtered_sorted_tasks() {
                                TaskCard {
                                    task: task.clone(),
                                    selected: if *bulk_mode.read() {
                                        selected_tasks.read().contains(&task.id)
                                    } else {
                                        selected_task_id() == Some(task.id)
                                    },
                                    on_select: move |id| {
                                        if *bulk_mode.read() {
                                            let mut current = selected_tasks.read().clone();
                                            if current.contains(&id) {
                                                current.remove(&id);
                                            } else {
                                                current.insert(id);
                                            }
                                            selected_tasks.set(current);
                                        } else {
                                            selected_task_id.set(Some(id));
                                        }
                                    },
                                    on_status_change: update_task_status.clone(),
                                    on_edit: move |task| {
                                        if !*bulk_mode.read() {
                                            editing_task.set(Some(task));
                                        }
                                    },
                                    bulk_mode: *bulk_mode.read(),
                                }
                            }
                        }
                    }
                }
            }
            
            // Keyboard shortcuts help
            div {
                style: "margin-top: 20px; padding: 12px; background: #f9fafb; border-radius: 6px; font-size: 12px; color: #6b7280;",
                
                div {
                    style: "font-weight: 600; margin-bottom: 8px;",
                    "Keyboard Shortcuts:"
                }
                
                div {
                    style: "display: flex; gap: 20px; flex-wrap: wrap;",
                    
                    span { 
                        style: "display: flex; align-items: center; gap: 4px;",
                        span { style: "background: white; padding: 2px 6px; border-radius: 3px; border: 1px solid #e5e7eb; font-family: monospace;", 
                            if cfg!(target_os = "macos") { "⌘N" } else { "Ctrl+N" }
                        }
                        " New Task"
                    }
                    
                    span { 
                        style: "display: flex; align-items: center; gap: 4px;",
                        span { style: "background: white; padding: 2px 6px; border-radius: 3px; border: 1px solid #e5e7eb; font-family: monospace;", "↑↓" }
                        " Navigate"
                    }
                    
                    span { 
                        style: "display: flex; align-items: center; gap: 4px;",
                        span { style: "background: white; padding: 2px 6px; border-radius: 3px; border: 1px solid #e5e7eb; font-family: monospace;", "Enter" }
                        " Edit"
                    }
                    
                    span { 
                        style: "display: flex; align-items: center; gap: 4px;",
                        span { style: "background: white; padding: 2px 6px; border-radius: 3px; border: 1px solid #e5e7eb; font-family: monospace;", "Esc" }
                        " Close/Deselect"
                    }
                }
            }
        }
        
        // Edit modal
        if let Some(task) = editing_task() {
            TaskEditModal {
                task: task.clone(),
                on_save: move |updated_task: Task| {
                    // Update the task in the list
                    let mut current_tasks = tasks();
                    if let Some(index) = current_tasks.iter().position(|t| t.id == updated_task.id) {
                        current_tasks[index] = updated_task;
                        tasks.set(current_tasks);
                    }
                    editing_task.set(None);
                },
                on_cancel: move |_| editing_task.set(None),
            }
        }
        
        // Create modal
        if *creating_task.read() {
            TaskCreateModal {
                on_create: move |new_task: Task| {
                    // Add the new task to the list
                    let mut current_tasks = tasks();
                    current_tasks.insert(0, new_task); // Add at the beginning
                    tasks.set(current_tasks);
                    creating_task.set(false);
                },
                on_cancel: move |_| creating_task.set(false),
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
    on_edit: EventHandler<Task>,
    bulk_mode: bool,
) -> Element {
    let time_tracking_service = use_context::<Arc<TimeTrackingService>>();
    let is_tracking = time_tracking_service.is_tracking(task.id);
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
                "padding: 20px 24px; background: {}; border: 2px solid {}; border-radius: 12px; margin-bottom: 16px; cursor: pointer; transition: all 0.3s ease; opacity: {}; min-height: 120px;",
                if selected { "#f0f9ff" } else if task.status == TaskStatus::Done { "#f0fdf4" } else { "white" },
                if selected { "#3b82f6" } else if task.status == TaskStatus::Done { "#10b981" } else { "#e5e7eb" },
                if task.status == TaskStatus::Done { "0.9" } else { "1" }
            ),
            onclick: move |_| on_select.call(task.id),
            
            div {
                style: "display: flex; justify-content: space-between; gap: 16px;",
                
                // Left side - content
                div {
                    style: "flex: 1; display: flex; flex-direction: column; gap: 12px;",
                    
                    // Header
                    div {
                        style: "display: flex; align-items: center; gap: 12px; flex-wrap: wrap;",
                        
                        // Checkbox for bulk mode
                        if bulk_mode {
                            input {
                                r#type: "checkbox",
                                style: "width: 18px; height: 18px; cursor: pointer;",
                                checked: selected,
                                onclick: move |e| {
                                    e.stop_propagation();
                                    on_select.call(task.id);
                                },
                            }
                        }
                        
                        // Status badge
                        div {
                            style: format!(
                                "padding: 4px 10px; background: {}; color: {}; border-radius: 4px; font-size: 12px; font-weight: 600;",
                                status_color.1, status_color.0
                            ),
                            "{task.status:?}"
                        }
                        
                        // Title with strikethrough for done tasks
                        h3 {
                            style: format!(
                                "font-size: 1.25rem; font-weight: 600; text-decoration: {}; color: {}; transition: all 0.3s; flex: 1;",
                                if task.status == TaskStatus::Done { "line-through" } else { "none" },
                                if task.status == TaskStatus::Done { "#9ca3af" } else { "#1f2937" }
                            ),
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
                        
                        // Time tracking indicator
                        if is_tracking {
                            div {
                                style: "padding: 2px 8px; background: #10b981; color: white; border-radius: 4px; font-size: 11px; display: flex; align-items: center; gap: 4px;",
                                "⏱️ Tracking"
                            }
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
                
                // Right side - actions container (vertical)
                div {
                    style: "display: flex; flex-direction: column; gap: 10px; align-items: stretch; min-width: 150px;",
                    
                    // Time tracking toggle
                    button {
                        style: format!(
                            "padding: 8px 12px; background: {}; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: 500; text-align: center; transition: all 0.2s;",
                            if is_tracking { "#ef4444" } else { "#10b981" }
                        ),
                        onclick: {
                            let service = time_tracking_service.clone();
                            let task_id = task.id;
                            move |e| {
                                e.stop_propagation();
                                let service = service.clone();
                                spawn(async move {
                                    if service.is_tracking(task_id) {
                                        let _ = service.stop_tracking(task_id).await;
                                    } else {
                                        let _ = service.start_tracking(task_id, "Time tracking".to_string()).await;
                                    }
                                });
                            }
                        },
                        if is_tracking { "⏹ Stop" } else { "▶ Track" }
                    }
                    
                    // Edit button
                    button {
                        style: "padding: 8px 12px; background: #3b82f6; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: 500; text-align: center; transition: all 0.2s;",
                        onclick: {
                            let task_clone = task.clone();
                            move |e| {
                                e.stop_propagation();
                                on_edit.call(task_clone.clone());
                            }
                        },
                        "Edit"
                    }
                    
                    // Status selector
                    select {
                    style: "padding: 8px 12px; border: 1px solid #e5e7eb; border-radius: 6px; font-size: 14px; background: white; cursor: pointer; width: 100%;",
                    value: format!("{:?}", task.status),
                    onclick: move |e| e.stop_propagation(),
                    onchange: {
                        let task_id = task.id;
                        let old_status = task.status;
                        move |e| {
                            let new_status = match e.value().as_str() {
                                "Todo" => TaskStatus::Todo,
                                "InProgress" => TaskStatus::InProgress,
                                "Done" => TaskStatus::Done,
                                "Blocked" => TaskStatus::Blocked,
                                "Review" => TaskStatus::Review,
                                "Cancelled" => TaskStatus::Cancelled,
                                _ => return,
                            };
                            
                            // No animation - just update status
                            on_status_change.call((task_id, new_status));
                        }
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
}