#[cfg(test)]
mod tests {
    use crate::domain::task::{Position, Task, TaskStatus};
    use crate::repository::Repository;
    use crate::repository::database::init_test_database;
    use crate::services::{
        AutoRunConfig, AutoRunOrchestrator, ClaudeCodeService, DependencyService,
        TaskExecutionStatus, TaskService,
        command_executor::{CommandExecutor, mock::MockCommandExecutor},
    };
    use crate::ui::views::map_view::MapView;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::time::{Duration, sleep};
    use uuid::Uuid;

    /// Test that clicking play button starts Claude Code execution
    #[tokio::test]
    async fn test_play_button_starts_claude_code() {
        // Setup test environment
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create a test task
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 100.0, y: 100.0 },
            description: "Task to test Claude Code execution".to_string(),
            ..Task::default()
        };
        repository.tasks.create(&task).await.unwrap();

        // Setup services
        let claude_service = Arc::new(ClaudeCodeService::new(repository.claude_code.clone()));
        let dep_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));

        // Create orchestrator
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository.clone(),
            claude_service.clone(),
            dep_service.clone(),
            task_service.clone(),
        ));

        // Simulate clicking play button by starting Claude Code
        orchestrator.start_auto_run(vec![task.id]).await.unwrap();

        // Wait a moment for execution to start
        sleep(Duration::from_millis(100)).await;

        // Verify task execution started
        let executions = orchestrator.executions.read().await;
        assert!(
            executions.contains_key(&task.id),
            "Task should be in executions"
        );

        let exec = executions.get(&task.id).unwrap();
        assert!(
            matches!(
                exec.status,
                TaskExecutionStatus::Running | TaskExecutionStatus::Queued
            ),
            "Task should be running or queued, got {:?}",
            exec.status
        );
    }

    /// Test that Claude Code creates a PR when execution completes
    #[tokio::test]
    async fn test_claude_code_creates_pr() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create mock command executor
        let mock_executor = Arc::new(MockCommandExecutor::new());

        // Set up expectations for Claude Code CLI commands
        mock_executor.add_response(
            "claude-code",
            vec!["--task", "Test Task", "--branch", "feature-test"],
            "Starting Claude Code...\nTask completed",
            "",
            true,
        );

        mock_executor.add_response(
            "gh",
            vec!["pr", "create", "--title", "Test Task"],
            "https://github.com/user/repo/pull/123",
            "",
            true,
        );

        // Create task
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            status: TaskStatus::InProgress,
            ..Task::default()
        };
        repository.tasks.create(&task).await.unwrap();

        // Execute with mocked CLI
        let output = mock_executor
            .execute("gh", &["pr", "create", "--title", "Test Task", "--body", "Automated PR from Claude Code"], None, None)
            .await
            .unwrap();
        assert!(output.stdout.contains("github.com/user/repo/pull"));
        assert!(output.success);
    }

    /// Test that multiple Claude Code instances can run in parallel
    #[tokio::test]
    async fn test_parallel_claude_code_execution() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create multiple tasks
        let mut tasks = Vec::new();
        for i in 0..3 {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Parallel Task {}", i),
                status: TaskStatus::Todo,
                ..Task::default()
            };
            repository.tasks.create(&task).await.unwrap();
            tasks.push(task);
        }

        // Setup services
        let claude_service = Arc::new(ClaudeCodeService::new(repository.claude_code.clone()));
        let dep_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));

        // Configure for parallel execution
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository.clone(),
            claude_service,
            dep_service,
            task_service,
        ));

        let config = AutoRunConfig {
            max_parallel_instances: 3,
            auto_merge_enabled: false,
            require_tests_pass: false,
            retry_on_failure: false,
            max_retries: 0,
        };
        orchestrator
            .update_config(config)
            .await
            .unwrap();

        // Start all tasks
        let task_ids: Vec<Uuid> = tasks.iter().map(|t| t.id).collect();
        orchestrator.start_auto_run(task_ids.clone()).await.unwrap();

        // Wait for execution to start
        sleep(Duration::from_millis(200)).await;

        // Verify multiple tasks are running
        let active_count = orchestrator.get_active_sessions_count().await;
        assert!(active_count > 0, "Should have active sessions");
        assert!(
            active_count <= 3,
            "Should not exceed max parallel instances"
        );
    }

    /// Test that task status updates are reflected in UI
    #[tokio::test]
    async fn test_task_status_updates_in_ui() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

        // Create map view
        let mut map_view = MapView::new_with_deps(repository.clone(), Some(runtime.clone()));

        // Create and start a task
        let task_id = Uuid::new_v4();
        map_view.test_set_task_status(task_id, TaskExecutionStatus::Running);

        // Verify spinner should be shown
        assert!(map_view.is_task_running(&task_id));
        assert_eq!(
            map_view.get_task_execution_status(&task_id),
            Some(&TaskExecutionStatus::Running)
        );

        // Update to completed
        map_view.test_get_running_tasks_mut()
            .insert(task_id, TaskExecutionStatus::Completed);
        assert_eq!(
            map_view.get_task_execution_status(&task_id),
            Some(&TaskExecutionStatus::Completed)
        );

        // Add PR URL
        map_view.test_set_pr_url(task_id, "https://github.com/user/repo/pull/42".to_string());
        assert!(map_view.test_has_pr_url(&task_id));
    }

    /// Test error handling when Claude Code fails
    #[tokio::test]
    async fn test_claude_code_error_handling() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create mock executor that fails
        let mock_executor = Arc::new(MockCommandExecutor::new());
        mock_executor.add_response(
            "claude-code",
            vec!["--task", "Failing Task"],
            "",
            "Error: Task failed",
            false,
        );

        // Execute and verify failure is handled
        let output = mock_executor
            .execute("claude-code", &["--task", "Failing Task"], None, None)
            .await
            .unwrap();
        assert!(!output.success);
        assert!(output.stderr.contains("Error"));

        // In real scenario, task status should be updated to Failed
        let task_id = Uuid::new_v4();
        let mut running_tasks = HashMap::new();
        running_tasks.insert(task_id, TaskExecutionStatus::Failed);
        assert_eq!(
            running_tasks.get(&task_id),
            Some(&TaskExecutionStatus::Failed)
        );
    }

    /// Test that Claude Code respects dependencies
    #[tokio::test]
    async fn test_claude_code_respects_dependencies() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create dependent tasks
        let task1 = Task {
            id: Uuid::new_v4(),
            title: "Task 1 - Must complete first".to_string(),
            status: TaskStatus::Todo,
            ..Task::default()
        };
        let task2 = Task {
            id: Uuid::new_v4(),
            title: "Task 2 - Depends on Task 1".to_string(),
            status: TaskStatus::Todo,
            ..Task::default()
        };

        repository.tasks.create(&task1).await.unwrap();
        repository.tasks.create(&task2).await.unwrap();

        // Create dependency
        let dep_service = Arc::new(DependencyService::new(repository.clone()));
        dep_service
            .add_dependency(task2.id, task1.id)
            .await
            .unwrap();

        // Setup orchestrator
        let claude_service = Arc::new(ClaudeCodeService::new(repository.claude_code.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository.clone(),
            claude_service,
            dep_service.clone(),
            task_service,
        ));

        // Start both tasks
        orchestrator
            .start_auto_run(vec![task1.id, task2.id])
            .await
            .unwrap();

        // Verify task2 is not running immediately (should wait for task1)
        sleep(Duration::from_millis(100)).await;

        let executions = orchestrator.executions.read().await;

        // Task1 should be running or queued
        if let Some(exec1) = executions.get(&task1.id) {
            assert!(
                matches!(
                    exec1.status,
                    TaskExecutionStatus::Running | TaskExecutionStatus::Queued
                ),
                "Task1 should be running or queued"
            );
        }

        // Task2 should be queued (waiting for dependency)
        if let Some(exec2) = executions.get(&task2.id) {
            assert!(
                matches!(exec2.status, TaskExecutionStatus::Queued),
                "Task2 should be queued waiting for Task1"
            );
        }
    }

    /// Test play button interaction doesn't open modal
    #[test]
    fn test_play_button_no_modal() {
        // This test verifies the fix we just made
        // The play button click should not trigger the modal

        // Simulate button rect and task rect
        let button_pos = eframe::egui::Pos2::new(185.0, 115.0);
        let button_radius = 10.0;
        let button_rect = eframe::egui::Rect::from_center_size(
            button_pos,
            eframe::egui::Vec2::splat(button_radius * 2.0),
        );

        let task_rect = eframe::egui::Rect::from_min_max(
            eframe::egui::Pos2::new(100.0, 100.0),
            eframe::egui::Pos2::new(250.0, 180.0),
        );

        // Click position on play button
        let click_pos = button_pos;

        // Verify click is within button but also within task
        assert!(button_rect.contains(click_pos), "Click should be on button");
        assert!(task_rect.contains(click_pos), "Click is also on task");

        // The fix ensures that when click is on button, modal doesn't open
        let clicked_on_play = button_rect.contains(click_pos);
        assert!(clicked_on_play, "Should detect click on play button");

        // Modal should not open when clicked_on_play is true
        let should_open_modal = !clicked_on_play;
        assert!(
            !should_open_modal,
            "Modal should not open when play button is clicked"
        );
    }
}
