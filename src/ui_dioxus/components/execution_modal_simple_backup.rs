use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn ExecutionDetailsModal(
    task_id: Uuid,
    task_title: String,
    onclose: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
                   background: rgba(0,0,0,0.5); z-index: 999;",
            onclick: move |_| onclose.call(()),
            
            div {
                style: "position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
                       width: 700px; max-height: 80vh; background: white; border-radius: 12px;
                       box-shadow: 0 10px 40px rgba(0,0,0,0.2); z-index: 1000; 
                       display: flex; flex-direction: column;",
                onclick: move |evt| evt.stop_propagation(),
                
                // Header
                div {
                    style: "padding: 20px; border-bottom: 1px solid #e0e0e0; 
                           display: flex; justify-content: space-between; align-items: center;",
                    
                    h2 {
                        style: "margin: 0; font-size: 20px; font-weight: 600;",
                        "Execution Details: {task_title}"
                    }
                    
                    button {
                        onclick: move |_| onclose.call(()),
                        style: "background: none; border: none; font-size: 24px; cursor: pointer;
                               color: #666; padding: 0; width: 30px; height: 30px;",
                        "×"
                    }
                }
                
                // Content
                div {
                    style: "flex: 1; overflow-y: auto; padding: 20px;",
                    
                    div {
                        style: "text-align: center; padding: 40px; color: #666;",
                        p {
                            style: "font-size: 18px; margin-bottom: 20px;",
                            "Task Execution Monitor"
                        }
                        p {
                            style: "font-size: 14px; color: #999;",
                            "Task ID: {task_id}"
                        }
                        p {
                            style: "font-size: 14px; color: #999; margin-top: 20px;",
                            "This modal will show execution details when properly implemented."
                        }
                        p {
                            style: "font-size: 14px; color: #999;",
                            "Features: Status, Branch, Logs, PR Link, Duration"
                        }
                        
                        div {
                            style: "margin-top: 30px;",
                            button {
                                onclick: move |_| {
                                    println!("Would open console for task: {}", task_id);
                                },
                                style: "padding: 8px 16px; background: #2196F3; color: white; 
                                       border: none; border-radius: 6px; cursor: pointer; margin: 0 5px;",
                                "Open Console (Demo)"
                            }
                            
                            button {
                                onclick: move |_| {
                                    println!("Would open logs for task: {}", task_id);
                                },
                                style: "padding: 8px 16px; background: #9C27B0; color: white; 
                                       border: none; border-radius: 6px; cursor: pointer; margin: 0 5px;",
                                "View Logs (Demo)"
                            }
                        }
                    }
                }
            }
        }
    }
}