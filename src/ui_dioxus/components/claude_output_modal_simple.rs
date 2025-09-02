use dioxus::prelude::*;
use crate::domain::claude_code::{ClaudeCodeSession, SessionStatus};
use crate::repository::Repository;
use std::sync::Arc;
use uuid::Uuid;

#[component]
pub fn ClaudeOutputModal(
    task_id: Uuid,
    on_close: EventHandler<()>,
) -> Element {
    let repository = use_context::<Arc<Repository>>();
    let mut sessions = use_signal(|| Vec::<ClaudeCodeSession>::new());
    let mut loading = use_signal(|| true);
    
    // Load sessions
    let _ = use_resource(move || {
        let repository = repository.clone();
        async move {
            loading.set(true);
            match repository.claude_code.get_sessions_by_task(task_id).await {
                Ok(session_list) => {
                    sessions.set(session_list);
                }
                Err(_) => {
                    sessions.set(Vec::new());
                }
            }
            loading.set(false);
        }
    });
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
                   background: rgba(0, 0, 0, 0.5); z-index: 999;
                   display: flex; align-items: center; justify-content: center;",
            onclick: move |_| on_close.call(()),
            
            div {
                style: "background: white; border-radius: 12px; padding: 24px;
                       width: 90%; max-width: 800px; max-height: 80vh; overflow-y: auto;
                       box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);",
                onclick: move |e| e.stop_propagation(),
                
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                    h2 { style: "margin: 0;", "⚡ Claude Code Output" }
                    button {
                        style: "background: none; border: none; font-size: 24px; cursor: pointer;",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                
                if *loading.read() {
                    div { "Loading sessions..." }
                } else if sessions.read().is_empty() {
                    div { "No Claude Code sessions found for this task." }
                } else {
                    div {
                        style: "font-family: monospace; white-space: pre-wrap; background: #f5f5f5; 
                               padding: 15px; border-radius: 8px; max-height: 500px; overflow-y: auto;",
                        
                        for session in sessions.read().iter() {
                            div {
                                style: "margin-bottom: 20px; padding-bottom: 20px; border-bottom: 1px solid #ddd;",
                                
                                div {
                                    style: "font-weight: bold; margin-bottom: 10px;",
                                    "Session: {session.id}"
                                }
                                
                                div {
                                    style: "margin-bottom: 10px;",
                                    "Status: {session.status:?}"
                                }
                                
                                if !session.session_log.is_empty() {
                                    div {
                                        style: "background: #1e1e1e; color: #d4d4d4; padding: 10px; 
                                               border-radius: 4px; font-size: 12px;",
                                        "{session.session_log}"
                                    }
                                } else {
                                    div { style: "color: #999;", "No output yet..." }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}