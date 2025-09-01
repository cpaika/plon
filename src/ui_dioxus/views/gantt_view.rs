use dioxus::prelude::*;
use fermi::prelude::*;
use crate::domain::task::{Task, TaskStatus};
use crate::domain::dependency::{Dependency, DependencyType};
use crate::ui_dioxus::state::{TASKS, DEPENDENCIES, SELECTED_TASK, RUNNING_TASKS};
use uuid::Uuid;
use chrono::{Local, Duration, NaiveDate};
use std::collections::HashMap;

#[component]
pub fn GanttView() -> Element {
    let tasks = use_atom_state(&TASKS);
    let dependencies = use_atom_state(&DEPENDENCIES);
    let selected_task = use_atom_state(&SELECTED_TASK);
    let running_tasks = use_atom_state(&RUNNING_TASKS);
    
    // View state
    let start_date = use_signal(|| Local::now().date_naive());
    let months_to_show = use_signal(|| 3i32);
    let expanded_tasks = use_signal(|| HashMap::<Uuid, bool>::new());
    
    // Calculate date range
    let end_date = start_date.read().clone() + Duration::days((*months_to_show.read() * 30) as i64);
    
    // Group tasks by parent/child relationships
    let task_tree = build_task_tree(&tasks.read(), &dependencies.read());
    
    rsx! {
        div {
            class: "gantt-view",
            
            // Gantt controls
            GanttControls {
                start_date: start_date.clone(),
                months_to_show: months_to_show.clone(),
            }
            
            div {
                class: "gantt-container",
                
                // Task list sidebar
                div {
                    class: "gantt-sidebar",
                    
                    div {
                        class: "gantt-header-row",
                        div { class: "task-name-header", "Task Name" }
                        div { class: "task-status-header", "Status" }
                        div { class: "task-duration-header", "Duration" }
                    }
                    
                    for (task, children) in task_tree.iter() {
                        GanttTaskRow {
                            task: task.clone(),
                            children: children.clone(),
                            expanded_tasks: expanded_tasks.clone(),
                            level: 0,
                            selected: selected_task.read().as_ref() == Some(&task.id),
                            running: running_tasks.read().contains_key(&task.id),
                        }
                    }
                }
                
                // Gantt chart area
                div {
                    class: "gantt-chart",
                    
                    // Date headers
                    GanttDateHeaders {
                        start_date: *start_date.read(),
                        months_to_show: *months_to_show.read(),
                    }
                    
                    // Task bars
                    div {
                        class: "gantt-bars",
                        
                        for (task, children) in task_tree.iter() {
                            GanttTaskBars {
                                task: task.clone(),
                                children: children.clone(),
                                expanded_tasks: expanded_tasks.clone(),
                                start_date: *start_date.read(),
                                end_date: end_date,
                                dependencies: dependencies.read().clone(),
                                level: 0,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GanttControls(
    start_date: Signal<NaiveDate>,
    months_to_show: Signal<i32>,
) -> Element {
    rsx! {
        div {
            class: "gantt-controls",
            
            button {
                class: "btn",
                onclick: move |_| {
                    let new_date = *start_date.read() - Duration::days(30);
                    start_date.set(new_date);
                },
                "← Previous Month"
            }
            
            button {
                class: "btn",
                onclick: move |_| {
                    start_date.set(Local::now().date_naive());
                },
                "Today"
            }
            
            button {
                class: "btn",
                onclick: move |_| {
                    let new_date = *start_date.read() + Duration::days(30);
                    start_date.set(new_date);
                },
                "Next Month →"
            }
            
            select {
                class: "gantt-zoom",
                onchange: move |evt| {
                    if let Ok(months) = evt.value().parse::<i32>() {
                        months_to_show.set(months);
                    }
                },
                option { value: "1", "1 Month" }
                option { value: "3", selected: true, "3 Months" }
                option { value: "6", "6 Months" }
                option { value: "12", "1 Year" }
            }
            
            button {
                class: "btn",
                onclick: move |_| {
                    // Export to image/PDF
                },
                "📥 Export"
            }
        }
    }
}

#[component]
fn GanttDateHeaders(start_date: NaiveDate, months_to_show: i32) -> Element {
    let days_total = months_to_show * 30;
    let day_width = 100.0 / days_total as f32;
    
    rsx! {
        div {
            class: "gantt-date-headers",
            
            // Month headers
            div {
                class: "month-headers",
                
                for month in 0..months_to_show {
                    let month_date = start_date + Duration::days((month * 30) as i64);
                    div {
                        class: "month-header",
                        style: "width: {100.0 / months_to_show as f32}%",
                        "{month_date.format(\"%B %Y\")}"
                    }
                }
            }
            
            // Week markers
            div {
                class: "week-markers",
                
                for week in 0..(days_total / 7) {
                    div {
                        class: "week-marker",
                        style: "width: {day_width * 7.0}%; left: {week as f32 * day_width * 7.0}%",
                        "W{week + 1}"
                    }
                }
            }
        }
    }
}

#[component]
fn GanttTaskRow(
    task: Task,
    children: Vec<Task>,
    expanded_tasks: Signal<HashMap<Uuid, bool>>,
    level: usize,
    selected: bool,
    running: bool,
) -> Element {
    let selected_task = use_atom_state(&SELECTED_TASK);
    let running_tasks = use_atom_state(&RUNNING_TASKS);
    let is_expanded = expanded_tasks.read().get(&task.id).copied().unwrap_or(false);
    
    let status_icon = match task.status {
        TaskStatus::Todo => "⭕",
        TaskStatus::InProgress => "🔄",
        TaskStatus::Done => "✅",
        TaskStatus::Blocked => "🚫",
    };
    
    rsx! {
        Fragment {
            div {
                class: if selected { "gantt-task-row selected" } else { "gantt-task-row" },
                style: "padding-left: {level * 20}px",
                onclick: move |_| {
                    selected_task.set(Some(task.id));
                },
                
                div {
                    class: "task-name",
                    
                    if !children.is_empty() {
                        button {
                            class: "expand-btn",
                            onclick: move |evt| {
                                evt.stop_propagation();
                                let mut expanded = expanded_tasks.write();
                                let current = expanded.get(&task.id).copied().unwrap_or(false);
                                expanded.insert(task.id, !current);
                            },
                            if is_expanded { "▼" } else { "▶" }
                        }
                    }
                    
                    span { "{task.title}" }
                    
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
                
                div { class: "task-status", "{status_icon}" }
                
                div {
                    class: "task-duration",
                    if let Some(due_date) = task.due_date {
                        "{due_date.format(\"%m/%d\")}"
                    }
                }
            }
            
            if is_expanded {
                for child in children.iter() {
                    GanttTaskRow {
                        task: child.clone(),
                        children: vec![],
                        expanded_tasks: expanded_tasks.clone(),
                        level: level + 1,
                        selected: selected_task.read().as_ref() == Some(&child.id),
                        running: running_tasks.read().contains_key(&child.id),
                    }
                }
            }
        }
    }
}

#[component]
fn GanttTaskBars(
    task: Task,
    children: Vec<Task>,
    expanded_tasks: Signal<HashMap<Uuid, bool>>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    dependencies: Vec<Dependency>,
    level: usize,
) -> Element {
    let is_expanded = expanded_tasks.read().get(&task.id).copied().unwrap_or(false);
    
    // Calculate task bar position and width
    let (left, width) = calculate_task_position(&task, start_date, end_date);
    
    if width == 0.0 {
        return rsx! { Fragment {} };
    }
    
    let status_color = match task.status {
        TaskStatus::Todo => "#808080",
        TaskStatus::InProgress => "#FFA500",
        TaskStatus::Done => "#00FF00",
        TaskStatus::Blocked => "#FF0000",
    };
    
    // Find dependencies for this task
    let task_dependencies: Vec<Dependency> = dependencies
        .iter()
        .filter(|d| d.to_task_id == task.id)
        .cloned()
        .collect();
    
    rsx! {
        Fragment {
            div {
                class: "gantt-bar-row",
                style: "height: 40px; position: relative;",
                
                // Task bar
                div {
                    class: "gantt-bar",
                    style: "position: absolute; left: {left}%; width: {width}%; background: {status_color}; height: 20px; top: 10px; border-radius: 4px;",
                    
                    // Progress indicator (if in progress)
                    if task.status == TaskStatus::InProgress {
                        div {
                            class: "progress-bar",
                            style: "width: 50%; background: #4CAF50; height: 100%; border-radius: 4px;",
                        }
                    }
                }
                
                // Dependency arrows
                for dep in task_dependencies.iter() {
                    GanttDependencyArrow {
                        dependency: dep.clone(),
                        start_date: start_date,
                        end_date: end_date,
                    }
                }
            }
            
            if is_expanded {
                for child in children.iter() {
                    GanttTaskBars {
                        task: child.clone(),
                        children: vec![],
                        expanded_tasks: expanded_tasks.clone(),
                        start_date: start_date,
                        end_date: end_date,
                        dependencies: dependencies.clone(),
                        level: level + 1,
                    }
                }
            }
        }
    }
}

#[component]
fn GanttDependencyArrow(
    dependency: Dependency,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Element {
    let arrow_color = match dependency.dependency_type {
        DependencyType::FinishToStart => "#4CAF50",
        DependencyType::StartToStart => "#2196F3",
        DependencyType::FinishToFinish => "#FF9800",
        DependencyType::StartToFinish => "#F44336",
    };
    
    rsx! {
        svg {
            class: "dependency-arrow",
            style: "position: absolute; width: 100%; height: 40px; pointer-events: none;",
            
            path {
                d: "M 10 20 L 50 20",
                stroke: arrow_color,
                stroke_width: "2",
                fill: "none",
                marker_end: "url(#arrowhead)",
            }
        }
    }
}

// Helper functions
fn build_task_tree(tasks: &[Task], dependencies: &[Dependency]) -> Vec<(Task, Vec<Task>)> {
    let mut tree = Vec::new();
    let mut parent_map = HashMap::<Uuid, Vec<Task>>::new();
    
    // Build parent-child relationships based on dependencies
    for dep in dependencies {
        if dep.dependency_type == DependencyType::FinishToStart {
            // Consider finish-to-start as parent-child for tree structure
            let parent_id = dep.from_task_id;
            let child_id = dep.to_task_id;
            
            if let Some(child_task) = tasks.iter().find(|t| t.id == child_id) {
                parent_map.entry(parent_id)
                    .or_insert_with(Vec::new)
                    .push(child_task.clone());
            }
        }
    }
    
    // Add root tasks (tasks without incoming dependencies)
    for task in tasks {
        let has_parent = dependencies.iter().any(|d| d.to_task_id == task.id);
        if !has_parent {
            let children = parent_map.get(&task.id).cloned().unwrap_or_default();
            tree.push((task.clone(), children));
        }
    }
    
    tree
}

fn calculate_task_position(task: &Task, start_date: NaiveDate, end_date: NaiveDate) -> (f32, f32) {
    if let Some(due_date) = task.due_date {
        let total_days = (end_date - start_date).num_days() as f32;
        let days_from_start = (due_date - start_date).num_days() as f32;
        
        // Default duration of 3 days for tasks
        let duration = 3.0;
        
        if days_from_start >= 0.0 && days_from_start <= total_days {
            let left = (days_from_start / total_days) * 100.0;
            let width = (duration / total_days) * 100.0;
            (left, width.min(100.0 - left))
        } else {
            (0.0, 0.0)
        }
    } else {
        (0.0, 0.0)
    }
}