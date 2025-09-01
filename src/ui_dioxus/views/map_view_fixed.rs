use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Position, Priority};
use crate::domain::goal::Goal;
use crate::domain::dependency::{Dependency, DependencyType};
use crate::ui_dioxus::state_simple::{TaskExecutionStatus, sample_tasks};
use uuid::Uuid;
use std::collections::HashMap;

#[component]
pub fn MapView() -> Element {
    // State for tasks and goals
    let mut tasks = use_signal(|| sample_tasks());
    let mut goals = use_signal(|| Vec::<Goal>::new());
    let mut dependencies = use_signal(|| Vec::<Dependency>::new());
    
    // View state
    let mut camera_x = use_signal(|| 0.0f32);
    let mut camera_y = use_signal(|| 0.0f32);
    let mut zoom_level = use_signal(|| 1.0f32);
    let mut selected_task = use_signal(|| None::<Uuid>);
    let mut selected_goal = use_signal(|| None::<Uuid>);
    
    // Interaction state
    let mut dragging_task = use_signal(|| None::<Uuid>);
    let mut drag_offset_x = use_signal(|| 0.0f32);
    let mut drag_offset_y = use_signal(|| 0.0f32);
    let mut panning = use_signal(|| false);
    let mut pan_start_x = use_signal(|| 0.0f32);
    let mut pan_start_y = use_signal(|| 0.0f32);
    
    // Claude Code execution state
    let mut running_tasks = use_signal(|| HashMap::<Uuid, TaskExecutionStatus>::new());
    
    // Calculate viewBox values
    let zoom = *zoom_level.read();
    let cam_x = *camera_x.read();
    let cam_y = *camera_y.read();
    let view_width = 800.0 / zoom;
    let view_height = 600.0 / zoom;
    let view_x = -400.0 / zoom - cam_x;
    let view_y = -300.0 / zoom - cam_y;
    
    rsx! {
        div {
            style: "width: 100%; height: 100vh; position: relative; overflow: hidden; background: #f5f5f5;",
            
            // Controls toolbar
            div {
                style: "position: absolute; top: 10px; left: 10px; z-index: 100; background: white; padding: 10px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                
                button {
                    onclick: move |_| {
                        let new_zoom = (*zoom_level.read() * 1.2).min(5.0);
                        zoom_level.set(new_zoom);
                    },
                    style: "margin: 0 5px; padding: 5px 10px; cursor: pointer;",
                    "🔍+"
                }
                
                button {
                    onclick: move |_| {
                        let new_zoom = (*zoom_level.read() / 1.2).max(0.1);
                        zoom_level.set(new_zoom);
                    },
                    style: "margin: 0 5px; padding: 5px 10px; cursor: pointer;",
                    "🔍-"
                }
                
                button {
                    onclick: move |_| {
                        zoom_level.set(1.0);
                        camera_x.set(0.0);
                        camera_y.set(0.0);
                    },
                    style: "margin: 0 5px; padding: 5px 10px; cursor: pointer;",
                    "🏠 Reset"
                }
                
                span {
                    style: "margin: 0 10px;",
                    "Zoom: {(zoom * 100.0) as i32}%"
                }
                
                button {
                    onclick: move |_| {
                        tasks.with_mut(|tasks| {
                            let cols = ((tasks.len() as f32).sqrt().ceil()) as usize;
                            for (i, task) in tasks.iter_mut().enumerate() {
                                let row = i / cols;
                                let col = i % cols;
                                task.position.x = (col as f64 * 200.0) - 400.0;
                                task.position.y = (row as f64 * 120.0) - 300.0;
                            }
                        });
                    },
                    style: "margin: 0 5px; padding: 5px 10px; cursor: pointer;",
                    "📊 Auto-arrange"
                }
                
                button {
                    onclick: move |_| {
                        let new_task = Task {
                            id: Uuid::new_v4(),
                            title: "New Task".to_string(),
                            description: String::new(),
                            status: TaskStatus::Todo,
                            priority: Priority::Medium,
                            position: Position { x: -cam_x as f64, y: -cam_y as f64 },
                            ..Default::default()
                        };
                        tasks.write().push(new_task);
                    },
                    style: "margin: 0 5px; padding: 5px 10px; cursor: pointer;",
                    "➕ Add Task"
                }
            }
            
            // Main SVG canvas
            svg {
                style: "width: 100%; height: 100%; cursor: move;",
                view_box: "{view_x} {view_y} {view_width} {view_height}",
                xmlns: "http://www.w3.org/2000/svg",
                
                onmousedown: move |evt| {
                    if evt.modifiers().shift {
                        panning.set(true);
                        pan_start_x.set(evt.page_coordinates().x as f32);
                        pan_start_y.set(evt.page_coordinates().y as f32);
                    }
                },
                
                onmousemove: move |evt| {
                    if *panning.read() {
                        let delta_x = (evt.page_coordinates().x as f32 - *pan_start_x.read()) / zoom;
                        let delta_y = (evt.page_coordinates().y as f32 - *pan_start_y.read()) / zoom;
                        camera_x.set(*camera_x.read() - delta_x);
                        camera_y.set(*camera_y.read() - delta_y);
                        pan_start_x.set(evt.page_coordinates().x as f32);
                        pan_start_y.set(evt.page_coordinates().y as f32);
                    } else if let Some(task_id) = *dragging_task.read() {
                        // Update position of dragged task
                        let world_x = (evt.page_coordinates().x as f32 - 400.0) / zoom - cam_x;
                        let world_y = (evt.page_coordinates().y as f32 - 300.0) / zoom - cam_y;
                        
                        tasks.with_mut(|tasks| {
                            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                task.position.x = (world_x - *drag_offset_x.read()) as f64;
                                task.position.y = (world_y - *drag_offset_y.read()) as f64;
                            }
                        });
                    }
                },
                
                onmouseup: move |_| {
                    panning.set(false);
                    dragging_task.set(None);
                },
                
                onwheel: move |evt| {
                    let delta = evt.delta().y;
                    if delta < 0.0 {
                        let new_zoom = (*zoom_level.read() * 1.1).min(5.0);
                        zoom_level.set(new_zoom);
                    } else {
                        let new_zoom = (*zoom_level.read() / 1.1).max(0.1);
                        zoom_level.set(new_zoom);
                    }
                },
                
                // Grid pattern
                defs {
                    pattern {
                        id: "grid",
                        width: "50",
                        height: "50",
                        pattern_units: "userSpaceOnUse",
                        
                        path {
                            d: "M 50 0 L 0 0 0 50",
                            fill: "none",
                            stroke: "#e0e0e0",
                            stroke_width: "1",
                        }
                    }
                    
                    marker {
                        id: "arrowhead",
                        marker_width: "10",
                        marker_height: "10",
                        ref_x: "9",
                        ref_y: "3",
                        orient: "auto",
                        
                        polygon {
                            points: "0 0, 10 3, 0 6",
                            fill: "#666",
                        }
                    }
                }
                
                rect {
                    x: "-5000",
                    y: "-5000",
                    width: "10000",
                    height: "10000",
                    fill: "url(#grid)",
                }
                
                // Draw dependencies
                for dep in dependencies.read().iter() {
                    if let (Some(from_task), Some(to_task)) = (
                        tasks.read().iter().find(|t| t.id == dep.from_task_id),
                        tasks.read().iter().find(|t| t.id == dep.to_task_id)
                    ) {
                        line {
                            x1: "{from_task.position.x + 75.0}",
                            y1: "{from_task.position.y}",
                            x2: "{to_task.position.x - 75.0}",
                            y2: "{to_task.position.y}",
                            stroke: match dep.dependency_type {
                                DependencyType::FinishToStart => "#4CAF50",
                                DependencyType::StartToStart => "#2196F3",
                                DependencyType::FinishToFinish => "#FF9800",
                                DependencyType::StartToFinish => "#F44336",
                            },
                            stroke_width: "2",
                            marker_end: "url(#arrowhead)",
                        }
                    }
                }
                
                // Draw tasks
                for task in tasks.read().iter() {
                    g {
                        key: "{task.id}",
                        transform: "translate({task.position.x - 75.0}, {task.position.y - 40.0})",
                        
                        // Task rectangle
                        rect {
                            x: "0",
                            y: "0",
                            width: "150",
                            height: "80",
                            rx: "5",
                            fill: match task.status {
                                TaskStatus::Todo => "#c8c8c8",
                                TaskStatus::InProgress => "#6495ff",
                                TaskStatus::Done => "#64ff64",
                                TaskStatus::Blocked => "#ff6464",
                                _ => "#b4b4b4",
                            },
                            stroke: if selected_task.read().as_ref() == Some(&task.id) { "#ffc800" } else { "#646464" },
                            stroke_width: if selected_task.read().as_ref() == Some(&task.id) { "3" } else { "1" },
                            style: "cursor: move;",
                            onmousedown: move |evt| {
                                evt.stop_propagation();
                                selected_task.set(Some(task.id));
                                if !evt.modifiers().shift {
                                    dragging_task.set(Some(task.id));
                                    let world_x = (evt.page_coordinates().x as f32 - 400.0) / zoom - cam_x;
                                    let world_y = (evt.page_coordinates().y as f32 - 300.0) / zoom - cam_y;
                                    drag_offset_x.set(world_x - task.position.x as f32);
                                    drag_offset_y.set(world_y - task.position.y as f32);
                                }
                            },
                        }
                        
                        // Task title
                        text {
                            x: "75",
                            y: "30",
                            text_anchor: "middle",
                            font_size: "14",
                            font_weight: "bold",
                            fill: "#333",
                            style: "pointer-events: none; user-select: none;",
                            "{task.title}"
                        }
                        
                        // Task description (truncated)
                        text {
                            x: "75",
                            y: "50",
                            text_anchor: "middle",
                            font_size: "11",
                            fill: "#666",
                            style: "pointer-events: none; user-select: none;",
                            "{task.description.chars().take(20).collect::<String>()}"
                        }
                        
                        // Priority badge
                        if task.priority != Priority::Medium {
                            rect {
                                x: "5",
                                y: "5",
                                width: "20",
                                height: "15",
                                rx: "3",
                                fill: match task.priority {
                                    Priority::Critical => "#ff0000",
                                    Priority::High => "#ff8800",
                                    Priority::Low => "#888888",
                                    _ => "#666666",
                                },
                            }
                            text {
                                x: "15",
                                y: "15",
                                text_anchor: "middle",
                                font_size: "10",
                                fill: "white",
                                font_weight: "bold",
                                "{match task.priority {
                                    Priority::Critical => \"C\",
                                    Priority::High => \"H\",
                                    Priority::Low => \"L\",
                                    _ => \"M\",
                                }}"
                            }
                        }
                        
                        // Play button for Todo tasks
                        if task.status == TaskStatus::Todo && !running_tasks.read().contains_key(&task.id) {
                            g {
                                transform: "translate(120, 55)",
                                style: "cursor: pointer;",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    running_tasks.write().insert(task.id, TaskExecutionStatus::Running);
                                },
                                
                                circle {
                                    cx: "0",
                                    cy: "0",
                                    r: "12",
                                    fill: "#4CAF50",
                                    stroke: "#fff",
                                    stroke_width: "2",
                                }
                                
                                polygon {
                                    points: "-5,-7 -5,7 7,0",
                                    fill: "white",
                                }
                            }
                        }
                        
                        // Running indicator
                        if running_tasks.read().contains_key(&task.id) {
                            g {
                                transform: "translate(120, 55)",
                                
                                circle {
                                    cx: "0",
                                    cy: "0",
                                    r: "12",
                                    fill: "#FFA500",
                                    stroke: "#fff",
                                    stroke_width: "2",
                                }
                                
                                rect {
                                    x: "-4",
                                    y: "-4",
                                    width: "8",
                                    height: "8",
                                    fill: "white",
                                }
                            }
                        }
                    }
                }
            }
            
            // Task details panel
            if let Some(task_id) = selected_task.read().as_ref() {
                if let Some(task) = tasks.read().iter().find(|t| t.id == *task_id) {
                    div {
                        style: "position: absolute; right: 10px; top: 10px; width: 300px; background: white; 
                                border-radius: 8px; padding: 15px; box-shadow: 0 4px 12px rgba(0,0,0,0.15); z-index: 200;",
                        
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;",
                            
                            h3 {
                                style: "margin: 0;",
                                "{task.title}"
                            }
                            
                            button {
                                onclick: move |_| selected_task.set(None),
                                style: "background: none; border: none; font-size: 20px; cursor: pointer;",
                                "×"
                            }
                        }
                        
                        div {
                            style: "margin-bottom: 10px;",
                            label { style: "font-weight: bold;", "Status: " }
                            span { 
                                style: match task.status {
                                    TaskStatus::Todo => "color: #808080;",
                                    TaskStatus::InProgress => "color: #6495ff;",
                                    TaskStatus::Done => "color: #64ff64;",
                                    TaskStatus::Blocked => "color: #ff6464;",
                                    _ => "color: #666;",
                                },
                                "{task.status:?}" 
                            }
                        }
                        
                        div {
                            style: "margin-bottom: 10px;",
                            label { style: "font-weight: bold;", "Priority: " }
                            span { 
                                style: match task.priority {
                                    Priority::Critical => "color: #ff0000;",
                                    Priority::High => "color: #ff8800;",
                                    Priority::Low => "color: #888888;",
                                    _ => "color: #666;",
                                },
                                "{task.priority:?}" 
                            }
                        }
                        
                        div {
                            style: "margin-bottom: 10px;",
                            label { style: "font-weight: bold; display: block; margin-bottom: 5px;", "Description:" }
                            p {
                                style: "margin: 0; padding: 10px; background: #f5f5f5; border-radius: 4px; min-height: 50px;",
                                "{task.description}"
                                if task.description.is_empty() { "No description" }
                            }
                        }
                        
                        if let Some(due_date) = task.due_date {
                            div {
                                style: "margin-bottom: 10px;",
                                label { style: "font-weight: bold;", "Due Date: " }
                                span { "{due_date.format(\"%Y-%m-%d\")}" }
                            }
                        }
                        
                        div {
                            style: "margin-top: 15px; display: flex; gap: 10px;",
                            
                            button {
                                onclick: move |_| {
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
                                style: "flex: 1; padding: 8px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                "Change Status"
                            }
                            
                            button {
                                onclick: move |_| {
                                    tasks.with_mut(|tasks| {
                                        tasks.retain(|t| t.id != task.id);
                                    });
                                    selected_task.set(None);
                                },
                                style: "flex: 1; padding: 8px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}