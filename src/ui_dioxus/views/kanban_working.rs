use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;
use chrono::Utc;

#[component]
pub fn KanbanView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut dragging_task = use_signal(|| None::<Uuid>);
    let mut new_task_column = use_signal(|| None::<TaskStatus>);
    
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
            
            // Board controls
            div {
                style: "margin-bottom: 20px; display: flex; gap: 10px; align-items: center;",
                
                button {
                    onclick: move |_| {
                        let new_task = Task::new("New Task".to_string(), String::new());
                        tasks.write().push(new_task);
                    },
                    style: "padding: 8px 16px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "➕ Add Task"
                }
                
                div {
                    style: "margin-left: auto; display: flex; gap: 20px;",
                    
                    span {
                        style: "padding: 8px; background: white; border-radius: 4px;",
                        "Total: {tasks.read().len()}"
                    }
                    
                    span {
                        style: "padding: 8px; background: white; border-radius: 4px; color: #4CAF50;",
                        "Completed: {tasks.read().iter().filter(|t| t.status == TaskStatus::Done).count()}"
                    }
                }
            }
            
            // Kanban columns
            div {
                style: "display: flex; gap: 15px; height: calc(100vh - 120px); overflow-x: auto;",
                
                for (name, status, color) in columns {
                    KanbanColumn {
                        name: name.to_string(),
                        status: status.clone(),
                        color: color.to_string(),
                        tasks: tasks.read().iter().filter(|t| t.status == status).cloned().collect(),
                        dragging_task: dragging_task.read().clone(),
                        ondragstart: move |task_id| dragging_task.set(Some(task_id)),
                        ondragend: move |_| dragging_task.set(None),
                        ondrop: move |_| {
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
                        },
                        onstatuschange: move |task_id| {
                            tasks.with_mut(|tasks| {
                                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                    task.status = match task.status {
                                        TaskStatus::Todo => TaskStatus::InProgress,
                                        TaskStatus::InProgress => TaskStatus::Review,
                                        TaskStatus::Review => TaskStatus::Done,
                                        TaskStatus::Done => TaskStatus::Todo,
                                        TaskStatus::Blocked => TaskStatus::Todo,
                                        _ => TaskStatus::Todo,
                                    };
                                }
                            });
                        },
                        ondelete: move |task_id| {
                            tasks.with_mut(|tasks| {
                                tasks.retain(|t| t.id != task_id);
                            });
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn KanbanColumn(
    name: String,
    status: TaskStatus,
    color: String,
    tasks: Vec<Task>,
    dragging_task: Option<Uuid>,
    ondragstart: EventHandler<Uuid>,
    ondragend: EventHandler<()>,
    ondrop: EventHandler<()>,
    onstatuschange: EventHandler<Uuid>,
    ondelete: EventHandler<Uuid>,
) -> Element {
    let is_drop_target = use_signal(|| false);
    
    rsx! {
        div {
            style: "flex: 0 0 280px; background: white; border-radius: 8px; 
                   padding: 15px; display: flex; flex-direction: column;
                   border-top: 4px solid {color};
                   background: {if *is_drop_target.read() { \"#f0f8ff\" } else { \"white\" }};",
            ondragover: move |evt| {
                evt.prevent_default();
                is_drop_target.set(true);
            },
            ondragleave: move |_| {
                is_drop_target.set(false);
            },
            ondrop: move |evt| {
                evt.prevent_default();
                ondrop.call(());
                is_drop_target.set(false);
            },
            
            // Column header
            div {
                style: "margin-bottom: 15px; display: flex; justify-content: space-between; align-items: center;",
                
                h3 {
                    style: "margin: 0; color: {color}; font-size: 16px;",
                    "{name}"
                }
                
                span {
                    style: "padding: 2px 8px; background: {color}; color: white; 
                           border-radius: 12px; font-size: 14px; font-weight: bold;",
                    "{tasks.len()}"
                }
            }
            
            // Cards container
            div {
                style: "flex: 1; overflow-y: auto;",
                
                for task in tasks {
                    KanbanCard {
                        task: task.clone(),
                        is_dragging: dragging_task.as_ref() == Some(&task.id),
                        ondragstart: move |_| ondragstart.call(task.id),
                        ondragend: move |_| ondragend.call(()),
                        onstatuschange: move |_| onstatuschange.call(task.id),
                        ondelete: move |_| ondelete.call(task.id),
                    }
                }
                
                if tasks.is_empty() {
                    div {
                        style: "padding: 40px 20px; text-align: center; color: #999; 
                               border: 2px dashed #ddd; border-radius: 8px; margin-top: 10px;",
                        "Drop tasks here"
                    }
                }
            }
        }
    }
}

#[component]
fn KanbanCard(
    task: Task,
    is_dragging: bool,
    ondragstart: EventHandler<()>,
    ondragend: EventHandler<()>,
    onstatuschange: EventHandler<()>,
    ondelete: EventHandler<()>,
) -> Element {
    let priority_color = match task.priority {
        Priority::Critical => "#ff0000",
        Priority::High => "#ff8800",
        Priority::Medium => "#ffaa00",
        Priority::Low => "#888888",
    };
    
    let is_overdue = task.due_date.map_or(false, |due| due < Utc::now());
    
    rsx! {
        div {
            draggable: "true",
            ondragstart: move |_| ondragstart.call(()),
            ondragend: move |_| ondragend.call(()),
            style: "background: #f8f8f8; border-radius: 6px; padding: 12px; 
                   margin-bottom: 10px; cursor: move; transition: all 0.2s;
                   opacity: {if is_dragging { \"0.5\" } else { \"1\" }};
                   box-shadow: 0 1px 3px rgba(0,0,0,0.1);",
            onmouseover: move |evt| {
                let elem = evt.target();
                // Would set hover style here in real implementation
            },
            
            // Card header with priority indicator
            div {
                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 8px;",
                
                h4 {
                    style: "margin: 0; font-size: 14px; flex: 1;",
                    "{task.title}"
                }
                
                div {
                    style: "width: 8px; height: 8px; border-radius: 50%; 
                           background: {priority_color}; margin-left: 8px;",
                    title: "{task.priority:?} priority",
                }
            }
            
            if !task.description.is_empty() {
                p {
                    style: "margin: 0 0 8px 0; font-size: 12px; color: #666; line-height: 1.4;",
                    "{task.description}"
                }
            }
            
            // Card metadata
            div {
                style: "display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 8px;",
                
                if let Some(due) = task.due_date {
                    span {
                        style: "font-size: 11px; padding: 2px 6px; 
                               background: {if is_overdue { \"#ffebee\" } else { \"#f0f0f0\" }}; 
                               color: {if is_overdue { \"#c62828\" } else { \"#666\" }}; 
                               border-radius: 3px; font-weight: {if is_overdue { \"bold\" } else { \"normal\" }};",
                        "📅 {due.format(\"%m/%d\")}"
                    }
                }
                
                if !task.subtasks.is_empty() {
                    let completed = task.subtasks.iter().filter(|s| s.completed).count();
                    let total = task.subtasks.len();
                    span {
                        style: "font-size: 11px; padding: 2px 6px; 
                               background: #e3f2fd; color: #1976d2; border-radius: 3px;",
                        "✓ {completed}/{total}"
                    }
                }
                
                for tag in task.tags.iter().take(2) {
                    span {
                        style: "font-size: 11px; padding: 2px 6px; 
                               background: #f5f5f5; color: #666; border-radius: 3px;",
                        "#{tag}"
                    }
                }
                
                if task.tags.len() > 2 {
                    span {
                        style: "font-size: 11px; padding: 2px 6px; 
                               background: #f5f5f5; color: #999; border-radius: 3px;",
                        "+{}"
                    }
                }
            }
            
            // Card actions
            div {
                style: "display: flex; gap: 4px; justify-content: flex-end;",
                
                button {
                    onclick: move |evt| {
                        evt.stop_propagation();
                        onstatuschange.call(());
                    },
                    style: "padding: 4px 8px; background: #2196F3; color: white; 
                           border: none; border-radius: 4px; font-size: 11px; cursor: pointer;",
                    title: "Move to next status",
                    "→"
                }
                
                button {
                    onclick: move |evt| {
                        evt.stop_propagation();
                        ondelete.call(());
                    },
                    style: "padding: 4px 8px; background: #f44336; color: white; 
                           border: none; border-radius: 4px; font-size: 11px; cursor: pointer;",
                    title: "Delete task",
                    "×"
                }
            }
        }
    }
}