use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Position, Priority};
use crate::domain::dependency::{Dependency, DependencyType, DependencyGraph};
use crate::repository::{Repository, database::init_database};
use uuid::Uuid;
use std::env::current_dir;

#[derive(Clone, Debug, PartialEq)]
struct DragState {
    from_task: Uuid,
    start_pos: (f64, f64),
}

#[component]
pub fn MapView() -> Element {
    let mut tasks = use_signal(|| Vec::<Task>::new());
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
        
        match init_database(db_path.to_str().unwrap_or("plon.db")).await {
            Ok(pool) => Some(Repository::new(pool)),
            Err(e) => {
                eprintln!("Failed to initialize database: {}", e);
                None
            }
        }
    });
    
    // Load tasks and dependencies from database once repository is ready
    use_effect(move || {
        spawn(async move {
            println!("Waiting for repository to be ready...");
            
            // Wait for repository resource to be ready (poll every 100ms for up to 5 seconds)
            let mut attempts = 0;
            loop {
                if let Some(Some(repo)) = repository.read().as_ref() {
                    println!("Repository ready, loading data...");
                    
                    // Load tasks
                    use crate::repository::task_repository::TaskFilters;
                    match repo.tasks.list(TaskFilters::default()).await {
                        Ok(loaded_tasks) => {
                            println!("Loaded {} tasks from database", loaded_tasks.len());
                            tasks.set(loaded_tasks);
                        }
                        Err(e) => {
                            eprintln!("Failed to load tasks: {}", e);
                        }
                    }
                    
                    // Load dependencies
                    match repo.dependencies.list_all().await {
                        Ok(deps) => {
                            println!("Loaded {} dependencies from database", deps.len());
                            dependencies.set(deps);
                        }
                        Err(e) => {
                            eprintln!("Failed to load dependencies: {}", e);
                        }
                    }
                    
                    break;
                }
                
                attempts += 1;
                if attempts > 50 {
                    eprintln!("Repository failed to initialize after 5 seconds");
                    break;
                }
                
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
    });
    
    // Helper to get task connection points
    let get_task_left_point = |task: &Task| -> (f64, f64) {
        (task.position.x, task.position.y + 30.0)
    };
    
    let get_task_right_point = |task: &Task| -> (f64, f64) {
        (task.position.x + 200.0, task.position.y + 30.0)
    };
    
    // Helper to check for circular dependency
    let would_create_cycle = move |from: Uuid, to: Uuid| -> bool {
        let mut graph = DependencyGraph::new();
        
        for dep in dependencies.read().iter() {
            let _ = graph.add_dependency(dep);
        }
        
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
                
                if dragging_connection.read().is_some() {
                    span {
                        style: "margin-left: 20px; padding: 8px 12px; background: linear-gradient(90deg, #FFC107, #FFD54F); color: #333; border-radius: 4px; font-weight: 500;",
                        "📍 Drop on left node to connect"
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
                        
                        tasks.write().push(new_task.clone());
                        
                        // Persist to database
                        let repository = repository.clone();
                        spawn(async move {
                            if let Some(Some(repo)) = repository.read().as_ref() {
                                let _ = repo.tasks.create(&new_task).await;
                            }
                        });
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
                id: "map-container",
                style: "flex: 1; position: relative; overflow: auto; background: #fafafa;",
                
                
                // SVG layer for dependencies
                svg {
                    style: "position: absolute; top: 0; left: 0; width: 2000px; height: 2000px; pointer-events: none; transform: scale({zoom.read()}); transform-origin: top left;",
                    
                    // Define gradients and markers
                    defs {
                        linearGradient {
                            id: "dep-gradient",
                            x1: "0%", y1: "0%", x2: "100%", y2: "0%",
                            stop { offset: "0%", style: "stop-color:#666;stop-opacity:1" }
                            stop { offset: "100%", style: "stop-color:#999;stop-opacity:0.8" }
                        }
                        
                        linearGradient {
                            id: "dep-gradient-hover",
                            x1: "0%", y1: "0%", x2: "100%", y2: "0%",
                            stop { offset: "0%", style: "stop-color:#4CAF50;stop-opacity:1" }
                            stop { offset: "50%", style: "stop-color:#66BB6A;stop-opacity:1" }
                            stop { offset: "100%", style: "stop-color:#81C784;stop-opacity:1" }
                        }
                        
                        marker {
                            id: "arrowhead",
                            "markerWidth": "10",
                            "markerHeight": "10",
                            "refX": "9",
                            "refY": "3",
                            orient: "auto",
                            "markerUnits": "strokeWidth",
                            path {
                                d: "M0,0 L0,6 L9,3 z",
                                fill: "#999"
                            }
                        }
                        
                        marker {
                            id: "arrowhead-hover",
                            "markerWidth": "12",
                            "markerHeight": "12",
                            "refX": "11",
                            "refY": "4",
                            orient: "auto",
                            "markerUnits": "strokeWidth",
                            path {
                                d: "M0,0 L0,8 L11,4 z",
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
                                
                                let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
                                let ctrl_offset = (distance / 3.0).min(80.0).max(20.0);
                                
                                let path = format!("M {} {} C {} {}, {} {}, {} {}", 
                                    x1, y1, 
                                    x1 + ctrl_offset, y1,
                                    x2 - ctrl_offset, y2,
                                    x2, y2
                                );
                                
                                let stroke = if is_hovered { "url(#dep-gradient-hover)" } else { "url(#dep-gradient)" };
                                let marker = if is_hovered { "url(#arrowhead-hover)" } else { "url(#arrowhead)" };
                                
                                rsx! {
                                    if is_hovered {
                                        path {
                                            d: "{path}",
                                            stroke: "#4CAF50",
                                            "stroke-width": "6",
                                            fill: "none",
                                            opacity: "0.2",
                                            "stroke-linecap": "round",
                                        }
                                    }
                                    
                                    path {
                                        d: "{path}",
                                        stroke: stroke,
                                        "stroke-width": "2",
                                        fill: "none",
                                        "marker-end": marker,
                                        "stroke-dasharray": "10,5",
                                        "stroke-linecap": "round",
                                        style: "pointer-events: stroke; cursor: pointer;",
                                        
                                        onmouseenter: move |_| {
                                            hover_dependency.set(Some((dep.from_task_id, dep.to_task_id)));
                                        },
                                        
                                        onmouseleave: move |_| {
                                            hover_dependency.set(None);
                                        },
                                        
                                        oncontextmenu: move |evt| {
                                            evt.stop_propagation();
                                            
                                            dependencies.with_mut(|deps| {
                                                deps.retain(|d| d.id != dep.id);
                                            });
                                            
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
                    
                    // Preview line when dragging
                    if let Some(drag_state) = dragging_connection.read().as_ref() {
                        {
                            let (x1, y1) = drag_state.start_pos;
                            let (mouse_x, mouse_y) = *mouse_position.read();
                            let zoom_factor = *zoom.read() as f64;
                            
                            // Correct mouse position calculation (simplified)
                            let x2 = mouse_x / zoom_factor;
                            let y2 = mouse_y / zoom_factor;
                            
                            let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
                            let ctrl_offset = (distance / 3.0).min(80.0).max(20.0);
                            
                            let path = format!("M {} {} C {} {}, {} {}, {} {}", 
                                x1, y1, 
                                x1 + ctrl_offset, y1,
                                x2 - ctrl_offset, y2,
                                x2, y2
                            );
                            
                            rsx! {
                                path {
                                    d: "{path}",
                                    stroke: "url(#dep-gradient-hover)",
                                    "stroke-width": "2",
                                    "stroke-dasharray": "8,4",
                                    fill: "none",
                                    "marker-end": "url(#arrowhead-hover)",
                                    "stroke-linecap": "round",
                                    opacity: "0.8"
                                }
                            }
                        }
                    }
                }
                
                // Task cards container
                div {
                    style: "position: relative; width: 2000px; height: 2000px; transform: scale({zoom.read()}); transform-origin: top left;",
                    
                    onmousemove: move |evt| {
                        // Update mouse position
                        let coords = evt.client_coordinates();
                        mouse_position.set((coords.x as f64, coords.y as f64));
                        
                        // Handle task dragging
                        if let Some(task_id) = *dragging_task.read() {
                            if dragging_connection.read().is_none() {
                                let zoom_factor = *zoom.read();
                                let start_mouse = *drag_start_mouse.read();
                                let start_pos = *drag_start_task_pos.read();
                                
                                let delta_x = (coords.x as f64 - start_mouse.0) / zoom_factor as f64;
                                let delta_y = (coords.y as f64 - start_mouse.1) / zoom_factor as f64;
                                
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
                        // Save task position if we were dragging a task
                        if let Some(task_id) = *dragging_task.read() {
                            if let Some(task) = tasks.read().iter().find(|t| t.id == task_id).cloned() {
                                spawn(async move {
                                    if let Some(Some(repo)) = repository.read().as_ref() {
                                        let _ = repo.tasks.update(&task).await;
                                    }
                                });
                            }
                        }
                        
                        dragging_task.set(None);
                        
                        if let Some(drag_state) = dragging_connection.read().as_ref() {
                            if let Some(target_task_id) = *hover_left_node.read() {
                                let from_id = drag_state.from_task;
                                
                                if from_id != target_task_id {
                                    if would_create_cycle(from_id, target_task_id) {
                                        error_message.set(Some("Cannot create dependency: would create a circular dependency".to_string()));
                                    } else {
                                        let new_dep = Dependency::new(from_id, target_task_id, DependencyType::FinishToStart);
                                        
                                        dependencies.with_mut(|deps| {
                                            deps.push(new_dep.clone());
                                        });
                                        
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
                    
                    // Render tasks
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
                                    tasks_signal: tasks.clone(),
                                    
                                    onclick: move |_| {
                                        if dragging_connection.read().is_none() {
                                            selected_task.set(Some(task.id));
                                        }
                                    },
                                    
                                    onmousedown: move |evt: MouseEvent| {
                                        if dragging_connection.read().is_none() {
                                            dragging_task.set(Some(task.id));
                                            let coords = evt.client_coordinates();
                                            drag_start_mouse.set((coords.x as f64, coords.y as f64));
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
    tasks_signal: Signal<Vec<Task>>,
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
    
    let transform = if is_highlighted || dragging {
        "scale(1.05)"
    } else {
        "scale(1)"
    };
    
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; \
         width: 200px; height: 60px; background: white; border-radius: 8px; \
         box-shadow: {}; cursor: {}; \
         border: 2px solid {}; opacity: {}; \
         transform: {}; \
         transition: all 0.2s ease; \
         user-select: none; display: flex; align-items: center;",
        task.position.x, task.position.y, shadow, cursor, border_color, opacity, transform
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
            
            // Left connection node
            {
                let node_transform = if is_left_node_hover { "translateY(-50%) scale(1.15)" } else { "translateY(-50%)" };
                let node_bg = if is_left_node_hover { "#4CAF50" } else { "white" };
                let node_border = if is_left_node_hover { "#4CAF50" } else { "#999" };
                let node_cursor = if is_connection_dragging { "pointer" } else { "default" };
                let node_shadow = if is_left_node_hover { "0 4px 10px rgba(76, 175, 80, 0.4)" } else { "0 2px 6px rgba(0,0,0,0.2)" };
                
                let node_style = format!(
                    "position: absolute; left: -10px; top: 50%; \
                     transform: {}; \
                     width: 20px; height: 20px; border-radius: 50%; \
                     background: {}; \
                     border: 3px solid {}; \
                     cursor: {}; \
                     transition: all 0.2s; z-index: 10; \
                     box-shadow: {};",
                    node_transform, node_bg, node_border, node_cursor, node_shadow
                );
                
                rsx! {
                    div {
                        style: "{node_style}",
                
                onmouseenter: move |evt| {
                    evt.stop_propagation();
                    on_left_node_enter.call(evt);
                },
                
                onmouseleave: move |evt| {
                    evt.stop_propagation();
                    on_left_node_leave.call(evt);
                },
                
                        if is_left_node_hover {
                            div {
                                style: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
                                       width: 8px; height: 8px; border-radius: 50%; background: white;",
                            }
                        }
                    }
                }
            }
            
            // Right connection node
            div {
                style: "position: absolute; right: -10px; top: 50%; transform: translateY(-50%);
                       width: 20px; height: 20px; border-radius: 50%;
                       background: white; border: 3px solid #666;
                       cursor: grab; transition: all 0.2s; z-index: 10;
                       box-shadow: 0 2px 6px rgba(0,0,0,0.2);",
                
                onmousedown: move |evt| {
                    evt.stop_propagation();
                    on_right_node_drag_start.call(evt);
                },
                
                div {
                    style: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
                           width: 8px; height: 8px; border-radius: 50%; background: #666;",
                }
            }
            
            // Task content
            div {
                style: "flex: 1; padding: 8px 24px; display: flex; flex-direction: column; justify-content: center; pointer-events: none;",
                
                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    
                    h4 { 
                        style: "margin: 0; font-size: 13px; font-weight: 500; flex: 1;", 
                        "{task.title}" 
                    }
                    
                    span {
                        style: "padding: 2px 6px; border-radius: 3px; font-size: 10px; 
                               color: white; background: {status_color};",
                        "{task.status:?}"
                    }
                }
                
                if !task.description.is_empty() {
                    p {
                        style: "margin: 2px 0 0 0; font-size: 11px; color: #666;",
                        "{task.description}"
                    }
                }
            }
            
            if !is_connection_dragging {
                // Claude Code button (play/in-progress indicator)
                {
                    let bg_color = if task.status == TaskStatus::InProgress { "#FF9800" } else { "#4CAF50" };
                    let icon = if task.status == TaskStatus::InProgress { "⚡" } else { "▶" };
                    let tooltip = if task.status == TaskStatus::InProgress { 
                        "Task in progress" 
                    } else { 
                        "Launch Claude Code to work on this task" 
                    };
                    
                    rsx! {
                        button {
                            onclick: move |evt| {
                                evt.stop_propagation();
                                
                                // Launch Claude Code for this task
                                let task_clone = task.clone();
                                let task_id = task.id;
                                spawn(async move {
                                    use crate::services::ClaudeAutomation;
                                    use std::env::current_dir;
                                    
                                    let workspace_dir = current_dir().unwrap_or_default();
                                    let automation = ClaudeAutomation::new(workspace_dir);
                                    
                                    // For now, using local repo - in future could get from config
                                    let repo_url = "https://github.com/user/repo.git";
                                    
                                    match automation.execute_task(&task_clone, repo_url).await {
                                        Ok(_) => {
                                            println!("✅ Claude Code launched for task: {}", task_clone.title);
                                            
                                            // Update task status to InProgress in UI
                                            tasks_signal.with_mut(|tasks| {
                                                if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id) {
                                                    t.status = TaskStatus::InProgress;
                                                }
                                            });
                                        }
                                        Err(e) => eprintln!("❌ Failed to launch Claude Code: {}", e),
                                    }
                                });
                            },
                            style: "position: absolute; top: 2px; right: 26px; width: 24px; height: 20px; 
                                   background: {bg_color}; color: white; border: none; border-radius: 3px; 
                                   cursor: pointer; font-size: 12px; display: flex; align-items: center; justify-content: center;
                                   transition: background 0.2s;",
                            title: "{tooltip}",
                            
                            "{icon}"
                        }
                    }
                }
                
                // Delete button
                button {
                    onclick: move |evt| {
                        evt.stop_propagation();
                        ondelete.call(evt);
                    },
                    style: "position: absolute; top: 2px; right: 2px; width: 20px; height: 20px; 
                           background: #ffcdd2; color: #c62828; border: none; border-radius: 3px; 
                           cursor: pointer; font-size: 12px;",
                    "×"
                }
            }
        }
    }
}