use dioxus::prelude::*;
use uuid::Uuid;
use crate::domain::task_execution::{TaskExecution, ExecutionStatus};
use crate::services::ClaudeConsole;
use std::env::current_dir;

#[component]
pub fn ExecutionDetailsModal(
    task_id: Uuid,
    task_title: String,
    onclose: EventHandler<()>,
) -> Element {
    let mut execution = use_signal(|| None::<TaskExecution>);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut refresh_counter = use_signal(|| 0);
    
    // Load execution details
    use_effect(move || {
        spawn(async move {
            use crate::repository::{Repository, database::init_database};
            
            let current = current_dir().unwrap_or_default();
            let db_path = current.join("plon.db");
            
            // Force refresh when counter changes
            let _ = refresh_counter.read();
            
            match init_database(db_path.to_str().unwrap_or("plon.db")).await {
                Ok(pool) => {
                    let repo = Repository::new(pool);
                    match ClaudeConsole::get_active_execution(&repo, task_id).await {
                        Ok(exec) => {
                            execution.set(exec);
                            loading.set(false);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to load execution: {}", e)));
                            loading.set(false);
                        }
                    }
                }
                Err(e) => {
                    error.set(Some(format!("Failed to connect to database: {}", e)));
                    loading.set(false);
                }
            }
        });
    });
    
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
                    
                    {
                        if *loading.read() {
                            rsx! { LoadingContent {} }
                        } else if let Some(err) = error.read().as_ref() {
                            rsx! { ErrorContent { error: err.clone() } }
                        } else if let Some(exec) = execution.read().as_ref() {
                            rsx! { 
                                ExecutionContent { 
                                    execution: exec.clone(),
                                    on_refresh: move |_| {
                                        loading.set(true);
                                        refresh_counter.with_mut(|c| *c += 1);
                                    }
                                }
                            }
                        } else {
                            rsx! { NoExecutionContent {} }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LoadingContent() -> Element {
    rsx! {
        div {
            style: "text-align: center; padding: 40px; color: #666;",
            "Loading execution details..."
        }
    }
}

#[component]
fn ErrorContent(error: String) -> Element {
    rsx! {
        div {
            style: "background: #ffebee; color: #c62828; padding: 16px; 
                   border-radius: 8px;",
            "{error}"
        }
    }
}

#[component]
fn NoExecutionContent() -> Element {
    rsx! {
        div {
            style: "text-align: center; padding: 40px; color: #999;",
            "No active execution found for this task"
        }
    }
}

#[component]
fn ExecutionContent(
    execution: TaskExecution,
    on_refresh: EventHandler<()>
) -> Element {
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
    
    let exec_for_console = execution.clone();
    let exec_for_logs = execution.clone();
    
    rsx! {
        div {
            // Status Section
            StatusSection {
                execution: execution.clone(),
                status_color: status_color.to_string(),
                status_icon: status_icon.to_string(),
            }
            
            // Error Message Section
            {execution.error_message.as_ref().map(|error_msg| rsx! {
                ErrorSection { error_message: error_msg.clone() }
            })}
            
            // Logs Section
            {(!execution.output_log.is_empty()).then(|| rsx! {
                LogsSection { logs: execution.output_log.clone() }
            })}
            
            // Actions Section
            ActionsSection {
                is_running: execution.status == ExecutionStatus::Running,
                exec_for_console: exec_for_console,
                exec_for_logs: exec_for_logs,
                on_refresh: on_refresh,
            }
        }
    }
}

#[component]
fn StatusSection(
    execution: TaskExecution,
    status_color: String,
    status_icon: String,
) -> Element {
    let duration_text = execution.duration()
        .map(|d| format!("Duration: {} minutes", d.num_minutes()))
        .unwrap_or_default();
    
    let completed_text = execution.completed_at
        .map(|c| c.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default();
    
    let started_text = execution.started_at.format("%Y-%m-%d %H:%M:%S").to_string();
    
    rsx! {
        div {
            style: "background: #f5f5f5; padding: 16px; border-radius: 8px; margin-bottom: 20px;",
            
            div {
                style: "display: flex; align-items: center; gap: 12px; margin-bottom: 12px;",
                
                span {
                    style: "font-size: 24px;",
                    "{status_icon}"
                }
                
                span {
                    style: "font-size: 18px; font-weight: 500; color: {status_color};",
                    "{execution.status:?}"
                }
                
                {(!duration_text.is_empty()).then(|| rsx! {
                    span {
                        style: "margin-left: auto; color: #666; font-size: 14px;",
                        "{duration_text}"
                    }
                })}
            }
            
            div {
                style: "display: grid; grid-template-columns: auto 1fr; gap: 8px; font-size: 14px;",
                
                span { style: "color: #666;", "Branch:" }
                span { style: "font-family: monospace;", "{execution.branch_name}" }
                
                span { style: "color: #666;", "Started:" }
                span { "{started_text}" }
                
                {(!completed_text.is_empty()).then(|| rsx! {
                    Fragment {
                        span { style: "color: #666;", "Completed:" }
                        span { "{completed_text}" }
                    }
                })}
                
                {execution.pr_url.as_ref().map(|pr_url| rsx! {
                    Fragment {
                        span { style: "color: #666;", "Pull Request:" }
                        a {
                            href: "{pr_url}",
                            target: "_blank",
                            style: "color: {status_color}; text-decoration: none;",
                            "View PR →"
                        }
                    }
                })}
            }
        }
    }
}

#[component]
fn ErrorSection(error_message: String) -> Element {
    rsx! {
        div {
            style: "background: #ffebee; color: #c62828; padding: 12px; 
                   border-radius: 8px; margin-bottom: 20px; font-family: monospace; 
                   font-size: 12px;",
            "{error_message}"
        }
    }
}

#[component]
fn LogsSection(logs: Vec<String>) -> Element {
    rsx! {
        div {
            h3 {
                style: "margin: 0 0 12px 0; font-size: 16px; font-weight: 600;",
                "Execution Logs"
            }
            
            div {
                style: "background: #1e1e1e; color: #d4d4d4; padding: 16px; 
                       border-radius: 8px; font-family: monospace; font-size: 12px; 
                       max-height: 300px; overflow-y: auto;",
                
                for (i, log) in logs.iter().enumerate() {
                    div {
                        key: "{i}",
                        style: "margin-bottom: 4px;",
                        "{log}"
                    }
                }
            }
        }
    }
}

#[component]
fn ActionsSection(
    is_running: bool,
    exec_for_console: TaskExecution,
    exec_for_logs: TaskExecution,
    on_refresh: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "margin-top: 20px; display: flex; gap: 12px;",
            
            {is_running.then(|| rsx! {
                RunningActions {
                    exec_for_console: exec_for_console,
                    exec_for_logs: exec_for_logs,
                }
            })}
            
            button {
                onclick: move |_| on_refresh.call(()),
                style: "padding: 8px 16px; background: #f5f5f5; color: #333; 
                       border: 1px solid #ddd; border-radius: 6px; cursor: pointer;",
                "🔄 Refresh"
            }
        }
    }
}

#[component]
fn RunningActions(
    exec_for_console: TaskExecution,
    exec_for_logs: TaskExecution,
) -> Element {
    rsx! {
        Fragment {
            button {
                onclick: move |_| {
                    let exec = exec_for_console.clone();
                    spawn(async move {
                        let workspace_dir = current_dir().unwrap_or_default();
                        if let Err(e) = ClaudeConsole::open_console(&workspace_dir, &exec) {
                            eprintln!("Failed to open console: {}", e);
                        }
                    });
                },
                style: "padding: 8px 16px; background: #2196F3; color: white; 
                       border: none; border-radius: 6px; cursor: pointer;",
                "Open Console"
            }
            
            button {
                onclick: move |_| {
                    let exec = exec_for_logs.clone();
                    spawn(async move {
                        let workspace_dir = current_dir().unwrap_or_default();
                        if let Err(e) = ClaudeConsole::open_logs_terminal(&workspace_dir, &exec) {
                            eprintln!("Failed to open logs: {}", e);
                        }
                    });
                },
                style: "padding: 8px 16px; background: #9C27B0; color: white; 
                       border: none; border-radius: 6px; cursor: pointer;",
                "View Logs in Terminal"
            }
        }
    }
}