use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;
use chrono::Utc;

#[component]
pub fn KanbanView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut dragging_task = use_signal(|| None::<Uuid>);
    let mut drop_status = use_signal(|| None::<TaskStatus>);
    
    let columns = vec![
        ("Todo", TaskStatus::Todo, "#808080"),
        ("In Progress", TaskStatus::InProgress, "#2196F3"),
        ("Review", TaskStatus::Review, "#FF9800"),
        ("Done", TaskStatus::Done, "#4CAF50"),
        ("Blocked", TaskStatus::Blocked, "#f44336"),
    ];
    
    rsx! {
        div {
            style: "padding: 20px; height: 100vh; background: #f5f5f5;",
            
            h2 { "Kanban Board" }
            
            // Board header
            div {
                style: "margin-bottom: 20px; display: flex; gap: 10px;",
                
                button {
                    onclick: move |_| {
                        tasks.write().push(Task::new("New Task".to_string(), String::new()));
                    },
                    style: "padding: 8px 16px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "➕ Add Task"
                }
                
                div {
                    style: "margin-left: auto; padding: 8px; background: white; border-radius: 4px;",
                    "Total tasks: {tasks.read().len()}"
                }
            }
            
            // Kanban columns
            div {
                style: "display: flex; gap: 15px; height: calc(100vh - 120px); overflow-x: auto;",
                
                for (name, status, color) in columns {
                    div {
                        key: "{status:?}",
                        style: "flex: 0 0 280px; background: white; border-radius: 8px; 
                               padding: 15px; display: flex; flex-direction: column;
                               border-top: 4px solid {color};",
                        ondragover: move |evt| {
                            evt.prevent_default();
                            drop_status.set(Some(status.clone()));
                        },
                        ondrop: move |evt| {
                            evt.prevent_default();
                            if let Some(task_id) = *dragging_task.read() {
                                tasks.with_mut(|tasks| {
                                    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                        task.status = status.clone();
                                        if status == TaskStatus::Done {
                                            task.completed_at = Some(Utc::now());
                                        }
                                    }
                                });
                            }
                            dragging_task.set(None);
                            drop_status.set(None);
                        },
                        
                        // Column header
                        div {
                            style: "margin-bottom: 15px; display: flex; justify-content: space-between;",
                            
                            h3 {
                                style: "margin: 0; color: {color};",
                                "{name}"
                            }
                            
                            span {
                                style: "padding: 2px 8px; background: {color}; color: white; 
                                       border-radius: 12px; font-size: 14px;",
                                "{tasks.read().iter().filter(|t| t.status == status).count()}"
                            }
                        }
                        
                        // Cards container
                        div {
                            style: "flex: 1; overflow-y: auto;",
                            
                            for task in tasks.read().iter().filter(|t| t.status == status) {
                                div {
                                    key: "{task.id}",
                                    draggable: "true",
                                    ondragstart: move |_| dragging_task.set(Some(task.id)),
                                    ondragend: move |_| {
                                        dragging_task.set(None);
                                        drop_status.set(None);
                                    },
                                    style: "background: #f8f8f8; border-radius: 6px; padding: 12px; 
                                           margin-bottom: 10px; cursor: move; transition: all 0.2s;
                                           opacity: {if dragging_task.read().as_ref() == Some(&task.id) { \"0.5\" } else { \"1\" }};",
                                    
                                    // Card header
                                    div {
                                        style: "display: flex; justify-content: space-between; margin-bottom: 8px;",
                                        
                                        h4 {
                                            style: "margin: 0; font-size: 14px;",
                                            "{task.title}"
                                        }
                                        
                                        div {
                                            style: "width: 8px; height: 8px; border-radius: 50%; 
                                                   background: {match task.priority {
                                                       Priority::Critical => \"#ff0000\",
                                                       Priority::High => \"#ff8800\",
                                                       Priority::Medium => \"#ffaa00\",
                                                       Priority::Low => \"#888888\",
                                                   }};",
                                        }
                                    }
                                    
                                    if !task.description.is_empty() {
                                        p {
                                            style: "margin: 0 0 8px 0; font-size: 12px; color: #666;",
                                            "{task.description.chars().take(80).collect::<String>()}"
                                        }
                                    }
                                    
                                    // Card footer
                                    div {
                                        style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                        
                                        if let Some(due) = task.due_date {
                                            span {
                                                style: "font-size: 11px; padding: 2px 6px; 
                                                       background: {if due < Utc::now() { \"#ffebee\" } else { \"#f0f0f0\" }}; 
                                                       color: {if due < Utc::now() { \"#c62828\" } else { \"#666\" }}; 
                                                       border-radius: 3px;",
                                                "📅 {due.format(\"%m/%d\")}"
                                            }
                                        }
                                        
                                        if !task.subtasks.is_empty() {
                                            let completed = task.subtasks.iter().filter(|s| s.completed).count();
                                            span {
                                                style: "font-size: 11px; padding: 2px 6px; 
                                                       background: #e3f2fd; color: #1976d2; border-radius: 3px;",
                                                "✓ {completed}/{task.subtasks.len()}"
                                            }
                                        }
                                        
                                        for tag in task.tags.iter().take(2) {
                                            span {
                                                style: "font-size: 11px; padding: 2px 6px; 
                                                       background: #f5f5f5; color: #666; border-radius: 3px;",
                                                "#{tag}"
                                            }
                                        }
                                    }
                                    
                                    // Quick actions
                                    if status == TaskStatus::Todo {
                                        button {
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                tasks.with_mut(|tasks| {
                                                    if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
                                                        t.status = TaskStatus::InProgress;
                                                    }
                                                });
                                            },
                                            style: "margin-top: 8px; width: 100%; padding: 4px; 
                                                   background: #4CAF50; color: white; border: none; 
                                                   border-radius: 4px; font-size: 12px; cursor: pointer;",
                                            "Start Task"
                                        }
                                    }
                                }
                            }
                            
                            if tasks.read().iter().filter(|t| t.status == status).count() == 0 {
                                div {
                                    style: "padding: 20px; text-align: center; color: #999; 
                                           border: 2px dashed #ddd; border-radius: 8px;",
                                    "No tasks"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}