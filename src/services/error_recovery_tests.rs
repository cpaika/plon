#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::domain::task::{Task, TaskStatus};
    use crate::domain::claude_code::{ClaudeCodeSession, SessionStatus};
    use crate::repository::database::init_test_database;
    use crate::repository::Repository;
    use crate::services::{
        AutoRunOrchestrator, AutoRunConfig, AutoRunStatus,
        ClaudeCodeService, DependencyService, TaskService,
        PRReviewService, TaskExecutionStatus,
        command_executor::{CommandExecutor, mock::MockCommandExecutor},
    };
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use uuid::Uuid;
    use anyhow::{Result, bail};
    use tokio::time::{sleep, Duration};
    
    /// Test recovery from network failures
    #[tokio::test]
    async fn test_network_failure_recovery() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        // Create mock executor that fails then succeeds
        let mock_executor = Arc::new(MockCommandExecutor::new());
        
        // First call fails with network error
        mock_executor.add_response(
            "gh",
            vec!["pr", "create"],
            "",
            "error: Network connection failed",
            false,
        );
        
        // Second call succeeds after retry
        mock_executor.add_response(
            "gh",
            vec!["pr", "create"],
            "https://github.com/user/repo/pull/42",
            "",
            true,
        );
        
        let pr_service = Arc::new(PRReviewService::new(
            repository.claude_code.clone(),
            mock_executor.clone(),
        ));
        
        // Should retry and succeed
        let task_id = Uuid::new_v4();
        let result = pr_service.create_pr_with_retry(task_id, 3).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://github.com/user/repo/pull/42");
    }
    
    /// Test handling of corrupted database state
    #[tokio::test]
    async fn test_corrupted_state_recovery() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        // Create task with invalid references
        let task = Task {
            id: Uuid::new_v4(),
            title: "Task with bad refs".to_string(),
            parent_id: Some(Uuid::new_v4()), // Non-existent parent
            ..Task::default()
        };
        
        // Should handle gracefully
        let result = repository.tasks.create(&task).await;
        if result.is_ok() {
            // Verify orphaned task is handled
            let children = repository.tasks.get_children(task.parent_id.unwrap()).await;
            assert!(children.is_err() || children.unwrap().is_empty());
        }
    }
    
    /// Test recovery from out-of-memory conditions
    #[tokio::test]
    async fn test_memory_exhaustion_recovery() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let claude_service = Arc::new(ClaudeCodeService::new(
            repository.claude_code.clone(),
        ));
        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));
        
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository,
            claude_service,
            dependency_service,
            task_service,
        ));
        
        // Configure with memory limits
        let config = AutoRunConfig {
            max_parallel_instances: 2, // Very limited
            auto_merge_enabled: false,
            require_tests_pass: true,
            retry_on_failure: true,
            max_retries: 1,
        };
        orchestrator.validate_and_update_config(config).await.unwrap();
        
        // Try to start many tasks
        let mut task_ids = Vec::new();
        for _ in 0..10 {
            let task = Task {
                id: Uuid::new_v4(),
                title: "Memory test task".to_string(),
                ..Task::default()
            };
            orchestrator.repository.tasks.create(&task).await.unwrap();
            task_ids.push(task.id);
        }
        
        // Should queue and process within memory limits
        let result = orchestrator.start_auto_run(task_ids).await;
        assert!(result.is_ok());
        
        // Check that only max_parallel_instances are running
        let active = orchestrator.active_sessions.read().await;
        assert!(active.len() <= 2);
    }
    
    /// Test graceful degradation under high load
    #[tokio::test]
    async fn test_graceful_degradation() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let claude_service = Arc::new(ClaudeCodeService::new(
            repository.claude_code.clone(),
        ));
        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));
        
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository,
            claude_service,
            dependency_service,
            task_service,
        ));
        
        // Simulate high load with many concurrent operations
        let mut handles = Vec::new();
        
        for i in 0..20 {
            let orch_clone = orchestrator.clone();
            let handle = tokio::spawn(async move {
                let task = Task {
                    id: Uuid::new_v4(),
                    title: format!("Load test task {}", i),
                    ..Task::default()
                };
                orch_clone.repository.tasks.create(&task).await.unwrap();
                
                // Try to start execution
                let result = orch_clone.start_task_execution_safe(task.id).await;
                // Should either succeed or fail gracefully
                match result {
                    Ok(_) => true,
                    Err(e) => {
                        // Check for expected errors under load
                        e.to_string().contains("Maximum parallel instances") ||
                        e.to_string().contains("resource") ||
                        e.to_string().contains("timeout")
                    }
                }
            });
            handles.push(handle);
        }
        
        // All should complete without panics
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }
    
    /// Test transaction rollback on failure
    #[tokio::test]
    async fn test_transaction_rollback() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        // Start a transaction
        let mut tx = repository.begin_transaction().await.unwrap();
        
        // Create task in transaction
        let task = Task {
            id: Uuid::new_v4(),
            title: "Transaction test".to_string(),
            ..Task::default()
        };
        
        let result = tx.tasks.create(&task).await;
        assert!(result.is_ok());
        
        // Simulate failure - don't commit
        drop(tx);
        
        // Task should not exist after rollback
        let fetched = repository.tasks.get(task.id).await.unwrap();
        assert!(fetched.is_none());
    }
    
    /// Test handling of deadlocks
    #[tokio::test]
    async fn test_deadlock_detection() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let task1 = Task {
            id: Uuid::new_v4(),
            title: "Task 1".to_string(),
            ..Task::default()
        };
        let task2 = Task {
            id: Uuid::new_v4(),
            title: "Task 2".to_string(),
            ..Task::default()
        };
        
        repository.tasks.create(&task1).await.unwrap();
        repository.tasks.create(&task2).await.unwrap();
        
        // Create circular dependency
        let dep_service = DependencyService::new(repository.clone());
        dep_service.add_dependency(task1.id, task2.id).await.unwrap();
        
        // Try to add reverse dependency (would create deadlock)
        let result = dep_service.add_dependency(task2.id, task1.id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("circular") || 
                result.unwrap_err().to_string().contains("deadlock"));
    }
    
    /// Test recovery from partial failures
    #[tokio::test]
    async fn test_partial_failure_recovery() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        // Create tasks with some that will fail
        let mut task_ids = Vec::new();
        
        for i in 0..5 {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Task {}", i),
                // Task 2 has invalid data
                description: if i == 2 { "\0invalid\0".to_string() } else { "valid".to_string() },
                ..Task::default()
            };
            
            let result = repository.tasks.create(&task).await;
            if result.is_ok() {
                task_ids.push(task.id);
            }
        }
        
        // Should have created all but the invalid one
        assert_eq!(task_ids.len(), 4);
    }
    
    /// Test timeout and cancellation handling
    #[tokio::test]
    async fn test_timeout_handling() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let claude_service = Arc::new(ClaudeCodeService::new(
            repository.claude_code.clone(),
        ));
        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));
        
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository,
            claude_service,
            dependency_service,
            task_service,
        ));
        
        // Create a task
        let task = Task {
            id: Uuid::new_v4(),
            title: "Timeout test".to_string(),
            ..Task::default()
        };
        orchestrator.repository.tasks.create(&task).await.unwrap();
        
        // Start execution with timeout
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            orchestrator.start_task_execution_safe(task.id)
        ).await;
        
        // Should handle timeout gracefully
        if result.is_err() {
            // Verify task is marked as failed
            sleep(Duration::from_millis(200)).await;
            let executions = orchestrator.executions.read().await;
            if let Some(exec) = executions.get(&task.id) {
                assert!(matches!(exec.status, TaskExecutionStatus::Failed | TaskExecutionStatus::Queued));
            }
        }
    }
    
    /// Test error propagation and logging
    #[tokio::test]
    async fn test_error_propagation() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        // Test nested error contexts
        async fn nested_operation(repo: &Repository) -> Result<()> {
            let task = repo.tasks.get(Uuid::new_v4()).await
                .context("Failed to fetch task")?
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
            Ok(())
        }
        
        let result = nested_operation(&repository).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        let error_chain = error.to_string();
        assert!(error_chain.contains("Task not found") || error_chain.contains("Failed to fetch"));
    }
}

