use dioxus::prelude::*;
use crate::domain::task::{Task, TaskStatus, Priority};
use crate::ui_dioxus::state_simple::sample_tasks;
use uuid::Uuid;
use chrono::{Utc, Datelike, Duration, NaiveDate};

#[component]
pub fn TimelineView() -> Element {
    let mut tasks = use_signal(|| sample_tasks());
    let mut selected_date = use_signal(|| Utc::now().naive_local().date());
    let mut selected_task = use_signal(|| None::<Uuid>);
    let mut view_mode = use_signal(|| "week"); // "day", "week", "month"
    
    // Get current date info
    let today = Utc::now().naive_local().date();
    let current_month = selected_date.read().month();
    let current_year = selected_date.read().year();
    
    // Calculate week dates
    let weekday = selected_date.read().weekday();
    let days_from_monday = weekday.num_days_from_monday();
    let week_start = *selected_date.read() - Duration::days(days_from_monday as i64);
    
    // Get tasks for the current view
    let visible_tasks: Vec<Task> = tasks.read().iter()
        .filter(|t| {
            if let Some(due) = t.due_date {
                let task_date = due.naive_local().date();
                match view_mode.read().as_ref() {
                    "day" => task_date == *selected_date.read(),
                    "week" => {
                        task_date >= week_start && task_date < week_start + Duration::days(7)
                    },
                    "month" => {
                        task_date.month() == current_month && task_date.year() == current_year
                    },
                    _ => false
                }
            } else {
                false
            }
        })
        .cloned()
        .collect();
    
    rsx! {
        div {
            style: "padding: 20px; height: 100vh; background: #f5f5f5;",
            
            h2 { "Timeline View" }
            
            // Controls
            div {
                style: "margin-bottom: 20px; padding: 15px; background: white; border-radius: 8px; display: flex; gap: 10px; align-items: center;",
                
                // View mode selector
                div {
                    style: "display: flex; gap: 5px; background: #f0f0f0; padding: 4px; border-radius: 6px;",
                    
                    button {
                        onclick: move |_| view_mode.set("day"),
                        style: "padding: 6px 12px; border: none; border-radius: 4px; cursor: pointer;
                               background: {if view_mode.read().as_ref() == \"day\" { \"#2196F3\" } else { \"transparent\" }};
                               color: {if view_mode.read().as_ref() == \"day\" { \"white\" } else { \"#333\" }};",
                        "Day"
                    }
                    
                    button {
                        onclick: move |_| view_mode.set("week"),
                        style: "padding: 6px 12px; border: none; border-radius: 4px; cursor: pointer;
                               background: {if view_mode.read().as_ref() == \"week\" { \"#2196F3\" } else { \"transparent\" }};
                               color: {if view_mode.read().as_ref() == \"week\" { \"white\" } else { \"#333\" }};",
                        "Week"
                    }
                    
                    button {
                        onclick: move |_| view_mode.set("month"),
                        style: "padding: 6px 12px; border: none; border-radius: 4px; cursor: pointer;
                               background: {if view_mode.read().as_ref() == \"month\" { \"#2196F3\" } else { \"transparent\" }};
                               color: {if view_mode.read().as_ref() == \"month\" { \"white\" } else { \"#333\" }};",
                        "Month"
                    }
                }
                
                // Navigation
                div {
                    style: "display: flex; gap: 10px; align-items: center; margin-left: 20px;",
                    
                    button {
                        onclick: move |_| {
                            let new_date = match view_mode.read().as_ref() {
                                "day" => *selected_date.read() - Duration::days(1),
                                "week" => *selected_date.read() - Duration::days(7),
                                "month" => {
                                    let year = if current_month == 1 { current_year - 1 } else { current_year };
                                    let month = if current_month == 1 { 12 } else { current_month - 1 };
                                    NaiveDate::from_ymd_opt(year, month, 1).unwrap()
                                },
                                _ => *selected_date.read()
                            };
                            selected_date.set(new_date);
                        },
                        style: "padding: 6px 12px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "◀"
                    }
                    
                    span {
                        style: "font-weight: bold; min-width: 150px; text-align: center;",
                        "{format_date_range(&selected_date.read(), &view_mode.read())}"
                    }
                    
                    button {
                        onclick: move |_| {
                            let new_date = match view_mode.read().as_ref() {
                                "day" => *selected_date.read() + Duration::days(1),
                                "week" => *selected_date.read() + Duration::days(7),
                                "month" => {
                                    let year = if current_month == 12 { current_year + 1 } else { current_year };
                                    let month = if current_month == 12 { 1 } else { current_month + 1 };
                                    NaiveDate::from_ymd_opt(year, month, 1).unwrap()
                                },
                                _ => *selected_date.read()
                            };
                            selected_date.set(new_date);
                        },
                        style: "padding: 6px 12px; background: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "▶"
                    }
                    
                    button {
                        onclick: move |_| selected_date.set(today),
                        style: "padding: 6px 12px; background: #FF9800; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "Today"
                    }
                }
                
                // Add task button
                button {
                    onclick: move |_| {
                        let mut new_task = Task::new("New Task".to_string(), String::new());
                        new_task.due_date = Some(Utc::now());
                        tasks.write().push(new_task);
                    },
                    style: "margin-left: auto; padding: 8px 16px; background: #9C27B0; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Add Task"
                }
            }
            
            // Timeline content
            div {
                style: "background: white; border-radius: 8px; padding: 20px; height: calc(100vh - 140px); overflow-y: auto;",
                
                match view_mode.read().as_ref() {
                    "day" => rsx! { DayView { date: *selected_date.read(), tasks: visible_tasks } },
                    "week" => rsx! { WeekView { week_start: week_start, tasks: visible_tasks, today: today } },
                    "month" => rsx! { MonthView { year: current_year, month: current_month, tasks: tasks.read().clone(), today: today } },
                    _ => rsx! { div { "Invalid view" } }
                }
            }
        }
    }
}

#[component]
fn DayView(date: NaiveDate, tasks: Vec<Task>) -> Element {
    let hours = (0..24).collect::<Vec<_>>();
    
    rsx! {
        div {
            style: "display: flex; flex-direction: column;",
            
            for hour in hours {
                div {
                    style: "display: flex; border-bottom: 1px solid #eee; min-height: 60px;",
                    
                    // Hour label
                    div {
                        style: "width: 80px; padding: 10px; color: #666; font-size: 14px;",
                        "{hour:02}:00"
                    }
                    
                    // Tasks for this hour
                    div {
                        style: "flex: 1; padding: 5px;",
                        
                        for task in tasks.iter().filter(|t| {
                            t.due_date.map_or(false, |due| due.hour() == hour as u32)
                        }) {
                            TaskTimelineCard { task: task.clone() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn WeekView(week_start: NaiveDate, tasks: Vec<Task>, today: NaiveDate) -> Element {
    let days = (0..7).map(|i| week_start + Duration::days(i)).collect::<Vec<_>>();
    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    
    rsx! {
        div {
            style: "display: flex; gap: 10px;",
            
            for (i, day) in days.iter().enumerate() {
                div {
                    style: "flex: 1; min-height: 400px;",
                    
                    // Day header
                    div {
                        style: "padding: 10px; background: {if *day == today { \"#e3f2fd\" } else { \"#f5f5f5\" }}; 
                               border-radius: 6px 6px 0 0; text-align: center;",
                        
                        div {
                            style: "font-weight: bold; color: {if *day == today { \"#2196F3\" } else { \"#333\" }};",
                            "{day_names[i]}"
                        }
                        
                        div {
                            style: "font-size: 20px; margin-top: 5px;",
                            "{day.day()}"
                        }
                    }
                    
                    // Tasks for this day
                    div {
                        style: "border: 1px solid #ddd; border-top: none; padding: 10px; min-height: 300px;",
                        
                        for task in tasks.iter().filter(|t| {
                            t.due_date.map_or(false, |due| due.naive_local().date() == *day)
                        }) {
                            TaskTimelineCard { task: task.clone() }
                        }
                        
                        if !tasks.iter().any(|t| t.due_date.map_or(false, |due| due.naive_local().date() == *day)) {
                            div {
                                style: "color: #999; text-align: center; padding: 20px;",
                                "No tasks"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MonthView(year: i32, month: u32, tasks: Vec<Task>, today: NaiveDate) -> Element {
    // Calculate first day of month and number of days
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let days_in_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    }.signed_duration_since(first_day).num_days() as u32;
    
    let first_weekday = first_day.weekday().num_days_from_monday();
    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    
    // Create calendar grid
    let mut weeks = vec![];
    let mut current_week = vec![];
    
    // Add empty cells for days before month starts
    for _ in 0..first_weekday {
        current_week.push(None);
    }
    
    // Add days of the month
    for day in 1..=days_in_month {
        current_week.push(Some(NaiveDate::from_ymd_opt(year, month, day).unwrap()));
        
        if current_week.len() == 7 {
            weeks.push(current_week.clone());
            current_week.clear();
        }
    }
    
    // Add remaining week if not empty
    if !current_week.is_empty() {
        while current_week.len() < 7 {
            current_week.push(None);
        }
        weeks.push(current_week);
    }
    
    rsx! {
        div {
            // Day headers
            div {
                style: "display: grid; grid-template-columns: repeat(7, 1fr); gap: 5px; margin-bottom: 10px;",
                
                for day_name in day_names {
                    div {
                        style: "padding: 10px; text-align: center; font-weight: bold; color: #666;",
                        "{day_name}"
                    }
                }
            }
            
            // Calendar grid
            div {
                style: "display: grid; grid-template-rows: repeat({}, 1fr); gap: 5px;",
                
                for week in weeks {
                    div {
                        style: "display: grid; grid-template-columns: repeat(7, 1fr); gap: 5px;",
                        
                        for day_opt in week {
                            if let Some(day) = day_opt {
                                div {
                                    style: "border: 1px solid #ddd; border-radius: 6px; padding: 8px; min-height: 100px;
                                           background: {if day == today { \"#e3f2fd\" } else { \"white\" }};",
                                    
                                    div {
                                        style: "font-weight: bold; margin-bottom: 5px; color: {if day == today { \"#2196F3\" } else { \"#333\" }};",
                                        "{day.day()}"
                                    }
                                    
                                    div {
                                        style: "font-size: 12px;",
                                        
                                        for task in tasks.iter().filter(|t| {
                                            t.due_date.map_or(false, |due| due.naive_local().date() == day)
                                        }).take(3) {
                                            div {
                                                style: "padding: 2px 4px; margin: 2px 0; background: {task_status_color(&task.status)}; 
                                                       color: white; border-radius: 3px; font-size: 11px; 
                                                       white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                                "{task.title}"
                                            }
                                        }
                                        
                                        {
                                            let task_count = tasks.iter().filter(|t| {
                                                t.due_date.map_or(false, |due| due.naive_local().date() == day)
                                            }).count();
                                            
                                            if task_count > 3 {
                                                rsx! {
                                                    div {
                                                        style: "color: #666; font-size: 11px; margin-top: 2px;",
                                                        "+{task_count - 3} more"
                                                    }
                                                }
                                            } else {
                                                rsx! { }
                                            }
                                        }
                                    }
                                }
                            } else {
                                div {
                                    style: "border: 1px solid #f0f0f0; border-radius: 6px; background: #fafafa;",
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
fn TaskTimelineCard(task: Task) -> Element {
    let status_color = task_status_color(&task.status);
    let priority_color = match task.priority {
        Priority::Critical => "#ff0000",
        Priority::High => "#ff8800",
        Priority::Medium => "#ffaa00",
        Priority::Low => "#888888",
    };
    
    rsx! {
        div {
            style: "padding: 8px; margin-bottom: 5px; background: #f8f8f8; border-radius: 6px;
                   border-left: 3px solid {status_color}; cursor: pointer;",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                
                div {
                    style: "flex: 1;",
                    
                    div {
                        style: "font-weight: bold; font-size: 13px; margin-bottom: 2px;",
                        "{task.title}"
                    }
                    
                    if let Some(due) = task.due_date {
                        div {
                            style: "font-size: 11px; color: #666;",
                            "⏰ {due.format(\"%H:%M\")}"
                        }
                    }
                }
                
                div {
                    style: "width: 6px; height: 6px; border-radius: 50%; background: {priority_color};",
                    title: "{task.priority:?} priority",
                }
            }
        }
    }
}

fn task_status_color(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "#808080",
        TaskStatus::InProgress => "#2196F3",
        TaskStatus::Review => "#FF9800",
        TaskStatus::Done => "#4CAF50",
        TaskStatus::Blocked => "#f44336",
        _ => "#666",
    }
}

fn format_date_range(date: &NaiveDate, mode: &str) -> String {
    match mode {
        "day" => date.format("%B %d, %Y").to_string(),
        "week" => {
            let weekday = date.weekday();
            let days_from_monday = weekday.num_days_from_monday();
            let week_start = *date - Duration::days(days_from_monday as i64);
            let week_end = week_start + Duration::days(6);
            format!("{} - {}", 
                week_start.format("%b %d"),
                week_end.format("%b %d, %Y"))
        },
        "month" => date.format("%B %Y").to_string(),
        _ => date.format("%Y-%m-%d").to_string(),
    }
}