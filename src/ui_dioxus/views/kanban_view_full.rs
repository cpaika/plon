use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::state_simple::{TaskExecutionStatus, sample_tasks};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use chrono::Utc;

#[component]
pub fn KanbanView() -> Element {
    // State
    let mut tasks = use_signal(|| sample_tasks());
    let mut selected_tasks = use_signal(|| HashSet::<Uuid>::new());
    let mut search_filter = use_signal(|| String::new());
    let mut running_tasks = use_signal(|| HashMap::<Uuid, TaskExecutionStatus>::new());
    
    // Drag and drop state
    let mut dragging_task = use_signal(|| None::<DragState>);
    let mut drop_target = use_signal(|| None::<DropTarget>);
    
    // Quick add state
    let mut quick_add_column = use_signal(|| None::<TaskStatus>);
    let mut quick_add_text = use_signal(|| String::new());
    
    // WIP limits
    let wip_limits = HashMap::from([
        (TaskStatus::InProgress, 3),
        (TaskStatus::Review, 2),
    ]);
    
    // Filter tasks
    let filtered_tasks = tasks.read().iter()
        .filter(|task| {
            if search_filter.read().is_empty() {
                return true;
            }
            let search = search_filter.read().to_lowercase();
            task.title.to_lowercase().contains(&search) ||
            task.description.to_lowercase().contains(&search)
        })
        .cloned()
        .collect::<Vec<_>>();
    
    // Group tasks by status
    let mut columns = vec![
        ("Todo", TaskStatus::Todo, "#c8c8c8"),
        ("In Progress", TaskStatus::InProgress, "#6495ff"),
        ("Review", TaskStatus::Review, "#ff9800"),
        ("Done", TaskStatus::Done, "#64ff64"),
        ("Blocked", TaskStatus::Blocked, "#ff6464"),
    ];
    
    rsx! {
        div {
            class: "kanban-view",
            style: "padding: 20px; height: 100vh; overflow-x: auto;",
            
            // Header
            div {
                style: "margin-bottom: 20px;",
                
                h2 { "Kanban Board" }
                
                // Search bar
                div {
                    style: "display: flex; gap: 10px; margin-bottom: 20px;",
                    
                    input {
                        r#type: "text",
                        placeholder: "Search tasks...",
                        value: "{search_filter}",
                        oninput: move |evt| search_filter.set(evt.value()),
                        style: "flex: 1; max-width: 400px; padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                    }
                    
                    button {
                        onclick: move |_| {
                            let new_task = Task::new("New Task".to_string(), String::new());
                            tasks.write().push(new_task);
                        },
                        style: "padding: 8px 16px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "➕ Add Task"
                    }
                    
                    if !selected_tasks.read().is_empty() {
                        span {
                            style: "padding: 8px; background: #e3f2fd; border-radius: 4px;",
                            "{selected_tasks.read().len()} selected"
                        }
                        
                        button {
                            onclick: move |_| selected_tasks.write().clear(),
                            style: "padding: 8px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            "Clear Selection"
                        }
                    }
                }
            }
            
            // Kanban columns
            div {
                style: "display: flex; gap: 20px; height: calc(100vh - 150px); overflow-x: auto;",
                
                for (title, status, color) in columns {
                    KanbanColumn {
                        title: title,
                        status: status.clone(),
                        color: color,
                        tasks: filtered_tasks.iter()
                            .filter(|t| t.status == status)
                            .cloned()
                            .collect(),
                        wip_limit: wip_limits.get(&status).copied(),
                        selected_tasks: selected_tasks.clone(),
                        running_tasks: running_tasks.clone(),
                        dragging_task: dragging_task.clone(),
                        drop_target: drop_target.clone(),
                        quick_add_active: quick_add_column.read().as_ref() == Some(&status),
                        quick_add_text: quick_add_text.clone(),
                        ondrop: move |task_id| {
                            tasks.with_mut(|tasks| {
                                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                    task.status = status.clone();
                                    if status == TaskStatus::Done {
                                        task.completed_at = Some(Utc::now());
                                    }
                                }
                            });
                            dragging_task.set(None);
                            drop_target.set(None);
                        },
                        onquickadd: move |text| {
                            if !text.is_empty() {
                                let mut new_task = Task::new(text, String::new());
                                new_task.status = status.clone();
                                tasks.write().push(new_task);
                                quick_add_text.set(String::new());
                                quick_add_column.set(None);
                            }
                        },
                        onquickaddtoggle: move |_| {
                            if quick_add_column.read().as_ref() == Some(&status) {
                                quick_add_column.set(None);
                            } else {
                                quick_add_column.set(Some(status.clone()));
                                quick_add_text.set(String::new());
                            }
                        },
                        onselect: move |task_id, multi| {
                            selected_tasks.with_mut(|selected| {
                                if multi {
                                    if selected.contains(&task_id) {
                                        selected.remove(&task_id);
                                    } else {
                                        selected.insert(task_id);
                                    }
                                } else {
                                    selected.clear();
                                    selected.insert(task_id);
                                }
                            });
                        },
                        onplay: move |task_id| {
                            running_tasks.write().insert(task_id, TaskExecutionStatus::Running);
                        },
                        onupdate: move |updated| {
                            tasks.with_mut(|tasks| {
                                if let Some(task) = tasks.iter_mut().find(|t| t.id == updated.id) {
                                    *task = updated;
                                }
                            });
                        },
                        ondelete: move |task_id| {
                            tasks.with_mut(|tasks| {
                                tasks.retain(|t| t.id != task_id);
                            });
                            selected_tasks.with_mut(|selected| {
                                selected.remove(&task_id);
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
    title: &'static str,
    status: TaskStatus,
    color: &'static str,
    tasks: Vec<Task>,
    wip_limit: Option<usize>,
    selected_tasks: Signal<HashSet<Uuid>>,
    running_tasks: Signal<HashMap<Uuid, TaskExecutionStatus>>,
    dragging_task: Signal<Option<DragState>>,
    drop_target: Signal<Option<DropTarget>>,
    quick_add_active: bool,
    quick_add_text: Signal<String>,
    ondrop: EventHandler<Uuid>,
    onquickadd: EventHandler<String>,
    onquickaddtoggle: EventHandler<MouseEvent>,
    onselect: EventHandler<(Uuid, bool)>,
    onplay: EventHandler<Uuid>,
    onupdate: EventHandler<Task>,
    ondelete: EventHandler<Uuid>,
) -> Element {
    let is_over_limit = wip_limit.map_or(false, |limit| tasks.len() > limit);
    let is_drop_target = drop_target.read().as_ref()
        .map_or(false, |target| target.status == status);
    
    rsx! {
        div {
            class: "kanban-column",
            style: format!("flex: 0 0 300px; background: #f5f5f5; border-radius: 8px; 
                           padding: 15px; display: flex; flex-direction: column; 
                           border: 2px solid {}; transition: all 0.3s;",
                           if is_drop_target { "#4CAF50" } else { "transparent" }),
            ondragover: move |evt| {
                evt.prevent_default();
                if dragging_task.read().is_some() {
                    drop_target.set(Some(DropTarget { status: status.clone() }));
                }
            },
            ondragleave: move |_| {
                drop_target.set(None);
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(drag) = dragging_task.read().as_ref() {
                    ondrop.call(drag.task_id);
                }
            },
            
            // Column header
            div {
                style: "margin-bottom: 15px;",
                
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;",
                    
                    h3 {
                        style: format!("margin: 0; color: {};", color),
                        "{title}"
                    }
                    
                    div {
                        style: "display: flex; gap: 5px; align-items: center;",
                        
                        span {
                            style: format!("padding: 2px 8px; background: {}; color: white; 
                                          border-radius: 4px; font-size: 12px;",
                                          if is_over_limit { "#ff0000" } else { color }),
                            "{tasks.len()}"
                        }
                        
                        if let Some(limit) = wip_limit {
                            span {
                                style: "font-size: 12px; color: #666;",
                                "/ {limit}"
                            }
                        }
                        
                        button {
                            onclick: move |evt| onquickaddtoggle.call(evt),
                            style: "padding: 2px 6px; background: none; border: none; 
                                   font-size: 16px; cursor: pointer;",
                            "➕"
                        }
                    }
                }
                
                if is_over_limit {
                    div {
                        style: "padding: 5px; background: #ffebee; color: #c62828; 
                               border-radius: 4px; font-size: 12px;",
                        "⚠️ WIP limit exceeded!"
                    }
                }
            }
            
            // Quick add form
            if quick_add_active {
                div {
                    style: "margin-bottom: 10px; padding: 10px; background: white; 
                           border-radius: 4px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);",
                    
                    input {
                        r#type: "text",
                        placeholder: "Enter task title...",
                        value: "{quick_add_text}",
                        oninput: move |evt| quick_add_text.set(evt.value()),
                        onkeypress: move |evt| {
                            if evt.key() == "Enter" {
                                onquickadd.call(quick_add_text.read().clone());
                            }
                        },
                        style: "width: 100%; padding: 8px; border: 1px solid #ddd; 
                               border-radius: 4px; margin-bottom: 8px;",
                        autofocus: true,
                    }
                    
                    div {
                        style: "display: flex; gap: 5px;",
                        
                        button {
                            onclick: move |_| onquickadd.call(quick_add_text.read().clone()),
                            style: "flex: 1; padding: 6px; background: #4CAF50; color: white; 
                                   border: none; border-radius: 4px; cursor: pointer;",
                            "Add"
                        }
                        
                        button {
                            onclick: move |evt| onquickaddtoggle.call(evt),
                            style: "flex: 1; padding: 6px; background: #f44336; color: white; 
                                   border: none; border-radius: 4px; cursor: pointer;",
                            "Cancel"
                        }
                    }
                }
            }
            
            // Task cards
            div {
                style: "flex: 1; overflow-y: auto;",
                
                if tasks.is_empty() && !is_drop_target {
                    div {
                        style: "padding: 20px; text-align: center; color: #999;",
                        "No tasks"
                    }
                } else if tasks.is_empty() && is_drop_target {
                    div {
                        style: "padding: 20px; text-align: center; border: 2px dashed #4CAF50; 
                               border-radius: 4px; background: #e8f5e9;",
                        "Drop here"
                    }
                }
                
                for task in tasks {
                    KanbanCard {
                        task: task.clone(),
                        selected: selected_tasks.read().contains(&task.id),
                        running: running_tasks.read().contains_key(&task.id),
                        dragging: dragging_task.read().as_ref()
                            .map_or(false, |d| d.task_id == task.id),
                        ondragstart: move |_| {
                            dragging_task.set(Some(DragState {
                                task_id: task.id,
                                original_status: task.status.clone(),
                            }));
                        },
                        ondragend: move |_| {
                            dragging_task.set(None);
                            drop_target.set(None);
                        },
                        onclick: move |evt| {
                            let multi = evt.modifiers().shift() || evt.modifiers().ctrl();
                            onselect.call((task.id, multi));
                        },
                        onplay: move |_| onplay.call(task.id),
                        onupdate: move |updated| onupdate.call(updated),
                        ondelete: move |_| ondelete.call(task.id),
                    }
                }
            }
        }
    }
}

#[component]
fn KanbanCard(
    task: Task,
    selected: bool,
    running: bool,
    dragging: bool,
    ondragstart: EventHandler<DragEvent>,
    ondragend: EventHandler<DragEvent>,
    onclick: EventHandler<MouseEvent>,
    onplay: EventHandler<MouseEvent>,
    onupdate: EventHandler<Task>,
    ondelete: EventHandler<MouseEvent>,
) -> Element {
    let mut editing_title = use_signal(|| false);
    let mut title_value = use_signal(|| task.title.clone());
    
    let priority_color = match task.priority {
        Priority::Critical => "#ff0000",
        Priority::High => "#ff8800",
        Priority::Medium => "#ffaa00",
        Priority::Low => "#888888",
    };
    
    let is_overdue = task.due_date.map_or(false, |due| due < Utc::now());
    
    rsx! {
        div {
            class: "kanban-card",
            style: format!("background: white; border-radius: 6px; padding: 12px; 
                           margin-bottom: 10px; cursor: move; box-shadow: 0 2px 4px rgba(0,0,0,0.1); 
                           border: 2px solid {}; opacity: {}; transition: all 0.2s;",
                           if selected { "#4CAF50" } else { "transparent" },
                           if dragging { "0.5" } else { "1" }),
            draggable: "true",
            ondragstart: move |evt| ondragstart.call(evt),
            ondragend: move |evt| ondragend.call(evt),
            onclick: move |evt| onclick.call(evt),
            
            // Card header
            div {
                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 8px;",
                
                if editing_title {
                    input {
                        value: "{title_value}",
                        oninput: move |evt| title_value.set(evt.value()),
                        onkeypress: move |evt| {
                            if evt.key() == "Enter" {
                                let mut updated = task.clone();
                                updated.title = title_value.read().clone();
                                onupdate.call(updated);
                                editing_title.set(false);
                            }
                        },
                        onblur: move |_| editing_title.set(false),
                        style: "flex: 1; padding: 4px; font-weight: bold; border: 1px solid #4CAF50; 
                               border-radius: 4px;",
                        autofocus: true,
                    }
                } else {
                    h4 {
                        style: "margin: 0; font-size: 14px; flex: 1; cursor: text;",
                        ondoubleclick: move |_| editing_title.set(true),
                        "{task.title}"
                    }
                }
                
                // Priority badge
                div {
                    style: format!("width: 8px; height: 8px; border-radius: 50%; 
                                   background: {}; margin-left: 8px;", priority_color),
                    title: "{task.priority:?} priority",
                }
            }
            
            // Description
            if !task.description.is_empty() {
                p {
                    style: "margin: 0 0 8px 0; font-size: 12px; color: #666; 
                           line-height: 1.4;",
                    "{task.description.chars().take(100).collect::<String>()}"
                    if task.description.len() > 100 { "..." }
                }
            }
            
            // Metadata
            div {
                style: "display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 8px;",
                
                // Due date
                if let Some(due) = task.due_date {
                    span {
                        style: format!("font-size: 11px; padding: 2px 6px; 
                                       background: {}; color: {}; border-radius: 3px;",
                                       if is_overdue { "#ffebee" } else { "#f0f0f0" },
                                       if is_overdue { "#c62828" } else { "#666" }),
                        if is_overdue { "⚠️ " } else { "📅 " }
                        "{due.format(\"%m/%d\")}"
                    }
                }
                
                // Subtasks
                if !task.subtasks.is_empty() {
                    let completed = task.subtasks.iter().filter(|s| s.completed).count();
                    let total = task.subtasks.len();
                    span {
                        style: "font-size: 11px; padding: 2px 6px; background: #e3f2fd; 
                               color: #1976d2; border-radius: 3px;",
                        "✓ {completed}/{total}"
                    }
                }
                
                // Tags
                for tag in task.tags.iter().take(2) {
                    span {
                        style: "font-size: 11px; padding: 2px 6px; background: #f5f5f5; 
                               color: #666; border-radius: 3px;",
                        "#{tag}"
                    }
                }
            }
            
            // Actions
            div {
                style: "display: flex; gap: 5px;",
                
                // Play button for Todo tasks
                if task.status == TaskStatus::Todo && !running {
                    button {
                        onclick: move |evt| {
                            evt.stop_propagation();
                            onplay.call(evt);
                        },
                        style: "padding: 4px 8px; background: #4CAF50; color: white; 
                               border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        "▶ Start"
                    }
                }
                
                // Running indicator
                if running {
                    div {
                        style: "padding: 4px 8px; background: #FFA500; color: white; 
                               border-radius: 4px; font-size: 12px;",
                        "🔄 Running"
                    }
                }
                
                // Delete button
                button {
                    onclick: move |evt| {
                        evt.stop_propagation();
                        ondelete.call(evt);
                    },
                    style: "margin-left: auto; padding: 4px; background: none; 
                           border: none; cursor: pointer; opacity: 0.5;",
                    "🗑"
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct DragState {
    task_id: Uuid,
    original_status: TaskStatus,
}

#[derive(Clone, Debug)]
struct DropTarget {
    status: TaskStatus,
}