use dioxus::prelude::*;
use crate::ui_dioxus::state_simple::sample_tasks;

#[component]
pub fn GanttView() -> Element {
    let tasks = use_signal(|| sample_tasks());
    let task_count = tasks.read().len();
    
    rsx! {
        div {
            style: "padding: 20px; height: 100vh; background: #f5f5f5;",
            
            h2 { "Gantt Chart" }
            p { "Total tasks: {task_count}" }
            
            // Legend
            div {
                style: "margin-bottom: 20px; padding: 15px; background: white; border-radius: 8px; display: flex; gap: 15px;",
                
                span {
                    style: "display: flex; gap: 5px; align-items: center;",
                    div { style: "width: 20px; height: 10px; background: #808080;", }
                    "Todo"
                }
                span {
                    style: "display: flex; gap: 5px; align-items: center;",
                    div { style: "width: 20px; height: 10px; background: #2196F3;", }
                    "In Progress"
                }
                span {
                    style: "display: flex; gap: 5px; align-items: center;",
                    div { style: "width: 20px; height: 10px; background: #4CAF50;", }
                    "Done"
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
                        for task in tasks.read().iter() {
                            div {
                                style: "padding: 10px 15px; border-bottom: 1px solid #eee; min-height: 50px; display: flex; align-items: center;",
                                
                                div {
                                    style: "flex: 1;",
                                    
                                    div {
                                        style: "font-weight: 500; font-size: 14px; margin-bottom: 2px;",
                                        "{task.title}"
                                    }
                                    
                                    div {
                                        style: "font-size: 11px; color: #666;",
                                        "Status: Active"
                                    }
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
                            
                            div { style: "flex: 0 0 100px; padding: 10px 5px; text-align: center; border-right: 1px solid #ddd; font-size: 12px;", "Week 1" }
                            div { style: "flex: 0 0 100px; padding: 10px 5px; text-align: center; border-right: 1px solid #ddd; font-size: 12px;", "Week 2" }
                            div { style: "flex: 0 0 100px; padding: 10px 5px; text-align: center; border-right: 1px solid #ddd; font-size: 12px;", "Week 3" }
                            div { style: "flex: 0 0 100px; padding: 10px 5px; text-align: center; border-right: 1px solid #ddd; font-size: 12px;", "Week 4" }
                            div { style: "flex: 0 0 100px; padding: 10px 5px; text-align: center; border-right: 1px solid #ddd; font-size: 12px;", "Week 5" }
                            div { style: "flex: 0 0 100px; padding: 10px 5px; text-align: center; border-right: 1px solid #ddd; font-size: 12px;", "Week 6" }
                        }
                        
                        // Task bars - simplified
                        for (i, task) in tasks.read().iter().enumerate() {
                            if i < 8 {
                                div {
                                    style: "position: absolute; height: 30px; background: #2196F3; border-radius: 4px; opacity: 0.8; display: flex; align-items: center; padding: 0 8px; left: {i * 60 + 20}px; top: {51 + i * 50}px; width: {80 + i * 15}px;",
                                    
                                    div {
                                        style: "color: white; font-size: 12px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                        "{task.title}"
                                    }
                                }
                            }
                        }
                        
                        // Today line
                        div {
                            style: "position: absolute; left: 200px; top: 0; bottom: 0; width: 2px; background: #ff0000; opacity: 0.5; z-index: 5;",
                        }
                    }
                }
            }
        }
    }
}