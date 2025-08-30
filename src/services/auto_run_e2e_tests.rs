#[cfg(test)]
mod tests {
    use crate::domain::task::Task;
    use crate::repository::Repository;
    use crate::repository::database::init_test_database;
    use crate::services::command_executor::{CommandExecutor, mock::MockCommandExecutor};
    use crate::services::{
        AutoRunConfig, AutoRunOrchestrator, AutoRunStatus, ClaudeCodeService, DependencyService, 
        PRReviewService, TaskExecutionStatus, TaskService,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;
    use uuid::Uuid;

    struct E2ETestContext {
        orchestrator: Arc<AutoRunOrchestrator>,
        repository: Arc<Repository>,
        task_service: Arc<TaskService>,
        dependency_service: Arc<DependencyService>,
        claude_service: Arc<ClaudeCodeService>,
        pr_review_service: Arc<PRReviewService>,
        mock_executor: Arc<MockCommandExecutor>,
    }

    async fn setup_e2e_context() -> E2ETestContext {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create mock command executor with realistic responses
        let mock_executor = Arc::new(MockCommandExecutor::new());
        setup_mock_responses(&mock_executor);

        let claude_service = Arc::new(ClaudeCodeService::with_executor(
            repository.claude_code.clone(),
            mock_executor.clone() as Arc<dyn CommandExecutor>,
        ));

        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));