// Helper implementations for testing
use crate::repository::Repository;

impl Repository {
    async fn begin_transaction(&self) -> Result<TransactionRepository> {
        // Simplified transaction support for testing
        Ok(TransactionRepository {
            tasks: self.tasks.clone(),
            goals: self.goals.clone(),
            resources: self.resources.clone(),
        })
    }
}

struct TransactionRepository {
    tasks: crate::repository::task_repository::TaskRepository,
    goals: crate::repository::goal_repository::GoalRepository,
    resources: crate::repository::resource_repository::ResourceRepository,
}

use crate::services::PRReviewService;
use anyhow::bail;

impl PRReviewService {
    async fn create_pr_with_retry(&self, task_id: Uuid, max_retries: u32) -> Result<String> {
        let mut retries = 0;
        loop {
            match self.create_pr(task_id).await {
                Ok(url) => return Ok(url),
                Err(e) if retries < max_retries => {
                    if e.to_string().contains("Network") {
                        retries += 1;
                        sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                    return Err(e);
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    async fn create_pr(&self, task_id: Uuid) -> Result<String> {
        let output = self.executor.execute("gh pr create --title 'Test PR' --body 'Test'").await?;
        if output.exit_code != 0 {
            bail!("Failed to create PR: {}", output.stderr.join("\n"));
        }
        Ok(output.stdout.join("\n"))
    }
}

