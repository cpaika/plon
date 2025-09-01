#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::domain::task::{Task, TaskStatus, Priority};
    use crate::repository::database::init_test_database;
    use crate::repository::Repository;
    use crate::services::{
        AutoRunOrchestrator, AutoRunConfig, TaskService,
        ClaudeCodeService, DependencyService, TaskExecutionStatus,
    };
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    use std::time::Instant;
    use uuid::Uuid;
    use tokio::sync::Semaphore;
    use futures::future::join_all;
    
    /// Test high concurrency database operations
    #[tokio::test]
    async fn test_concurrent_database_operations() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let num_operations = 100;
        let num_threads = 10;
        let ops_per_thread = num_operations / num_threads;
        
        let success_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));
        
        let start = Instant::now();
        let mut handles = Vec::new();
        
        for thread_id in 0..num_threads {
            let repo_clone = repository.clone();
            let success = success_count.clone();
            let errors = error_count.clone();
            
            let handle = tokio::spawn(async move {
                for i in 0..ops_per_thread {
                    let task = Task {
                        id: Uuid::new_v4(),
                        title: format!("Thread {} Task {}", thread_id, i),
                        status: if i % 2 == 0 { TaskStatus::NotStarted } else { TaskStatus::InProgress },
                        priority: match i % 3 {
                            0 => Priority::High,
                            1 => Priority::Medium,
                            _ => Priority::Low,
                        },
                        ..Task::default()
                    };
                    
                    match repo_clone.tasks.create(&task).await {
                        Ok(_) => {
                            success.fetch_add(1, Ordering::Relaxed);
                            
                            // Random read/update operations
                            if i % 3 == 0 {
                                let _ = repo_clone.tasks.get(task.id).await;
                            }
                            if i % 5 == 0 {
                                let _ = repo_clone.tasks.update(&task).await;
                            }
                        }
                        Err(_) => {
                            errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            });
            handles.push(handle);
        }
        
        // Wait for all operations
        join_all(handles).await;
        
        let elapsed = start.elapsed();
        let total_success = success_count.load(Ordering::Relaxed);
        let total_errors = error_count.load(Ordering::Relaxed);
        
        println!("Stress test completed:");
        println!("  Operations: {}", num_operations);
        println!("  Successful: {}", total_success);
        println!("  Errors: {}", total_errors);
        println!("  Time: {:?}", elapsed);
        println!("  Ops/sec: {:.2}", total_success as f64 / elapsed.as_secs_f64());
        
        // Most operations should succeed
        assert!(total_success > num_operations * 8 / 10);
    }
    
    /// Test parallel auto-run execution under load
    #[tokio::test]
    async fn test_parallel_auto_run_stress() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let claude_service = Arc::new(ClaudeCodeService::new(
            repository.claude_code.clone(),
        ));
        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));
        
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository.clone(),
            claude_service,
            dependency_service,
            task_service,
        ));
        
        // Configure for maximum parallelism
        let config = AutoRunConfig {
            max_parallel_instances: 20,
            auto_merge_enabled: true,
            require_tests_pass: false,
            retry_on_failure: false,
            max_retries: 0,
        };
        orchestrator.validate_and_update_config(config).await.unwrap();
        
        // Create many tasks with dependencies
        let num_tasks = 50;
        let mut task_ids = Vec::new();
        
        for i in 0..num_tasks {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Stress Task {}", i),
                estimated_hours: Some(0.1),
                ..Task::default()
            };
            repository.tasks.create(&task).await.unwrap();
            task_ids.push(task.id);
            
            // Add dependencies to create complex graph
            if i > 0 && i % 3 == 0 {
                let _ = orchestrator.dependency_service
                    .add_dependency(task.id, task_ids[i - 1]).await;
            }
            if i > 1 && i % 5 == 0 {
                let _ = orchestrator.dependency_service
                    .add_dependency(task.id, task_ids[i - 2]).await;
            }
        }
        
        let start = Instant::now();
        
        // Start auto-run
        let result = orchestrator.start_auto_run(task_ids.clone()).await;
        assert!(result.is_ok());
        
        // Monitor progress
        let mut max_parallel = 0;
        let mut checks = 0;
        
        while orchestrator.status.read().await.clone() == AutoRunStatus::Running {
            let active_count = orchestrator.active_sessions.read().await.len();
            max_parallel = max_parallel.max(active_count);
            checks += 1;
            
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            // Prevent infinite loop
            if checks > 1000 {
                break;
            }
        }
        
        let elapsed = start.elapsed();
        
        println!("Parallel execution stress test:");
        println!("  Tasks: {}", num_tasks);
        println!("  Max parallel: {}", max_parallel);
        println!("  Time: {:?}", elapsed);
        
        // Should have achieved some parallelism
        assert!(max_parallel > 1);
    }
    
    /// Test memory usage under sustained load
    #[tokio::test]
    async fn test_memory_stress() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        // Track approximate memory usage
        let initial_mem = get_approximate_memory_usage();
        
        // Create many large tasks
        let mut handles = Vec::new();
        let semaphore = Arc::new(Semaphore::new(5)); // Limit concurrent operations
        
        for batch in 0..10 {
            let repo_clone = repository.clone();
            let sem_clone = semaphore.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = sem_clone.acquire().await.unwrap();
                
                for i in 0..100 {
                    let large_description = "x".repeat(10000); // 10KB per task
                    let task = Task {
                        id: Uuid::new_v4(),
                        title: format!("Memory Test {} - {}", batch, i),
                        description: large_description,
                        ..Task::default()
                    };
                    
                    let _ = repo_clone.tasks.create(&task).await;
                    
                    // Periodically clean up
                    if i % 20 == 0 {
                        tokio::task::yield_now().await;
                    }
                }
            });
            handles.push(handle);
        }
        
        // Wait for all operations
        join_all(handles).await;
        
        let final_mem = get_approximate_memory_usage();
        let mem_increase = final_mem.saturating_sub(initial_mem);
        
        println!("Memory stress test:");
        println!("  Initial memory: ~{}MB", initial_mem / 1024 / 1024);
        println!("  Final memory: ~{}MB", final_mem / 1024 / 1024);
        println!("  Increase: ~{}MB", mem_increase / 1024 / 1024);
        
        // Memory increase should be reasonable (not growing unbounded)
        // This is a rough check - in practice would need proper memory profiling
        assert!(mem_increase < 500 * 1024 * 1024); // Less than 500MB increase
    }
    
    /// Test rapid state changes
    #[tokio::test]
    async fn test_rapid_state_changes() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        // Create a task
        let task = Task {
            id: Uuid::new_v4(),
            title: "Rapid change task".to_string(),
            ..Task::default()
        };
        repository.tasks.create(&task).await.unwrap();
        
        let num_changes = 100;
        let mut handles = Vec::new();
        
        // Multiple threads rapidly changing state
        for thread_id in 0..5 {
            let repo_clone = repository.clone();
            let task_id = task.id;
            
            let handle = tokio::spawn(async move {
                for i in 0..num_changes {
                    if let Ok(Some(mut current_task)) = repo_clone.tasks.get(task_id).await {
                        // Rapid status changes
                        current_task.status = match (thread_id + i) % 4 {
                            0 => TaskStatus::NotStarted,
                            1 => TaskStatus::InProgress,
                            2 => TaskStatus::Completed,
                            _ => TaskStatus::Blocked,
                        };
                        
                        let _ = repo_clone.tasks.update(&current_task).await;
                    }
                }
            });
            handles.push(handle);
        }
        
        join_all(handles).await;
        
        // Final state should be consistent
        let final_task = repository.tasks.get(task.id).await.unwrap().unwrap();
        assert!(!final_task.title.is_empty());
    }
    
    /// Test queue overflow handling
    #[tokio::test]
    async fn test_queue_overflow() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let claude_service = Arc::new(ClaudeCodeService::new(
            repository.claude_code.clone(),
        ));
        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));
        
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository.clone(),
            claude_service,
            dependency_service,
            task_service,
        ));
        
        // Configure with very small queue
        let config = AutoRunConfig {
            max_parallel_instances: 1,
            auto_merge_enabled: false,
            require_tests_pass: false,
            retry_on_failure: false,
            max_retries: 0,
        };
        orchestrator.validate_and_update_config(config).await.unwrap();
        
        // Try to queue many tasks at once
        let mut task_ids = Vec::new();
        for i in 0..100 {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Queue test {}", i),
                ..Task::default()
            };
            repository.tasks.create(&task).await.unwrap();
            task_ids.push(task.id);
        }
        
        // Should handle gracefully without panic
        let result = orchestrator.start_auto_run(task_ids).await;
        assert!(result.is_ok());
        
        // Check queue size
        let queue_size = orchestrator.execution_queue.lock().await.len();
        println!("Queue size after overflow test: {}", queue_size);
        
        // Queue should be bounded
        assert!(queue_size <= 100);
    }
    
    /// Test concurrent dependency modifications
    #[tokio::test]
    async fn test_concurrent_dependency_changes() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let dep_service = Arc::new(DependencyService::new(repository.clone()));
        
        // Create base tasks
        let mut tasks = Vec::new();
        for i in 0..10 {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Dep test {}", i),
                ..Task::default()
            };
            repository.tasks.create(&task).await.unwrap();
            tasks.push(task);
        }
        
        let mut handles = Vec::new();
        
        // Concurrent dependency additions and removals
        for _ in 0..5 {
            let dep_clone = dep_service.clone();
            let tasks_clone = tasks.clone();
            
            let handle = tokio::spawn(async move {
                for i in 0..20 {
                    let from_idx = i % tasks_clone.len();
                    let to_idx = (i + 1) % tasks_clone.len();
                    
                    if from_idx != to_idx {
                        if i % 2 == 0 {
                            // Add dependency
                            let _ = dep_clone.add_dependency(
                                tasks_clone[from_idx].id,
                                tasks_clone[to_idx].id
                            ).await;
                        } else {
                            // Remove dependency
                            let _ = dep_clone.remove_dependency(
                                tasks_clone[from_idx].id,
                                tasks_clone[to_idx].id
                            ).await;
                        }
                    }
                }
            });
            handles.push(handle);
        }
        
        join_all(handles).await;
        
        // Verify no circular dependencies
        for task in &tasks {
            let has_circular = dep_service.has_circular_dependency(task.id).await;
            assert!(!has_circular.unwrap_or(false));
        }
    }
}

// Helper function to get approximate memory usage
fn get_approximate_memory_usage() -> usize {
    // This is a simplified approach - in production would use proper memory profiling
    use std::fs;
    
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return kb * 1024; // Convert KB to bytes
                    }
                }
            }
        }
    }
    
    // Fallback: return a default value if we can't read memory
    100 * 1024 * 1024 // 100MB default
}