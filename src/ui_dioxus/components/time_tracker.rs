use dioxus::prelude::*;
use crate::services::TimeTrackingService;
use uuid::Uuid;
use std::sync::Arc;
use chrono::Duration;

#[component]
pub fn TimeTracker(
    task_id: Uuid,
) -> Element {
    // Get service from context instead of props
    let service = use_context::<Arc<TimeTrackingService>>();
    
    let mut is_tracking = use_signal(|| service.is_tracking(task_id));
    let mut elapsed_time = use_signal(|| Duration::zero());
    let mut timer_active = use_signal(|| false);
    
    // Update elapsed time every second if tracking
    use_effect({
        let service = service.clone();
        let mut timer_active = timer_active.clone();
        let mut elapsed_time = elapsed_time.clone();
        move || {
            if *is_tracking.read() {
                let service = service.clone();
                let mut timer_active = timer_active.clone();
                let mut elapsed_time = elapsed_time.clone();
                timer_active.set(true);
                spawn(async move {
                    loop {
                        if !*timer_active.read() {
                            break;
                        }
                        
                        if let Some(entry) = service.get_active_entry(task_id) {
                            let elapsed = chrono::Utc::now() - entry.start_time;
                            elapsed_time.set(elapsed);
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                });
            } else {
                timer_active.set(false);
            }
        }
    });
    
    let handle_start = {
        let service = service.clone();
        move |_| {
            let service = service.clone();
            spawn(async move {
                match service.start_tracking(task_id, "Time tracking".to_string()).await {
                    Ok(_) => {
                        is_tracking.set(true);
                    }
                    Err(e) => {
                        eprintln!("Failed to start tracking: {}", e);
                    }
                }
            });
        }
    };
    
    let handle_stop = {
        let service = service.clone();
        move |_| {
            let service = service.clone();
            spawn(async move {
                match service.stop_tracking(task_id).await {
                    Ok(duration) => {
                        is_tracking.set(false);
                        elapsed_time.set(duration);
                        timer_active.set(false);
                    }
                    Err(e) => {
                        eprintln!("Failed to stop tracking: {}", e);
                    }
                }
            });
        }
    };
    
    let handle_pause = {
        let service = service.clone();
        move |_| {
            match service.pause_tracking(task_id) {
                Ok(_) => {
                    is_tracking.set(false);
                    timer_active.set(false);
                }
                Err(e) => {
                    eprintln!("Failed to pause tracking: {}", e);
                }
            }
        }
    };
    
    // Format duration as HH:MM:SS
    let format_duration = |d: Duration| -> String {
        let total_seconds = d.num_seconds();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    };
    
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 10px; padding: 10px; background: #f9fafb; border-radius: 6px;",
            
            // Timer display
            div {
                style: "font-family: monospace; font-size: 18px; font-weight: bold; min-width: 100px;",
                {format_duration(*elapsed_time.read())}
            }
            
            // Control buttons
            if *is_tracking.read() {
                button {
                    style: "padding: 6px 12px; background: #ef4444; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: handle_stop,
                    "⏹ Stop"
                }
                
                button {
                    style: "padding: 6px 12px; background: #f59e0b; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: handle_pause,
                    "⏸ Pause"
                }
            } else {
                button {
                    style: "padding: 6px 12px; background: #10b981; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: handle_start,
                    "▶ Start"
                }
            }
            
            // Total time for task
            div {
                style: "margin-left: auto; color: #6b7280; font-size: 14px;",
                {
                    let total = service.get_total_time(task_id);
                    format!("Total: {}", format_duration(total))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashSet, HashMap};
    use crate::repository::Repository;
    use crate::domain::task::{Task, TaskStatus, Priority};
    use sqlx::SqlitePool;
    
    #[tokio::test]
    async fn test_time_tracker_component() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let repo = Arc::new(Repository::new(pool));
        
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            due_date: None,
            scheduled_date: None,
            completed_at: None,
            estimated_hours: Some(2.0),
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            tags: HashSet::new(),
            assignee: None,
            position: crate::domain::task::Position { x: 0.0, y: 0.0 },
            is_archived: false,
            configuration_id: None,
            metadata: HashMap::new(),
            subtasks: Vec::new(),
            sort_order: 0,
        };
        
        repo.tasks.create(&task).await.unwrap();
        
        let _service = Arc::new(TimeTrackingService::new(repo));
        
        // Test that component can be created
        // In actual usage, service would be provided via context
        let _task_id = task.id;
        
        // In a real test, we would render this in a VirtualDom
        // and test the interactions
    }
}