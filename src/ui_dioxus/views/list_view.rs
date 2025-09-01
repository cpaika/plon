use dioxus::prelude::*;
use fermi::prelude::*;
use crate::domain::task::{Task, TaskStatus};
use crate::ui_dioxus::state::{TASKS, SELECTED_TASK};
use uuid::Uuid;

#[component]
pub fn ListView() -> Element {
    let tasks = use_atom_state(&TASKS);
    let selected_task = use_atom_state(&SELECTED_TASK);
    
    // Filter and sort controls
    let filter = use_signal(|| TaskFilter::All);
    let sort_by = use_signal(|| SortBy::Title);
    
    let filtered_tasks = filter_tasks(&tasks.read(), &filter.read());
    let sorted_tasks = sort_tasks(filtered_tasks, &sort_by.read());
    
    rsx! {
        div {
            class: "list-view",
            
            // Controls
            div {
                class: "list-controls",
                
                select {
                    class: "filter-select",
                    onchange: move |evt| {
                        filter.set(TaskFilter::from_str(&evt.value()).unwrap_or(TaskFilter::All));
                    },
                    option { value: "all", "All Tasks" }
                    option { value: "todo", "To Do" }
                    option { value: "in_progress", "In Progress" }
                    option { value: "done", "Done" }
                    option { value: "blocked", "Blocked" }
                }
                
                select {
                    class: "sort-select",
                    onchange: move |evt| {
                        sort_by.set(SortBy::from_str(&evt.value()).unwrap_or(SortBy::Title));
                    },
                    option { value: "title", "Sort by Title" }
                    option { value: "status", "Sort by Status" }
                    option { value: "priority", "Sort by Priority" }
                    option { value: "due_date", "Sort by Due Date" }
                }
            }
            
            // Task list
            div {
                class: "task-list",
                
                for task in sorted_tasks {
                    TaskListItem {
                        key: "{task.id}",
                        task: task.clone(),
                        selected: selected_task.read().as_ref() == Some(&task.id),
                    }
                }
            }
        }
    }
}

#[component]
fn TaskListItem(task: Task, selected: bool) -> Element {
    let selected_task = use_atom_state(&SELECTED_TASK);
    let editing = use_signal(|| false);
    
    let status_icon = match task.status {
        TaskStatus::Todo => "⭕",
        TaskStatus::InProgress => "🔄",
        TaskStatus::Done => "✅",
        TaskStatus::Blocked => "🚫",
    };
    
    rsx! {
        div {
            class: if selected { "task-list-item selected" } else { "task-list-item" },
            onclick: move |_| {
                selected_task.set(Some(task.id));
            },
            
            div {
                class: "task-item-header",
                
                span { class: "status-icon", "{status_icon}" }
                
                if editing.read() {
                    input {
                        class: "task-title-input",
                        value: "{task.title}",
                        onkeypress: move |evt| {
                            if evt.key() == "Enter" {
                                editing.set(false);
                                // Save task
                            }
                        },
                    }
                } else {
                    h3 {
                        class: "task-title",
                        ondoubleclick: move |_| {
                            editing.set(true);
                        },
                        "{task.title}"
                    }
                }
                
                div {
                    class: "task-actions",
                    
                    button {
                        class: "btn-icon",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            // Start Claude Code
                        },
                        "▶️"
                    }
                    
                    button {
                        class: "btn-icon",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            editing.set(true);
                        },
                        "✏️"
                    }
                    
                    button {
                        class: "btn-icon danger",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            // Delete task
                        },
                        "🗑️"
                    }
                }
            }
            
            p {
                class: "task-description",
                "{task.description}"
            }
            
            div {
                class: "task-metadata",
                
                if let Some(due_date) = task.due_date {
                    span {
                        class: "due-date",
                        "📅 {due_date}"
                    }
                }
                
                if let Some(priority) = task.priority {
                    span {
                        class: "priority",
                        "⚡ Priority: {priority}"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum TaskFilter {
    All,
    Todo,
    InProgress,
    Done,
    Blocked,
}

impl TaskFilter {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "all" => Some(Self::All),
            "todo" => Some(Self::Todo),
            "in_progress" => Some(Self::InProgress),
            "done" => Some(Self::Done),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum SortBy {
    Title,
    Status,
    Priority,
    DueDate,
}

impl SortBy {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "title" => Some(Self::Title),
            "status" => Some(Self::Status),
            "priority" => Some(Self::Priority),
            "due_date" => Some(Self::DueDate),
            _ => None,
        }
    }
}

fn filter_tasks(tasks: &[Task], filter: &TaskFilter) -> Vec<Task> {
    match filter {
        TaskFilter::All => tasks.to_vec(),
        TaskFilter::Todo => tasks.iter().filter(|t| t.status == TaskStatus::Todo).cloned().collect(),
        TaskFilter::InProgress => tasks.iter().filter(|t| t.status == TaskStatus::InProgress).cloned().collect(),
        TaskFilter::Done => tasks.iter().filter(|t| t.status == TaskStatus::Done).cloned().collect(),
        TaskFilter::Blocked => tasks.iter().filter(|t| t.status == TaskStatus::Blocked).cloned().collect(),
    }
}

fn sort_tasks(mut tasks: Vec<Task>, sort_by: &SortBy) -> Vec<Task> {
    match sort_by {
        SortBy::Title => {
            tasks.sort_by(|a, b| a.title.cmp(&b.title));
        }
        SortBy::Status => {
            tasks.sort_by_key(|t| format!("{:?}", t.status));
        }
        SortBy::Priority => {
            tasks.sort_by_key(|t| t.priority.unwrap_or(999));
        }
        SortBy::DueDate => {
            tasks.sort_by_key(|t| t.due_date);
        }
    }
    tasks
}