use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Position, Priority};
use crate::domain::goal::Goal;
use crate::domain::dependency::{Dependency, DependencyType};
use crate::ui_dioxus::state_simple::{TaskExecutionStatus, sample_tasks};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Utc;

#[component]
pub fn MapView() -> Element {
    // State for tasks and goals
    let mut tasks = use_signal(|| sample_tasks());
    let mut goals = use_signal(|| Vec::<Goal>::new());
    let mut dependencies = use_signal(|| Vec::<Dependency>::new());
    
    // View state
    let mut camera_pos = use_signal(|| (0.0f32, 0.0f32));
    let mut zoom_level = use_signal(|| 1.0f32);
    let mut selected_task = use_signal(|| None::<Uuid>);
    let mut selected_goal = use_signal(|| None::<Uuid>);
    
    // Interaction state
    let mut dragging_item = use_signal(|| None::<DragState>);
    let mut panning = use_signal(|| false);
    let mut pan_start = use_signal(|| (0.0f32, 0.0f32));
    let mut creating_dependency = use_signal(|| false);
    let mut dependency_source = use_signal(|| None::<Uuid>);
    
    // Claude Code execution state
    let mut running_tasks = use_signal(|| HashMap::<Uuid, TaskExecutionStatus>::new());
    let mut spinner_rotation = use_signal(|| 0.0f32);
    
    // Update spinner rotation
    use_future(move || async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            spinner_rotation.with_mut(|r| *r = (*r + 0.1) % (std::f32::consts::PI * 2.0));
        }
    });
    
    rsx! {
        div {
            class: "map-view",
            style: "width: 100%; height: 100vh; position: relative; overflow: hidden;",
            
            // Controls toolbar
            div {
                class: "map-controls",
                style: "position: absolute; top: 10px; left: 10px; z-index: 100; background: white; padding: 10px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                
                button {
                    onclick: move |_| zoom_level.with_mut(|z| *z = (*z * 1.2).min(5.0)),
                    style: "margin: 0 5px; padding: 5px 10px;",
                    "🔍+"
                }
                
                button {
                    onclick: move |_| zoom_level.with_mut(|z| *z = (*z / 1.2).max(0.1)),
                    style: "margin: 0 5px; padding: 5px 10px;",
                    "🔍-"
                }
                
                button {
                    onclick: move |_| {
                        zoom_level.set(1.0);
                        camera_pos.set((0.0, 0.0));
                    },
                    style: "margin: 0 5px; padding: 5px 10px;",
                    "🏠 Reset"
                }
                
                span {
                    style: "margin: 0 10px;",
                    "Zoom: {(*zoom_level.read() * 100.0) as i32}%"
                }
                
                button {
                    onclick: move |_| auto_arrange(&mut tasks),
                    style: "margin: 0 5px; padding: 5px 10px;",
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
                            position: Position { 
                                x: -camera_pos.read().0 as f64, 
                                y: -camera_pos.read().1 as f64 
                            },
                            ..Default::default()
                        };
                        tasks.write().push(new_task);
                    },
                    style: "margin: 0 5px; padding: 5px 10px;",
                    "➕ Add Task"
                }
            }
            
            // Main SVG canvas
            svg {
                style: "width: 100%; height: 100%; background: #f5f5f5;",
                viewBox: format!("{} {} {} {}", 
                    -400.0 / zoom_level.read() - camera_pos.read().0,
                    -300.0 / zoom_level.read() - camera_pos.read().1,
                    800.0 / zoom_level.read(),
                    600.0 / zoom_level.read()
                ),
                
                // Event handlers for panning
                onmousedown: move |evt| {
                    if evt.modifiers().shift() || evt.buttons() == 4 { // Middle mouse
                        panning.set(true);
                        pan_start.set((evt.client_x() as f32, evt.client_y() as f32));
                    }
                },
                
                onmousemove: move |evt| {
                    if *panning.read() {
                        let delta_x = (evt.client_x() as f32 - pan_start.read().0) / zoom_level.read();
                        let delta_y = (evt.client_y() as f32 - pan_start.read().1) / zoom_level.read();
                        camera_pos.with_mut(|pos| {
                            pos.0 -= delta_x;
                            pos.1 -= delta_y;
                        });
                        pan_start.set((evt.client_x() as f32, evt.client_y() as f32));
                    } else if let Some(drag) = dragging_item.read().as_ref() {
                        // Update position of dragged item
                        let world_x = evt.client_x() as f32 / zoom_level.read() - camera_pos.read().0;
                        let world_y = evt.client_y() as f32 / zoom_level.read() - camera_pos.read().1;
                        
                        tasks.with_mut(|tasks| {
                            if let Some(task) = tasks.iter_mut().find(|t| t.id == drag.item_id) {
                                task.position.x = (world_x - drag.offset.0) as f64;
                                task.position.y = (world_y - drag.offset.1) as f64;
                            }
                        });
                    }
                },
                
                onmouseup: move |_| {
                    panning.set(false);
                    dragging_item.set(None);
                    if *creating_dependency.read() {
                        creating_dependency.set(false);
                        dependency_source.set(None);
                    }
                },
                
                onwheel: move |evt| {
                    let delta = evt.delta_y();
                    if delta < 0.0 {
                        zoom_level.with_mut(|z| *z = (*z * 1.1).min(5.0));
                    } else {
                        zoom_level.with_mut(|z| *z = (*z / 1.1).max(0.1));
                    }
                },
                
                // Grid pattern
                defs {
                    pattern {
                        id: "grid",
                        width: "50",
                        height: "50",
                        patternUnits: "userSpaceOnUse",
                        
                        path {
                            d: "M 50 0 L 0 0 0 50",
                            fill: "none",
                            stroke: "#e0e0e0",
                            stroke_width: "1",
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
                        DependencyArrow {
                            from_pos: from_task.position.clone(),
                            to_pos: to_task.position.clone(),
                            dep_type: dep.dependency_type.clone(),
                        }
                    }
                }
                
                // Draw dependency creation preview
                if *creating_dependency.read() {
                    if let Some(source_id) = dependency_source.read() {
                        if let Some(source) = tasks.read().iter().find(|t| t.id == *source_id) {
                            line {
                                x1: "{source.position.x + 75.0}",
                                y1: "{source.position.y}",
                                x2: "{source.position.x + 150.0}",
                                y2: "{source.position.y}",
                                stroke: "#6495ED",
                                stroke_width: "2",
                                stroke_dasharray: "5,5",
                            }
                        }
                    }
                }
                
                // Draw goals
                for goal in goals.read().iter() {
                    GoalNode {
                        goal: goal.clone(),
                        selected: selected_goal.read().as_ref() == Some(&goal.id),
                        onclick: move |_| selected_goal.set(Some(goal.id)),
                    }
                }
                
                // Draw tasks
                for task in tasks.read().iter() {
                    TaskNode {
                        task: task.clone(),
                        selected: selected_task.read().as_ref() == Some(&task.id),
                        running: running_tasks.read().contains_key(&task.id),
                        spinner_rotation: *spinner_rotation.read(),
                        onclick: move |_| selected_task.set(Some(task.id)),
                        onmousedown: move |evt| {
                            evt.stop_propagation();
                            if !evt.modifiers().shift() {
                                dragging_item.set(Some(DragState {
                                    item_id: task.id,
                                    offset: (
                                        evt.client_x() as f32 / zoom_level.read() - task.position.x as f32,
                                        evt.client_y() as f32 / zoom_level.read() - task.position.y as f32,
                                    ),
                                }));
                            }
                        },
                        onplay: move |_| {
                            running_tasks.write().insert(task.id, TaskExecutionStatus::Running);
                            // Here you would start actual Claude Code execution
                        },
                        ondependency: move |_| {
                            creating_dependency.set(true);
                            dependency_source.set(Some(task.id));
                        },
                    }
                }
            }
            
            // Task details panel
            if let Some(task_id) = selected_task.read() {
                if let Some(task) = tasks.read().iter().find(|t| t.id == *task_id) {
                    TaskDetailsPanel {
                        task: task.clone(),
                        onclose: move |_| selected_task.set(None),
                        onupdate: move |updated_task| {
                            tasks.with_mut(|tasks| {
                                if let Some(t) = tasks.iter_mut().find(|t| t.id == updated_task.id) {
                                    *t = updated_task;
                                }
                            });
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn TaskNode(
    task: Task,
    selected: bool,
    running: bool,
    spinner_rotation: f32,
    onclick: EventHandler<MouseEvent>,
    onmousedown: EventHandler<MouseEvent>,
    onplay: EventHandler<MouseEvent>,
    ondependency: EventHandler<MouseEvent>,
) -> Element {
    let fill_color = match task.status {
        TaskStatus::Todo => "#c8c8c8",
        TaskStatus::InProgress => "#6495ff",
        TaskStatus::Done => "#64ff64",
        TaskStatus::Blocked => "#ff6464",
        _ => "#b4b4b4",
    };
    
    let stroke_color = if selected { "#ffc800" } else { "#646464" };
    let stroke_width = if selected { "3" } else { "1" };
    
    rsx! {
        g {
            transform: "translate({task.position.x - 75.0}, {task.position.y - 40.0})",
            
            // Task rectangle
            rect {
                x: "0",
                y: "0",
                width: "150",
                height: "80",
                rx: "5",
                fill: fill_color,
                stroke: stroke_color,
                stroke_width: stroke_width,
                onclick: move |evt| onclick.call(evt),
                onmousedown: move |evt| onmousedown.call(evt),
                style: "cursor: move;",
            }
            
            // Left connection point (dependency target)
            circle {
                cx: "0",
                cy: "40",
                r: "6",
                fill: "#4CAF50",
                stroke: "#fff",
                stroke_width: "2",
                style: "cursor: crosshair;",
            }
            
            // Right connection point (dependency source)
            circle {
                cx: "150",
                cy: "40",
                r: "6",
                fill: "#2196F3",
                stroke: "#fff",
                stroke_width: "2",
                style: "cursor: crosshair;",
                onclick: move |evt| {
                    evt.stop_propagation();
                    ondependency.call(evt);
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
            if task.status == TaskStatus::Todo && !running {
                g {
                    transform: "translate(120, 55)",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        onplay.call(evt);
                    },
                    style: "cursor: pointer;",
                    
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
            if running {
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
                    
                    g {
                        transform: format!("rotate({} 0 0)", spinner_rotation * 180.0 / std::f32::consts::PI),
                        
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
}

#[component]
fn GoalNode(goal: Goal, selected: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let stroke_color = if selected { "#ffc800" } else { "#FFA500" };
    let stroke_width = if selected { "3" } else { "2" };
    
    rsx! {
        g {
            transform: "translate({500.0}, {300.0})", // Default position
            onclick: move |evt| onclick.call(evt),
            style: "cursor: pointer;",
            
            circle {
                cx: "0",
                cy: "0",
                r: "30",
                fill: "#FFD700",
                stroke: stroke_color,
                stroke_width: stroke_width,
            }
            
            text {
                x: "0",
                y: "5",
                text_anchor: "middle",
                font_size: "14",
                font_weight: "bold",
                fill: "#333",
                "{goal.title}"
            }
        }
    }
}

#[component]
fn DependencyArrow(from_pos: Position, to_pos: Position, dep_type: DependencyType) -> Element {
    let color = match dep_type {
        DependencyType::FinishToStart => "#4CAF50",
        DependencyType::StartToStart => "#2196F3",
        DependencyType::FinishToFinish => "#FF9800",
        DependencyType::StartToFinish => "#F44336",
    };
    
    rsx! {
        line {
            x1: "{from_pos.x + 75.0}",
            y1: "{from_pos.y}",
            x2: "{to_pos.x - 75.0}",
            y2: "{to_pos.y}",
            stroke: color,
            stroke_width: "2",
            marker_end: "url(#arrowhead)",
        }
    }
}

#[component]
fn TaskDetailsPanel(task: Task, onclose: EventHandler<MouseEvent>, onupdate: EventHandler<Task>) -> Element {
    let mut editing_title = use_signal(|| false);
    let mut title_value = use_signal(|| task.title.clone());
    let mut description_value = use_signal(|| task.description.clone());
    
    rsx! {
        div {
            class: "task-details-panel",
            style: "position: absolute; right: 10px; top: 10px; width: 300px; background: white; 
                    border-radius: 8px; padding: 15px; box-shadow: 0 4px 12px rgba(0,0,0,0.15); z-index: 200;",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;",
                
                if *editing_title.read() {
                    input {
                        value: "{title_value}",
                        oninput: move |evt| title_value.set(evt.value()),
                        onkeypress: move |evt| {
                            if evt.key() == "Enter" {
                                editing_title.set(false);
                                let mut updated = task.clone();
                                updated.title = title_value.read().clone();
                                onupdate.call(updated);
                            }
                        },
                        style: "font-size: 18px; font-weight: bold; border: 1px solid #ddd; padding: 5px;",
                    }
                } else {
                    h3 {
                        style: "margin: 0; cursor: pointer;",
                        ondoubleclick: move |_| editing_title.set(true),
                        "{task.title}"
                    }
                }
                
                button {
                    onclick: move |evt| onclose.call(evt),
                    style: "background: none; border: none; font-size: 20px; cursor: pointer;",
                    "×"
                }
            }
            
            div {
                style: "margin-bottom: 10px;",
                label { style: "font-weight: bold;", "Status: " }
                select {
                    value: "{task.status:?}",
                    onchange: move |evt| {
                        let mut updated = task.clone();
                        updated.status = match evt.value().as_str() {
                            "Todo" => TaskStatus::Todo,
                            "InProgress" => TaskStatus::InProgress,
                            "Done" => TaskStatus::Done,
                            "Blocked" => TaskStatus::Blocked,
                            _ => TaskStatus::Todo,
                        };
                        onupdate.call(updated);
                    },
                    style: "margin-left: 10px; padding: 5px;",
                    
                    option { value: "Todo", "Todo" }
                    option { value: "InProgress", "In Progress" }
                    option { value: "Done", "Done" }
                    option { value: "Blocked", "Blocked" }
                }
            }
            
            div {
                style: "margin-bottom: 10px;",
                label { style: "font-weight: bold;", "Priority: " }
                select {
                    value: "{task.priority:?}",
                    onchange: move |evt| {
                        let mut updated = task.clone();
                        updated.priority = match evt.value().as_str() {
                            "Critical" => Priority::Critical,
                            "High" => Priority::High,
                            "Low" => Priority::Low,
                            _ => Priority::Medium,
                        };
                        onupdate.call(updated);
                    },
                    style: "margin-left: 10px; padding: 5px;",
                    
                    option { value: "Critical", "Critical" }
                    option { value: "High", "High" }
                    option { value: "Medium", "Medium" }
                    option { value: "Low", "Low" }
                }
            }
            
            div {
                style: "margin-bottom: 10px;",
                label { style: "font-weight: bold; display: block; margin-bottom: 5px;", "Description:" }
                textarea {
                    value: "{description_value}",
                    oninput: move |evt| description_value.set(evt.value()),
                    onblur: move |_| {
                        let mut updated = task.clone();
                        updated.description = description_value.read().clone();
                        onupdate.call(updated);
                    },
                    style: "width: 100%; min-height: 100px; padding: 5px; border: 1px solid #ddd; border-radius: 4px;",
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
                    style: "flex: 1; padding: 8px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Save"
                }
                
                button {
                    style: "flex: 1; padding: 8px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Delete"
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct DragState {
    item_id: Uuid,
    offset: (f32, f32),
}

fn auto_arrange(tasks: &mut Signal<Vec<Task>>) {
    tasks.with_mut(|tasks| {
        let cols = ((tasks.len() as f32).sqrt().ceil()) as usize;
        for (i, task) in tasks.iter_mut().enumerate() {
            let row = i / cols;
            let col = i % cols;
            task.position.x = (col as f64 * 200.0) - 400.0;
            task.position.y = (row as f64 * 120.0) - 300.0;
        }
    });
}