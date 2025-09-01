use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Position, Priority};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;

#[component]
pub fn MapView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut selected_task: Signal<Option<Uuid>> = use_signal(|| None);
    let mut zoom = use_signal(|| 1.0f32);
    let mut dragging_task = use_signal(|| None::<Uuid>);
    let mut drag_start_mouse = use_signal(|| (0.0f64, 0.0f64));
    let mut drag_start_task_pos = use_signal(|| (0.0f64, 0.0f64));
    
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
            
            // Map area
            div {
                style: "flex: 1; position: relative; overflow: auto; background: #fafafa;",
                
                // Task cards container - apply zoom transform
                div {
                    style: "position: relative; width: 2000px; height: 2000px; transform: scale({zoom.read()}); transform-origin: top left;",
                    
                    onmousemove: move |evt| {
                        // Update task position while dragging
                        if let Some(task_id) = *dragging_task.read() {
                            let zoom_factor = *zoom.read();
                            let start_mouse = *drag_start_mouse.read();
                            let start_pos = *drag_start_task_pos.read();
                            
                            // Calculate delta from drag start
                            let delta_x = (evt.client_coordinates().x as f64 - start_mouse.0) / zoom_factor as f64;
                            let delta_y = (evt.client_coordinates().y as f64 - start_mouse.1) / zoom_factor as f64;
                            
                            // Calculate new position
                            let new_x = (start_pos.0 + delta_x).max(0.0).min(1900.0);
                            let new_y = (start_pos.1 + delta_y).max(0.0).min(1900.0);
                            
                            tasks.with_mut(|tasks| {
                                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                    task.position.x = new_x;
                                    task.position.y = new_y;
                                }
                            });
                        }
                    },
                    
                    onmouseup: move |_| {
                        // End drag
                        dragging_task.set(None);
                    },
                    
                    // Render each task as a card
                    for task in tasks.read().clone() {
                        TaskCard {
                            task: task.clone(),
                            selected: selected_task.read().as_ref() == Some(&task.id),
                            dragging: dragging_task.read().as_ref() == Some(&task.id),
                            onclick: move |_| selected_task.set(Some(task.id)),
                            onmousedown: move |evt: MouseEvent| {
                                // Start drag
                                dragging_task.set(Some(task.id));
                                drag_start_mouse.set((evt.client_coordinates().x as f64, evt.client_coordinates().y as f64));
                                drag_start_task_pos.set((task.position.x, task.position.y));
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
    onclick: EventHandler<MouseEvent>,
    onmousedown: EventHandler<MouseEvent>,
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
    
    let border_color = if selected { "#4CAF50" } else { "transparent" };
    let opacity = if dragging { "0.6" } else { "1" };
    let cursor = if dragging { "grabbing" } else { "grab" };
    let shadow = if dragging { "0 4px 16px rgba(0,0,0,0.3)" } else { "0 2px 8px rgba(0,0,0,0.1)" };
    let transform = if dragging { "scale(1.05)" } else { "scale(1)" };
    let pos_x = task.position.x;
    let pos_y = task.position.y;
    
    rsx! {
        div {
            style: "position: absolute; left: {pos_x}px; top: {pos_y}px; 
                   width: 200px; padding: 12px; background: white; border-radius: 8px; 
                   box-shadow: {shadow}; cursor: {cursor};
                   border: 2px solid {border_color}; opacity: {opacity}; 
                   transform: {transform};
                   transition: box-shadow 0.2s, opacity 0.2s, transform 0.2s;
                   user-select: none;",
            
            onmousedown: move |evt| {
                evt.stop_propagation();
                onmousedown.call(evt);
            },
            
            onclick: move |evt| {
                if !dragging {
                    evt.stop_propagation();
                    onclick.call(evt);
                }
            },
            
            div {
                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 8px; pointer-events: none;",
                
                h4 { 
                    style: "margin: 0; flex: 1; font-size: 14px;", 
                    "{task.title}" 
                }
                
                span {
                    style: "padding: 2px 6px; border-radius: 4px; font-size: 11px; color: white; background: {status_color};",
                    "{task.status:?}"
                }
            }
            
            if !task.description.is_empty() {
                p {
                    style: "margin: 8px 0; font-size: 13px; color: #666; line-height: 1.4; pointer-events: none;",
                    "{task.description}"
                }
            }
            
            div {
                style: "display: flex; gap: 8px; align-items: center; margin-top: 8px;",
                
                span {
                    style: "font-size: 12px; padding: 2px 6px; background: {priority_color}; color: white; border-radius: 3px; pointer-events: none;",
                    "{task.priority:?}"
                }
                
                if let Some(due) = task.due_date {
                    span {
                        style: "font-size: 12px; color: #666; pointer-events: none;",
                        "📅 {due.format(\"%m/%d\")}"
                    }
                }
                
                div {
                    style: "margin-left: auto; display: flex; gap: 4px;",
                    
                    button {
                        onclick: move |evt| {
                            evt.stop_propagation();
                            onstatuschange.call(evt);
                        },
                        onmousedown: move |evt| evt.stop_propagation(),
                        style: "padding: 4px 8px; background: #4CAF50; color: white; border: none; border-radius: 4px; font-size: 11px; cursor: pointer; pointer-events: auto;",
                        "→"
                    }
                    
                    button {
                        onclick: move |evt| {
                            evt.stop_propagation();
                            ondelete.call(evt);
                        },
                        onmousedown: move |evt| evt.stop_propagation(),
                        style: "padding: 4px 8px; background: #f44336; color: white; border: none; border-radius: 4px; font-size: 11px; cursor: pointer; pointer-events: auto;",
                        "×"
                    }
                }
            }
        }
    }
}

#[component]
fn TaskDetailsPanel(task: Task, onclose: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div {
            style: "position: absolute; right: 20px; top: 70px; width: 300px; 
                   background: white; border-radius: 8px; padding: 20px; 
                   box-shadow: 0 4px 12px rgba(0,0,0,0.15); z-index: 100;",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;",
                
                h3 { style: "margin: 0;", "{task.title}" }
                
                button {
                    onclick: move |evt| onclose.call(evt),
                    style: "background: none; border: none; font-size: 20px; cursor: pointer;",
                    "×"
                }
            }
            
            div { style: "margin-bottom: 10px;",
                strong { "Status: " }
                "{task.status:?}"
            }
            
            div { style: "margin-bottom: 10px;",
                strong { "Priority: " }
                "{task.priority:?}"
            }
            
            div { style: "margin-bottom: 10px;",
                strong { "Description: " }
                p { style: "margin: 5px 0;", "{task.description}" }
            }
            
            if let Some(due) = task.due_date {
                div { style: "margin-bottom: 10px;",
                    strong { "Due Date: " }
                    "{due.format(\"%Y-%m-%d\")}"
                }
            }
            
            div { style: "margin-bottom: 10px;",
                strong { "Position: " }
                "({task.position.x:.0}, {task.position.y:.0})"
            }
            
            if !task.subtasks.is_empty() {
                div { style: "margin-bottom: 10px;",
                    strong { "Subtasks: " }
                    "{task.subtasks.iter().filter(|s| s.completed).count()}/{task.subtasks.len()} completed"
                }
            }
        }
    }
}