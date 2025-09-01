use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Position, Priority};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;

#[component]
pub fn MapView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut selected_task = use_signal(|| None::<Uuid>);
    let mut zoom = use_signal(|| 1.0f32);
    
    rsx! {
        div {
            style: "width: 100%; height: 100vh; display: flex; flex-direction: column; background: #f5f5f5;",
            
            // Toolbar
            div {
                style: "padding: 10px; background: white; box-shadow: 0 2px 4px rgba(0,0,0,0.1); display: flex; gap: 10px; align-items: center;",
                
                h2 { style: "margin: 0; margin-right: 20px;", "Task Map" }
                
                button {
                    onclick: move |_| zoom.set((*zoom.read() * 1.2).min(3.0)),
                    style: "padding: 8px 12px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Zoom In"
                }
                
                button {
                    onclick: move |_| zoom.set((*zoom.read() / 1.2).max(0.3)),
                    style: "padding: 8px 12px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Zoom Out"
                }
                
                button {
                    onclick: move |_| zoom.set(1.0),
                    style: "padding: 8px 12px; background: #FF9800; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Reset"
                }
                
                span { style: "margin-left: 20px;", "Zoom: {(*zoom.read() * 100.0) as i32}%" }
                
                button {
                    onclick: move |_| {
                        tasks.write().push(Task {
                            id: Uuid::new_v4(),
                            title: "New Task".to_string(),
                            description: String::new(),
                            status: TaskStatus::Todo,
                            priority: Priority::Medium,
                            position: Position { x: 200.0, y: 200.0 },
                            ..Default::default()
                        });
                    },
                    style: "margin-left: auto; padding: 8px 16px; background: #9C27B0; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Add Task"
                }
            }
            
            // Map area
            div {
                style: "flex: 1; position: relative; overflow: auto;",
                
                // Task cards
                div {
                    style: "position: relative; width: 2000px; height: 2000px; transform: scale({zoom});",
                    
                    for task in tasks.read().iter() {
                        div {
                            key: "{task.id}",
                            style: "position: absolute; left: {task.position.x}px; top: {task.position.y}px; 
                                   width: 200px; padding: 12px; background: white; border-radius: 8px; 
                                   box-shadow: 0 2px 8px rgba(0,0,0,0.1); cursor: pointer;
                                   border: 2px solid {if selected_task.read().as_ref() == Some(&task.id) { \"#4CAF50\" } else { \"transparent\" }};",
                            onclick: move |_| selected_task.set(Some(task.id)),
                            
                            div {
                                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 8px;",
                                
                                h4 { 
                                    style: "margin: 0; flex: 1;", 
                                    "{task.title}" 
                                }
                                
                                span {
                                    style: "padding: 2px 6px; border-radius: 4px; font-size: 11px; color: white; background: {match task.status {
                                        TaskStatus::Todo => \"#808080\",
                                        TaskStatus::InProgress => \"#2196F3\",
                                        TaskStatus::Done => \"#4CAF50\",
                                        TaskStatus::Blocked => \"#f44336\",
                                        _ => \"#666\",
                                    }};",
                                    "{task.status:?}"
                                }
                            }
                            
                            if !task.description.is_empty() {
                                p {
                                    style: "margin: 8px 0; font-size: 13px; color: #666;",
                                    "{task.description.chars().take(100).collect::<String>()}"
                                    if task.description.len() > 100 { "..." }
                                }
                            }
                            
                            div {
                                style: "display: flex; gap: 8px; align-items: center;",
                                
                                span {
                                    style: "font-size: 12px; padding: 2px 6px; background: {match task.priority {
                                        Priority::Critical => \"#ff0000\",
                                        Priority::High => \"#ff8800\",
                                        Priority::Medium => \"#ffaa00\",
                                        Priority::Low => \"#888888\",
                                    }}; color: white; border-radius: 3px;",
                                    "{task.priority:?}"
                                }
                                
                                if let Some(due) = task.due_date {
                                    span {
                                        style: "font-size: 12px; color: #666;",
                                        "📅 {due.format(\"%m/%d\")}"
                                    }
                                }
                                
                                if task.status == TaskStatus::Todo {
                                    button {
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            tasks.with_mut(|tasks| {
                                                if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                                    t.status = TaskStatus::InProgress;
                                                }
                                            });
                                        },
                                        style: "margin-left: auto; padding: 4px 8px; background: #4CAF50; 
                                               color: white; border: none; border-radius: 4px; font-size: 12px; cursor: pointer;",
                                        "Start"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Selected task details
            if let Some(task_id) = selected_task.read().as_ref() {
                if let Some(task) = tasks.read().iter().find(|t| t.id == *task_id) {
                    div {
                        style: "position: absolute; right: 20px; top: 70px; width: 300px; 
                               background: white; border-radius: 8px; padding: 20px; 
                               box-shadow: 0 4px 12px rgba(0,0,0,0.15);",
                        
                        h3 { style: "margin-top: 0;", "{task.title}" }
                        p { style: "color: #666;", "{task.description}" }
                        
                        div { style: "margin: 10px 0;",
                            strong { "Status: " }
                            "{task.status:?}"
                        }
                        
                        div { style: "margin: 10px 0;",
                            strong { "Priority: " }
                            "{task.priority:?}"
                        }
                        
                        button {
                            onclick: move |_| selected_task.set(None),
                            style: "width: 100%; padding: 10px; background: #f44336; color: white; 
                                   border: none; border-radius: 4px; cursor: pointer;",
                            "Close"
                        }
                    }
                }
            }
        }
    }
}