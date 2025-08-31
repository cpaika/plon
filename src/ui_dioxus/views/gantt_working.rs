use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate, Datelike};

#[component]
pub fn GanttView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut selected_task = use_signal(|| None::<Uuid>);
    let mut zoom_level = use_signal(|| "week"); // "day", "week", "month"
    
    // Calculate date range
    let today = Utc::now().naive_local().date();
    let start_date = today - Duration::days(7);
    let end_date = today + Duration::days(30);
    
    // Generate date columns based on zoom level
    let date_columns = generate_date_columns(&start_date, &end_date, &zoom_level.read());
    
    // Sort tasks by start date
    let mut sorted_tasks = tasks.read().clone();
    sorted_tasks.sort_by(|a, b| {
        let a_date = a.created_at.naive_local().date();
        let b_date = b.created_at.naive_local().date();
        a_date.cmp(&b_date)
    });
    
    rsx! {
        div {
            style: "padding: 20px; height: 100vh; background: #f5f5f5;",
            
            h2 { "Gantt Chart" }
            
            // Controls
            div {
                style: "margin-bottom: 20px; padding: 15px; background: white; border-radius: 8px; display: flex; gap: 10px; align-items: center;",
                
                // Zoom controls
                div {
                    style: "display: flex; gap: 5px; background: #f0f0f0; padding: 4px; border-radius: 6px;",
                    
                    button {
                        onclick: move |_| zoom_level.set("day"),
                        style: "padding: 6px 12px; border: none; border-radius: 4px; cursor: pointer;
                               background: {if zoom_level.read().as_ref() == \"day\" { \"#2196F3\" } else { \"transparent\" }};
                               color: {if zoom_level.read().as_ref() == \"day\" { \"white\" } else { \"#333\" }};",
                        "Day"
                    }
                    
                    button {
                        onclick: move |_| zoom_level.set("week"),
                        style: "padding: 6px 12px; border: none; border-radius: 4px; cursor: pointer;
                               background: {if zoom_level.read().as_ref() == \"week\" { \"#2196F3\" } else { \"transparent\" }};
                               color: {if zoom_level.read().as_ref() == \"week\" { \"white\" } else { \"#333\" }};",
                        "Week"
                    }
                    
                    button {
                        onclick: move |_| zoom_level.set("month"),
                        style: "padding: 6px 12px; border: none; border-radius: 4px; cursor: pointer;
                               background: {if zoom_level.read().as_ref() == \"month\" { \"#2196F3\" } else { \"transparent\" }};
                               color: {if zoom_level.read().as_ref() == \"month\" { \"white\" } else { \"#333\" }};",
                        "Month"
                    }
                }
                
                // Legend
                div {
                    style: "margin-left: auto; display: flex; gap: 15px; align-items: center;",
                    
                    div {
                        style: "display: flex; gap: 5px; align-items: center;",
                        div { style: "width: 20px; height: 10px; background: #808080;", }
                        span { style: "font-size: 12px;", "Todo" }
                    }
                    
                    div {
                        style: "display: flex; gap: 5px; align-items: center;",
                        div { style: "width: 20px; height: 10px; background: #2196F3;", }
                        span { style: "font-size: 12px;", "In Progress" }
                    }
                    
                    div {
                        style: "display: flex; gap: 5px; align-items: center;",
                        div { style: "width: 20px; height: 10px; background: #4CAF50;", }
                        span { style: "font-size: 12px;", "Done" }
                    }
                }
            }
            
            // Gantt chart
            div {
                style: "background: white; border-radius: 8px; overflow: auto; height: calc(100vh - 140px);",
                
                div {
                    style: "display: flex; min-width: 1200px;",
                    
                    // Task list (left side)
                    div {
                        style: "flex: 0 0 250px; border-right: 2px solid #ddd;",
                        
                        // Header
                        div {
                            style: "padding: 15px; background: #f5f5f5; border-bottom: 1px solid #ddd; font-weight: bold;",
                            "Tasks"
                        }
                        
                        // Task rows
                        for task in sorted_tasks.iter() {
                            div {
                                style: "padding: 10px 15px; border-bottom: 1px solid #eee; min-height: 50px;
                                       background: {if selected_task.read().as_ref() == Some(&task.id) { \"#e3f2fd\" } else { \"white\" }};
                                       cursor: pointer; display: flex; align-items: center;",
                                onclick: move |_| selected_task.set(Some(task.id)),
                                
                                div {
                                    style: "flex: 1;",
                                    
                                    div {
                                        style: "font-weight: 500; font-size: 14px; margin-bottom: 2px;",
                                        "{task.title}"
                                    }
                                    
                                    div {
                                        style: "font-size: 11px; color: #666;",
                                        "{task.status:?}"
                                    }
                                }
                                
                                // Priority indicator
                                div {
                                    style: "width: 6px; height: 6px; border-radius: 50%; 
                                           background: {match task.priority {
                                               Priority::Critical => \"#ff0000\",
                                               Priority::High => \"#ff8800\",
                                               Priority::Medium => \"#ffaa00\",
                                               Priority::Low => \"#888888\",
                                           }};",
                                }
                            }
                        }
                    }
                    
                    // Timeline (right side)
                    div {
                        style: "flex: 1; position: relative;",
                        
                        // Date headers
                        div {
                            style: "display: flex; background: #f5f5f5; border-bottom: 1px solid #ddd; position: sticky; top: 0; z-index: 10;",
                            
                            for (date, label) in date_columns.iter() {
                                div {
                                    style: "flex: 0 0 {get_column_width(&zoom_level.read())}px; padding: 10px 5px; 
                                           text-align: center; border-right: 1px solid #ddd; font-size: 12px;",
                                    
                                    div { style: "font-weight: bold;", "{label}" }
                                    
                                    if zoom_level.read().as_ref() != "month" {
                                        div { 
                                            style: "font-size: 10px; color: #666; margin-top: 2px;",
                                            "{date.format(\"%b\")}" 
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Task bars
                        for (index, task) in sorted_tasks.iter().enumerate() {
                            GanttBar {
                                task: task.clone(),
                                index: index,
                                start_date: start_date,
                                column_width: get_column_width(&zoom_level.read()),
                                zoom_level: zoom_level.read().clone(),
                                is_selected: selected_task.read().as_ref() == Some(&task.id),
                            }
                        }
                        
                        // Today line
                        {
                            let today_offset = calculate_offset(&start_date, &today, &zoom_level.read());
                            rsx! {
                                div {
                                    style: "position: absolute; left: {today_offset}px; top: 0; bottom: 0; 
                                           width: 2px; background: #ff0000; opacity: 0.5; z-index: 5;",
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
fn GanttBar(
    task: Task,
    index: usize,
    start_date: NaiveDate,
    column_width: usize,
    zoom_level: String,
    is_selected: bool,
) -> Element {
    let task_start = task.created_at.naive_local().date();
    let task_end = task.due_date
        .map(|d| d.naive_local().date())
        .unwrap_or(task_start + Duration::days(7));
    
    let start_offset = calculate_offset(&start_date, &task_start, &zoom_level);
    let duration_days = (task_end - task_start).num_days().max(1) as usize;
    let bar_width = calculate_bar_width(duration_days, &zoom_level, column_width);
    
    let status_color = match task.status {
        TaskStatus::Todo => "#808080",
        TaskStatus::InProgress => "#2196F3",
        TaskStatus::Done => "#4CAF50",
        TaskStatus::Blocked => "#f44336",
        _ => "#666",
    };
    
    let top_position = 51 + (index * 50); // Header height + row height * index
    
    rsx! {
        div {
            style: "position: absolute; left: {start_offset}px; top: {top_position}px; 
                   width: {bar_width}px; height: 30px;",
            
            div {
                style: "height: 100%; background: {status_color}; border-radius: 4px; 
                       opacity: {if is_selected { \"1\" } else { \"0.8\" }};
                       box-shadow: {if is_selected { \"0 2px 8px rgba(0,0,0,0.2)\" } else { \"0 1px 3px rgba(0,0,0,0.1)\" }};
                       display: flex; align-items: center; padding: 0 8px; cursor: pointer;
                       position: relative;",
                
                // Task title (if it fits)
                if bar_width > 60 {
                    div {
                        style: "color: white; font-size: 12px; font-weight: 500; 
                               white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{task.title}"
                    }
                }
                
                // Progress indicator for in-progress tasks
                if task.status == TaskStatus::InProgress {
                    let progress = calculate_progress(&task_start, &task_end);
                    div {
                        style: "position: absolute; left: 0; top: 0; bottom: 0; 
                               width: {progress}%; background: rgba(255,255,255,0.2); 
                               border-radius: 4px 0 0 4px;",
                    }
                }
                
                // Resize handles
                if is_selected {
                    div {
                        style: "position: absolute; left: 0; top: 0; bottom: 0; width: 5px; 
                               cursor: ew-resize; background: rgba(0,0,0,0.2);",
                    }
                    div {
                        style: "position: absolute; right: 0; top: 0; bottom: 0; width: 5px; 
                               cursor: ew-resize; background: rgba(0,0,0,0.2);",
                    }
                }
            }
            
            // Dependencies (if any)
            if !task.dependencies.is_empty() {
                div {
                    style: "position: absolute; left: -20px; top: 15px; width: 20px; height: 1px; 
                           background: #999;",
                }
                div {
                    style: "position: absolute; left: -20px; top: 10px; width: 10px; height: 10px; 
                           border-left: 1px solid #999; border-bottom: 1px solid #999;",
                }
            }
        }
    }
}

fn generate_date_columns(start: &NaiveDate, end: &NaiveDate, zoom: &str) -> Vec<(NaiveDate, String)> {
    let mut columns = vec![];
    let mut current = *start;
    
    while current <= *end {
        let label = match zoom {
            "day" => current.format("%d").to_string(),
            "week" => format!("W{}", current.iso_week().week()),
            "month" => current.format("%B").to_string(),
            _ => current.format("%d").to_string(),
        };
        
        columns.push((current, label));
        
        current = match zoom {
            "day" => current + Duration::days(1),
            "week" => current + Duration::days(7),
            "month" => {
                let next_month = if current.month() == 12 {
                    NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).unwrap()
                };
                next_month
            },
            _ => current + Duration::days(1),
        };
    }
    
    columns
}

fn get_column_width(zoom: &str) -> usize {
    match zoom {
        "day" => 40,
        "week" => 80,
        "month" => 120,
        _ => 40,
    }
}

fn calculate_offset(start: &NaiveDate, target: &NaiveDate, zoom: &str) -> usize {
    let days_diff = (*target - *start).num_days() as usize;
    
    match zoom {
        "day" => days_diff * 40,
        "week" => (days_diff / 7) * 80 + ((days_diff % 7) * 80 / 7),
        "month" => {
            let months_diff = ((target.year() - start.year()) * 12 + 
                             (target.month() as i32 - start.month() as i32)) as usize;
            months_diff * 120 + (target.day() as usize * 120 / 30)
        },
        _ => days_diff * 40,
    }
}

fn calculate_bar_width(duration_days: usize, zoom: &str, column_width: usize) -> usize {
    match zoom {
        "day" => duration_days * column_width,
        "week" => ((duration_days as f32 / 7.0) * column_width as f32) as usize,
        "month" => ((duration_days as f32 / 30.0) * column_width as f32) as usize,
        _ => duration_days * column_width,
    }.max(20) // Minimum width
}

fn calculate_progress(start: &NaiveDate, end: &NaiveDate) -> usize {
    let today = Utc::now().naive_local().date();
    if today < *start {
        0
    } else if today > *end {
        100
    } else {
        let total_days = (*end - *start).num_days();
        let elapsed_days = (today - *start).num_days();
        ((elapsed_days as f32 / total_days as f32) * 100.0) as usize
    }
}