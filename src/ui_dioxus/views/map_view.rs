use dioxus::prelude::*;
use fermi::prelude::*;
use crate::domain::task::{Task, TaskStatus, Position};
use crate::domain::goal::Goal;
use crate::domain::dependency::{Dependency, DependencyType};
use crate::ui_dioxus::state::{TASKS, GOALS, DEPENDENCIES, SELECTED_TASK, SELECTED_GOAL, ZOOM_LEVEL, CAMERA_POSITION, RUNNING_TASKS, TaskExecutionStatus};
use uuid::Uuid;
use std::collections::HashMap;

#[component]
pub fn MapView() -> Element {
    let tasks = use_atom_state(&TASKS);
    let goals = use_atom_state(&GOALS);
    let dependencies = use_atom_state(&DEPENDENCIES);
    let selected_task = use_atom_state(&SELECTED_TASK);
    let selected_goal = use_atom_state(&SELECTED_GOAL);
    let zoom = use_atom_state(&ZOOM_LEVEL);
    let camera = use_atom_state(&CAMERA_POSITION);
    let running_tasks = use_atom_state(&RUNNING_TASKS);
    
    // Mouse interaction state
    let dragging = use_signal(|| None::<DragState>);
    let hovering = use_signal(|| None::<Uuid>);
    
    // Canvas reference for interactions
    let canvas_ref = use_node_ref();
    
    rsx! {
        div {
            class: "map-view",
            
            // Toolbar
            MapToolbar {}
            
            // Main canvas area
            div {
                class: "map-canvas-container",
                onmousedown: move |evt| handle_mouse_down(&evt, &dragging, &selected_task, &tasks, &camera, &zoom),
                onmousemove: move |evt| handle_mouse_move(&evt, &dragging, &hovering, &tasks, &camera, &zoom),
                onmouseup: move |_| handle_mouse_up(&dragging),
                onwheel: move |evt| handle_wheel(&evt, &zoom, &camera),
                
                svg {
                    class: "map-canvas",
                    viewBox: format!("{} {} {} {}", camera.read().0, camera.read().1, 1920.0 / zoom.read(), 1080.0 / zoom.read()),
                    width: "100%",
                    height: "100%",
                    
                    // Render dependencies as arrows
                    for dep in dependencies.read().iter() {
                        DependencyArrow {
                            key: "{dep.from_task_id}_{dep.to_task_id}",
                            dependency: dep.clone(),
                            tasks: tasks.read().clone(),
                        }
                    }
                    
                    // Render goals
                    for goal in goals.read().iter() {
                        GoalNode {
                            key: "{goal.id}",
                            goal: goal.clone(),
                            selected: selected_goal.read().as_ref() == Some(&goal.id),
                        }
                    }
                    
                    // Render tasks
                    for task in tasks.read().iter() {
                        TaskNode {
                            key: "{task.id}",
                            task: task.clone(),
                            selected: selected_task.read().as_ref() == Some(&task.id),
                            running: running_tasks.read().contains_key(&task.id),
                            hovering: hovering.read().as_ref() == Some(&task.id),
                        }
                    }
                }
            }
            
            // Minimap
            MiniMap {
                tasks: tasks.read().clone(),
                camera: camera.read().clone(),
                zoom: zoom.read().clone(),
            }
        }
    }
}

#[component]
fn MapToolbar() -> Element {
    let zoom = use_atom_state(&ZOOM_LEVEL);
    
    rsx! {
        div {
            class: "map-toolbar",
            
            button {
                class: "toolbar-btn",
                onclick: move |_| {
                    zoom.set((*zoom.read() * 1.2).min(5.0));
                },
                "🔍+"
            }
            
            button {
                class: "toolbar-btn",
                onclick: move |_| {
                    zoom.set((*zoom.read() / 1.2).max(0.2));
                },
                "🔍-"
            }
            
            button {
                class: "toolbar-btn",
                onclick: move |_| {
                    zoom.set(1.0);
                },
                "Reset Zoom"
            }
            
            button {
                class: "toolbar-btn",
                onclick: move |_| {
                    // Auto-arrange tasks
                },
                "Auto Arrange"
            }
        }
    }
}

