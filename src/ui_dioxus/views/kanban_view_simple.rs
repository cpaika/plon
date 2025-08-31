use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;

#[component]
pub fn KanbanView() -> Element {
    let tasks = use_signal(|| sample_tasks());
    let dragging_task = use_signal(|| None::<Uuid>);
    let drag_over_status = use_signal(|| None::<TaskStatus>);
    
    rsx! {
        div {
            style: "padding: 20px; height: 100vh; background: #f5f5f5;",
            
            h2 { "Kanban Board" }
            p { "Drag cards between columns to change their status" }
            
            // Kanban columns
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
                }
                
                // In Progress column
                KanbanColumn {
                    title: "In Progress",
                    color: "#2196F3",
                    status: TaskStatus::InProgress,
                    tasks: tasks,
                    dragging_task: dragging_task,
                    drag_over_status: drag_over_status,
                }
                
                // Review column
                KanbanColumn {
                    title: "Review",
                    color: "#FF9800",
                    status: TaskStatus::Review,
                    tasks: tasks,
                    dragging_task: dragging_task,
                    drag_over_status: drag_over_status,
                }
                
                // Done column
                KanbanColumn {
                    title: "Done",
                    color: "#4CAF50",
                    status: TaskStatus::Done,
                    tasks: tasks,
                    dragging_task: dragging_task,
                    drag_over_status: drag_over_status,
                }
                
                // Blocked column
                KanbanColumn {
                    title: "Blocked",
                    color: "#f44336",
                    status: TaskStatus::Blocked,
                    tasks: tasks,
                    dragging_task: dragging_task,
                    drag_over_status: drag_over_status,
                }
            }
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
) -> Element {
    let column_tasks: Vec<Task> = tasks.read().iter()
        .filter(|t| t.status == status)
        .cloned()
        .collect();
    
    let is_drag_over = drag_over_status.read().as_ref() == Some(&status);
    let background = if is_drag_over { "#e8f5e9" } else { "white" };
    
    rsx! {
        div {
            style: "flex: 0 0 280px; background: {background}; border-radius: 8px; padding: 15px; border-top: 4px solid {color}; transition: background 0.2s;",
            
            ondragover: move |evt| {
                evt.stop_propagation();
                drag_over_status.set(Some(status));
            },
            
            ondragleave: move |_| {
                if drag_over_status.read().as_ref() == Some(&status) {
                    drag_over_status.set(None);
                }
            },
            
            ondrop: move |_| {
                drag_over_status.set(None);
                
                if let Some(task_id) = *dragging_task.read() {
                    tasks.with_mut(|tasks| {
                        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                            task.status = status;
                        }
                    });
                }
                dragging_task.set(None);
            },
            
            div {
                style: "margin-bottom: 15px; display: flex; justify-content: space-between;",
                h3 { style: "margin: 0; color: {color};", "{title}" }
                span {
                    style: "padding: 2px 8px; background: {color}; color: white; border-radius: 12px; font-size: 14px;",
                    "{column_tasks.len()}"
                }
            }
            
            div {
                style: "overflow-y: auto; max-height: calc(100vh - 200px);",
                
                if column_tasks.is_empty() {
                    div {
                        style: "padding: 20px; text-align: center; color: #999; border: 2px dashed #ddd; border-radius: 8px;",
                        "Drop tasks here"
                    }
                }
                
                for task in column_tasks {
                    KanbanCard {
                        task: task.clone(),
                        dragging_task: dragging_task,
                        tasks: tasks,
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
) -> Element {
    let is_dragging = dragging_task.read().as_ref() == Some(&task.id);
    let card_opacity = if is_dragging { "0.5" } else { "1" };
    let card_transform = if is_dragging { "scale(1.02) rotate(2deg)" } else { "scale(1)" };
    let card_shadow = if is_dragging { "0 8px 16px rgba(0,0,0,0.3)" } else { "0 2px 4px rgba(0,0,0,0.1)" };
    
    rsx! {
        div {
            style: "background: #f8f8f8; border-radius: 6px; padding: 12px; margin-bottom: 10px; 
                   cursor: move; opacity: {card_opacity}; transform: {card_transform}; 
                   box-shadow: {card_shadow}; transition: all 0.2s ease;",
            draggable: "true",
            
            ondragstart: move |_| {
                dragging_task.set(Some(task.id));
            },
            
            ondragend: move |_| {
                dragging_task.set(None);
            },
            
            div {
                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 8px;",
                
                h4 { 
                    style: "margin: 0; font-size: 14px; flex: 1;", 
                    "{task.title}" 
                }
                
                button {
                    onclick: move |evt| {
                        evt.stop_propagation();
                        tasks.with_mut(|tasks| {
                            tasks.retain(|t| t.id != task.id);
                        });
                    },
                    style: "background: none; border: none; color: #999; cursor: pointer; font-size: 16px; padding: 0; margin: -4px -4px 0 0;",
                    "×"
                }
            }
            
            if !task.description.is_empty() {
                p { 
                    style: "margin: 0 0 8px 0; font-size: 12px; color: #666;", 
                    "{task.description}" 
                }
            }
            
            div {
                style: "display: flex; gap: 8px; flex-wrap: wrap;",
                
                // Simplified priority badge
                span {
                    style: "font-size: 11px; padding: 2px 6px; background: #ff8800; color: white; border-radius: 3px;",
                    "Priority"
                }
                
                if let Some(_due) = task.due_date {
                    span {
                        style: "font-size: 11px; color: #666;",
                        "📅 Due"
                    }
                }
            }
        }
    }
}