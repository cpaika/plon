use dioxus::prelude::*;
use fermi::prelude::*;
use crate::domain::task::{Task, TaskStatus};
use crate::ui_dioxus::state::{TASKS, SELECTED_TASK, RUNNING_TASKS};
use uuid::Uuid;
use chrono::{Local, Duration, NaiveDate};

#[component]
pub fn TimelineView() -> Element {
    let tasks = use_atom_state(&TASKS);
    let selected_task = use_atom_state(&SELECTED_TASK);
    let running_tasks = use_atom_state(&RUNNING_TASKS);
    
    // Timeline navigation state
    let current_date = use_signal(|| Local::now().date_naive());
    let days_to_show = use_signal(|| 30i64);
    let scroll_offset = use_signal(|| 0.0f32);
    
    // Calculate date range
    let start_date = current_date.read().clone();
    let end_date = start_date + Duration::days(*days_to_show.read());
    
    // Filter tasks with due dates
    let timeline_tasks: Vec<Task> = tasks.read()
        .iter()
        .filter(|t| t.due_date.is_some())
        .cloned()
        .collect();
    
    rsx! {
        div {
            class: "timeline-view",
            
            // Timeline controls
            TimelineControls {
                current_date: current_date.clone(),
                days_to_show: days_to_show.clone(),
            }
            
            // Timeline content
            div {
                class: "timeline-container",
                onwheel: move |evt| {
                    // Horizontal scroll with mouse wheel
                    let delta = evt.delta_y() as f32;
                    scroll_offset.set((scroll_offset.read() + delta).max(0.0));
                },
                
                // Date headers
                div {
                    class: "timeline-header",
                    style: "transform: translateX(-{scroll_offset.read()}px)",
                    
                    for day in 0..=*days_to_show.read() {
                        let date = start_date + Duration::days(day);
                        DateColumn {
                            date: date,
                            is_today: date == Local::now().date_naive(),
                        }
                    }
                }
                
                // Task rows
                div {
                    class: "timeline-body",
                    style: "transform: translateX(-{scroll_offset.read()}px)",
                    
                    for task in timeline_tasks.iter() {
                        TimelineTask {
                            key: "{task.id}",
                            task: task.clone(),
                            start_date: start_date,
                            days_to_show: *days_to_show.read(),
                            selected: selected_task.read().as_ref() == Some(&task.id),
                            running: running_tasks.read().contains_key(&task.id),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TimelineControls(
    current_date: Signal<NaiveDate>,
    days_to_show: Signal<i64>,
) -> Element {
    rsx! {
        div {
            class: "timeline-controls",
            
            button {
                class: "btn",
                onclick: move |_| {
                    let new_date = *current_date.read() - Duration::days(7);
                    current_date.set(new_date);
                },
                "← Previous Week"
            }
            
            button {
                class: "btn",
                onclick: move |_| {
                    current_date.set(Local::now().date_naive());
                },
                "Today"
            }
            
            button {
                class: "btn",
                onclick: move |_| {
                    let new_date = *current_date.read() + Duration::days(7);
                    current_date.set(new_date);
                },
                "Next Week →"
            }
            
            select {
                class: "timeline-zoom",
                onchange: move |evt| {
                    if let Ok(days) = evt.value().parse::<i64>() {
                        days_to_show.set(days);
                    }
                },
                option { value: "7", "1 Week" }
                option { value: "14", "2 Weeks" }
                option { value: "30", selected: true, "1 Month" }
                option { value: "90", "3 Months" }
            }
        }
    }
}

#[component]
fn DateColumn(date: NaiveDate, is_today: bool) -> Element {
    let day_of_week = date.format("%a").to_string();
    let day_number = date.format("%d").to_string();
    let month = date.format("%b").to_string();
    
    rsx! {
        div {
            class: if is_today { "date-column today" } else { "date-column" },
            style: "width: 100px; min-width: 100px;",
            
            div { class: "day-of-week", "{day_of_week}" }
            div { class: "day-number", "{day_number}" }
            div { class: "month", "{month}" }
        }
    }
}

#[component]
fn TimelineTask(
    task: Task,
    start_date: NaiveDate,
    days_to_show: i64,
    selected: bool,
    running: bool,
) -> Element {
    let selected_task = use_atom_state(&SELECTED_TASK);
    let running_tasks = use_atom_state(&RUNNING_TASKS);
    
    // Calculate task position and width
    let (left, width) = if let Some(due_date) = task.due_date {
        let days_from_start = (due_date - start_date).num_days();
        let task_duration = 1; // Default to 1 day for now
        
        if days_from_start >= 0 && days_from_start <= days_to_show {
            let left = days_from_start as f32 * 100.0;
            let width = task_duration as f32 * 100.0;
            (left, width)
        } else {
            (0.0, 0.0)
        }
    } else {
        (0.0, 0.0)
    };
    
    if width == 0.0 {
        return rsx! { Fragment {} };
    }
    
    let status_color = match task.status {
        TaskStatus::Todo => "#808080",
        TaskStatus::InProgress => "#FFA500",
        TaskStatus::Done => "#00FF00",
        TaskStatus::Blocked => "#FF0000",
    };
    
    rsx! {
        div {
            class: if selected { "timeline-task selected" } else { "timeline-task" },
            style: "position: absolute; left: {left}px; width: {width}px; background: {status_color};",
            onclick: move |_| {
                selected_task.set(Some(task.id));
            },
            
            div {
                class: "task-content",
                
                span { class: "task-title", "{task.title}" }
                
                if task.status == TaskStatus::Todo && !running {
                    button {
                        class: "play-btn",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            let mut tasks = running_tasks.write();
                            tasks.insert(task.id, crate::ui_dioxus::state::TaskExecutionStatus::Running);
                        },
                        "▶"
                    }
                }
                
                if running {
                    span { class: "running-badge", "🔄" }
                }
            }
        }
    }
}