#[component]
fn TaskNode(task: Task, selected: bool, running: bool, hovering: bool) -> Element {
    let selected_task = use_atom_state(&SELECTED_TASK);
    let running_tasks = use_atom_state(&RUNNING_TASKS);
    
    let status_color = match task.status {
        TaskStatus::Todo => "#808080",
        TaskStatus::InProgress => "#FFA500",
        TaskStatus::Done => "#00FF00",
        TaskStatus::Blocked => "#FF0000",
    };
    
    let x = task.position.x;
    let y = task.position.y;
    
    rsx! {
        g {
            transform: "translate({x}, {y})",
            
            // Task card background
            rect {
                class: if selected { "task-node selected" } else { "task-node" },
                x: "0",
                y: "0",
                width: "150",
                height: "80",
                rx: "8",
                fill: if hovering { "#f0f0f0" } else { "#ffffff" },
                stroke: if selected { "#4CAF50" } else { "#cccccc" },
                stroke_width: if selected { "3" } else { "1" },
                onclick: move |_| {
                    selected_task.set(Some(task.id));
                },
            }
            
            // Status indicator
            circle {
                cx: "10",
                cy: "10",
                r: "5",
                fill: status_color,
            }
            
            // Task title
            text {
                x: "20",
                y: "15",
                class: "task-title",
                font_size: "14",
                font_weight: "bold",
                "{task.title}"
            }
            
            // Task description (truncated)
            text {
                x: "10",
                y: "35",
                class: "task-description",
                font_size: "12",
                "{task.description.chars().take(50).collect::<String>()}..."
            }
            
            // Play button for Todo tasks
            if task.status == TaskStatus::Todo && !running {
                PlayButton {
                    task_id: task.id,
                    x: 135.0,
                    y: 15.0,
                }
            }
            
            // Running indicator
            if running {
                RunningIndicator {
                    x: 135.0,
                    y: 15.0,
                }
            }
        }
    }
}

#[component]
fn GoalNode(goal: Goal, selected: bool) -> Element {
    let selected_goal = use_atom_state(&SELECTED_GOAL);
    
    let x = goal.position.map(|p| p.x).unwrap_or(500.0);
    let y = goal.position.map(|p| p.y).unwrap_or(500.0);
    
    rsx! {
        g {
            transform: "translate({x}, {y})",
            
            circle {
                class: if selected { "goal-node selected" } else { "goal-node" },
                cx: "0",
                cy: "0",
                r: "30",
                fill: "#FFD700",
                stroke: if selected { "#4CAF50" } else { "#FFA500" },
                stroke_width: if selected { "3" } else { "2" },
                onclick: move |_| {
                    selected_goal.set(Some(goal.id));
                },
            }
            
            text {
                x: "0",
                y: "5",
                text_anchor: "middle",
                font_size: "14",
                font_weight: "bold",
                "{goal.title}"
            }
        }
    }
}

#[component]
fn DependencyArrow(dependency: Dependency, tasks: Vec<Task>) -> Element {
    let from_task = tasks.iter().find(|t| t.id == dependency.from_task_id);
    let to_task = tasks.iter().find(|t| t.id == dependency.to_task_id);
    
    if let (Some(from), Some(to)) = (from_task, to_task) {
        let x1 = from.position.x + 75.0;
        let y1 = from.position.y + 40.0;
        let x2 = to.position.x + 75.0;
        let y2 = to.position.y + 40.0;
        
        let color = match dependency.dependency_type {
            DependencyType::FinishToStart => "#4CAF50",
            DependencyType::StartToStart => "#2196F3",
            DependencyType::FinishToFinish => "#FF9800",
            DependencyType::StartToFinish => "#F44336",
        };
        
        rsx! {
            g {
                line {
                    x1: "{x1}",
                    y1: "{y1}",
                    x2: "{x2}",
                    y2: "{y2}",
                    stroke: color,
                    stroke_width: "2",
                    marker_end: "url(#arrowhead)",
                }
            }
        }
    } else {
        rsx! { Fragment {} }
    }
}

#[component]
fn PlayButton(task_id: Uuid, x: f32, y: f32) -> Element {
    let running_tasks = use_atom_state(&RUNNING_TASKS);
    
    rsx! {
        g {
            transform: "translate({x}, {y})",
            onclick: move |evt| {
                evt.stop_propagation();
                let mut tasks = running_tasks.write();
                tasks.insert(task_id, TaskExecutionStatus::Running);
                // Start Claude Code execution
            },
            
            circle {
                cx: "0",
                cy: "0",
                r: "12",
                fill: "#4CAF50",
                class: "play-button",
            }
            
            polygon {
                points: "-4,-6 -4,6 6,0",
                fill: "white",
            }
        }
    }
}

