use dioxus::prelude::*;
use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use crate::domain::task::{Task, TaskStatus, Priority};
use std::sync::Arc;

#[component]
pub fn Dashboard() -> Element {
    let repository = use_context::<Arc<Repository>>();
    let mut tasks = use_signal(|| Vec::<Task>::new());
    let mut loading = use_signal(|| true);
    
    // Statistics
    let mut total_count = use_signal(|| 0);
    let mut todo_count = use_signal(|| 0);
    let mut in_progress_count = use_signal(|| 0);
    let mut done_count = use_signal(|| 0);
    let mut blocked_count = use_signal(|| 0);
    let mut review_count = use_signal(|| 0);
    
    // Priority counts
    let mut critical_count = use_signal(|| 0);
    let mut high_count = use_signal(|| 0);
    let mut medium_count = use_signal(|| 0);
    let mut low_count = use_signal(|| 0);
    
    // Load tasks and calculate statistics
    use_effect({
        let repo = repository.clone();
        move || {
            let repo = repo.clone();
            spawn(async move {
                loading.set(true);
                
                // Fetch all tasks
                let filters = TaskFilters {
                    status: None,
                    assigned_resource_id: None,
                    goal_id: None,
                    overdue: false,
                    limit: None,
                };
                
                match repo.tasks.list(filters).await {
                    Ok(task_list) => {
                        // Calculate statistics
                        let total = task_list.len();
                        let mut todo = 0;
                        let mut in_progress = 0;
                        let mut done = 0;
                        let mut blocked = 0;
                        let mut review = 0;
                        
                        let mut critical = 0;
                        let mut high = 0;
                        let mut medium = 0;
                        let mut low = 0;
                        
                        for task in &task_list {
                            match task.status {
                                TaskStatus::Todo => todo += 1,
                                TaskStatus::InProgress => in_progress += 1,
                                TaskStatus::Done => done += 1,
                                TaskStatus::Blocked => blocked += 1,
                                TaskStatus::Review => review += 1,
                                TaskStatus::Cancelled => {},
                            }
                            
                            match task.priority {
                                Priority::Critical => critical += 1,
                                Priority::High => high += 1,
                                Priority::Medium => medium += 1,
                                Priority::Low => low += 1,
                            }
                        }
                        
                        tasks.set(task_list);
                        total_count.set(total);
                        todo_count.set(todo);
                        in_progress_count.set(in_progress);
                        done_count.set(done);
                        blocked_count.set(blocked);
                        review_count.set(review);
                        
                        critical_count.set(critical);
                        high_count.set(high);
                        medium_count.set(medium);
                        low_count.set(low);
                    }
                    Err(e) => {
                        eprintln!("Failed to load tasks: {}", e);
                    }
                }
                
                loading.set(false);
            });
        }
    });
    
    rsx! {
        div {
            style: "padding: 20px; background: #f5f5f5; min-height: 100vh;",
            
            h1 { style: "margin-bottom: 30px;", "Dashboard" }
            
            // Main statistics
            div {
                style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 30px;",
                
                // Total Tasks
                div {
                    style: "background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); display: flex; align-items: center; gap: 15px;",
                    
                    div { style: "font-size: 40px; color: #2196F3;", "📋" }
                    div {
                        div { style: "font-size: 14px; color: #666; margin-bottom: 5px;", "Total Tasks" }
                        div { style: "font-size: 28px; font-weight: bold; color: #333;", "{total_count}" }
                    }
                }
                
                // In Progress
                div {
                    style: "background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); display: flex; align-items: center; gap: 15px;",
                    
                    div { style: "font-size: 40px; color: #FF9800;", "⏳" }
                    div {
                        div { style: "font-size: 14px; color: #666; margin-bottom: 5px;", "In Progress" }
                        div { style: "font-size: 28px; font-weight: bold; color: #333;", "{in_progress_count}" }
                    }
                }
                
                // Completed
                div {
                    style: "background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); display: flex; align-items: center; gap: 15px;",
                    
                    div { style: "font-size: 40px; color: #4CAF50;", "✅" }
                    div {
                        div { style: "font-size: 14px; color: #666; margin-bottom: 5px;", "Completed" }
                        div { style: "font-size: 28px; font-weight: bold; color: #333;", "{done_count}" }
                    }
                }
                
                // Blocked
                div {
                    style: "background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); display: flex; align-items: center; gap: 15px;",
                    
                    div { style: "font-size: 40px; color: #f44336;", "🚫" }
                    div {
                        div { style: "font-size: 14px; color: #666; margin-bottom: 5px;", "Blocked" }
                        div { style: "font-size: 28px; font-weight: bold; color: #333;", "{blocked_count}" }
                    }
                }
            }
            
            // Two column layout
            div {
                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                
                // Left column - Status breakdown
                div {
                    style: "background: white; border-radius: 8px; padding: 20px;",
                    
                    h3 { style: "margin: 0 0 20px 0;", "Task Status" }
                    
                    // Todo
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "Todo" }
                            span { style: "font-size: 14px; color: #666;", "{todo_count}" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: format!("height: 100%; background: #808080; width: {}%;", 
                                    if *total_count.read() > 0 { (*todo_count.read() * 100 / *total_count.read()) as u32 } else { 0 }),
                            }
                        }
                    }
                    
                    // In Progress
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "In Progress" }
                            span { style: "font-size: 14px; color: #666;", "{in_progress_count}" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: format!("height: 100%; background: #2196F3; width: {}%;",
                                    if *total_count.read() > 0 { (*in_progress_count.read() * 100 / *total_count.read()) as u32 } else { 0 }),
                            }
                        }
                    }
                    
                    // Done
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "Done" }
                            span { style: "font-size: 14px; color: #666;", "{done_count}" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: format!("height: 100%; background: #4CAF50; width: {}%;",
                                    if *total_count.read() > 0 { (*done_count.read() * 100 / *total_count.read()) as u32 } else { 0 }),
                            }
                        }
                    }
                }
                
                // Right column - Priority distribution
                div {
                    style: "background: white; border-radius: 8px; padding: 20px;",
                    
                    h3 { style: "margin: 0 0 20px 0;", "Priority Distribution" }
                    
                    // High
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "High" }
                            span { style: "font-size: 14px; color: #666;", "{high_count}" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: format!("height: 100%; background: #ff8800; width: {}%;",
                                    if *total_count.read() > 0 { (*high_count.read() * 100 / *total_count.read()) as u32 } else { 0 }),
                            }
                        }
                    }
                    
                    // Medium
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "Medium" }
                            span { style: "font-size: 14px; color: #666;", "{medium_count}" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: format!("height: 100%; background: #ffaa00; width: {}%;",
                                    if *total_count.read() > 0 { (*medium_count.read() * 100 / *total_count.read()) as u32 } else { 0 }),
                            }
                        }
                    }
                    
                    // Low
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "Low" }
                            span { style: "font-size: 14px; color: #666;", "{low_count}" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: format!("height: 100%; background: #888888; width: {}%;",
                                    if *total_count.read() > 0 { (*low_count.read() * 100 / *total_count.read()) as u32 } else { 0 }),
                            }
                        }
                    }
                }
            }
            
            // Recent tasks and upcoming due dates
            div {
                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-top: 20px;",
                
                // Recent tasks column
                div {
                    style: "background: white; border-radius: 8px; padding: 20px;",
                    
                    h3 { style: "margin: 0 0 20px 0; display: flex; align-items: center; gap: 8px;",
                        "🕐 Recent Tasks"
                    }
                    
                    div {
                        style: "text-align: center; padding: 20px; color: #9ca3af;",
                        "Recent tasks: {tasks().len()}"
                    }
                }
                
                // Upcoming due dates column
                div {
                    style: "background: white; border-radius: 8px; padding: 20px;",
                    
                    h3 { style: "margin: 0 0 20px 0; display: flex; align-items: center; gap: 8px;",
                        "📅 Upcoming Due Dates"
                    }
                    
                    div {
                        style: "text-align: center; padding: 40px 20px; color: #9ca3af;",
                        "Upcoming due dates"
                    }
                }
            }
        }
    }
}