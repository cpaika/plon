use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus};
use crate::ui_dioxus::state_simple::sample_tasks;
use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use uuid::Uuid;
use std::sync::Arc;

#[component]
pub fn KanbanViewWithDB() -> Element {
    // Initialize repository once using use_future
    let mut repository = use_signal(|| None::<Arc<Repository>>);
    let mut tasks = use_signal(|| Vec::<Task>::new());
    
    // Load repository and tasks asynchronously
    let _ = use_resource(move || async move {
        // Get the current directory to ensure we know where the DB will be created
        let current_dir = std::env::current_dir().unwrap_or_default();
        println!("Current directory: {:?}", current_dir);
        
        // Connect to database - create file if it doesn't exist
        // Use absolute path for reliability
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
                // Use in-memory database as fallback
                sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap()
            }
        };
        
        // Run migrations
        match sqlx::migrate!("./migrations").run(&pool).await {
            Ok(_) => println!("Migrations successful"),
            Err(e) => println!("Migration error: {}", e),
        }
        
        let repo = Arc::new(Repository::new(pool));
        
        // Load tasks
        let loaded_tasks = match repo.tasks.list(TaskFilters::default()).await {
            Ok(t) if !t.is_empty() => {
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
    
    let mut dragging_task = use_signal(|| None::<Uuid>);
    let mut drag_over_status = use_signal(|| None::<TaskStatus>);
    let mut is_dragging = use_signal(|| false);
    let mut ghost_position = use_signal(|| (0.0, 0.0));
    
    rsx! {
        div {
            style: "padding: 20px; height: 100vh; background: #f5f5f5; position: relative; 
                   user-select: none; -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none;",
            
            // Global mouse move handler for ghost
            onmousemove: move |evt| {
                if is_dragging() {
                    ghost_position.set((evt.client_coordinates().x, evt.client_coordinates().y));
                }
            },
            
            // Global mouse up handler
            onmouseup: move |_| {
                dragging_task.set(None);
                drag_over_status.set(None);
                is_dragging.set(false);
            },
            
            h2 { "Kanban Board" }
            p { "Drag cards between columns to change their status" }
            
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
                    
                    // Todo column
                    KanbanColumn {
                        title: "Todo",
                        color: "#808080",
                        status: TaskStatus::Todo,
                        tasks: tasks,
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        is_dragging: is_dragging,
                        ghost_position: ghost_position,
                        repository: repository,
                    }
                    
                    // In Progress column
                    KanbanColumn {
                        title: "In Progress",
                        color: "#2196F3",
                        status: TaskStatus::InProgress,
                        tasks: tasks,
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        is_dragging: is_dragging,
                        ghost_position: ghost_position,
                        repository: repository,
                    }
                    
                    // Review column
                    KanbanColumn {
                        title: "Review",
                        color: "#FF9800",
                        status: TaskStatus::Review,
                        tasks: tasks,
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        is_dragging: is_dragging,
                        ghost_position: ghost_position,
                        repository: repository,
                    }
                    
                    // Done column
                    KanbanColumn {
                        title: "Done",
                        color: "#4CAF50",
                        status: TaskStatus::Done,
                        tasks: tasks,
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        is_dragging: is_dragging,
                        ghost_position: ghost_position,
                        repository: repository,
                    }
                    
                    // Blocked column
                    KanbanColumn {
                        title: "Blocked",
                        color: "#f44336",
                        status: TaskStatus::Blocked,
                        tasks: tasks,
                        dragging_task: dragging_task,
                        drag_over_status: drag_over_status,
                        is_dragging: is_dragging,
                        ghost_position: ghost_position,
                        repository: repository,
                    }
                }
            }
            
            // Drag ghost overlay
            {if let Some(task_id) = dragging_task() {
                if let Some(task) = tasks.read().iter().find(|t| t.id == task_id) {
                    let (x, y) = ghost_position();
                    rsx! {
                        div {
                            style: "position: fixed; left: {x}px; top: {y}px; 
                                   width: 250px; pointer-events: none; z-index: 9999;
                                   transform: translate(-50%, -50%) rotate(3deg);
                                   opacity: 0.8;",
                            
                            // Ghost card
                            div {
                                style: "background: white; border-radius: 6px; padding: 12px;
                                       box-shadow: 0 10px 30px rgba(0,0,0,0.3);
                                       border: 1px solid #e0e0e0;",
                                
                                h4 {
                                    style: "margin: 0; font-size: 14px; font-weight: 500; color: #333;",
                                    "{task.title}"
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            } else {
                rsx! {}
            }}
        }
    }
}

#[component]
fn KanbanColumn(
    title: &'static str,
    color: &'static str,
    status: TaskStatus,
    tasks: Signal<Vec<Task>>,
    dragging_task: Signal<Option<Uuid>>,
    drag_over_status: Signal<Option<TaskStatus>>,
    is_dragging: Signal<bool>,
    ghost_position: Signal<(f64, f64)>,
    repository: Signal<Option<Arc<Repository>>>,
) -> Element {
    let column_tasks: Vec<Task> = tasks.read().iter()
        .filter(|t| t.status == status)
        .cloned()
        .collect();
    
    let is_drag_over = drag_over_status.read().as_ref() == Some(&status);
    let background = if is_drag_over { "#e8f5e9" } else { "white" };
    let border_color = if is_drag_over { "#4CAF50" } else { color };
    
    rsx! {
        div {
            style: "flex: 0 0 280px; background: {background}; border-radius: 8px; 
                   padding: 15px; border-top: 4px solid {border_color}; 
                   transition: all 0.3s ease; position: relative;
                   min-height: 200px;",
            
            // Mouse enter for drag over
            onmouseenter: move |_| {
                if *is_dragging.read() {
                    drag_over_status.set(Some(status));
                }
            },
            
            // Mouse leave
            onmouseleave: move |_| {
                if drag_over_status.read().as_ref() == Some(&status) {
                    drag_over_status.set(None);
                }
            },
            
            // Handle drop
            onmouseup: move |evt| {
                evt.stop_propagation();
                
                if let Some(task_id) = *dragging_task.read() {
                    if drag_over_status.read().as_ref() == Some(&status) {
                        // Move task to this column
                        tasks.with_mut(|tasks| {
                            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                task.status = status;
                                
                                // Persist to database if repository is available
                                if let Some(repo) = repository() {
                                    let task_clone = task.clone();
                                    spawn(async move {
                                        let _ = repo.tasks.update(&task_clone).await;
                                    });
                                }
                            }
                        });
                    }
                }
                
                // Clear drag state
                dragging_task.set(None);
                drag_over_status.set(None);
                is_dragging.set(false);
            },
            
            // Column header
            div {
                style: "margin-bottom: 15px; display: flex; justify-content: space-between; align-items: center;
                       user-select: none; -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none;",
                
                h3 { 
                    style: "margin: 0; color: {color}; font-size: 16px; font-weight: 600;
                           user-select: none; -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none;", 
                    "{title}" 
                }
                
                span {
                    style: "padding: 2px 8px; background: {color}; color: white; 
                           border-radius: 12px; font-size: 14px; font-weight: 500;",
                    "{column_tasks.len()}"
                }
            }
            
            // Cards container
            div {
                style: "overflow-y: auto; max-height: calc(100vh - 200px); 
                       min-height: 100px; position: relative;",
                
                if column_tasks.is_empty() && is_drag_over {
                    div {
                        style: "padding: 40px 20px; text-align: center; color: #4CAF50; 
                               border: 2px dashed #4CAF50; border-radius: 8px; 
                               background: rgba(76, 175, 80, 0.05);",
                        "Drop here"
                    }
                } else if column_tasks.is_empty() {
                    div {
                        style: "padding: 40px 20px; text-align: center; color: #999; 
                               border: 2px dashed #ddd; border-radius: 8px;",
                        "No tasks"
                    }
                }
                
                for task in column_tasks {
                    KanbanCard {
                        task: task.clone(),
                        dragging_task: dragging_task,
                        tasks: tasks,
                        is_dragging: is_dragging,
                        ghost_position: ghost_position,
                        repository: repository,
                    }
                }
            }
        }
    }
}

#[component]
fn KanbanCard(
    task: Task,
    dragging_task: Signal<Option<Uuid>>,
    tasks: Signal<Vec<Task>>,
    is_dragging: Signal<bool>,
    ghost_position: Signal<(f64, f64)>,
    repository: Signal<Option<Arc<Repository>>>,
) -> Element {
    let is_this_card_dragging = dragging_task.read().as_ref() == Some(&task.id);
    let visibility = if is_this_card_dragging { "hidden" } else { "visible" };
    let cursor = if is_this_card_dragging { "grabbing" } else { "grab" };
    let task_id_short = format!("#{}", &task.id.to_string()[..8]);
    
    rsx! {
        div {
            style: "background: white; border-radius: 6px; padding: 12px; 
                   margin-bottom: 10px; cursor: {cursor}; visibility: {visibility}; 
                   box-shadow: 0 2px 4px rgba(0,0,0,0.1); 
                   transition: all 0.2s ease; user-select: none; 
                   -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none;
                   border: 1px solid #e0e0e0;",
            
            // Start drag on mouse down
            onmousedown: move |evt| {
                evt.stop_propagation();
                dragging_task.set(Some(task.id));
                is_dragging.set(true);
                ghost_position.set((evt.client_coordinates().x, evt.client_coordinates().y));
            },
            
            // Card header with delete button
            div {
                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 8px;",
                
                h4 { 
                    style: "margin: 0; font-size: 14px; flex: 1; font-weight: 500; color: #333;
                           user-select: none; -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none;", 
                    "{task.title}" 
                }
                
                button {
                    onclick: move |evt| {
                        evt.stop_propagation();
                        let task_id = task.id;
                        tasks.with_mut(|tasks| {
                            tasks.retain(|t| t.id != task_id);
                        });
                        
                        // Delete from database if repository is available
                        if let Some(repo) = repository() {
                            spawn(async move {
                                let _ = repo.tasks.delete(task_id).await;
                            });
                        }
                    },
                    onmousedown: move |evt| evt.stop_propagation(),
                    style: "background: none; border: none; color: #999; cursor: pointer; 
                           font-size: 18px; padding: 0; margin: -4px -4px 0 0; 
                           line-height: 1; hover: color: #666;",
                    "×"
                }
            }
            
            // Description
            if !task.description.is_empty() {
                p { 
                    style: "margin: 0 0 8px 0; font-size: 12px; color: #666; line-height: 1.4;", 
                    "{task.description}" 
                }
            }
            
            // Tags and metadata  
            div {
                style: "display: flex; gap: 8px; flex-wrap: wrap; align-items: center;",
                
                // Priority badge - simplified
                span {
                    style: "font-size: 11px; padding: 2px 6px; 
                           background: #ff8800; 
                           color: white; border-radius: 3px; font-weight: 500;",
                    "Priority"
                }
                
                // Due date
                if let Some(due) = task.due_date {
                    span {
                        style: "font-size: 11px; color: #666; display: flex; align-items: center; gap: 2px;",
                        "📅 {due.format(\"%m/%d\")}"
                    }
                }
                
                // Task ID for debugging
                span {
                    style: "font-size: 10px; color: #ccc; margin-left: auto;",
                    "{task_id_short}"
                }
            }
        }
    }
}