#[component]
fn RunningIndicator(x: f32, y: f32) -> Element {
    rsx! {
        g {
            transform: "translate({x}, {y})",
            
            circle {
                cx: "0",
                cy: "0",
                r: "12",
                fill: "#FFA500",
                class: "running-indicator",
            }
            
            // Spinner animation
            animateTransform {
                attributeName: "transform",
                attributeType: "XML",
                r#type: "rotate",
                from: "0 0 0",
                to: "360 0 0",
                dur: "1s",
                repeatCount: "indefinite",
            }
            
            rect {
                x: "-3",
                y: "-3",
                width: "6",
                height: "6",
                fill: "white",
            }
        }
    }
}

#[component]
fn MiniMap(tasks: Vec<Task>, camera: (f32, f32), zoom: f32) -> Element {
    let scale = 0.1;
    
    rsx! {
        div {
            class: "minimap",
            
            svg {
                width: "200",
                height: "150",
                viewBox: "0 0 2000 1500",
                
                // Tasks as dots
                for task in tasks.iter() {
                    circle {
                        cx: "{task.position.x * scale}",
                        cy: "{task.position.y * scale}",
                        r: "2",
                        fill: "#4CAF50",
                    }
                }
                
                // Camera viewport
                rect {
                    x: "{camera.0 * scale}",
                    y: "{camera.1 * scale}",
                    width: "{(1920.0 / zoom) * scale}",
                    height: "{(1080.0 / zoom) * scale}",
                    fill: "none",
                    stroke: "#FF0000",
                    stroke_width: "2",
                }
            }
        }
    }
}

// Helper types and functions
#[derive(Clone, Debug)]
struct DragState {
    item_id: Uuid,
    start_pos: (f32, f32),
    offset: (f32, f32),
}

fn handle_mouse_down(
    evt: &MouseEvent,
    dragging: &Signal<Option<DragState>>,
    selected_task: &AtomState<Option<Uuid>>,
    tasks: &AtomState<Vec<Task>>,
    camera: &AtomState<(f32, f32)>,
    zoom: &AtomState<f32>,
) {
    // Convert screen coordinates to world coordinates
    let world_x = evt.client_x() as f32 / zoom.read() + camera.read().0;
    let world_y = evt.client_y() as f32 / zoom.read() + camera.read().1;
    
    // Check if clicking on a task
    for task in tasks.read().iter() {
        if world_x >= task.position.x && world_x <= task.position.x + 150.0
            && world_y >= task.position.y && world_y <= task.position.y + 80.0 {
            dragging.set(Some(DragState {
                item_id: task.id,
                start_pos: (world_x, world_y),
                offset: (world_x - task.position.x, world_y - task.position.y),
            }));
            selected_task.set(Some(task.id));
            break;
        }
    }
}

fn handle_mouse_move(
    evt: &MouseEvent,
    dragging: &Signal<Option<DragState>>,
    hovering: &Signal<Option<Uuid>>,
    tasks: &AtomState<Vec<Task>>,
    camera: &AtomState<(f32, f32)>,
    zoom: &AtomState<f32>,
) {
    let world_x = evt.client_x() as f32 / zoom.read() + camera.read().0;
    let world_y = evt.client_y() as f32 / zoom.read() + camera.read().1;
    
    if let Some(drag) = dragging.read().as_ref() {
        // Update task position
        let mut tasks_mut = tasks.write();
        if let Some(task) = tasks_mut.iter_mut().find(|t| t.id == drag.item_id) {
            task.position.x = world_x - drag.offset.0;
            task.position.y = world_y - drag.offset.1;
        }
    } else {
        // Update hover state
        let mut hovered = None;
        for task in tasks.read().iter() {
            if world_x >= task.position.x && world_x <= task.position.x + 150.0
                && world_y >= task.position.y && world_y <= task.position.y + 80.0 {
                hovered = Some(task.id);
                break;
            }
        }
        hovering.set(hovered);
    }
}

fn handle_mouse_up(dragging: &Signal<Option<DragState>>) {
    dragging.set(None);
}

fn handle_wheel(evt: &WheelEvent, zoom: &AtomState<f32>, camera: &AtomState<(f32, f32)>) {
    let delta = evt.delta_y() as f32;
    let zoom_factor = if delta > 0.0 { 0.9 } else { 1.1 };
    let new_zoom = (*zoom.read() * zoom_factor).clamp(0.2, 5.0);
    zoom.set(new_zoom);
}