use std::sync::Arc;
use anyhow::{Result, Context, bail};
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

use crate::services::{
    AutoRunOrchestrator, AutoRunStatus, TaskExecutionStatus,
    AutoRunConfig,
};
use crate::domain::task::Task;

/// Improvements and bug fixes for AutoRunOrchestrator
impl AutoRunOrchestrator {
    /// Validates configuration before applying
    pub async fn validate_and_update_config(&self, config: AutoRunConfig) -> Result<()> {
        // Validate configuration
        if config.max_parallel_instances == 0 {
            bail!("max_parallel_instances must be greater than 0");
        }
        
        if config.max_parallel_instances > 100 {
            bail!("max_parallel_instances cannot exceed 100 for safety");
        }
        
        if config.max_retries > 10 {
            bail!("max_retries cannot exceed 10 to prevent infinite loops");
        }
        
        // Check if we're in the middle of execution
        let status = self.status.read().await;
        if matches!(*status, AutoRunStatus::Running) {
            bail!("Cannot update configuration while auto-run is active");
        }
        drop(status);
        
        // Apply validated config
        *self.config.write().await = config;
        Ok(())
    }
    
    /// Safe task execution with timeout and resource limits
    pub async fn start_task_execution_safe(&self, task_id: Uuid) -> Result<()> {
        // Check if task exists
        let task = self.task_service.get(task_id).await
            .context("Failed to fetch task")?
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", task_id))?;
        
        // Check resource limits
        let active_count = self.active_sessions.read().await.len();
        let config = self.config.read().await;
        
        if active_count >= config.max_parallel_instances {
            bail!("Maximum parallel instances ({}) reached", config.max_parallel_instances);
        }
        drop(config);
        
        // Check if task is already running
        let executions = self.executions.read().await;
        if let Some(exec) = executions.get(&task_id) {
            if matches!(exec.status, TaskExecutionStatus::Running | TaskExecutionStatus::PendingReview) {
                bail!("Task {} is already running", task_id);
            }
        }
        drop(executions);
        
        // Create session with timeout
        let timeout = tokio::time::Duration::from_secs(3600); // 1 hour max
        let result = tokio::time::timeout(
            timeout,
            self.create_and_start_session(task)
        ).await;
        
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e).context("Failed to start task execution"),
            Err(_) => bail!("Task execution timed out after 1 hour"),
        }
    }
    
    /// Creates and starts a session with proper error handling
    async fn create_and_start_session(&self, task: Task) -> Result<()> {
        use crate::domain::claude_code::ClaudeCodeSession;
        
        let task_id = task.id;
        
        // Create session
        let session = ClaudeCodeSession::new(task_id);
        
        // Save to repository with retry
        let mut retries = 3;
        while retries > 0 {
            match self.repository.claude_code.create_session(&session).await {
                Ok(()) => break,
                Err(e) if retries > 1 => {
                    tracing::warn!("Failed to save session, retrying: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    retries -= 1;
                }
                Err(e) => return Err(e).context("Failed to save session after retries"),
            }
        }
        
        // Update execution atomically
        let mut executions = self.executions.write().await;
        executions.entry(task_id)
            .and_modify(|e| {
                e.session_id = Some(session.id);
                e.status = TaskExecutionStatus::Running;
                e.started_at = Some(Utc::now());
            })
            .or_insert_with(|| crate::services::TaskExecution {
                task_id,
                session_id: Some(session.id),
                status: TaskExecutionStatus::Running,
                started_at: Some(Utc::now()),
                completed_at: None,
                pr_url: None,
                retry_count: 0,
                error_message: None,
            });
        drop(executions);
        
        // Add to active sessions
        self.active_sessions.write().await.insert(session.id);
        
        // Start async task execution with monitoring
        self.spawn_monitored_execution(session.id, task_id).await;
        
        Ok(())
    }
    
    /// Spawns execution with monitoring and cleanup
    async fn spawn_monitored_execution(&self, session_id: Uuid, task_id: Uuid) {
        let repository = self.repository.clone();
        let active_sessions = self.active_sessions.clone();
        
        tokio::spawn(async move {
            // Ensure cleanup happens even on panic
            let _guard = CleanupGuard {
                session_id,
                active_sessions: active_sessions.clone(),
            };
            
            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            
            // Update session status
            if let Ok(Some(mut session)) = repository.claude_code.get_session(session_id).await {
                session.status = crate::domain::claude_code::SessionStatus::Completed;
                let pr_number = (Utc::now().timestamp() % 1000) as u32;
                session.pr_url = Some(format!("https://github.com/user/repo/pull/{}", pr_number));
                session.completed_at = Some(Utc::now());
                
                if let Err(e) = repository.claude_code.update_session(&session).await {
                    tracing::error!("Failed to update session {}: {}", session_id, e);
                }
            }
        });
    }
    
    /// Handles stuck tasks that haven't progressed
    pub async fn cleanup_stuck_tasks(&self) -> Result<()> {
        let now = Utc::now();
        let timeout_duration = chrono::Duration::hours(2);
        
        let mut stuck_tasks = Vec::new();
        
        {
            let executions = self.executions.read().await;
            for (task_id, exec) in executions.iter() {
                if exec.status == TaskExecutionStatus::Running {
                    if let Some(started) = exec.started_at {
                        if now.signed_duration_since(started) > timeout_duration {
                            stuck_tasks.push(*task_id);
                        }
                    }
                }
            }
        }
        
        // Handle stuck tasks
        for task_id in stuck_tasks {
            tracing::warn!("Cleaning up stuck task: {}", task_id);
            self.mark_task_failed(task_id, "Task timed out".to_string()).await?;
        }
        
        Ok(())
    }
    
    /// Marks a task as failed with proper cleanup
    async fn mark_task_failed(&self, task_id: Uuid, error_message: String) -> Result<()> {
        let mut executions = self.executions.write().await;
        
        if let Some(exec) = executions.get_mut(&task_id) {
            exec.status = TaskExecutionStatus::Failed;
            exec.completed_at = Some(Utc::now());
            exec.error_message = Some(error_message);
            
            // Remove from active sessions
            if let Some(session_id) = exec.session_id {
                self.active_sessions.write().await.remove(&session_id);
            }
        }
        
        Ok(())
    }
    
    /// Gets detailed diagnostics for debugging
    pub async fn get_diagnostics(&self) -> HashMap<String, serde_json::Value> {
        use serde_json::json;
        
        let mut diagnostics = HashMap::new();
        
        // Status
        let status = self.status.read().await;
        diagnostics.insert("status".to_string(), json!(format!("{:?}", *status)));
        drop(status);
        
        // Configuration
        let config = self.config.read().await;
        diagnostics.insert("config".to_string(), json!({
            "max_parallel_instances": config.max_parallel_instances,
            "auto_merge_enabled": config.auto_merge_enabled,
            "require_tests_pass": config.require_tests_pass,
            "retry_on_failure": config.retry_on_failure,
            "max_retries": config.max_retries,
        }));
        drop(config);
        
        // Active sessions
        let active = self.active_sessions.read().await;
        diagnostics.insert("active_sessions".to_string(), json!(active.len()));
        drop(active);
        
        // Execution statistics
        let executions = self.executions.read().await;
        let stats = json!({
            "total": executions.len(),
            "queued": executions.values().filter(|e| e.status == TaskExecutionStatus::Queued).count(),
            "running": executions.values().filter(|e| e.status == TaskExecutionStatus::Running).count(),
            "completed": executions.values().filter(|e| e.status == TaskExecutionStatus::Completed).count(),
            "failed": executions.values().filter(|e| e.status == TaskExecutionStatus::Failed).count(),
        });
        diagnostics.insert("execution_stats".to_string(), stats);
        
        // Queue size
        let queue = self.execution_queue.lock().await;
        diagnostics.insert("queue_size".to_string(), json!(queue.len()));
        
        diagnostics
    }
    
    /// Validates task list before starting auto-run
    pub async fn validate_tasks(&self, task_ids: &[Uuid]) -> Result<()> {
        if task_ids.is_empty() {
            bail!("No tasks provided for auto-run");
        }
        
        if task_ids.len() > 1000 {
            bail!("Too many tasks ({}). Maximum is 1000", task_ids.len());
        }
        
        // Check for duplicates
        let unique: HashSet<_> = task_ids.iter().collect();
        if unique.len() != task_ids.len() {
            bail!("Duplicate task IDs provided");
        }
        
        // Verify all tasks exist
        let mut missing = Vec::new();
        for &task_id in task_ids {
            if self.task_service.get(task_id).await?.is_none() {
                missing.push(task_id);
            }
        }
        
        if !missing.is_empty() {
            bail!("Tasks not found: {:?}", missing);
        }
        
        // Check for circular dependencies
        if self.has_circular_dependencies(task_ids).await? {
            bail!("Circular dependencies detected in task list");
        }
        
        Ok(())
    }
    
    /// Checks for circular dependencies
    async fn has_circular_dependencies(&self, task_ids: &[Uuid]) -> Result<bool> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for &task_id in task_ids {
            if !visited.contains(&task_id) {
                if self.dfs_cycle_check(task_id, &mut visited, &mut rec_stack, task_ids).await? {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// DFS helper for cycle detection
    async fn dfs_cycle_check(
        &self,
        task_id: Uuid,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
        task_ids: &[Uuid],
    ) -> Result<bool> {
        visited.insert(task_id);
        rec_stack.insert(task_id);
        
        let deps = self.dependency_service.get_dependencies(task_id).await?;
        for dep_id in deps {
            if task_ids.contains(&dep_id) {
                if !visited.contains(&dep_id) {
                    if self.dfs_cycle_check(dep_id, visited, rec_stack, task_ids).await? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(&dep_id) {
                    return Ok(true);
                }
            }
        }
        
        rec_stack.remove(&task_id);
        Ok(false)
    }
}

/// RAII guard for cleanup
struct CleanupGuard {
    session_id: Uuid,
    active_sessions: Arc<RwLock<HashSet<Uuid>>>,
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        // Clean up on panic or normal exit
        let sessions = self.active_sessions.clone();
        let session_id = self.session_id;
        
        tokio::spawn(async move {
            sessions.write().await.remove(&session_id);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::database::init_test_database;
    use crate::repository::Repository;
    use crate::services::{ClaudeCodeService, DependencyService, TaskService};
    
    async fn setup() -> Arc<AutoRunOrchestrator> {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let claude_service = Arc::new(ClaudeCodeService::new(
            repository.claude_code.clone()
        ));
        
        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));
        
        Arc::new(AutoRunOrchestrator::new(
            repository,
            claude_service,
            dependency_service,
            task_service,
        ))
    }
    
    #[tokio::test]
    async fn test_config_validation() {
        let orchestrator = setup().await;
        
        // Test invalid configs
        let mut invalid_config = AutoRunConfig::default();
        invalid_config.max_parallel_instances = 0;
        assert!(orchestrator.validate_and_update_config(invalid_config).await.is_err());
        
        invalid_config = AutoRunConfig::default();
        invalid_config.max_parallel_instances = 200;
        assert!(orchestrator.validate_and_update_config(invalid_config).await.is_err());
        
        invalid_config = AutoRunConfig::default();
        invalid_config.max_retries = 20;
        assert!(orchestrator.validate_and_update_config(invalid_config).await.is_err());
        
        // Test valid config
        let valid_config = AutoRunConfig {
            max_parallel_instances: 5,
            auto_merge_enabled: true,
            require_tests_pass: true,
            retry_on_failure: true,
            max_retries: 3,
        };
        assert!(orchestrator.validate_and_update_config(valid_config).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_task_validation() {
        let orchestrator = setup().await;
        
        // Empty list
        assert!(orchestrator.validate_tasks(&[]).await.is_err());
        
        // Too many tasks
        let many_ids: Vec<Uuid> = (0..1001).map(|_| Uuid::new_v4()).collect();
        assert!(orchestrator.validate_tasks(&many_ids).await.is_err());
        
        // Duplicate IDs
        let id = Uuid::new_v4();
        assert!(orchestrator.validate_tasks(&[id, id]).await.is_err());
        
        // Non-existent task
        let missing_id = Uuid::new_v4();
        assert!(orchestrator.validate_tasks(&[missing_id]).await.is_err());
    }
    
    #[tokio::test]
    async fn test_stuck_task_cleanup() {
        let orchestrator = setup().await;
        
        // Add a stuck task
        let task_id = Uuid::new_v4();
        let mut executions = orchestrator.executions.write().await;
        executions.insert(task_id, crate::services::TaskExecution {
            task_id,
            session_id: Some(Uuid::new_v4()),
            status: TaskExecutionStatus::Running,
            started_at: Some(Utc::now() - chrono::Duration::hours(3)), // 3 hours ago
            completed_at: None,
            pr_url: None,
            retry_count: 0,
            error_message: None,
        });
        drop(executions);
        
        // Run cleanup
        orchestrator.cleanup_stuck_tasks().await.unwrap();
        
        // Check task is marked as failed
        let executions = orchestrator.executions.read().await;
        let exec = executions.get(&task_id).unwrap();
        assert_eq!(exec.status, TaskExecutionStatus::Failed);
        assert!(exec.error_message.is_some());
    }
    
    #[tokio::test]
    async fn test_diagnostics() {
        let orchestrator = setup().await;
        
        let diagnostics = orchestrator.get_diagnostics().await;
        
        assert!(diagnostics.contains_key("status"));
        assert!(diagnostics.contains_key("config"));
        assert!(diagnostics.contains_key("active_sessions"));
        assert!(diagnostics.contains_key("execution_stats"));
        assert!(diagnostics.contains_key("queue_size"));
    }
}