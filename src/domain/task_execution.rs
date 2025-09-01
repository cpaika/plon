use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a single execution of a task by Claude Code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskExecution {
    pub id: Uuid,
    pub task_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ExecutionStatus,
    pub branch_name: String,
    pub pr_url: Option<String>,
    pub error_message: Option<String>,
    pub output_log: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    /// Claude Code is currently working on the task
    Running,
    /// Task completed successfully
    Success,
    /// Task failed with an error
    Failed,
    /// User cancelled the execution
    Cancelled,
    /// Waiting for PR review
    PendingReview,
    /// PR was merged
    Merged,
}

impl TaskExecution {
    pub fn new(task_id: Uuid, branch_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            started_at: Utc::now(),
            completed_at: None,
            status: ExecutionStatus::Running,
            branch_name,
            pr_url: None,
            error_message: None,
            output_log: Vec::new(),
        }
    }
    
    pub fn complete_success(&mut self, pr_url: Option<String>) {
        self.completed_at = Some(Utc::now());
        self.status = ExecutionStatus::Success;
        self.pr_url = pr_url;
    }
    
    pub fn complete_failure(&mut self, error: String) {
        self.completed_at = Some(Utc::now());
        self.status = ExecutionStatus::Failed;
        self.error_message = Some(error);
    }
    
    pub fn cancel(&mut self) {
        self.completed_at = Some(Utc::now());
        self.status = ExecutionStatus::Cancelled;
    }
    
    pub fn add_log(&mut self, message: String) {
        self.output_log.push(format!("[{}] {}", Utc::now().format("%H:%M:%S"), message));
    }
    
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.completed_at.map(|end| end - self.started_at)
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.status, ExecutionStatus::Running | ExecutionStatus::PendingReview)
    }
}