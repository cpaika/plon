use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus};
use crate::ui_dioxus::state_simple::sample_tasks;
use crate::ui_dioxus::components::TaskEditModal;
use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use uuid::Uuid;
use std::sync::Arc;

#[component]
pub fn KanbanViewOrdered() -> Element {
    // Initialize repository once using use_resource
    let mut repository = use_signal(|| None::<Arc<Repository>>);
    let mut tasks = use_signal(|| Vec::<Task>::new());
    
    // Track dragging state
    let mut dragging_task = use_signal(|| None::<Uuid>);
    let mut drag_over_status = use_signal(|| None::<TaskStatus>);
    let mut drag_over_position = use_signal(|| None::<usize>); // Position in the column where we're hovering
    let mut mouse_position = use_signal(|| (0.0, 0.0));
    let mut editing_task = use_signal(|| None::<Task>);
    
    // Load repository and tasks asynchronously
    let _ = use_resource(move || async move {
        // Get the current directory to ensure we know where the DB will be created
        let current_dir = std::env::current_dir().unwrap_or_default();
        println!("Current directory: {:?}", current_dir);
        
        // Connect to database - create file if it doesn't exist
        let db_path = current_dir.join("plon.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        println!("Attempting to connect to: {}", db_url);
        
        let pool = match sqlx::SqlitePool::connect(&db_url).await {
            Ok(pool) => {
                println!("Successfully connected to database at: {:?}", db_path);
                pool
            },
            Err(e) => {
                println!("Failed to connect to database: {}", e);
                println!("Using in-memory database instead");
                sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap()
            }
        };
        
        // Run migrations
        match sqlx::migrate!("./migrations").run(&pool).await {
            Ok(_) => println!("Migrations successful"),
            Err(e) => println!("Migration error: {}", e),
        }
        
        let repo = Arc::new(Repository::new(pool));
        
        // Load tasks sorted by sort_order
        let loaded_tasks = match repo.tasks.list(TaskFilters::default()).await {
            Ok(mut t) if !t.is_empty() => {
                // Sort by sort_order within each status
                t.sort_by_key(|task| task.sort_order);
                println!("Loaded {} tasks from database", t.len());
                t
            },
            _ => {
                println!("No tasks in database, using sample data");
                let sample = sample_tasks();
                for task in &sample {
                    let _ = repo.tasks.create(task).await;
                }
                sample
            }
        };
        
        repository.set(Some(repo));
        tasks.set(loaded_tasks);
    });
    
    // Helper function to get tasks for a specific column, sorted by sort_order
    let get_column_tasks = move |status: TaskStatus| -> Vec<Task> {
        let mut column_tasks: Vec<Task> = tasks.read()
            .iter()
            .filter(|t| t.status == status)
            .cloned()
            .collect();
        column_tasks.sort_by_key(|t| t.sort_order);
        column_tasks
    };
    
    // Helper to recalculate sort_order values when reordering
    let mut recalculate_sort_orders = move |status: TaskStatus, moved_task_id: Uuid, new_position: usize| {
        tasks.with_mut(|tasks| {
            // Get all tasks in this column
            let mut column_tasks: Vec<&mut Task> = tasks
                .iter_mut()
                .filter(|t| t.status == status)
                .collect();
            
            // Sort by current sort_order
            column_tasks.sort_by_key(|t| t.sort_order);
            
            // Find the task we're moving
            if let Some(moved_task_idx) = column_tasks.iter().position(|t| t.id == moved_task_id) {
                // Remove the task from its current position
                let moved_task = column_tasks.remove(moved_task_idx);
                
                // Insert at new position
                let insert_pos = new_position.min(column_tasks.len());
                column_tasks.insert(insert_pos, moved_task);
                
                // Reassign sort_order values
                for (i, task) in column_tasks.iter_mut().enumerate() {
                    task.sort_order = (i as i32 + 1) * 100;
                    
                    // Persist to database if repository is available
                    if let Some(repo) = repository() {
                        let task_clone = (*task).clone();
                        spawn(async move {
                            let _ = repo.tasks.update(&task_clone).await;
                        });
                    }
                }
            }
        });
    };
    
    rsx! {
        div {
            style: "padding: 20px; height: 100vh; background: #f5f5f5; position: relative; 
                   user-select: none; -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none;",
            
            // Global mouse move handler
            onmousemove: move |evt| {
                mouse_position.set((evt.client_coordinates().x, evt.client_coordinates().y));
            },
            
            // Global mouse up handler
            onmouseup: move |_| {
                // If we were dragging and have a target position, reorder the cards
                if let (Some(task_id), Some(status), Some(position)) = 
                    (*dragging_task.read(), *drag_over_status.read(), *drag_over_position.read()) {
                    
                    // First update the task's status if it changed
                    let old_status = tasks.read().iter()
                        .find(|t| t.id == task_id)
                        .map(|t| t.status);
                    
                    if old_status != Some(status) {
                        tasks.with_mut(|tasks| {
                            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                task.status = status;
                            }
                        });
                    }
                    
                    // Then recalculate sort orders
                    recalculate_sort_orders(status, task_id, position);
                }
                
                // Clear drag state
                dragging_task.set(None);
                drag_over_status.set(None);
                drag_over_position.set(None);
            },
            
            h2 { "Kanban Board with Ordering" }
            p { "Drag cards to reorder them within columns or move between columns" }
            
            // Show loading if repository not ready
            if repository().is_none() {
                div {
                    style: "padding: 20px; text-align: center;",
                    "Loading..."
                }
            } else {
                // Kanban columns container
                div {
                    style: "display: flex; gap: 15px; height: calc(100vh - 100px); overflow-x: auto;",
                    
                    // Render each column
                    KanbanColumnOrdered {
                        status: TaskStatus::Todo,
                        tasks: get_column_tasks(TaskStatus::Todo),
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        drag_over_position: drag_over_position,
                        mouse_position: mouse_position,
                        editing_task: editing_task,
                    }
                    
                    KanbanColumnOrdered {
                        status: TaskStatus::InProgress,
                        tasks: get_column_tasks(TaskStatus::InProgress),
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        drag_over_position: drag_over_position,
                        mouse_position: mouse_position,
                        editing_task: editing_task,
                    }
                    
                    KanbanColumnOrdered {
                        status: TaskStatus::Review,
                        tasks: get_column_tasks(TaskStatus::Review),
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        drag_over_position: drag_over_position,
                        mouse_position: mouse_position,
                        editing_task: editing_task,
                    }
                    
                    KanbanColumnOrdered {
                        status: TaskStatus::Done,
                        tasks: get_column_tasks(TaskStatus::Done),
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        drag_over_position: drag_over_position,
                        mouse_position: mouse_position,
                        editing_task: editing_task,
                    }
                    
                    KanbanColumnOrdered {
                        status: TaskStatus::Blocked,
                        tasks: get_column_tasks(TaskStatus::Blocked),
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        drag_over_position: drag_over_position,
                        mouse_position: mouse_position,
                        editing_task: editing_task,
                    }
                }
            }
            
            // Dragging card (follows mouse)
            if let Some(task_id) = dragging_task() {
                if let Some(task) = tasks.read().iter().find(|t| t.id == task_id) {
                    {
                        let (x, y) = mouse_position();
                        rsx! {
                            div {
                                style: "position: fixed; left: {x}px; top: {y}px; 
                                       width: 250px; pointer-events: none; z-index: 9999;
                                       transform: translate(-50%, -50%) rotate(2deg);",
                                
                                // Dragging card visual
                                div {
                            style: "background: white; border-radius: 6px; padding: 12px;
                                   box-shadow: 0 10px 30px rgba(0,0,0,0.3);
                                   border: 1px solid #e0e0e0; opacity: 0.9;",
                            
                            h4 {
                                style: "margin: 0; font-size: 14px; font-weight: 500; color: #333;",
                                "{task.title}"
                            }
                            
                            if !task.description.is_empty() {
                                p { 
                                    style: "margin: 4px 0 0 0; font-size: 12px; color: #666;", 
                                    "{task.description}" 
                                }
                            }
                                }
                            }
                        }
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
                    tasks.with_mut(|tasks| {
                        if let Some(index) = tasks.iter().position(|t| t.id == updated_task.id) {
                            tasks[index] = updated_task;
                        }
                    });
                    editing_task.set(None);
                },
                on_cancel: move |_| editing_task.set(None),
            }
        }
    }
}

#[component]
fn KanbanColumnOrdered(
    status: TaskStatus,
    tasks: Vec<Task>,
    dragging_task: Signal<Option<Uuid>>,
    drag_over_status: Signal<Option<TaskStatus>>,
    drag_over_position: Signal<Option<usize>>,
    mouse_position: Signal<(f64, f64)>,
    editing_task: Signal<Option<Task>>,
) -> Element {
    let column_name = match status {
        TaskStatus::Todo => "Todo",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Review => "Review",
        TaskStatus::Done => "Done",
        TaskStatus::Blocked => "Blocked",
        _ => "Unknown",
    };
    
    let column_color = match status {
        TaskStatus::Todo => "#808080",
        TaskStatus::InProgress => "#2196F3",
        TaskStatus::Review => "#FF9800",
        TaskStatus::Done => "#4CAF50",
        TaskStatus::Blocked => "#f44336",
        _ => "#999999",
    };
    
    let is_drag_over = drag_over_status.read().as_ref() == Some(&status);
    let background = if is_drag_over { "#e8f5e9" } else { "white" };
    let border_color = if is_drag_over { "#4CAF50" } else { column_color };
    
    rsx! {
        div {
            style: "flex: 0 0 280px; background: {background}; border-radius: 8px; 
                   padding: 15px; border-top: 4px solid {border_color}; 
                   transition: all 0.3s ease; position: relative;
                   min-height: 200px;",
            
            // Column header
            div {
                style: "margin-bottom: 15px; display: flex; justify-content: space-between; align-items: center;",
                
                h3 { 
                    style: "margin: 0; color: {column_color}; font-size: 16px; font-weight: 600;", 
                    "{column_name}" 
                }
                
                span {
                    style: "padding: 2px 8px; background: {column_color}; color: white; 
                           border-radius: 12px; font-size: 14px; font-weight: 500;",
                    "{tasks.len()}"
                }
            }
            
            // Cards container
            div {
                style: "overflow-y: auto; max-height: calc(100vh - 200px); 
                       min-height: 100px; position: relative;",
                
                onmouseenter: move |_| {
                    if dragging_task.read().is_some() {
                        drag_over_status.set(Some(status));
                    }
                },
                
                onmouseleave: move |_| {
                    if *drag_over_status.read() == Some(status) {
                        drag_over_status.set(None);
                        drag_over_position.set(None);
                    }
                },
                
                // Empty column placeholder
                if tasks.is_empty() {
                    div {
                        style: "padding: 40px 20px; text-align: center; color: #999; 
                               border: 2px dashed #ddd; border-radius: 8px;",
                        
                        onmouseenter: move |_| {
                            if dragging_task.read().is_some() {
                                drag_over_position.set(Some(0));
                            }
                        },
                        
                        if is_drag_over { "Drop here" } else { "No tasks" }
                    }
                }
                
                // Render cards with drop zones
                {tasks.iter().enumerate().map(|(index, task)| {
                    rsx! {
                        // Drop zone before this card
                        {if dragging_task.read().is_some() && *drag_over_status.read() == Some(status) {
                            rsx! {
                                DropZone {
                                    index: index,
                                    drag_over_position: drag_over_position,
                                    is_active: *drag_over_position.read() == Some(index),
                                }
                            }
                        } else {
                            rsx! {}
                        }}
                        
                        // The card itself
                        KanbanCardOrdered {
                            key: "{task.id}",
                            task: task.clone(),
                            index: index,
                            dragging_task: dragging_task,
                            drag_over_position: drag_over_position,
                            is_column_active: is_drag_over,
                            on_edit: move |task| editing_task.set(Some(task)),
                        }
                    }
                })}
                
                // Drop zone after the last card
                {if dragging_task.read().is_some() && *drag_over_status.read() == Some(status) {
                    rsx! {
                        DropZone {
                            index: tasks.len(),
                            drag_over_position: drag_over_position,
                            is_active: *drag_over_position.read() == Some(tasks.len()),
                        }
                    }
                } else {
                    rsx! {}
                }}
            }
        }
    }
}

#[component]
fn DropZone(
    index: usize,
    drag_over_position: Signal<Option<usize>>,
    is_active: bool,
) -> Element {
    let height = if is_active { "40px" } else { "2px" };
    let background = if is_active { 
        "linear-gradient(90deg, transparent, #4CAF50, transparent)" 
    } else { 
        "transparent" 
    };
    let opacity = if is_active { "1" } else { "0" };
    
    rsx! {
        div {
            style: "height: {height}; background: {background}; 
                   margin: 4px 0; transition: all 0.2s ease; border-radius: 2px;
                   opacity: {opacity};",
            
            onmouseenter: move |_| {
                drag_over_position.set(Some(index));
            }
        }
    }
}

#[component]
fn KanbanCardOrdered(
    task: Task,
    index: usize,
    dragging_task: Signal<Option<Uuid>>,
    drag_over_position: Signal<Option<usize>>,
    is_column_active: bool,
    on_edit: EventHandler<Task>,
) -> Element {
    let is_dragging = dragging_task.read().as_ref() == Some(&task.id);
    let opacity = if is_dragging { "0.3" } else { "1" };
    let transform = if is_dragging { "scale(0.95)" } else { "scale(1)" };
    
    rsx! {
        div {
            style: "background: white; border-radius: 6px; padding: 12px; 
                   margin-bottom: 8px; cursor: grab; opacity: {opacity};
                   transform: {transform}; transition: all 0.2s ease;
                   box-shadow: 0 2px 4px rgba(0,0,0,0.1); 
                   border: 1px solid #e0e0e0;
                   user-select: none; -webkit-user-select: none;",
            
            onmousedown: move |evt| {
                evt.stop_propagation();
                dragging_task.set(Some(task.id));
            },
            
            onmouseenter: move |_| {
                if dragging_task.read().is_some() && is_column_active {
                    // Set drop position based on where we're hovering
                    drag_over_position.set(Some(index + 1));
                }
            },
            
            // Card content
            div {
                style: "position: relative;",
                
                // Edit button
                button {
                    style: "position: absolute; top: 2px; right: 2px; padding: 2px 6px;
                           background: #e3f2fd; color: #1976d2; border: none; border-radius: 3px;
                           cursor: pointer; font-size: 11px; font-weight: 500;",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_edit.call(task.clone());
                    },
                    "Edit"
                }
                
                h4 { 
                    style: "margin: 0 0 4px 0; font-size: 14px; font-weight: 500; color: #333; padding-right: 40px;", 
                    "{task.title}" 
                }
                
                if !task.description.is_empty() {
                    p { 
                        style: "margin: 0 0 8px 0; font-size: 12px; color: #666; line-height: 1.4;", 
                        "{task.description}" 
                    }
                }
                
                // Tags
                div {
                    style: "display: flex; gap: 6px; flex-wrap: wrap;",
                    
                    span {
                        style: "font-size: 11px; padding: 2px 6px; 
                               background: #ff8800; color: white; 
                               border-radius: 3px; font-weight: 500;",
                        "Priority"
                    }
                    
                    if let Some(due) = task.due_date {
                        span {
                            style: "font-size: 11px; color: #666;",
                            "📅 {due.format(\"%m/%d\")}"
                        }
                    }
                }
            }
        }
    }
}