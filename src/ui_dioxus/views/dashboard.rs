use dioxus::prelude::*;
use crate::ui_dioxus::state_simple::sample_tasks;

#[component]
pub fn Dashboard() -> Element {
    let tasks = use_signal(|| sample_tasks());
    let task_count = tasks.read().len();
    
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
                        div { style: "font-size: 28px; font-weight: bold; color: #333;", "{task_count}" }
                    }
                }
                
                // In Progress
                div {
                    style: "background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); display: flex; align-items: center; gap: 15px;",
                    
                    div { style: "font-size: 40px; color: #FF9800;", "⏳" }
                    div {
                        div { style: "font-size: 14px; color: #666; margin-bottom: 5px;", "In Progress" }
                        div { style: "font-size: 28px; font-weight: bold; color: #333;", "5" }
                    }
                }
                
                // Completed
                div {
                    style: "background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); display: flex; align-items: center; gap: 15px;",
                    
                    div { style: "font-size: 40px; color: #4CAF50;", "✅" }
                    div {
                        div { style: "font-size: 14px; color: #666; margin-bottom: 5px;", "Completed" }
                        div { style: "font-size: 28px; font-weight: bold; color: #333;", "3" }
                    }
                }
                
                // Blocked
                div {
                    style: "background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); display: flex; align-items: center; gap: 15px;",
                    
                    div { style: "font-size: 40px; color: #f44336;", "🚫" }
                    div {
                        div { style: "font-size: 14px; color: #666; margin-bottom: 5px;", "Blocked" }
                        div { style: "font-size: 28px; font-weight: bold; color: #333;", "1" }
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
                            span { style: "font-size: 14px; color: #666;", "4" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: "height: 100%; background: #808080; width: 30%;",
                            }
                        }
                    }
                    
                    // In Progress
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "In Progress" }
                            span { style: "font-size: 14px; color: #666;", "5" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: "height: 100%; background: #2196F3; width: 40%;",
                            }
                        }
                    }
                    
                    // Done
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "Done" }
                            span { style: "font-size: 14px; color: #666;", "3" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: "height: 100%; background: #4CAF50; width: 25%;",
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
                            span { style: "font-size: 14px; color: #666;", "3" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: "height: 100%; background: #ff8800; width: 25%;",
                            }
                        }
                    }
                    
                    // Medium
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "Medium" }
                            span { style: "font-size: 14px; color: #666;", "6" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: "height: 100%; background: #ffaa00; width: 50%;",
                            }
                        }
                    }
                    
                    // Low
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; justify-content: space-between; margin-bottom: 5px;",
                            span { style: "font-size: 14px;", "Low" }
                            span { style: "font-size: 14px; color: #666;", "3" }
                        }
                        div {
                            style: "height: 8px; background: #f0f0f0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: "height: 100%; background: #888888; width: 25%;",
                            }
                        }
                    }
                }
            }
            
            // Recent tasks
            div {
                style: "background: white; border-radius: 8px; padding: 20px; margin-top: 20px;",
                
                h3 { style: "margin: 0 0 20px 0;", "Recent Tasks" }
                
                for (i, task) in tasks.read().iter().enumerate() {
                    if i < 5 {
                        div {
                            style: "padding: 10px; border-bottom: 1px solid #eee; display: flex; justify-content: space-between; align-items: center;",
                            
                            div {
                                span {
                                    style: "font-weight: 500;",
                                    "{task.title}"
                                }
                            }
                            
                            span {
                                style: "padding: 2px 8px; border-radius: 4px; font-size: 12px; background: #2196F3; color: white;",
                                "Active"
                            }
                        }
                    }
                }
            }
        }
    }
}