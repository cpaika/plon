use dioxus::prelude::*;
use fermi::prelude::*;
use crate::domain::task::{Task, TaskStatus};
use crate::ui_dioxus::state::{TASKS, SELECTED_TASK};
use uuid::Uuid;

#[component]
pub fn KanbanView() -> Element {
    let tasks = use_atom_state(&TASKS);
    let selected_task = use_atom_state(&SELECTED_TASK);
    
    // Drag and drop state
    let dragging_task = use_signal(|| None::<Uuid>);
    let drop_zone = use_signal(|| None::<TaskStatus>);
    
    rsx! {
        div {
            class: "kanban-view",
            
            div {
                class: "kanban-board",
                
                KanbanColumn {
                    status: TaskStatus::Todo,
                    title: "To Do",
                    tasks: tasks.read().iter().filter(|t| t.status == TaskStatus::Todo).cloned().collect(),
                    dragging_task: dragging_task.clone(),
                    drop_zone: drop_zone.clone(),
                }
                
                KanbanColumn {
                    status: TaskStatus::InProgress,
                    title: "In Progress",
                    tasks: tasks.read().iter().filter(|t| t.status == TaskStatus::InProgress).cloned().collect(),
                    dragging_task: dragging_task.clone(),
                    drop_zone: drop_zone.clone(),
                }
                
                KanbanColumn {
                    status: TaskStatus::Done,
                    title: "Done",
                    tasks: tasks.read().iter().filter(|t| t.status == TaskStatus::Done).cloned().collect(),
                    dragging_task: dragging_task.clone(),
                    drop_zone: drop_zone.clone(),
                }
                
                KanbanColumn {
                    status: TaskStatus::Blocked,
                    title: "Blocked",
                    tasks: tasks.read().iter().filter(|t| t.status == TaskStatus::Blocked).cloned().collect(),
                    dragging_task: dragging_task.clone(),
                    drop_zone: drop_zone.clone(),
                }
            }
        }
    }
}

#[component]
fn KanbanColumn(
    status: TaskStatus,
    title: &'static str,
    tasks: Vec<Task>,
    dragging_task: Signal<Option<Uuid>>,
    drop_zone: Signal<Option<TaskStatus>>,
) -> Element {
    let tasks_state = use_atom_state(&TASKS);
    let is_drop_zone = drop_zone.read().as_ref() == Some(&status);
    
    rsx! {
        div {
            class: if is_drop_zone { "kanban-column drop-zone" } else { "kanban-column" },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_zone.set(Some(status.clone()));
            },
            ondragleave: move |_| {
                drop_zone.set(None);
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(task_id) = dragging_task.read().as_ref() {
                    // Update task status
                    let mut tasks_mut = tasks_state.write();
                    if let Some(task) = tasks_mut.iter_mut().find(|t| t.id == *task_id) {
                        task.status = status.clone();
                    }
                }
                dragging_task.set(None);
                drop_zone.set(None);
            },
            
            div {
                class: "column-header",
                h3 { "{title}" }
                span { class: "task-count", "({tasks.len()})" }
            }
            
            div {
                class: "column-content",
                
                for task in tasks {
                    KanbanCard {
                        key: "{task.id}",
                        task: task.clone(),
                        dragging_task: dragging_task.clone(),
                    }
                }
                
                button {
                    class: "add-task-btn",
                    onclick: move |_| {
                        // Create new task in this column
                    },
                    "➕ Add Task"
                }
            }
        }
    }
}

#[component]
fn KanbanCard(task: Task, dragging_task: Signal<Option<Uuid>>) -> Element {
    let selected_task = use_atom_state(&SELECTED_TASK);
    let is_dragging = dragging_task.read().as_ref() == Some(&task.id);
    
    rsx! {
        div {
            class: if is_dragging { "kanban-card dragging" } else { "kanban-card" },
            draggable: "true",
            ondragstart: move |_| {
                dragging_task.set(Some(task.id));
            },
            ondragend: move |_| {
                dragging_task.set(None);
            },
            onclick: move |_| {
                selected_task.set(Some(task.id));
            },
            
            div {
                class: "card-header",
                h4 { "{task.title}" }
                
                div {
                    class: "card-actions",
                    
                    button {
                        class: "btn-icon",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            // Start Claude Code
                        },
                        "▶️"
                    }
                }
            }
            
            p {
                class: "card-description",
                "{task.description.chars().take(100).collect::<String>()}"
                if task.description.len() > 100 {
                    "..."
                }
            }
            
            div {
                class: "card-footer",
                
                if let Some(due_date) = task.due_date {
                    span {
                        class: "due-date",
                        "📅 {due_date}"
                    }
                }
                
                if let Some(priority) = task.priority {
                    span {
                        class: if priority <= 2 { "priority high" } else { "priority" },
                        "⚡ {priority}"
                    }
                }
            }
        }
    }
}