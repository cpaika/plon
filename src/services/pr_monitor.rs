use std::time::Duration;
use tokio::time::interval;
use crate::repository::Repository;
use crate::services::ClaudeAutomation;
use std::path::PathBuf;
use anyhow::Result;

pub struct PrMonitor {
    repository: Repository,
    automation: ClaudeAutomation,
    check_interval: Duration,
}

impl PrMonitor {
    pub fn new(repository: Repository, workspace_dir: PathBuf) -> Self {
        let automation = ClaudeAutomation::with_repository(workspace_dir, repository.clone());
        Self {
            repository,
            automation,
            check_interval: Duration::from_secs(60), // Check every minute
        }
    }
    
    /// Start monitoring PR status for active task executions
    pub async fn start_monitoring(&self) {
        let mut interval = interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_all_active_executions().await {
                eprintln!("Error checking PR status: {}", e);
            }
        }
    }
    
    /// Check all active task executions for PR status updates
    async fn check_all_active_executions(&self) -> Result<()> {
        use crate::domain::task_execution::ExecutionStatus;
        
        // Get all active executions (Running or PendingReview status)
        let running_status = serde_json::to_string(&ExecutionStatus::Running)?;
        let pending_status = serde_json::to_string(&ExecutionStatus::PendingReview)?;
        
        let active_executions = sqlx::query!(
            r#"
            SELECT id 
            FROM task_executions 
            WHERE status = ? OR status = ?
            "#,
            running_status,
            pending_status
        )
        .fetch_all(&*self.repository.pool)
        .await?;
        
        for row in active_executions {
            // row.id is Option<String> from sqlx::query!
            if let Some(ref id_str) = row.id {
                if let Ok(id) = uuid::Uuid::parse_str(id_str) {
                    // Update each execution's status
                    if let Err(e) = self.automation.update_execution_status(id).await {
                        eprintln!("Failed to update execution {}: {}", id, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get recent PR activity for reporting
    pub async fn get_recent_pr_activity(&self, hours: i64) -> Result<Vec<PrActivity>> {
        use chrono::{Utc, Duration as ChronoDuration};
        use crate::domain::task_execution::ExecutionStatus;
        
        let since = Utc::now() - ChronoDuration::hours(hours);
        let pending_status = serde_json::to_string(&ExecutionStatus::PendingReview)?;
        
        let rows = sqlx::query!(
            r#"
            SELECT te.id, te.task_id, te.pr_url, te.status, te.completed_at, t.title
            FROM task_executions te
            JOIN tasks t ON te.task_id = t.id
            WHERE te.pr_url IS NOT NULL 
                AND (te.completed_at > ? OR te.status = ?)
            ORDER BY te.completed_at DESC
            "#,
            since,
            pending_status
        )
        .fetch_all(&*self.repository.pool)
        .await?;
        
        let mut activities = Vec::new();
        for row in rows {
            if let Some(ref id_str) = row.id {
                let status: ExecutionStatus = serde_json::from_str(&row.status)?;
                let execution_id = uuid::Uuid::parse_str(id_str)?;
                let task_id = uuid::Uuid::parse_str(&row.task_id)?;
                
                // Convert NaiveDateTime to DateTime<Utc> if present
                let completed_at = row.completed_at.map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc));
                
                activities.push(PrActivity {
                    execution_id,
                    task_id,
                    task_title: row.title,
                    pr_url: row.pr_url.unwrap_or_default(),
                    status,
                    completed_at,
                });
            }
        }
        
        Ok(activities)
    }
}

#[derive(Debug, Clone)]
pub struct PrActivity {
    pub execution_id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub task_title: String,
    pub pr_url: String,
    pub status: crate::domain::task_execution::ExecutionStatus,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Start the PR monitor in the background
pub async fn start_pr_monitor_background(repository: Repository, workspace_dir: PathBuf) {
    tokio::spawn(async move {
        let monitor = PrMonitor::new(repository, workspace_dir);
        monitor.start_monitoring().await;
    });
}