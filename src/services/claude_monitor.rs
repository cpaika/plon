use anyhow::Result;
use std::process::Command;
use std::path::PathBuf;
use tokio::time::{interval, Duration};
use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use crate::domain::task::TaskStatus;
use uuid::Uuid;

/// Service to monitor Claude terminal status and update task status
/// when a PR is created
pub struct ClaudeMonitor {
    repository: Repository,
    workspace_dir: PathBuf,
}

impl ClaudeMonitor {
    pub fn new(repository: Repository, workspace_dir: PathBuf) -> Self {
        Self {
            repository,
            workspace_dir,
        }
    }
    
    /// Start monitoring Claude status for active tasks
    pub async fn start_monitoring(&self) {
        let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_and_update_task_statuses().await {
                eprintln!("Error checking Claude status: {}", e);
            }
        }
    }
    
    /// Check all InProgress tasks for PR creation and update to Review status
    async fn check_and_update_task_statuses(&self) -> Result<()> {
        // Get all tasks that are InProgress
        let filters = TaskFilters {
            status: Some(TaskStatus::InProgress),
            ..Default::default()
        };
        let in_progress_tasks = self.repository.tasks.list(filters).await?;
        
        for task in in_progress_tasks {
            // Check if a PR exists for this task using gh CLI
            if self.check_pr_exists(task.id).await? {
                // Update task status to Review
                let mut task = task;
                task.status = TaskStatus::Review;
                self.repository.tasks.update(&task).await?;
                println!("✅ Task '{}' status updated to Review (PR detected)", task.title);
            }
        }
        
        Ok(())
    }
    
    /// Check if a PR exists for a given task
    async fn check_pr_exists(&self, task_id: Uuid) -> Result<bool> {
        // Use the task ID prefix for branch/PR search pattern
        let task_id_str = task_id.to_string();
        let task_id_prefix = task_id_str.split('-').next().unwrap_or("unknown");
        
        // Check for PRs with gh CLI
        let output = Command::new("gh")
            .current_dir(&self.workspace_dir)
            .args(&[
                "pr", 
                "list", 
                "--search", 
                &format!("task/{}", task_id_prefix),
                "--json",
                "number",
            ])
            .output()?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // If the JSON output contains any PR data, a PR exists
            if !stdout.trim().is_empty() && stdout.trim() != "[]" {
                return Ok(true);
            }
        }
        
        // Also check for Claude-prefixed branches
        let output = Command::new("gh")
            .current_dir(&self.workspace_dir)
            .args(&[
                "pr",
                "list",
                "--search",
                &format!("claude/{}", task_id_prefix),
                "--json",
                "number",
            ])
            .output()?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() && stdout.trim() != "[]" {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Monitor a specific Claude Code execution and update task when PR is created
    pub async fn monitor_claude_execution(&self, task_id: Uuid) -> Result<()> {
        // This method can be called when Claude Code is launched
        // to specifically monitor that execution
        
        // Start a background task to monitor this specific task
        let repository = self.repository.clone();
        let workspace_dir = self.workspace_dir.clone();
        
        tokio::spawn(async move {
            let monitor = ClaudeMonitor::new(repository, workspace_dir);
            
            // Check every 10 seconds for this specific task
            let mut interval = interval(Duration::from_secs(10));
            let mut checks = 0;
            let max_checks = 60; // Check for up to 10 minutes
            
            while checks < max_checks {
                interval.tick().await;
                checks += 1;
                
                if let Ok(has_pr) = monitor.check_pr_exists(task_id).await {
                    if has_pr {
                        // Update task status to Review
                        if let Ok(Some(mut task)) = monitor.repository.tasks.get(task_id).await {
                            if task.status == TaskStatus::InProgress {
                                task.status = TaskStatus::Review;
                                let _ = monitor.repository.tasks.update(&task).await;
                                println!("✅ Task '{}' automatically marked as Review (PR created)", task.title);
                            }
                        }
                        break; // Stop monitoring once PR is created
                    }
                }
            }
        });
        
        Ok(())
    }
}

/// Start the Claude monitor in the background
pub async fn start_claude_monitor_background(repository: Repository, workspace_dir: PathBuf) {
    tokio::spawn(async move {
        let monitor = ClaudeMonitor::new(repository, workspace_dir);
        monitor.start_monitoring().await;
    });
}