        let pr_review_service = Arc::new(PRReviewService::new(
            repository.clone(),
            mock_executor.clone() as Arc<dyn CommandExecutor>,
        ));

        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository.clone(),
            claude_service.clone(),
            dependency_service.clone(),
            task_service.clone(),
        ));

        E2ETestContext {
            orchestrator,
            repository,
            task_service,
            dependency_service,
            claude_service,
            pr_review_service,
            mock_executor,
        }
    }

    fn setup_mock_responses(mock: &MockCommandExecutor) {
        // Mock Claude Code execution
        mock.add_response(
            "claude",
            vec!["code"],
            "Task completed successfully\nAll tests passed\nPR created: https://github.com/user/repo/pull/123",
            "",
            true,
        );

        // Mock git operations
        mock.add_response("git", vec!["checkout"], "Switched to branch", "", true);
        mock.add_response("git", vec!["merge"], "Merge successful", "", true);
        mock.add_response("git", vec!["push"], "Pushed to remote", "", true);

        // Mock GitHub CLI operations
        mock.add_response(
            "gh",
            vec!["pr", "create"],
            "https://github.com/user/repo/pull/123",
            "",
            true,
        );

        mock.add_response(
            "gh",
            vec!["pr", "view"],
            r#"{
                "title": "Implement feature",
                "body": "This PR implements the feature",
                "headRefName": "feature-branch",
                "baseRefName": "main",
                "author": {"login": "claude"},
                "state": "OPEN"
            }"#,
            "",
            true,
        );

        mock.add_response("gh", vec!["pr", "review"], "Approved", "", true);
        mock.add_response("gh", vec!["pr", "merge"], "Merged successfully", "", true);

        // Mock test execution
        mock.add_response(
            "cargo",
            vec!["test"],
            "test result: ok. 100 passed",
            "",
            true,
        );
    }

    #[tokio::test]
    async fn test_e2e_simple_linear_workflow() {
        let ctx = setup_e2e_context().await;

        // Create a simple linear workflow: Task A -> Task B -> Task C
        let task_a = Task::new("Task A".to_string(), "First task".to_string());
        let task_b = Task::new("Task B".to_string(), "Second task".to_string());
        let task_c = Task::new("Task C".to_string(), "Third task".to_string());

        ctx.repository.tasks.create(&task_a).await.unwrap();
        ctx.repository.tasks.create(&task_b).await.unwrap();
        ctx.repository.tasks.create(&task_c).await.unwrap();

        // Set up dependencies: B depends on A, C depends on B
        ctx.dependency_service
            .add_dependency(task_b.id, task_a.id)
            .await
            .unwrap();
        ctx.dependency_service
            .add_dependency(task_c.id, task_b.id)
            .await
            .unwrap();

        // Start auto-run
        let task_ids = vec![task_a.id, task_b.id, task_c.id];
        let orchestrator = ctx.orchestrator.clone();

        let handle = tokio::spawn(async move {
            orchestrator.start_auto_run(task_ids).await.unwrap();
        });

        // Wait for completion with timeout
        let mut attempts = 0;
        loop {
            sleep(Duration::from_millis(100)).await;
            let status = ctx.orchestrator.get_status().await;

            if status == AutoRunStatus::Completed {
                break;
            }

            attempts += 1;
            if attempts > 100 {
                // 10 second timeout
                panic!("Auto-run did not complete in time");
            }
        }

        // Verify all tasks completed
        let progress = ctx.orchestrator.get_progress().await;
        assert_eq!(progress.total_tasks, 3);
        assert_eq!(progress.completed_tasks, 3);
        assert_eq!(progress.failed_tasks, 0);

        // Verify execution order
        let executions = ctx.orchestrator.get_execution_details().await;
        assert_eq!(executions.len(), 3);

        // Task A should complete before B, B before C
        let exec_a = executions.iter().find(|e| e.task_id == task_a.id).unwrap();
        let exec_b = executions.iter().find(|e| e.task_id == task_b.id).unwrap();
        let exec_c = executions.iter().find(|e| e.task_id == task_c.id).unwrap();

        assert!(exec_a.completed_at.unwrap() <= exec_b.started_at.unwrap());
        assert!(exec_b.completed_at.unwrap() <= exec_c.started_at.unwrap());

        handle.abort();
    }

    #[tokio::test]
    async fn test_e2e_parallel_execution() {
        let ctx = setup_e2e_context().await;

        // Create parallel tasks with a common dependency
        //     A
        //    / \
        //   B   C
        //    \ /
        //     D
        let task_a = Task::new("Task A".to_string(), "Root task".to_string());
        let task_b = Task::new("Task B".to_string(), "Parallel task 1".to_string());
        let task_c = Task::new("Task C".to_string(), "Parallel task 2".to_string());
        let task_d = Task::new("Task D".to_string(), "Final task".to_string());

        ctx.repository.tasks.create(&task_a).await.unwrap();
        ctx.repository.tasks.create(&task_b).await.unwrap();
        ctx.repository.tasks.create(&task_c).await.unwrap();
        ctx.repository.tasks.create(&task_d).await.unwrap();

        // Set up dependencies
        ctx.dependency_service
            .add_dependency(task_b.id, task_a.id)
            .await
            .unwrap();
        ctx.dependency_service
            .add_dependency(task_c.id, task_a.id)
            .await
            .unwrap();
        ctx.dependency_service
            .add_dependency(task_d.id, task_b.id)
            .await
            .unwrap();
        ctx.dependency_service
            .add_dependency(task_d.id, task_c.id)
            .await
            .unwrap();

        // Configure for parallel execution
        ctx.orchestrator
            .update_config(AutoRunConfig {
                max_parallel_instances: 2,
                auto_merge_enabled: true,
                require_tests_pass: true,
                retry_on_failure: false,
                max_retries: 0,
            })
            .await
            .unwrap();

        // Start auto-run
        let task_ids = vec![task_a.id, task_b.id, task_c.id, task_d.id];
        let orchestrator = ctx.orchestrator.clone();

        let handle = tokio::spawn(async move {
            orchestrator.start_auto_run(task_ids).await.unwrap();
        });

        // Monitor execution
        let mut max_parallel = 0;
        let mut attempts = 0;

        loop {
            sleep(Duration::from_millis(100)).await;
            let progress = ctx.orchestrator.get_progress().await;

            max_parallel = max_parallel.max(progress.running_tasks);

            if ctx.orchestrator.get_status().await == AutoRunStatus::Completed {
                break;
            }

            attempts += 1;
            if attempts > 100 {
                panic!("Auto-run did not complete in time");
            }
        }

        // Verify parallel execution occurred
        assert!(
            max_parallel >= 2,
            "Should have run at least 2 tasks in parallel"
        );

        // Verify all tasks completed
        let progress = ctx.orchestrator.get_progress().await;
        assert_eq!(progress.completed_tasks, 4);
        assert_eq!(progress.failed_tasks, 0);

        handle.abort();
    }

    #[tokio::test]
    async fn test_e2e_failure_and_retry() {
        let ctx = setup_e2e_context().await;

        // Set up a task that fails initially then succeeds on retry
        let task = Task::new("Flaky Task".to_string(), "Sometimes fails".to_string());
        ctx.repository.tasks.create(&task).await.unwrap();

        // Mock a failure followed by success
        let mock = ctx.mock_executor.clone();
        mock.add_response("cargo", vec!["test"], "", "test failed", false);
        mock.add_response("cargo", vec!["test"], "test passed", "", true);

        // Configure with retry
        ctx.orchestrator
            .update_config(AutoRunConfig {
                max_parallel_instances: 1,
                auto_merge_enabled: true,
                require_tests_pass: true,
                retry_on_failure: true,
                max_retries: 2,
            })
            .await
            .unwrap();

        // Start auto-run
        let orchestrator = ctx.orchestrator.clone();
        let task_id = task.id;

        let handle = tokio::spawn(async move {
            orchestrator.start_auto_run(vec![task_id]).await.unwrap();
        });

        // Wait for completion
        let mut attempts = 0;
        loop {
            sleep(Duration::from_millis(100)).await;

            if ctx.orchestrator.get_status().await == AutoRunStatus::Completed {
                break;
            }

            attempts += 1;
            if attempts > 100 {
                panic!("Auto-run did not complete in time");
            }
        }

        // Verify task succeeded after retry
        let executions = ctx.orchestrator.get_execution_details().await;
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].retry_count, 1);
        assert_eq!(executions[0].status, TaskExecutionStatus::Completed);

        handle.abort();
    }

    #[tokio::test]
    async fn test_e2e_complex_dependency_graph() {
        let ctx = setup_e2e_context().await;

        // Create a complex dependency graph
        // Level 0: A, B
        // Level 1: C (depends on A), D (depends on A, B)
        // Level 2: E (depends on C, D)
        // Level 3: F (depends on E)

        let tasks = vec![
            Task::new("Task A".to_string(), "Level 0".to_string()),
            Task::new("Task B".to_string(), "Level 0".to_string()),
            Task::new("Task C".to_string(), "Level 1".to_string()),
            Task::new("Task D".to_string(), "Level 1".to_string()),
            Task::new("Task E".to_string(), "Level 2".to_string()),
            Task::new("Task F".to_string(), "Level 3".to_string()),
        ];

        for task in &tasks {
            ctx.repository.tasks.create(task).await.unwrap();
        }

        // Set up dependencies
        ctx.dependency_service
            .add_dependency(tasks[2].id, tasks[0].id)
            .await
            .unwrap(); // C -> A
        ctx.dependency_service
            .add_dependency(tasks[3].id, tasks[0].id)
            .await
            .unwrap(); // D -> A
        ctx.dependency_service
            .add_dependency(tasks[3].id, tasks[1].id)
            .await
            .unwrap(); // D -> B
        ctx.dependency_service
            .add_dependency(tasks[4].id, tasks[2].id)
            .await
            .unwrap(); // E -> C
        ctx.dependency_service
            .add_dependency(tasks[4].id, tasks[3].id)
            .await
            .unwrap(); // E -> D
        ctx.dependency_service
            .add_dependency(tasks[5].id, tasks[4].id)
            .await
            .unwrap(); // F -> E

        // Configure for parallel execution
        ctx.orchestrator
            .update_config(AutoRunConfig {
                max_parallel_instances: 3,
                auto_merge_enabled: true,
                require_tests_pass: true,
                retry_on_failure: false,
                max_retries: 0,
            })
            .await
            .unwrap();

        // Start auto-run
        let task_ids: Vec<Uuid> = tasks.iter().map(|t| t.id).collect();
        let orchestrator = ctx.orchestrator.clone();

        let handle = tokio::spawn(async move {
            orchestrator.start_auto_run(task_ids).await.unwrap();
        });

        // Wait for completion
        let mut attempts = 0;
        loop {
            sleep(Duration::from_millis(100)).await;

            if ctx.orchestrator.get_status().await == AutoRunStatus::Completed {
                break;
            }

            attempts += 1;
            if attempts > 150 {
                panic!("Auto-run did not complete in time");
            }
        }

        // Verify all tasks completed
        let progress = ctx.orchestrator.get_progress().await;
        assert_eq!(progress.completed_tasks, 6);
        assert_eq!(progress.failed_tasks, 0);

        // Verify dependency order was respected
        let executions = ctx.orchestrator.get_execution_details().await;

        let get_exec = |idx: usize| {
            executions
                .iter()
                .find(|e| e.task_id == tasks[idx].id)
                .unwrap()
        };

        // Level 0 should complete before Level 1
        assert!(get_exec(0).completed_at.unwrap() <= get_exec(2).started_at.unwrap());
        assert!(get_exec(1).completed_at.unwrap() <= get_exec(3).started_at.unwrap());

        // Level 1 should complete before Level 2
        assert!(get_exec(2).completed_at.unwrap() <= get_exec(4).started_at.unwrap());
        assert!(get_exec(3).completed_at.unwrap() <= get_exec(4).started_at.unwrap());

        // Level 2 should complete before Level 3
        assert!(get_exec(4).completed_at.unwrap() <= get_exec(5).started_at.unwrap());

        handle.abort();
    }

    #[tokio::test]
    async fn test_e2e_pause_resume_stop() {
        let ctx = setup_e2e_context().await;

        // Create several tasks
        let mut tasks = Vec::new();
        for i in 0..5 {
            let task = Task::new(format!("Task {}", i), "".to_string());
            ctx.repository.tasks.create(&task).await.unwrap();
            tasks.push(task);
        }

        // Configure with slow execution
        ctx.orchestrator
            .update_config(AutoRunConfig {
                max_parallel_instances: 1,
                auto_merge_enabled: true,
                require_tests_pass: true,
                retry_on_failure: false,
                max_retries: 0,
            })
            .await
            .unwrap();

        // Add delay to mock responses
        ctx.mock_executor.add_response_with_delay(
            "claude",
            vec!["slow"],
            "Slow task",
            "",
            true,
            500,
        );

        // Start auto-run
        let task_ids: Vec<Uuid> = tasks.iter().map(|t| t.id).collect();
        let orchestrator = ctx.orchestrator.clone();

        let handle = tokio::spawn(async move { orchestrator.start_auto_run(task_ids).await });

        // Let it run briefly
        sleep(Duration::from_millis(200)).await;
        assert_eq!(ctx.orchestrator.get_status().await, AutoRunStatus::Running);

        // Pause
        ctx.orchestrator.pause().await.unwrap();
        assert_eq!(ctx.orchestrator.get_status().await, AutoRunStatus::Paused);

        let progress_paused = ctx.orchestrator.get_progress().await;

        // Wait and verify no progress while paused
        sleep(Duration::from_millis(200)).await;
        let progress_still_paused = ctx.orchestrator.get_progress().await;
        assert_eq!(
            progress_paused.completed_tasks,
            progress_still_paused.completed_tasks
        );

        // Resume
        ctx.orchestrator.resume().await.unwrap();
        assert_eq!(ctx.orchestrator.get_status().await, AutoRunStatus::Running);

        // Let it run a bit more
        sleep(Duration::from_millis(200)).await;

        // Stop
        ctx.orchestrator.stop().await.unwrap();
        assert_eq!(ctx.orchestrator.get_status().await, AutoRunStatus::Idle);

        // Verify executions were cleared
        let executions = ctx.orchestrator.get_execution_details().await;
        assert_eq!(executions.len(), 0);

        handle.abort();
    }

    #[tokio::test]
    async fn test_e2e_pr_review_integration() {
        let ctx = setup_e2e_context().await;

        // Create a task
        let task = Task::new(
            "Feature Task".to_string(),
            "Implement feature X".to_string(),
        );
        ctx.repository.tasks.create(&task).await.unwrap();

        // Create a mock PR review scenario
        ctx.mock_executor.add_response(
            "gh",
            vec!["pr", "view", "456"],
            r#"{
                "title": "Implement feature X",
                "body": "This PR implements feature X as requested",
                "headRefName": "feature-x",
                "baseRefName": "main",
                "author": {"login": "claude"},
                "state": "OPEN"
            }"#,
            "",
            true,
        );

        // Mock successful tests
        ctx.mock_executor.add_response(
            "cargo",
            vec!["test"],
            "test result: ok. 50 passed",
            "",
            true,
        );

        // Mock no conflicts
        ctx.mock_executor.add_response(
            "git",
            vec!["merge", "--no-commit"],
            "Merge successful",
            "",
            true,
        );

        // Review PR
        let pr_url = "https://github.com/owner/repo/pull/456";
        let review = ctx
            .pr_review_service
            .review_pr(pr_url.to_string())
            .await
            .unwrap();

        assert!(review.approved);
        assert!(review.tests_passed);
        assert!(!review.merge_conflicts);
        assert!(review.comments.iter().any(|c| c.contains("ready to merge")));

        // Approve and merge
        ctx.pr_review_service
            .approve_and_merge(pr_url.to_string())
            .await
            .unwrap();

        // Verify mock was called
        assert!(
            ctx.mock_executor
                .assert_called_with("gh", &["pr", "review", "456"])
        );
        assert!(
            ctx.mock_executor
                .assert_called_with("gh", &["pr", "merge", "456"])
        );
    }

    #[tokio::test]
    async fn test_e2e_concurrent_pr_reviews() {
        let ctx = setup_e2e_context().await;

        // Set up multiple PRs
        for i in 1..=3 {
            let pr_number = format!("{}", 100 + i);
            ctx.mock_executor.add_response(
                "gh",
                vec!["pr", "view", &pr_number],
                &format!(
                    r#"{{
                    "title": "Feature {}",
                    "body": "Implements feature {}",
                    "headRefName": "feature-{}",
                    "baseRefName": "main",
                    "author": {{"login": "claude-{}"}},
                    "state": "OPEN"
                }}"#,
                    i, i, i, i
                ),
                "",
                true,
            );
        }

        // Review multiple PRs concurrently
        let pr_service = ctx.pr_review_service.clone();
        let handles: Vec<_> = (1..=3)
            .map(|i| {
                let service = pr_service.clone();
                let url = format!("https://github.com/owner/repo/pull/{}", 100 + i);
                tokio::spawn(async move { service.review_pr(url).await })
            })
            .collect();

        // Wait for all reviews
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await);
        }

        // Verify all succeeded
        for result in results {
            let review = result.unwrap().unwrap();
            assert!(review.approved);
        }
    }

    #[tokio::test]
    async fn test_e2e_error_recovery() {
        let ctx = setup_e2e_context().await;

        // Create a task that will fail permanently
        let task = Task::new("Broken Task".to_string(), "Always fails".to_string());
        ctx.repository.tasks.create(&task).await.unwrap();

        // Mock permanent failure
        ctx.mock_executor.add_response(
            "cargo",
            vec!["test"],
            "",
            "Compilation error: syntax error",
            false,
        );

        // Configure with retries
        ctx.orchestrator
            .update_config(AutoRunConfig {
                max_parallel_instances: 1,
                auto_merge_enabled: true,
                require_tests_pass: true,
                retry_on_failure: true,
                max_retries: 2,
            })
            .await
            .unwrap();

        // Start auto-run
        let orchestrator = ctx.orchestrator.clone();
        let task_id = task.id;

        let handle = tokio::spawn(async move { orchestrator.start_auto_run(vec![task_id]).await });

        // Wait for completion
        let mut attempts = 0;
        loop {
            sleep(Duration::from_millis(100)).await;
            let status = ctx.orchestrator.get_status().await;

            if status == AutoRunStatus::Completed || matches!(status, AutoRunStatus::Failed(_)) {
                break;
            }

            attempts += 1;
            if attempts > 100 {
                break;
            }
        }

        // Verify task failed after retries
        let executions = ctx.orchestrator.get_execution_details().await;
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].retry_count, 2);
        assert_eq!(executions[0].status, TaskExecutionStatus::Failed);
        assert!(executions[0].error_message.is_some());

        handle.abort();
    }

    #[tokio::test]
    async fn test_e2e_full_workflow_with_metrics() {
        let ctx = setup_e2e_context().await;

        // Create a realistic project structure
        let tasks = vec![
            Task::new(
                "Setup Infrastructure".to_string(),
                "Initialize project".to_string(),
            ),
            Task::new(
                "Implement API".to_string(),
                "Create REST endpoints".to_string(),
            ),
            Task::new("Add Database".to_string(), "Set up PostgreSQL".to_string()),
            Task::new(
                "Write Tests".to_string(),
                "Unit and integration tests".to_string(),
            ),
            Task::new("Documentation".to_string(), "API documentation".to_string()),
            Task::new("Deploy".to_string(), "Deploy to production".to_string()),
        ];

        for task in &tasks {
            ctx.repository.tasks.create(task).await.unwrap();
        }

        // Set up realistic dependencies
        // API depends on Infrastructure
        ctx.dependency_service
            .add_dependency(tasks[1].id, tasks[0].id)
            .await
            .unwrap();
        // Database depends on Infrastructure
        ctx.dependency_service
            .add_dependency(tasks[2].id, tasks[0].id)
            .await
            .unwrap();
        // Tests depend on API and Database
        ctx.dependency_service
            .add_dependency(tasks[3].id, tasks[1].id)
            .await
            .unwrap();
        ctx.dependency_service
            .add_dependency(tasks[3].id, tasks[2].id)
            .await
            .unwrap();
        // Documentation depends on API
        ctx.dependency_service
            .add_dependency(tasks[4].id, tasks[1].id)
            .await
            .unwrap();
        // Deploy depends on Tests and Documentation
        ctx.dependency_service
            .add_dependency(tasks[5].id, tasks[3].id)
            .await
            .unwrap();
        ctx.dependency_service
            .add_dependency(tasks[5].id, tasks[4].id)
            .await
            .unwrap();

        // Configure for realistic execution
        ctx.orchestrator
            .update_config(AutoRunConfig {
                max_parallel_instances: 2,
                auto_merge_enabled: true,
                require_tests_pass: true,
                retry_on_failure: true,
                max_retries: 1,
            })
            .await
            .unwrap();

        // Track metrics
        let start_time = std::time::Instant::now();
        let mut progress_history = Vec::new();

        // Start auto-run
        let task_ids: Vec<Uuid> = tasks.iter().map(|t| t.id).collect();
        let orchestrator = ctx.orchestrator.clone();

        let handle = tokio::spawn(async move { orchestrator.start_auto_run(task_ids).await });

        // Monitor progress
        loop {
            sleep(Duration::from_millis(50)).await;

            let progress = ctx.orchestrator.get_progress().await;
            progress_history.push(progress.clone());

            if ctx.orchestrator.get_status().await == AutoRunStatus::Completed {
                break;
            }

            if progress_history.len() > 200 {
                panic!("Auto-run took too long");
            }
        }

        let elapsed = start_time.elapsed();

        // Verify successful completion
        let final_progress = ctx.orchestrator.get_progress().await;
        assert_eq!(final_progress.completed_tasks, 6);
        assert_eq!(final_progress.failed_tasks, 0);

        // Verify parallelism was utilized
        let max_parallel = progress_history
            .iter()
            .map(|p| p.running_tasks)
            .max()
            .unwrap_or(0);
        assert!(max_parallel >= 2, "Should have utilized parallel execution");

        // Verify reasonable execution time
        assert!(elapsed.as_secs() < 10, "Should complete within 10 seconds");

        // Verify execution order
        let executions = ctx.orchestrator.get_execution_details().await;

        // Infrastructure should complete first
        let infra_exec = executions
            .iter()
            .find(|e| e.task_id == tasks[0].id)
            .unwrap();

        // Deploy should complete last
        let deploy_exec = executions
            .iter()
            .find(|e| e.task_id == tasks[5].id)
            .unwrap();

        assert!(infra_exec.completed_at.unwrap() < deploy_exec.started_at.unwrap());

        println!("Full workflow completed in {:?}", elapsed);
        println!("Maximum parallel tasks: {}", max_parallel);
        println!("Total progress updates: {}", progress_history.len());

        handle.abort();
    }
}
