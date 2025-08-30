use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::domain::claude_code::{ClaudeCodeSession, SessionStatus};
use crate::repository::Repository;
use crate::services::claude_code_service::ClaudeCodeService;
use crate::services::dependency_service::DependencyService;
use crate::services::task_service::TaskService;

#[derive(Debug, Clone, PartialEq)]
pub enum AutoRunStatus {
    Idle,
    Planning,
    Running,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct AutoRunConfig {
    pub max_parallel_instances: usize,
    pub auto_merge_enabled: bool,
    pub require_tests_pass: bool,
    pub retry_on_failure: bool,
    pub max_retries: usize,
}

impl Default for AutoRunConfig {
    fn default() -> Self {
        Self {
            max_parallel_instances: 3,
            auto_merge_enabled: true,
            require_tests_pass: true,
            retry_on_failure: true,
            max_retries: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskExecution {
    pub task_id: Uuid,
    pub session_id: Option<Uuid>,
    pub status: TaskExecutionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub pr_url: Option<String>,
    pub retry_count: usize,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskExecutionStatus {
    Queued,
    Running,
    PendingReview,
    Merging,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct AutoRunProgress {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub running_tasks: usize,
    pub queued_tasks: usize,
    pub current_phase: String,
}

pub struct AutoRunOrchestrator {
    repository: Arc<Repository>,
    #[allow(dead_code)]
    claude_service: Arc<ClaudeCodeService>,
    dependency_service: Arc<DependencyService>,
    task_service: Arc<TaskService>,

    status: Arc<RwLock<AutoRunStatus>>,
    config: Arc<RwLock<AutoRunConfig>>,
    pub executions: Arc<RwLock<HashMap<Uuid, TaskExecution>>>,
    execution_queue: Arc<Mutex<Vec<Uuid>>>,
    active_sessions: Arc<RwLock<HashSet<Uuid>>>,
}

impl AutoRunOrchestrator {
    pub fn new(
        repository: Arc<Repository>,
        claude_service: Arc<ClaudeCodeService>,
        dependency_service: Arc<DependencyService>,
        task_service: Arc<TaskService>,
    ) -> Self {
        Self {
            repository,
            claude_service,
            dependency_service,
            task_service,
            status: Arc::new(RwLock::new(AutoRunStatus::Idle)),
            config: Arc::new(RwLock::new(AutoRunConfig::default())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            execution_queue: Arc::new(Mutex::new(Vec::new())),
            active_sessions: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn start_auto_run(&self, task_ids: Vec<Uuid>) -> Result<()> {
        // Update status
        *self.status.write().await = AutoRunStatus::Planning;

        // Build execution plan
        let execution_plan = self.build_execution_plan(task_ids).await?;

        // Initialize executions
        let mut executions = self.executions.write().await;
        for task_id in &execution_plan {
            executions.insert(
                *task_id,
                TaskExecution {
                    task_id: *task_id,
                    session_id: None,
                    status: TaskExecutionStatus::Queued,
                    started_at: None,
                    completed_at: None,
                    pr_url: None,
                    retry_count: 0,
                    error_message: None,
                },
            );
        }
        drop(executions);

        // Start with tasks that have no dependencies
        let initial_tasks = self.get_unblocked_tasks(&execution_plan).await?;

        // Add to queue
        let mut queue = self.execution_queue.lock().await;
        queue.extend(initial_tasks);
        drop(queue);

        // Update status and start processing
        *self.status.write().await = AutoRunStatus::Running;

        // Start the orchestration loop
        self.orchestration_loop().await?;

        Ok(())
    }

    async fn build_execution_plan(&self, task_ids: Vec<Uuid>) -> Result<Vec<Uuid>> {
        // Simple topological sort without recursion
        let mut plan = Vec::new();
        let mut in_degree = HashMap::new();
        let mut adjacency = HashMap::new();

        // Initialize all tasks
        for &task_id in &task_ids {
            adjacency.insert(task_id, Vec::new());
            in_degree.insert(task_id, 0);
        }

        // Build dependency graph for selected tasks
        for &task_id in &task_ids {
            let deps = self.dependency_service.get_dependencies(task_id).await?;

            for dep_id in deps {
                if task_ids.contains(&dep_id) {
                    // task_id depends on dep_id, so dep_id -> task_id in the graph
                    adjacency
                        .entry(dep_id)
                        .or_insert_with(Vec::new)
                        .push(task_id);
                    *in_degree.entry(task_id).or_insert(0) += 1;
                }
            }
        }

        // Find all tasks with no dependencies
        let mut queue: Vec<Uuid> = in_degree
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(id, _)| *id)
            .collect();

        // Process queue
        while let Some(task_id) = queue.pop() {
            plan.push(task_id);

            if let Some(dependents) = adjacency.get(&task_id) {
                for &dependent_id in dependents {
                    if let Some(count) = in_degree.get_mut(&dependent_id) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push(dependent_id);
                        }
                    }
                }
            }
        }

        // Check for cycles
        if plan.len() != task_ids.len() {
            return Err(anyhow::anyhow!("Circular dependency detected"));
        }

        Ok(plan)
    }

    async fn get_unblocked_tasks(&self, execution_plan: &[Uuid]) -> Result<Vec<Uuid>> {
        let mut unblocked = Vec::new();
        let executions = self.executions.read().await;

        for task_id in execution_plan {
            let execution = executions.get(task_id);
            if let Some(exec) = execution
                && exec.status == TaskExecutionStatus::Queued
            {
                // Check if all dependencies are completed
                let deps = self.dependency_service.get_dependencies(*task_id).await?;
                let all_deps_complete = deps.iter().all(|dep_id| {
                    let exec_opt = executions.get(dep_id);
                    // If not in execution plan, consider it complete
                    exec_opt
                        .map(|e| e.status == TaskExecutionStatus::Completed)
                        .unwrap_or(true)
                });

                if all_deps_complete {
                    unblocked.push(*task_id);
                }
            }
        }

        Ok(unblocked)
    }

    async fn orchestration_loop(&self) -> Result<()> {
        loop {
            let status = self.status.read().await.clone();
            match status {
                AutoRunStatus::Running => {
                    // Process queue
                    self.process_queue().await?;

                    // Check for completed sessions
                    self.check_completed_sessions().await?;

                    // Check if all tasks are done
                    if self.all_tasks_complete().await? {
                        *self.status.write().await = AutoRunStatus::Completed;
                        break;
                    }

                    // Small delay to prevent busy waiting
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                AutoRunStatus::Paused => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
                AutoRunStatus::Completed | AutoRunStatus::Failed(_) => {
                    break;
                }
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }

        Ok(())
    }

    async fn process_queue(&self) -> Result<()> {
        let config = self.config.read().await;
        let active_count = self.active_sessions.read().await.len();

        if active_count >= config.max_parallel_instances {
            return Ok(());
        }

        let slots_available = config.max_parallel_instances - active_count;
        drop(config);

        let mut queue = self.execution_queue.lock().await;
        let drain_end = slots_available.min(queue.len());
        let tasks_to_start: Vec<Uuid> = queue.drain(..drain_end).collect();
        drop(queue);

        for task_id in tasks_to_start {
            self.start_task_execution(task_id).await?;
        }

        Ok(())
    }

    async fn start_task_execution(&self, task_id: Uuid) -> Result<()> {
        // Get the task
        let task = self
            .task_service
            .get(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // Create Claude Code session
        let session = ClaudeCodeSession::new(task.id);

        // Save session to repository
        self.repository.claude_code.create_session(&session).await?;

        // Update execution
        let mut executions = self.executions.write().await;
        if let Some(exec) = executions.get_mut(&task_id) {
            exec.session_id = Some(session.id);
            exec.status = TaskExecutionStatus::Running;
            exec.started_at = Some(Utc::now());
        }
        drop(executions);

        // Add to active sessions
        self.active_sessions.write().await.insert(session.id);

        // Simulate Claude Code starting the task
        // In real implementation, this would launch actual Claude Code instance
        tokio::spawn({
            let repository = self.repository.clone();
            let session_id = session.id;
            async move {
                // Simulate work being done (shorter for tests)
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                // Update session status to completed
                if let Ok(Some(mut session)) = repository.claude_code.get_session(session_id).await
                {
                    session.status = SessionStatus::Completed;
                    // Use timestamp for unique PR number
                    let pr_number = (Utc::now().timestamp() % 1000) as u32;
                    session.pr_url =
                        Some(format!("https://github.com/user/repo/pull/{}", pr_number));
                    session.completed_at = Some(Utc::now());
                    let _ = repository.claude_code.update_session(&session).await;
                }
            }
        });

        Ok(())
    }

    async fn check_completed_sessions(&self) -> Result<()> {
        let active_sessions = self.active_sessions.read().await.clone();

        for session_id in active_sessions {
            let session = self.repository.claude_code.get_session(session_id).await?;

            if let Some(session) = session {
                match session.status {
                    SessionStatus::Completed => {
                        self.handle_completed_session(session).await?;
                    }
                    SessionStatus::Failed => {
                        self.handle_failed_session(session).await?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    async fn handle_completed_session(&self, session: ClaudeCodeSession) -> Result<()> {
        // Remove from active sessions
        self.active_sessions.write().await.remove(&session.id);

        // Update execution status
        let mut executions = self.executions.write().await;
        if let Some(exec) = executions.get_mut(&session.task_id) {
            exec.status = TaskExecutionStatus::PendingReview;
            exec.pr_url = session.pr_url.clone();
        }
        drop(executions);

        // Start PR review process
        if let Some(pr_url) = session.pr_url {
            self.start_pr_review(session.task_id, pr_url).await?;
        }

        Ok(())
    }

    async fn handle_failed_session(&self, session: ClaudeCodeSession) -> Result<()> {
        // Remove from active sessions
        self.active_sessions.write().await.remove(&session.id);

        let config = self.config.read().await;
        let should_retry = config.retry_on_failure;
        let max_retries = config.max_retries;
        drop(config);

        let mut executions = self.executions.write().await;
        if let Some(exec) = executions.get_mut(&session.task_id) {
            exec.error_message = session.error_message.clone();

            if should_retry && exec.retry_count < max_retries {
                // Retry the task
                exec.retry_count += 1;
                exec.status = TaskExecutionStatus::Queued;
                drop(executions);

                let mut queue = self.execution_queue.lock().await;
                queue.push(session.task_id);
            } else {
                // Mark as failed
                exec.status = TaskExecutionStatus::Failed;
                exec.completed_at = Some(Utc::now());
            }
        }

        Ok(())
    }

    async fn start_pr_review(&self, task_id: Uuid, _pr_url: String) -> Result<()> {
        // This will be implemented with the PR review service
        // For now, simulate auto-merge after review

        tokio::spawn({
            let executions = self.executions.clone();
            let queue = self.execution_queue.clone();
            let dependency_service = self.dependency_service.clone();

            async move {
                // Simulate PR review time (shorter for tests)
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                // Update execution status
                let mut execs = executions.write().await;
                if let Some(exec) = execs.get_mut(&task_id) {
                    exec.status = TaskExecutionStatus::Completed;
                    exec.completed_at = Some(Utc::now());
                }
                drop(execs);

                // Find newly unblocked tasks
                if let Ok(dependents) = dependency_service.get_dependents(task_id).await {
                    let execs = executions.read().await;
                    let mut newly_unblocked = Vec::new();

                    for dependent_id in dependents {
                        if let Some(exec) = execs.get(&dependent_id)
                            && exec.status == TaskExecutionStatus::Queued
                        {
                            // Check if all other dependencies are complete
                            if let Ok(deps) =
                                dependency_service.get_dependencies(dependent_id).await
                            {
                                let all_complete = deps.iter().all(|dep_id| {
                                    execs
                                        .get(dep_id)
                                        .map(|e| e.status == TaskExecutionStatus::Completed)
                                        .unwrap_or(true)
                                });

                                if all_complete {
                                    newly_unblocked.push(dependent_id);
                                }
                            }
                        }
                    }
                    drop(execs);

                    // Add newly unblocked tasks to queue
                    if !newly_unblocked.is_empty() {
                        let mut q = queue.lock().await;
                        q.extend(newly_unblocked);
                    }
                }
            }
        });

        Ok(())
    }

    async fn all_tasks_complete(&self) -> Result<bool> {
        let executions = self.executions.read().await;
        Ok(executions.values().all(|exec| {
            exec.status == TaskExecutionStatus::Completed
                || exec.status == TaskExecutionStatus::Failed
        }))
    }

    pub async fn pause(&self) -> Result<()> {
        *self.status.write().await = AutoRunStatus::Paused;
        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        *self.status.write().await = AutoRunStatus::Running;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        *self.status.write().await = AutoRunStatus::Idle;

        // Clear all state
        self.executions.write().await.clear();
        self.execution_queue.lock().await.clear();
        self.active_sessions.write().await.clear();

        Ok(())
    }

    pub async fn get_status(&self) -> AutoRunStatus {
        self.status.read().await.clone()
    }

    pub async fn get_progress(&self) -> AutoRunProgress {
        let executions = self.executions.read().await;

        let total_tasks = executions.len();
        let completed_tasks = executions
            .values()
            .filter(|e| e.status == TaskExecutionStatus::Completed)
            .count();
        let failed_tasks = executions
            .values()
            .filter(|e| e.status == TaskExecutionStatus::Failed)
            .count();
        let running_tasks = executions
            .values()
            .filter(|e| e.status == TaskExecutionStatus::Running)
            .count();
        let queued_tasks = executions
            .values()
            .filter(|e| e.status == TaskExecutionStatus::Queued)
            .count();

        let current_phase = if running_tasks > 0 {
            format!("Executing {} tasks", running_tasks)
        } else if queued_tasks > 0 {
            format!("{} tasks queued", queued_tasks)
        } else {
            "Idle".to_string()
        };

        AutoRunProgress {
            total_tasks,
            completed_tasks,
            failed_tasks,
            running_tasks,
            queued_tasks,
            current_phase,
        }
    }

    pub async fn update_config(&self, config: AutoRunConfig) -> Result<()> {
        *self.config.write().await = config;
        Ok(())
    }

    pub async fn get_execution_details(&self) -> Vec<TaskExecution> {
        self.executions.read().await.values().cloned().collect()
    }
    
    #[cfg(test)]
    pub async fn get_active_sessions_count(&self) -> usize {
        self.active_sessions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::Task;
    use crate::repository::database::init_test_database;

    async fn setup() -> (AutoRunOrchestrator, Arc<Repository>) {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        let claude_service = Arc::new(ClaudeCodeService::new(repository.claude_code.clone()));

        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));

        let orchestrator = AutoRunOrchestrator::new(
            repository.clone(),
            claude_service,
            dependency_service,
            task_service,
        );

        (orchestrator, repository)
    }

    #[tokio::test]
    async fn test_build_execution_plan() {
        let (orchestrator, repository) = setup().await;

        // Create tasks with dependencies
        let task1 = Task::new("Task 1".to_string(), "".to_string());
        let task2 = Task::new("Task 2".to_string(), "".to_string());
        let task3 = Task::new("Task 3".to_string(), "".to_string());

        repository.tasks.create(&task1).await.unwrap();
        repository.tasks.create(&task2).await.unwrap();
        repository.tasks.create(&task3).await.unwrap();

        // task2 depends on task1, task3 depends on task2
        orchestrator
            .dependency_service
            .add_dependency(task2.id, task1.id)
            .await
            .unwrap();
        orchestrator
            .dependency_service
            .add_dependency(task3.id, task2.id)
            .await
            .unwrap();

        let plan = orchestrator
            .build_execution_plan(vec![task1.id, task2.id, task3.id])
            .await
            .unwrap();

        // Should be in dependency order: task1, task2, task3
        assert_eq!(plan, vec![task1.id, task2.id, task3.id]);
    }

    #[tokio::test]
    async fn test_get_unblocked_tasks() {
        let (orchestrator, repository) = setup().await;

        // Create tasks
        let task1 = Task::new("Task 1".to_string(), "".to_string());
        let task2 = Task::new("Task 2".to_string(), "".to_string());
        let task3 = Task::new("Task 3".to_string(), "".to_string());

        repository.tasks.create(&task1).await.unwrap();
        repository.tasks.create(&task2).await.unwrap();
        repository.tasks.create(&task3).await.unwrap();

        // task2 depends on task1
        orchestrator
            .dependency_service
            .add_dependency(task2.id, task1.id)
            .await
            .unwrap();

        // Initialize executions
        let mut executions = orchestrator.executions.write().await;
        executions.insert(
            task1.id,
            TaskExecution {
                task_id: task1.id,
                session_id: None,
                status: TaskExecutionStatus::Queued,
                started_at: None,
                completed_at: None,
                pr_url: None,
                retry_count: 0,
                error_message: None,
            },
        );
        executions.insert(
            task2.id,
            TaskExecution {
                task_id: task2.id,
                session_id: None,
                status: TaskExecutionStatus::Queued,
                started_at: None,
                completed_at: None,
                pr_url: None,
                retry_count: 0,
                error_message: None,
            },
        );
        executions.insert(
            task3.id,
            TaskExecution {
                task_id: task3.id,
                session_id: None,
                status: TaskExecutionStatus::Queued,
                started_at: None,
                completed_at: None,
                pr_url: None,
                retry_count: 0,
                error_message: None,
            },
        );
        drop(executions);

        // Check dependencies were set correctly
        let task1_deps = orchestrator
            .dependency_service
            .get_dependencies(task1.id)
            .await
            .unwrap();
        let task2_deps = orchestrator
            .dependency_service
            .get_dependencies(task2.id)
            .await
            .unwrap();
        let task3_deps = orchestrator
            .dependency_service
            .get_dependencies(task3.id)
            .await
            .unwrap();

        println!("Task1 deps: {:?}", task1_deps);
        println!("Task2 deps: {:?}", task2_deps);
        println!("Task3 deps: {:?}", task3_deps);

        let unblocked = orchestrator
            .get_unblocked_tasks(&[task1.id, task2.id, task3.id])
            .await
            .unwrap();

        // Only task1 and task3 should be unblocked (task2 depends on task1)
        println!("Unblocked tasks: {:?}", unblocked);
        println!(
            "Task IDs: task1={:?}, task2={:?}, task3={:?}",
            task1.id, task2.id, task3.id
        );
        assert_eq!(unblocked.len(), 2);
        assert!(unblocked.contains(&task1.id));
        assert!(unblocked.contains(&task3.id));
    }

    #[tokio::test]
    async fn test_status_transitions() {
        let (orchestrator, _) = setup().await;

        assert_eq!(orchestrator.get_status().await, AutoRunStatus::Idle);

        orchestrator.pause().await.unwrap();
        assert_eq!(orchestrator.get_status().await, AutoRunStatus::Paused);

        orchestrator.resume().await.unwrap();
        assert_eq!(orchestrator.get_status().await, AutoRunStatus::Running);

        orchestrator.stop().await.unwrap();
        assert_eq!(orchestrator.get_status().await, AutoRunStatus::Idle);
    }

    #[tokio::test]
    async fn test_config_update() {
        let (orchestrator, _) = setup().await;

        let new_config = AutoRunConfig {
            max_parallel_instances: 5,
            auto_merge_enabled: false,
            require_tests_pass: false,
            retry_on_failure: false,
            max_retries: 0,
        };

        orchestrator
            .update_config(new_config.clone())
            .await
            .unwrap();

        let config = orchestrator.config.read().await;
        assert_eq!(config.max_parallel_instances, 5);
        assert!(!config.auto_merge_enabled);
        assert!(!config.require_tests_pass);
        assert!(!config.retry_on_failure);
        assert_eq!(config.max_retries, 0);
    }

    #[tokio::test]
    async fn test_progress_tracking() {
        let (orchestrator, _) = setup().await;

        // Add some mock executions
        let mut executions = orchestrator.executions.write().await;
        let task_id1 = Uuid::new_v4();
        let task_id2 = Uuid::new_v4();
        let task_id3 = Uuid::new_v4();

        executions.insert(
            task_id1,
            TaskExecution {
                task_id: task_id1,
                session_id: None,
                status: TaskExecutionStatus::Completed,
                started_at: None,
                completed_at: Some(Utc::now()),
                pr_url: None,
                retry_count: 0,
                error_message: None,
            },
        );

        executions.insert(
            task_id2,
            TaskExecution {
                task_id: task_id2,
                session_id: None,
                status: TaskExecutionStatus::Running,
                started_at: Some(Utc::now()),
                completed_at: None,
                pr_url: None,
                retry_count: 0,
                error_message: None,
            },
        );

        executions.insert(
            task_id3,
            TaskExecution {
                task_id: task_id3,
                session_id: None,
                status: TaskExecutionStatus::Queued,
                started_at: None,
                completed_at: None,
                pr_url: None,
                retry_count: 0,
                error_message: None,
            },
        );
        drop(executions);

        let progress = orchestrator.get_progress().await;
        assert_eq!(progress.total_tasks, 3);
        assert_eq!(progress.completed_tasks, 1);
        assert_eq!(progress.failed_tasks, 0);
        assert_eq!(progress.running_tasks, 1);
        assert_eq!(progress.queued_tasks, 1);
    }
}
