use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::domain::dependency::{Dependency, DependencyType};
use crate::repository::Repository;
use chrono::{DateTime, Utc, Duration, Datelike};
use uuid::Uuid;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum TimeRange {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GanttTask {
    pub task: Task,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub progress: f32,
    pub row: usize,
    pub is_milestone: bool,
    pub is_critical: bool,
}

#[derive(Clone, Debug)]
struct ViewSettings {
    time_range: TimeRange,
    start_date: DateTime<Utc>,
    show_dependencies: bool,
    show_progress: bool,
    show_resources: bool,
    zoom_level: f32,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            time_range: TimeRange::Month,
            start_date: Utc::now(),
            show_dependencies: true,
            show_progress: true,
            show_resources: false,
            zoom_level: 1.0,
        }
    }
}

#[component]
pub fn GanttViewEnhanced() -> Element {
    let repository = use_context::<Arc<Repository>>();
    
    // State
    let mut tasks = use_signal(|| Vec::<Task>::new());
    let mut dependencies = use_signal(|| Vec::<Dependency>::new());
    let mut gantt_tasks = use_signal(|| Vec::<GanttTask>::new());
    let mut view_settings = use_signal(ViewSettings::default);
    let mut selected_task = use_signal(|| None::<Uuid>);
    let mut dragging_task = use_signal(|| None::<(Uuid, String)>); // (task_id, "start" | "end" | "move")
    let mut drag_start_pos = use_signal(|| 0.0);
    let mut hover_task = use_signal(|| None::<Uuid>);
    
    // Load tasks and dependencies
    use_effect(move || {
        let repo = repository.clone();
        let mut tasks = tasks.clone();
        let mut dependencies = dependencies.clone();
        
        spawn(async move {
            // Load tasks
            match repo.tasks.list(Default::default()).await {
                Ok(loaded_tasks) => {
                    tasks.set(loaded_tasks);
                }
                Err(e) => eprintln!("Failed to load tasks: {}", e),
            }
            
            // Load dependencies
            match repo.dependencies.list_all().await {
                Ok(deps) => dependencies.set(deps),
                Err(e) => eprintln!("Failed to load dependencies: {}", e),
            }
        });
    });
    
    // Convert tasks to GanttTasks
    use_effect(move || {
        let tasks_list = tasks.read();
        let mut gantt_list = Vec::new();
        
        for (i, task) in tasks_list.iter().enumerate() {
            let start = task.scheduled_date.unwrap_or_else(|| Utc::now());
            let end = task.due_date.unwrap_or_else(|| start + Duration::days(
                (task.estimated_hours.unwrap_or(8.0) / 8.0) as i64
            ));
            
            // Calculate progress based on status
            let progress = match task.status {
                TaskStatus::Done => 1.0,
                TaskStatus::InProgress => 0.5,
                _ => 0.0,
            };
            
            // Check if milestone (zero duration)
            let is_milestone = start == end || task.estimated_hours == Some(0.0);
            
            gantt_list.push(GanttTask {
                task: task.clone(),
                start_date: start,
                end_date: end,
                progress,
                row: i,
                is_milestone,
                is_critical: false, // Will calculate later
            });
        }
        
        // Calculate critical path
        calculate_critical_path(&mut gantt_list, &dependencies.read());
        
        gantt_tasks.set(gantt_list);
    });
    
    // Helper functions
    let get_time_columns = move || -> Vec<String> {
        let settings = view_settings.read();
        let mut columns = Vec::new();
        
        match settings.time_range {
            TimeRange::Day => {
                for hour in 0..24 {
                    columns.push(format!("{:02}:00", hour));
                }
            }
            TimeRange::Week => {
                let start = settings.start_date;
                for day in 0..7 {
                    let date = start + Duration::days(day);
                    columns.push(format!("{}/{}", date.month(), date.day()));
                }
            }
            TimeRange::Month => {
                let start = settings.start_date;
                for week in 0..5 {
                    let date = start + Duration::weeks(week);
                    columns.push(format!("Week {}", week + 1));
                }
            }
            TimeRange::Quarter => {
                for month in 0..3 {
                    columns.push(format!("Month {}", month + 1));
                }
            }
            TimeRange::Year => {
                for month in 1..=12 {
                    columns.push(format!("{}", month_name(month)));
                }
            }
        }
        
        columns
    };
    
    let calculate_task_position = move |task: &GanttTask| -> (f64, f64) {
        let settings = view_settings.read();
        let chart_start = settings.start_date;
        let column_width = 100.0 * settings.zoom_level as f64;
        
        let days_per_column = match settings.time_range {
            TimeRange::Day => 1.0 / 24.0,
            TimeRange::Week => 1.0,
            TimeRange::Month => 7.0,
            TimeRange::Quarter => 30.0,
            TimeRange::Year => 30.0,
        };
        
        let start_offset = (task.start_date - chart_start).num_days() as f64 / days_per_column;
        let duration = (task.end_date - task.start_date).num_days() as f64 / days_per_column;
        
        let x = start_offset * column_width;
        let width = duration * column_width;
        
        (x.max(0.0), width.max(10.0))
    };
    
    rsx! {
        div {
            style: "width: 100%; height: 100vh; display: flex; flex-direction: column; background: #f5f5f5;",
            
            // Toolbar
            div {
                style: "padding: 10px; background: white; box-shadow: 0 2px 4px rgba(0,0,0,0.1);",
                
                div {
                    style: "display: flex; gap: 10px; align-items: center; flex-wrap: wrap;",
                    
                    h2 { style: "margin: 0; margin-right: 20px;", "📊 Enhanced Gantt Chart" }
                    
                    // Time range selector
                    select {
                        onchange: move |evt| {
                            let mut settings = view_settings.write();
                            settings.time_range = match evt.value().as_str() {
                                "day" => TimeRange::Day,
                                "week" => TimeRange::Week,
                                "month" => TimeRange::Month,
                                "quarter" => TimeRange::Quarter,
                                "year" => TimeRange::Year,
                                _ => TimeRange::Month,
                            };
                        },
                        style: "padding: 6px; border: 1px solid #ddd; border-radius: 4px;",
                        
                        option { value: "day", "Day View" }
                        option { value: "week", "Week View" }
                        option { value: "month", selected: true, "Month View" }
                        option { value: "quarter", "Quarter View" }
                        option { value: "year", "Year View" }
                    }
                    
                    // Navigation
                    button {
                        onclick: move |_| {
                            let mut settings = view_settings.write();
                            settings.start_date = settings.start_date - match settings.time_range {
                                TimeRange::Day => Duration::days(1),
                                TimeRange::Week => Duration::weeks(1),
                                TimeRange::Month => Duration::days(30),
                                TimeRange::Quarter => Duration::days(90),
                                TimeRange::Year => Duration::days(365),
                            };
                        },
                        style: "padding: 6px 10px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "◀"
                    }
                    
                    button {
                        onclick: move |_| {
                            let mut settings = view_settings.write();
                            settings.start_date = Utc::now();
                        },
                        style: "padding: 6px 10px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "Today"
                    }
                    
                    button {
                        onclick: move |_| {
                            let mut settings = view_settings.write();
                            settings.start_date = settings.start_date + match settings.time_range {
                                TimeRange::Day => Duration::days(1),
                                TimeRange::Week => Duration::weeks(1),
                                TimeRange::Month => Duration::days(30),
                                TimeRange::Quarter => Duration::days(90),
                                TimeRange::Year => Duration::days(365),
                            };
                        },
                        style: "padding: 6px 10px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "▶"
                    }
                    
                    div { style: "border-left: 1px solid #ddd; height: 30px; margin: 0 10px;" }
                    
                    // View options
                    label {
                        style: "display: flex; align-items: center; gap: 5px;",
                        input {
                            r#type: "checkbox",
                            checked: view_settings.read().show_dependencies,
                            onchange: move |evt| {
                                view_settings.write().show_dependencies = evt.checked();
                            },
                        }
                        "Dependencies"
                    }
                    
                    label {
                        style: "display: flex; align-items: center; gap: 5px;",
                        input {
                            r#type: "checkbox",
                            checked: view_settings.read().show_progress,
                            onchange: move |evt| {
                                view_settings.write().show_progress = evt.checked();
                            },
                        }
                        "Progress"
                    }
                    
                    label {
                        style: "display: flex; align-items: center; gap: 5px;",
                        input {
                            r#type: "checkbox",
                            checked: view_settings.read().show_resources,
                            onchange: move |evt| {
                                view_settings.write().show_resources = evt.checked();
                            },
                        }
                        "Resources"
                    }
                    
                    div { style: "border-left: 1px solid #ddd; height: 30px; margin: 0 10px;" }
                    
                    // Zoom controls
                    button {
                        onclick: move |_| {
                            view_settings.write().zoom_level = (view_settings.read().zoom_level * 1.2).min(3.0);
                        },
                        style: "padding: 6px 10px; background: #FF9800; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "🔍+"
                    }
                    
                    button {
                        onclick: move |_| {
                            view_settings.write().zoom_level = 1.0;
                        },
                        style: "padding: 6px 10px; background: #FF9800; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "100%"
                    }
                    
                    button {
                        onclick: move |_| {
                            view_settings.write().zoom_level = (view_settings.read().zoom_level / 1.2).max(0.5);
                        },
                        style: "padding: 6px 10px; background: #FF9800; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "🔍-"
                    }
                    
                    div { style: "flex: 1;" }
                    
                    button {
                        onclick: move |_| {
                            // Export functionality
                            export_to_image(&gantt_tasks.read());
                        },
                        style: "padding: 6px 12px; background: #9C27B0; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "📥 Export"
                    }
                }
            }
            
            // Legend
            div {
                style: "padding: 10px; background: white; border-bottom: 1px solid #ddd; display: flex; gap: 20px;",
                
                div {
                    style: "display: flex; align-items: center; gap: 5px;",
                    div { style: "width: 20px; height: 12px; background: #808080; border-radius: 2px;" }
                    "Todo"
                }
                div {
                    style: "display: flex; align-items: center; gap: 5px;",
                    div { style: "width: 20px; height: 12px; background: #2196F3; border-radius: 2px;" }
                    "In Progress"
                }
                div {
                    style: "display: flex; align-items: center; gap: 5px;",
                    div { style: "width: 20px; height: 12px; background: #4CAF50; border-radius: 2px;" }
                    "Done"
                }
                div {
                    style: "display: flex; align-items: center; gap: 5px;",
                    div { style: "width: 20px; height: 12px; background: #F44336; border-radius: 2px;" }
                    "Blocked"
                }
                div {
                    style: "display: flex; align-items: center; gap: 5px;",
                    div { style: "width: 12px; height: 12px; background: #FFD700; transform: rotate(45deg);" }
                    "Milestone"
                }
                div {
                    style: "display: flex; align-items: center; gap: 5px;",
                    div { style: "width: 20px; height: 3px; background: #FF5722;" }
                    "Critical Path"
                }
            }
            
            // Main chart area
            div {
                style: "flex: 1; background: white; overflow: auto; position: relative;",
                
                div {
                    style: "display: flex; min-width: 1200px;",
                    
                    // Task list (left side)
                    div {
                        style: "flex: 0 0 300px; border-right: 2px solid #ddd; position: sticky; left: 0; background: white; z-index: 10;",
                        
                        // Header
                        div {
                            style: "padding: 10px; background: #f5f5f5; border-bottom: 1px solid #ddd; font-weight: bold; display: flex; gap: 10px;",
                            div { style: "flex: 1;", "Task Name" }
                            div { style: "width: 80px;", "Duration" }
                            div { style: "width: 60px;", "Progress" }
                        }
                        
                        // Task rows
                        for task in gantt_tasks.read().iter() {
                            TaskRow {
                                task: task.clone(),
                                selected: selected_task.read().as_ref() == Some(&task.task.id),
                                onclick: move |_| selected_task.set(Some(task.task.id)),
                            }
                        }
                    }
                    
                    // Timeline (right side)
                    div {
                        style: "flex: 1; position: relative;",
                        
                        // Time headers
                        div {
                            style: "display: flex; background: #f5f5f5; border-bottom: 1px solid #ddd; position: sticky; top: 0; z-index: 5;",
                            
                            for column in get_time_columns() {
                                div {
                                    style: "width: {100.0 * view_settings.read().zoom_level as f64}px; padding: 10px 5px; text-align: center; border-right: 1px solid #ddd; font-size: 12px;",
                                    "{column}"
                                }
                            }
                        }
                        
                        // Task bars and dependencies
                        div {
                            style: "position: relative;",
                            
                            // Today line
                            {
                                let settings = view_settings.read();
                                let today_offset = (Utc::now() - settings.start_date).num_days() as f64;
                                let column_width = 100.0 * settings.zoom_level as f64;
                                let days_per_column = match settings.time_range {
                                    TimeRange::Day => 1.0 / 24.0,
                                    TimeRange::Week => 1.0,
                                    TimeRange::Month => 7.0,
                                    TimeRange::Quarter => 30.0,
                                    TimeRange::Year => 30.0,
                                };
                                let x = (today_offset / days_per_column) * column_width;
                                
                                rsx! {
                                    div {
                                        style: "position: absolute; left: {x}px; top: 0; bottom: 0; width: 2px; background: #FF5722; opacity: 0.5; z-index: 1;",
                                    }
                                }
                            }
                            
                            // Dependencies
                            if view_settings.read().show_dependencies {
                                for dep in dependencies.read().iter() {
                                    if let (Some(from), Some(to)) = (
                                        gantt_tasks.read().iter().find(|t| t.task.id == dep.from_task_id),
                                        gantt_tasks.read().iter().find(|t| t.task.id == dep.to_task_id)
                                    ) {
                                        DependencyLine {
                                            from_task: from.clone(),
                                            to_task: to.clone(),
                                            dep_type: dep.dependency_type,
                                        }
                                    }
                                }
                            }
                            
                            // Task bars
                            for task in gantt_tasks.read().iter() {
                                GanttBar {
                                    task: task.clone(),
                                    position: calculate_task_position(task),
                                    selected: selected_task.read().as_ref() == Some(&task.task.id),
                                    show_progress: view_settings.read().show_progress,
                                    onselect: move |_| selected_task.set(Some(task.task.id)),
                                    on_resize_start: move |delta| {
                                        // Handle resize start
                                        let mut tasks_list = gantt_tasks.write();
                                        if let Some(t) = tasks_list.iter_mut().find(|t| t.task.id == task.task.id) {
                                            t.start_date = t.start_date + Duration::days(delta as i64);
                                            
                                            // Update estimated hours
                                            let duration_days = (t.end_date - t.start_date).num_days();
                                            t.task.estimated_hours = Some((duration_days * 8) as f32);
                                        }
                                    },
                                    on_resize_end: move |delta| {
                                        // Handle resize end
                                        let mut tasks_list = gantt_tasks.write();
                                        if let Some(t) = tasks_list.iter_mut().find(|t| t.task.id == task.task.id) {
                                            t.end_date = t.end_date + Duration::days(delta as i64);
                                            
                                            // Update estimated hours
                                            let duration_days = (t.end_date - t.start_date).num_days();
                                            t.task.estimated_hours = Some((duration_days * 8) as f32);
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TaskRow(task: GanttTask, selected: bool, onclick: EventHandler<()>) -> Element {
    let duration_days = (task.end_date - task.start_date).num_days();
    let progress_percent = (task.progress * 100.0) as i32;
    
    rsx! {
        div {
            style: "padding: 10px; border-bottom: 1px solid #eee; min-height: 50px; display: flex; gap: 10px; align-items: center; background: {if selected { \"#f0f8ff\" } else { \"white\" }}; cursor: pointer;",
            onclick: move |_| onclick.call(()),
            
            div {
                style: "flex: 1;",
                div {
                    style: "font-weight: 500; font-size: 14px; color: {if task.is_critical { \"#FF5722\" } else { \"#333\" }};",
                    "{task.task.title}"
                    if task.is_milestone {
                        span { style: "margin-left: 5px; color: #FFD700;", "◆" }
                    }
                }
                if task.task.assignee.is_some() {
                    div {
                        style: "font-size: 11px; color: #666;",
                        "👤 {task.task.assignee.as_ref().unwrap()}"
                    }
                }
            }
            
            div {
                style: "width: 80px; font-size: 12px; color: #666;",
                "{duration_days}d"
            }
            
            div {
                style: "width: 60px; font-size: 12px;",
                div {
                    style: "background: #e0e0e0; border-radius: 10px; height: 8px; overflow: hidden;",
                    div {
                        style: "background: #4CAF50; height: 100%; width: {progress_percent}%;",
                    }
                }
                div {
                    style: "text-align: center; font-size: 10px; color: #666; margin-top: 2px;",
                    "{progress_percent}%"
                }
            }
        }
    }
}

#[component]
fn GanttBar(
    task: GanttTask,
    position: (f64, f64),
    selected: bool,
    show_progress: bool,
    onselect: EventHandler<()>,
    on_resize_start: EventHandler<f64>,
    on_resize_end: EventHandler<f64>,
) -> Element {
    let (x, width) = position;
    let y = (task.row * 50) as f64 + 45.0;
    
    let bar_color = match task.task.status {
        TaskStatus::Todo => "#808080",
        TaskStatus::InProgress => "#2196F3",
        TaskStatus::Done => "#4CAF50",
        TaskStatus::Blocked => "#F44336",
        _ => "#808080",
    };
    
    if task.is_milestone {
        // Render milestone as diamond
        rsx! {
            div {
                style: "position: absolute; left: {x}px; top: {y + 10.0}px; width: 20px; height: 20px; background: #FFD700; transform: rotate(45deg); cursor: pointer; z-index: 2;",
                onclick: move |_| onselect.call(()),
                title: "{task.task.title}",
            }
        }
    } else {
        // Render regular task bar
        rsx! {
            div {
                style: "position: absolute; left: {x}px; top: {y}px; width: {width}px; height: 30px; z-index: 2;",
                
                // Main bar
                div {
                    style: "position: relative; height: 100%; background: {bar_color}; border-radius: 4px; cursor: move; box-shadow: {if selected { \"0 0 0 2px #333\" } else { \"0 2px 4px rgba(0,0,0,0.1)\" }}; {if task.is_critical { \"border-bottom: 3px solid #FF5722;\" } else { \"\" }}",
                    onclick: move |_| onselect.call(()),
                    title: "{task.task.title}",
                    
                    // Progress bar
                    if show_progress {
                        div {
                            style: "position: absolute; top: 0; left: 0; height: 100%; width: {task.progress * 100.0}%; background: rgba(255,255,255,0.3); border-radius: 4px;",
                        }
                    }
                    
                    // Task title
                    div {
                        style: "color: white; font-size: 11px; padding: 5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{task.task.title}"
                    }
                    
                    // Resize handles
                    div {
                        style: "position: absolute; left: 0; top: 0; width: 5px; height: 100%; cursor: ew-resize; background: rgba(0,0,0,0.2);",
                        onmousedown: move |_| {
                            on_resize_start.call(-1.0);
                        },
                    }
                    
                    div {
                        style: "position: absolute; right: 0; top: 0; width: 5px; height: 100%; cursor: ew-resize; background: rgba(0,0,0,0.2);",
                        onmousedown: move |_| {
                            on_resize_end.call(1.0);
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn DependencyLine(from_task: GanttTask, to_task: GanttTask, dep_type: DependencyType) -> Element {
    // Simplified dependency rendering - just draw a line
    let from_y = (from_task.row * 50) as f64 + 60.0;
    let to_y = (to_task.row * 50) as f64 + 60.0;
    
    let color = match dep_type {
        DependencyType::FinishToStart => "#666",
        DependencyType::StartToStart => "#2196F3",
        DependencyType::FinishToFinish => "#FF9800",
        DependencyType::StartToFinish => "#F44336",
    };
    
    rsx! {
        svg {
            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 1;",
            
            line {
                x1: "100",
                y1: "{from_y}",
                x2: "200",
                y2: "{to_y}",
                stroke: color,
                "stroke-width": "1",
                "stroke-dasharray": "5,5",
                opacity: "0.5",
            }
        }
    }
}

// Helper functions
fn calculate_critical_path(tasks: &mut Vec<GanttTask>, _dependencies: &Vec<Dependency>) {
    // Simple critical path marking - in real implementation would use proper algorithm
    for task in tasks.iter_mut() {
        // Mark tasks with high priority as critical for now
        task.is_critical = task.task.priority == Priority::High || task.task.priority == Priority::Critical;
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
        5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
        9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
        _ => "",
    }
}

fn export_to_image(tasks: &Vec<GanttTask>) {
    // Mock export functionality
    println!("Exporting {} tasks to image...", tasks.len());
}