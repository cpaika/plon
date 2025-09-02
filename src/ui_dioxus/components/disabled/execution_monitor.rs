use dioxus::prelude::*;
use uuid::Uuid;
use crate::domain::task_execution::{TaskExecution, ExecutionStatus};
use crate::repository::Repository;
use std::time::Duration;

// TODO: Fix these components - they need PartialEq on Repository which is complex
// For now, monitoring is handled directly in the TaskCard buttons

/*
#[component]
pub fn ExecutionMonitor(
    repository: Option<Repository>,
    task_id: Uuid,
) -> Element {
    let mut execution = use_signal(|| None::<TaskExecution>);
    let mut last_update = use_signal(|| std::time::Instant::now());
    
    // Poll for execution updates every 5 seconds
    use_effect(move || {
        let repo_for_poll = repository.clone();
        spawn(async move {
            loop {
                if let Some(repo) = repo_for_poll.as_ref() {
                    if let Ok(Some(exec)) = repo.task_executions.get_active_for_task(task_id).await {
                        execution.set(Some(exec));
                        last_update.set(std::time::Instant::now());
                    }
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
    });
    
    match execution.read().as_ref() {
        Some(exec) => {
            let status_color = match exec.status {
                ExecutionStatus::Running => "#2196F3",
                ExecutionStatus::Success => "#4CAF50",
                ExecutionStatus::Failed => "#F44336",
                ExecutionStatus::Cancelled => "#FF9800",
                ExecutionStatus::PendingReview => "#9C27B0",
                ExecutionStatus::Merged => "#00BCD4",
            };
            
            let status_icon = match exec.status {
                ExecutionStatus::Running => "🔄",
                ExecutionStatus::Success => "✅",
                ExecutionStatus::Failed => "❌",
                ExecutionStatus::Cancelled => "⚠️",
                ExecutionStatus::PendingReview => "👀",
                ExecutionStatus::Merged => "🎉",
            };
            
            let duration = exec.duration()
                .map(|d| format!("{}m", d.num_minutes()))
                .unwrap_or_else(|| {
                    let elapsed = std::time::Instant::now() - *last_update.read();
                    format!("~{}m", elapsed.as_secs() / 60)
                });
            
            rsx! {
                div {
                    style: "position: absolute; bottom: 2px; left: 2px; right: 2px; height: 16px;
                           background: {status_color}15; border-top: 1px solid {status_color};
                           display: flex; align-items: center; justify-content: space-between;
                           padding: 0 4px; font-size: 9px; color: {status_color};",
                    
                    span {
                        style: "display: flex; align-items: center; gap: 2px;",
                        "{status_icon}"
                        span { 
                            style: "font-weight: 500;",
                            "{exec.status:?}" 
                        }
                    }
                    
                    span {
                        style: "opacity: 0.8;",
                        "{duration}"
                    }
                    
                    if let Some(pr_url) = &exec.pr_url {
                        a {
                            href: "{pr_url}",
                            target: "_blank",
                            style: "color: {status_color}; text-decoration: none; font-weight: bold;",
                            onclick: move |evt| {
                                evt.stop_propagation();
                            },
                            "PR →"
                        }
                    }
                }
            }
        }
        None => rsx! { }
    }
}

#[component]
pub fn ExecutionHistoryPanel(
    repository: Option<Repository>,
    task_id: Uuid,
    onclose: EventHandler<()>,
) -> Element {
    let mut executions = use_signal(|| Vec::<TaskExecution>::new());
    
    // Load execution history
    use_effect(move || {
        let repo_for_load = repository.clone();
        spawn(async move {
            if let Some(repo) = repo_for_load.as_ref() {
                if let Ok(execs) = repo.task_executions.list_for_task(task_id).await {
                    executions.set(execs);
                }
            }
        });
    });
    
    rsx! {
        div {
            style: "position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
                   width: 600px; max-height: 500px; background: white; border-radius: 12px;
                   box-shadow: 0 10px 40px rgba(0,0,0,0.2); z-index: 1000; overflow: hidden;
                   display: flex; flex-direction: column;",
            
            // Header
            div {
                style: "padding: 16px; border-bottom: 1px solid #e0e0e0; display: flex;
                       justify-content: space-between; align-items: center;",
                
                h3 {
                    style: "margin: 0; font-size: 18px; font-weight: 600;",
                    "Execution History"
                }
                
                button {
                    onclick: move |_| onclose.call(()),
                    style: "background: none; border: none; font-size: 20px; cursor: pointer;
                           color: #666; padding: 0; width: 24px; height: 24px;",
                    "×"
                }
            }
            
            // Content
            div {
                style: "flex: 1; overflow-y: auto; padding: 16px;",
                
                if executions.read().is_empty() {
                    div {
                        style: "text-align: center; color: #999; padding: 40px;",
                        "No executions yet"
                    }
                } else {
                    for exec in executions.read().iter() {
                        ExecutionHistoryItem {
                            execution: exec.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ExecutionHistoryItem(execution: TaskExecution) -> Element {
    let status_color = match execution.status {
        ExecutionStatus::Running => "#2196F3",
        ExecutionStatus::Success => "#4CAF50",
        ExecutionStatus::Failed => "#F44336",
        ExecutionStatus::Cancelled => "#FF9800",
        ExecutionStatus::PendingReview => "#9C27B0",
        ExecutionStatus::Merged => "#00BCD4",
    };
    
    let status_icon = match execution.status {
        ExecutionStatus::Running => "🔄",
        ExecutionStatus::Success => "✅",
        ExecutionStatus::Failed => "❌",
        ExecutionStatus::Cancelled => "⚠️",
        ExecutionStatus::PendingReview => "👀",
        ExecutionStatus::Merged => "🎉",
    };
    
    let card_style = format!(
        "background: #f8f8f8; border-radius: 8px; padding: 12px; margin-bottom: 8px; \
         border-left: 4px solid {};",
        status_color
    );
    
    let link_style = format!(
        "color: {}; text-decoration: none; margin-top: 4px; display: inline-block;",
        status_color
    );
    
    rsx! {
        div {
            style: "{card_style}",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: center;
                       margin-bottom: 8px;",
                
                span {
                    style: "display: flex; align-items: center; gap: 8px; font-weight: 500;",
                    span { "{status_icon}" }
                    span { "{execution.status:?}" }
                }
                
                span {
                    style: "font-size: 12px; color: #666;",
                    "{execution.started_at.format(\"%Y-%m-%d %H:%M\").to_string()}"
                }
            }
            
            div {
                style: "font-size: 12px; color: #666;",
                
                div { "Branch: {execution.branch_name}" }
                
                if let Some(duration) = execution.duration() {
                    div { "Duration: {duration.num_minutes()} minutes" }
                }
                
                if let Some(pr_url) = &execution.pr_url {
                    a {
                        href: "{pr_url}",
                        target: "_blank",
                        style: "{link_style}",
                        "View Pull Request →"
                    }
                }
                
                if let Some(error) = &execution.error_message {
                    div {
                        style: "color: #F44336; margin-top: 8px; padding: 8px; background: #ffebee;
                               border-radius: 4px; font-family: monospace; font-size: 11px;",
                        "{error}"
                    }
                }
            }
        }
    }
}
*/