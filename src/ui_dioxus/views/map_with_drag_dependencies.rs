use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Position, Priority};
use crate::domain::dependency::{Dependency, DependencyType, DependencyGraph};
use crate::ui_dioxus::state_simple::sample_tasks;
use crate::repository::Repository;
use uuid::Uuid;
use std::env::current_dir;
use sqlx::sqlite::SqlitePool;

#[derive(Clone, Debug, PartialEq)]
struct DragState {
    from_task: Uuid,
    start_pos: (f64, f64),
}

#[component]
pub fn MapView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut dependencies = use_signal(|| Vec::<Dependency>::new());
    let mut selected_task: Signal<Option<Uuid>> = use_signal(|| None);
    let mut zoom = use_signal(|| 1.0f32);
    let mut dragging_task = use_signal(|| None::<Uuid>);
    let mut drag_start_mouse = use_signal(|| (0.0f64, 0.0f64));
    let mut drag_start_task_pos = use_signal(|| (0.0f64, 0.0f64));
    let mut dragging_connection = use_signal(|| None::<DragState>);
    let mut hover_dependency = use_signal(|| None::<(Uuid, Uuid)>);
    let mut hover_left_node = use_signal(|| None::<Uuid>);
    let mut mouse_position = use_signal(|| (0.0, 0.0));
    let mut error_message = use_signal(|| None::<String>);
    
    // Load dependencies from database
    let repository = use_resource(move || async move {
        let current = current_dir().unwrap_or_default();
        let db_path = current.join("plon.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        
        match SqlitePool::connect(&db_url).await {
            Ok(pool) => Some(Repository::new(pool)),
            Err(_) => None
        }
    });
    
    // Load dependencies when repository is ready
    use_effect(move || {
        spawn(async move {
            if let Some(Some(repo)) = repository.read().as_ref() {
                if let Ok(deps) = repo.dependencies.list_all().await {
                    dependencies.set(deps);
                }
            }
        });
    });
    
    // Helper to get task connection points
    let get_task_left_point = |task: &Task| -> (f64, f64) {
        (task.position.x, task.position.y + 30.0) // Left middle of card
    };
    
    let get_task_right_point = |task: &Task| -> (f64, f64) {
        (task.position.x + 200.0, task.position.y + 30.0) // Right middle of card
    };
    
    // Helper to check for circular dependency
    let would_create_cycle = move |from: Uuid, to: Uuid| -> bool {
        let mut graph = DependencyGraph::new();
        
        // Add existing dependencies
        for dep in dependencies.read().iter() {
            let _ = graph.add_dependency(dep);
        }
        
        // Try adding new dependency
        let test_dep = Dependency::new(from, to, DependencyType::FinishToStart);
        graph.add_dependency(&test_dep).is_err()
    };
    
    rsx! {
        div {
            style: "width: 100%; height: 100vh; display: flex; flex-direction: column; background: #f5f5f5;",
            
            // Toolbar
            div {
                style: "padding: 10px; background: white; box-shadow: 0 2px 4px rgba(0,0,0,0.1); display: flex; gap: 10px; align-items: center;",
                
                h2 { style: "margin: 0; margin-right: 20px;", "Task Map" }
                
                button {
                    onclick: move |_| {
                        let current = *zoom.read();
                        let new_zoom = (current * 1.2).min(3.0);
                        zoom.set(new_zoom);
                    },
                    style: "padding: 8px 12px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Zoom In"
                }
                
                button {
                    onclick: move |_| {
                        let current = *zoom.read();
                        let new_zoom = (current / 1.2).max(0.3);
                        zoom.set(new_zoom);
                    },
                    style: "padding: 8px 12px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Zoom Out"
                }
                
                button {
                    onclick: move |_| {
                        zoom.set(1.0);
                    },
                    style: "padding: 8px 12px; background: #FF9800; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Reset"
                }
                
                span { style: "margin-left: 20px;", "Zoom: {(*zoom.read() * 100.0) as i32}%" }
                
                // Show when dragging connection
                if dragging_connection.read().is_some() {
                    span {
                        style: "margin-left: 20px; padding: 8px 12px; background: #FFC107; color: #333; border-radius: 4px;",
                        "Drop on left node to create dependency"
                    }
                }
                
                button {
                    onclick: move |_| {
                        let new_task = Task {
                            id: Uuid::new_v4(),
                            title: "New Task".to_string(),
                            description: String::new(),
                            status: TaskStatus::Todo,
                            priority: Priority::Medium,
                            position: Position { x: 200.0, y: 200.0 },
                            ..Default::default()
                        };
                        tasks.write().push(new_task);
                    },
                    style: "margin-left: auto; padding: 8px 16px; background: #9C27B0; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Add Task"
                }
            }
            
            // Error message
            if let Some(error) = error_message.read().as_ref() {
                div {
                    style: "padding: 10px; background: #ffebee; color: #c62828; border-left: 4px solid #f44336;",
                    "{error}"
                    button {
                        onclick: move |_| error_message.set(None),
                        style: "margin-left: 10px; padding: 2px 8px; background: #f44336; color: white; border: none; border-radius: 3px; cursor: pointer;",
                        "×"
                    }
                }
            }
            
            // Map area
            div {
                style: "flex: 1; position: relative; overflow: auto; background: #fafafa;",
                
                // SVG layer for dependencies
                svg {
                    style: "position: absolute; top: 0; left: 0; width: 2000px; height: 2000px; pointer-events: none; transform: scale({zoom.read()}); transform-origin: top left;",
                    
                    // Define arrowhead marker
                    defs {
                        marker {
                            id: "arrowhead",
                            "markerWidth": "10",
                            "markerHeight": "10",
                            "refX": "9",
                            "refY": "3",
                            "orient": "auto",
                            polygon {
                                points: "0 0, 10 3, 0 6",
                                fill: "#666"
                            }
                        }
                        marker {
                            id: "arrowhead-hover",
                            "markerWidth": "10",
                            "markerHeight": "10",
                            "refX": "9",
                            "refY": "3",
                            "orient": "auto",
                            polygon {
                                points: "0 0, 10 3, 0 6",
                                fill: "#4CAF50"
                            }
                        }
                    }
                    
                    // Render dependencies
                    for dep in dependencies.read().clone() {
                        if let (Some(from_task), Some(to_task)) = (
                            tasks.read().iter().find(|t| t.id == dep.from_task_id),
                            tasks.read().iter().find(|t| t.id == dep.to_task_id)
                        ) {
                            {
                                let (x1, y1) = get_task_right_point(from_task);
                                let (x2, y2) = get_task_left_point(to_task);
                                let is_hovered = hover_dependency.read().as_ref() == Some(&(dep.from_task_id, dep.to_task_id));
                                let stroke_color = if is_hovered { "#4CAF50" } else { "#666" };
                                let stroke_width = if is_hovered { "3" } else { "2" };
                                let marker = if is_hovered { "url(#arrowhead-hover)" } else { "url(#arrowhead)" };
                                
                                // Calculate bezier control points for curved line
                                let ctrl_offset = ((x2 - x1).abs() / 3.0).min(50.0);
                                let path = format!("M {} {} C {} {}, {} {}, {} {}", 
                                    x1, y1, 
                                    x1 + ctrl_offset, y1,
                                    x2 - ctrl_offset, y2,
                                    x2, y2
                                );
                                
                                rsx! {
                                    path {
                                        d: "{path}",
                                        stroke: stroke_color,
                                        "stroke-width": stroke_width,
                                        fill: "none",
                                        "marker-end": marker,
                                        style: "pointer-events: stroke; cursor: pointer;",
                                        
                                        onmouseenter: move |_| {
                                            hover_dependency.set(Some((dep.from_task_id, dep.to_task_id)));
                                        },
                                        
                                        onmouseleave: move |_| {
                                            hover_dependency.set(None);
                                        },
                                        
                                        oncontextmenu: move |evt| {
                                            evt.stop_propagation();
                                            
                                            // Delete dependency on right-click
                                            dependencies.with_mut(|deps| {
                                                deps.retain(|d| d.id != dep.id);
                                            });
                                            
                                            // Persist deletion
                                            spawn(async move {
                                                if let Some(Some(repo)) = repository.read().as_ref() {
                                                    let _ = repo.dependencies.delete(dep.from_task_id, dep.to_task_id).await;
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Preview line when dragging connection
                    if let Some(drag_state) = dragging_connection.read().as_ref() {
                        {
                            let (x1, y1) = drag_state.start_pos;
                            let (x2, y2) = *mouse_position.read();
                            let actual_x2 = x2 / *zoom.read() as f64;
                            let actual_y2 = y2 / *zoom.read() as f64;
                            
                            // Calculate bezier control points
                            let ctrl_offset = ((actual_x2 - x1).abs() / 3.0).min(50.0);
                            let path = format!("M {} {} C {} {}, {} {}, {} {}", 
                                x1, y1, 
                                x1 + ctrl_offset, y1,
                                actual_x2 - ctrl_offset, actual_y2,
                                actual_x2, actual_y2
                            );
                            
                            rsx! {
                                path {
                                    d: "{path}",
                                    stroke: "#999",
                                    "stroke-width": "2",
                                    "stroke-dasharray": "5,5",
                                    fill: "none",
                                    "marker-end": "url(#arrowhead)"
                                }
                            }
                        }
                    }
                }
                
                // Task cards container
                div {
                    style: "position: relative; width: 2000px; height: 2000px; transform: scale({zoom.read()}); transform-origin: top left;",
                    
                    onmousemove: move |evt| {
                        // Update mouse position for preview line
                        mouse_position.set((evt.client_coordinates().x as f64, evt.client_coordinates().y as f64));
                        
                        // Update task position while dragging
                        if let Some(task_id) = *dragging_task.read() {
                            if dragging_connection.read().is_none() {
                                let zoom_factor = *zoom.read();
                                let start_mouse = *drag_start_mouse.read();
                                let start_pos = *drag_start_task_pos.read();
                                
                                let delta_x = (evt.client_coordinates().x as f64 - start_mouse.0) / zoom_factor as f64;
                                let delta_y = (evt.client_coordinates().y as f64 - start_mouse.1) / zoom_factor as f64;
                                
                                let new_x = (start_pos.0 + delta_x).max(0.0).min(1900.0);
                                let new_y = (start_pos.1 + delta_y).max(0.0).min(1900.0);
                                
                                tasks.with_mut(|tasks| {
                                    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                        task.position.x = new_x;
                                        task.position.y = new_y;
                                    }
                                });
                            }
                        }
                    },
                    
                    onmouseup: move |_| {
                        // End drag
                        dragging_task.set(None);
                        
                        // Handle connection drop
                        if let Some(drag_state) = dragging_connection.read().as_ref() {
                            if let Some(target_task_id) = *hover_left_node.read() {
                                let from_id = drag_state.from_task;
                                
                                if from_id != target_task_id {
                                    // Check for cycle
                                    if would_create_cycle(from_id, target_task_id) {
                                        error_message.set(Some("Cannot create dependency: would create a circular dependency".to_string()));
                                    } else {
                                        // Create dependency
                                        let new_dep = Dependency::new(from_id, target_task_id, DependencyType::FinishToStart);
                                        
                                        dependencies.with_mut(|deps| {
                                            deps.push(new_dep.clone());
                                        });
                                        
                                        // Persist to database
                                        spawn(async move {
                                            if let Some(Some(repo)) = repository.read().as_ref() {
                                                let _ = repo.dependencies.create(&new_dep).await;
                                            }
                                        });
                                        
                                        error_message.set(None);
                                    }
                                }
                            }
                        }
                        
                        dragging_connection.set(None);
                        hover_left_node.set(None);
                    },
                    
                    // Render each task as a card
                    for task in tasks.read().clone() {
                        {
                            let is_highlighted = hover_dependency.read().as_ref()
                                .map(|(from, to)| *from == task.id || *to == task.id)
                                .unwrap_or(false);
                            
                            rsx! {
                                TaskCard {
                                    task: task.clone(),
                                    selected: selected_task.read().as_ref() == Some(&task.id),
                                    dragging: dragging_task.read().as_ref() == Some(&task.id),
                                    is_highlighted: is_highlighted,
                                    is_left_node_hover: hover_left_node.read().as_ref() == Some(&task.id),
                                    is_connection_dragging: dragging_connection.read().is_some(),
                                    
                                    onclick: move |_| {
                                        if dragging_connection.read().is_none() {
                                            selected_task.set(Some(task.id));
                                        }
                                    },
                                    
                                    onmousedown: move |evt: MouseEvent| {
                                        if dragging_connection.read().is_none() {
                                            dragging_task.set(Some(task.id));
                                            drag_start_mouse.set((evt.client_coordinates().x as f64, evt.client_coordinates().y as f64));
                                            drag_start_task_pos.set((task.position.x, task.position.y));
                                        }
                                    },
                                    
                                    on_right_node_drag_start: {
                                        let task = task.clone();
                                        move |_| {
                                            let (x, y) = get_task_right_point(&task);
                                            dragging_connection.set(Some(DragState {
                                                from_task: task.id,
                                                start_pos: (x, y),
                                            }));
                                        }
                                    },
                                    
                                    on_left_node_enter: move |_| {
                                        if dragging_connection.read().is_some() {
                                            hover_left_node.set(Some(task.id));
                                        }
                                    },
                                    
                                    on_left_node_leave: move |_| {
                                        hover_left_node.set(None);
                                    },
                                    
                                    onstatuschange: move |_| {
                                        tasks.with_mut(|tasks| {
                                            if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                                t.status = match t.status {
                                                    TaskStatus::Todo => TaskStatus::InProgress,
                                                    TaskStatus::InProgress => TaskStatus::Done,
                                                    TaskStatus::Done => TaskStatus::Todo,
                                                    _ => TaskStatus::Todo,
                                                };
                                            }
                                        });
                                    },
                                    
                                    ondelete: move |_| {
                                        // Remove task and its dependencies
                                        dependencies.with_mut(|deps| {
                                            deps.retain(|d| d.from_task_id != task.id && d.to_task_id != task.id);
                                        });
                                        
                                        tasks.with_mut(|tasks| {
                                            tasks.retain(|t| t.id != task.id);
                                        });
                                        
                                        if selected_task.read().as_ref() == Some(&task.id) {
                                            selected_task.set(None);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Selected task details panel
            {
                let task_id = selected_task.read().clone();
                if let Some(id) = task_id {
                    let task = tasks.read().iter().find(|t| t.id == id).cloned();
                    if let Some(task) = task {
                        rsx! {
                            TaskDetailsPanel {
                                task: task,
                                onclose: move |_| selected_task.set(None),
                            }
                        }
                    } else {
                        rsx! { }
                    }
                } else {
                    rsx! { }
                }
            }
        }
    }
}

#[component]
fn TaskCard(
    task: Task,
    selected: bool,
    dragging: bool,
    is_highlighted: bool,
    is_left_node_hover: bool,
    is_connection_dragging: bool,
    onclick: EventHandler<MouseEvent>,
    onmousedown: EventHandler<MouseEvent>,
    on_right_node_drag_start: EventHandler<MouseEvent>,
    on_left_node_enter: EventHandler<MouseEvent>,
    on_left_node_leave: EventHandler<MouseEvent>,
    onstatuschange: EventHandler<MouseEvent>,
    ondelete: EventHandler<MouseEvent>
) -> Element {
    let status_color = match task.status {
        TaskStatus::Todo => "#808080",
        TaskStatus::InProgress => "#2196F3",
        TaskStatus::Done => "#4CAF50",
        TaskStatus::Blocked => "#f44336",
        _ => "#666",
    };
    
    let priority_color = match task.priority {
        Priority::Critical => "#ff0000",
        Priority::High => "#ff8800",
        Priority::Medium => "#ffaa00",
        Priority::Low => "#888888",
    };
    
    let border_color = if selected {
        "#4CAF50"
    } else if is_highlighted {
        "#4CAF50"
    } else {
        "#ddd"
    };
    
    let opacity = if dragging { "0.6" } else { "1" };
    let cursor = if dragging { "grabbing" } else { "grab" };
    
    let shadow = if is_highlighted {
        "0 4px 20px rgba(76, 175, 80, 0.4)"
    } else if dragging {
        "0 4px 16px rgba(0,0,0,0.3)"
    } else {
        "0 2px 8px rgba(0,0,0,0.1)"
    };
    
    let transform = if is_highlighted {
        "scale(1.05)"
    } else if dragging {
        "scale(1.05)"
    } else {
        "scale(1)"
    };
    
    let pos_x = task.position.x;
    let pos_y = task.position.y;
    
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; \
         width: 200px; height: 60px; background: white; border-radius: 8px; \
         box-shadow: {}; cursor: {}; \
         border: 2px solid {}; opacity: {}; \
         transform: {}; \
         transition: box-shadow 0.2s, opacity 0.2s, transform 0.2s, border-color 0.2s; \
         user-select: none; display: flex; align-items: center;",
        pos_x, pos_y, shadow, cursor, border_color, opacity, transform
    );
    
    rsx! {
        div {
            style: "{card_style}",
            
            onmousedown: move |evt| {
                evt.stop_propagation();
                onmousedown.call(evt);
            },
            
            onclick: move |evt| {
                if !dragging && !is_connection_dragging {
                    evt.stop_propagation();
                    onclick.call(evt);
                }
            },
            
            // Left connection node (input)
            {
                let bg_color = if is_left_node_hover { "#4CAF50" } else { "white" };
                let border_color = if is_left_node_hover { "#4CAF50" } else { "#666" };
                let cursor = if is_connection_dragging { "pointer" } else { "default" };
                
                rsx! {
                    div {
                        style: "position: absolute; left: -8px; top: 50%; transform: translateY(-50%);
                               width: 16px; height: 16px; border-radius: 50%;
                               background: {bg_color};
                               border: 2px solid {border_color};
                               cursor: {cursor};
                               transition: all 0.2s; z-index: 10;
                               box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
                
                onmouseenter: move |evt| {
                    evt.stop_propagation();
                    on_left_node_enter.call(evt);
                },
                
                onmouseleave: move |evt| {
                    evt.stop_propagation();
                    on_left_node_leave.call(evt);
                },
                
                        // Visual indicator when hovering
                        if is_left_node_hover {
                            div {
                                style: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
                                       width: 6px; height: 6px; border-radius: 50%; background: #4CAF50;",
                            }
                        }
                    }
                }
            }
            
            // Right connection node (output)
            div {
                style: "position: absolute; right: -8px; top: 50%; transform: translateY(-50%);
                       width: 16px; height: 16px; border-radius: 50%;
                       background: white; border: 2px solid #666;
                       cursor: grab; transition: all 0.2s; z-index: 10;
                       box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
                
                onmousedown: move |evt| {
                    evt.stop_propagation();
                    on_right_node_drag_start.call(evt);
                },
                
                onmouseenter: move |evt| {
                    evt.stop_propagation();
                },
                
                // Visual indicator
                div {
                    style: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
                           width: 6px; height: 6px; border-radius: 50%; background: #666;",
                }
            }
            
            // Task content
            div {
                style: "flex: 1; padding: 8px 20px; display: flex; flex-direction: column; justify-content: center; pointer-events: none;",
                
                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    
                    h4 { 
                        style: "margin: 0; font-size: 13px; font-weight: 500; flex: 1; 
                               white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", 
                        "{task.title}" 
                    }
                    
                    span {
                        style: "padding: 2px 6px; border-radius: 3px; font-size: 10px; 
                               color: white; background: {status_color}; margin-left: 8px;",
                        "{task.status:?}"
                    }
                }
                
                if !task.description.is_empty() {
                    p {
                        style: "margin: 2px 0 0 0; font-size: 11px; color: #666; 
                               white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{task.description}"
                    }
                } else {
                    div {
                        style: "display: flex; gap: 4px; align-items: center; margin-top: 4px;",
                        
                        span {
                            style: "width: 6px; height: 6px; border-radius: 50%; background: {priority_color};",
                        }
                        
                        span {
                            style: "font-size: 10px; color: #888;",
                            "{task.priority:?}"
                        }
                    }
                }
            }
            
            // Action buttons (only visible when not dragging connection)
            if !is_connection_dragging {
                div {
                    style: "position: absolute; top: 2px; right: 2px; display: flex; gap: 2px;",
                    
                    button {
                        onclick: move |evt| {
                            evt.stop_propagation();
                            ondelete.call(evt);
                        },
                        style: "width: 20px; height: 20px; background: #ffcdd2; color: #c62828; 
                               border: none; border-radius: 3px; cursor: pointer; font-size: 12px;
                               display: flex; align-items: center; justify-content: center;",
                        "×"
                    }
                }
            }
        }
    }
}

#[component]
fn TaskDetailsPanel(
    task: Task,
    onclose: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            style: "position: fixed; right: 0; top: 60px; width: 300px; height: calc(100vh - 60px); 
                   background: white; box-shadow: -2px 0 8px rgba(0,0,0,0.1); padding: 20px; overflow-y: auto;",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                
                h3 { style: "margin: 0;", "Task Details" }
                
                button {
                    onclick: move |evt| onclose.call(evt),
                    style: "padding: 4px 8px; background: #f5f5f5; border: none; border-radius: 4px; cursor: pointer;",
                    "×"
                }
            }
            
            div {
                style: "margin-bottom: 15px;",
                label { style: "font-size: 12px; color: #666;", "Title" }
                h4 { style: "margin: 4px 0;", "{task.title}" }
            }
            
            div {
                style: "margin-bottom: 15px;",
                label { style: "font-size: 12px; color: #666;", "Description" }
                p { style: "margin: 4px 0; color: #333;", 
                    {if task.description.is_empty() { "No description" } else { &task.description }}
                }
            }
            
            div {
                style: "margin-bottom: 15px;",
                label { style: "font-size: 12px; color: #666;", "Status" }
                p { style: "margin: 4px 0;", "{task.status:?}" }
            }
            
            div {
                style: "margin-bottom: 15px;",
                label { style: "font-size: 12px; color: #666;", "Priority" }
                p { style: "margin: 4px 0;", "{task.priority:?}" }
            }
            
            div {
                style: "margin-bottom: 15px;",
                label { style: "font-size: 12px; color: #666;", "Position" }
                p { style: "margin: 4px 0;", "X: {task.position.x:.0}, Y: {task.position.y:.0}" }
            }
        }
    }
}