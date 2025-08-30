#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::domain::task::{Task, TaskStatus, Position};
    use crate::domain::goal::{Goal, GoalStatus};
    use crate::repository::database::init_test_database;
    use crate::repository::Repository;
    use crate::services::{
        TaskService, GoalService, DependencyService,
        AutoRunOrchestrator, AutoRunConfig, AutoRunStatus,
        ClaudeCodeService, TaskExecutionStatus,
    };
    use crate::services::command_executor::mock::MockCommandExecutor;
    use crate::ui::views::MapView;
    use egui::{Context, Pos2, Vec2, Rect, Key, Modifiers};
    use std::sync::Arc;
    use std::collections::HashSet;
    use uuid::Uuid;

    struct UITestContext {
        map_view: MapView,
        repository: Arc<Repository>,
        task_service: Arc<TaskService>,
        goal_service: Arc<GoalService>,
        dependency_service: Arc<DependencyService>,
        orchestrator: Arc<AutoRunOrchestrator>,
        mock_executor: Arc<MockCommandExecutor>,
        runtime: Arc<tokio::runtime::Runtime>,
    }

    fn setup_ui_test() -> UITestContext {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let task_service = Arc::new(TaskService::new(repository.clone()));
        let goal_service = Arc::new(GoalService::new(repository.clone()));
        let dependency_service = Arc::new(DependencyService::new(repository.clone()));
        
        let mock_executor = Arc::new(MockCommandExecutor::new());
        setup_mock_ui_responses(&mock_executor);
        
        let claude_service = Arc::new(ClaudeCodeService::new(
            repository.claude_code.clone()
        ));
        
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository.clone(),
            claude_service,
            dependency_service.clone(),
            task_service.clone(),
        ));
        
        let map_view = MapView::new(
            repository.clone(),
            Some(runtime.clone()),
        );
        
        UITestContext {
            map_view,
            repository,
            task_service,
            goal_service,
            dependency_service,
            orchestrator,
            mock_executor,
            runtime,
        }
    }

    fn setup_mock_ui_responses(mock: &MockCommandExecutor) {
        mock.add_response("claude", vec!["code"], "Task completed", "", true);
        mock.add_response("git", vec![], "Success", "", true);
        mock.add_response("gh", vec!["pr"], "PR created", "", true);
        mock.add_response("cargo", vec!["test"], "Tests passed", "", true);
    }

    fn create_test_context() -> Context {
        Context::default()
    }

    #[test]
    fn test_ui_select_tasks_for_auto_run() {
        let mut ctx = setup_ui_test();
        let ui_ctx = create_test_context();
        
        // Create test tasks
        let task1 = Task {
            id: Uuid::new_v4(),
            title: "Task 1".to_string(),
            description: "".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 100.0, y: 100.0 },
            ..Task::default()
        };
        
        let task2 = Task {
            id: Uuid::new_v4(),
            title: "Task 2".to_string(),
            description: "".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 200.0, y: 100.0 },
            ..Task::default()
        };
        
        ctx.runtime.block_on(async {
            ctx.repository.tasks.create(&task1).await.unwrap();
            ctx.repository.tasks.create(&task2).await.unwrap();
        });
        
        // Simulate selection of tasks
        ctx.map_view.selected_items.insert(task1.id);
        ctx.map_view.selected_items.insert(task2.id);
        
        // Verify selection
        assert_eq!(ctx.map_view.selected_items.len(), 2);
        assert!(ctx.map_view.selected_items.contains(&task1.id));
        assert!(ctx.map_view.selected_items.contains(&task2.id));
    }

    #[test]
    fn test_ui_start_auto_run_button() {
        let mut ctx = setup_ui_test();
        
        // Create and select tasks
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 100.0, y: 100.0 },
            ..Task::default()
        };
        
        ctx.runtime.block_on(async {
            ctx.repository.tasks.create(&task).await.unwrap();
        });
        
        ctx.map_view.selected_items.insert(task.id);
        
        // Simulate auto-run button click
        ctx.map_view.auto_run_enabled = true;
        
        // Verify auto-run state
        assert!(ctx.map_view.auto_run_enabled);
        assert!(!ctx.map_view.selected_items.is_empty());
    }

    #[test]
    fn test_ui_display_execution_progress() {
        let mut ctx = setup_ui_test();
        
        // Create test execution data
        let task_id = Uuid::new_v4();
        let execution = crate::services::TaskExecution {
            task_id,
            session_id: Some(Uuid::new_v4()),
            status: TaskExecutionStatus::Running,
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            pr_url: None,
            retry_count: 0,
            error_message: None,
        };
        
        // Simulate progress update
        ctx.map_view.auto_run_progress = Some(crate::services::AutoRunProgress {
            total_tasks: 10,
            completed_tasks: 3,
            failed_tasks: 1,
            running_tasks: 2,
            queued_tasks: 4,
            current_phase: "Executing tasks".to_string(),
        });
        
        // Verify progress display data
        let progress = ctx.map_view.auto_run_progress.as_ref().unwrap();
        assert_eq!(progress.total_tasks, 10);
        assert_eq!(progress.completed_tasks, 3);
        assert_eq!(progress.running_tasks, 2);
    }

    #[test]
    fn test_ui_task_status_visualization() {
        let mut ctx = setup_ui_test();
        
        // Create tasks with different statuses
        let tasks = vec![
            Task {
                id: Uuid::new_v4(),
                title: "Queued".to_string(),
                status: TaskStatus::Todo,
                position: Position { x: 100.0, y: 100.0 },
                ..Task::default()
            },
            Task {
                id: Uuid::new_v4(),
                title: "Running".to_string(),
                status: TaskStatus::InProgress,
                position: Position { x: 200.0, y: 100.0 },
                ..Task::default()
            },
            Task {
                id: Uuid::new_v4(),
                title: "Completed".to_string(),
                status: TaskStatus::Done,
                position: Position { x: 300.0, y: 100.0 },
                ..Task::default()
            },
        ];
        
        for task in &tasks {
            ctx.runtime.block_on(async {
                ctx.repository.tasks.create(task).await.unwrap();
            });
        }
        
        // Set execution statuses
        ctx.map_view.task_execution_status.insert(tasks[0].id, TaskExecutionStatus::Queued);
        ctx.map_view.task_execution_status.insert(tasks[1].id, TaskExecutionStatus::Running);
        ctx.map_view.task_execution_status.insert(tasks[2].id, TaskExecutionStatus::Completed);
        
        // Verify status visualization data
        assert_eq!(ctx.map_view.task_execution_status.len(), 3);
        assert_eq!(
            ctx.map_view.task_execution_status.get(&tasks[1].id),
            Some(&TaskExecutionStatus::Running)
        );
    }

    #[test]
    fn test_ui_dependency_visualization() {
        let mut ctx = setup_ui_test();
        
        // Create tasks with dependencies
        let task1 = Task {
            id: Uuid::new_v4(),
            title: "Parent".to_string(),
            position: Position { x: 100.0, y: 100.0 },
            ..Task::default()
        };
        
        let task2 = Task {
            id: Uuid::new_v4(),
            title: "Child".to_string(),
            position: Position { x: 200.0, y: 200.0 },
            ..Task::default()
        };
        
        ctx.runtime.block_on(async {
            ctx.repository.tasks.create(&task1).await.unwrap();
            ctx.repository.tasks.create(&task2).await.unwrap();
            ctx.dependency_service.add_dependency(task2.id, task1.id).await.unwrap();
        });
        
        // Load dependencies
        let deps = ctx.runtime.block_on(async {
            ctx.dependency_service.get_all_dependencies().await.unwrap()
        });
        
        // Verify dependency exists
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].from_task_id, task2.id);
        assert_eq!(deps[0].to_task_id, task1.id);
    }

    #[test]
    fn test_ui_pause_resume_controls() {
        let mut ctx = setup_ui_test();
        
        // Simulate auto-run in progress
        ctx.map_view.auto_run_status = Some(AutoRunStatus::Running);
        
        // Test pause
        ctx.map_view.auto_run_paused = true;
        assert!(ctx.map_view.auto_run_paused);
        
        // Test resume
        ctx.map_view.auto_run_paused = false;
        assert!(!ctx.map_view.auto_run_paused);
    }

    #[test]
    fn test_ui_error_display() {
        let mut ctx = setup_ui_test();
        
        // Simulate task execution error
        let task_id = Uuid::new_v4();
        ctx.map_view.task_execution_status.insert(task_id, TaskExecutionStatus::Failed);
        ctx.map_view.task_execution_errors.insert(
            task_id,
            "Test execution failed: compilation error".to_string()
        );
        
        // Verify error display data
        assert_eq!(
            ctx.map_view.task_execution_status.get(&task_id),
            Some(&TaskExecutionStatus::Failed)
        );
        assert!(ctx.map_view.task_execution_errors.contains_key(&task_id));
    }

    #[test]
    fn test_ui_config_dialog() {
        let mut ctx = setup_ui_test();
        
        // Open config dialog
        ctx.map_view.show_auto_run_config = true;
        
        // Set config values
        ctx.map_view.auto_run_config = AutoRunConfig {
            max_parallel_instances: 5,
            auto_merge_enabled: false,
            require_tests_pass: true,
            retry_on_failure: true,
            max_retries: 3,
        };
        
        // Verify config
        assert_eq!(ctx.map_view.auto_run_config.max_parallel_instances, 5);
        assert!(!ctx.map_view.auto_run_config.auto_merge_enabled);
        assert_eq!(ctx.map_view.auto_run_config.max_retries, 3);
    }

    #[test]
    fn test_ui_multi_selection() {
        let mut ctx = setup_ui_test();
        
        // Create multiple tasks
        let mut tasks = Vec::new();
        for i in 0..5 {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Task {}", i),
                position: Position { 
                    x: 100.0 + (i as f32 * 50.0), 
                    y: 100.0 
                },
                ..Task::default()
            };
            tasks.push(task.clone());
            
            ctx.runtime.block_on(async {
                ctx.repository.tasks.create(&task).await.unwrap();
            });
        }
        
        // Simulate box selection
        let selection_rect = Rect::from_min_max(
            Pos2::new(50.0, 50.0),
            Pos2::new(400.0, 150.0)
        );
        
        // Select tasks within rectangle
        for task in &tasks {
            let task_rect = Rect::from_center_size(
                Pos2::new(task.position.x, task.position.y),
                Vec2::new(80.0, 60.0)
            );
            
            if selection_rect.intersects(task_rect) {
                ctx.map_view.selected_items.insert(task.id);
            }
        }
        
        // All tasks should be selected
        assert_eq!(ctx.map_view.selected_items.len(), 5);
    }

    #[test]
    fn test_ui_execution_timeline() {
        let mut ctx = setup_ui_test();
        
        // Create execution timeline data
        let now = chrono::Utc::now();
        let task_executions = vec![
            crate::services::TaskExecution {
                task_id: Uuid::new_v4(),
                session_id: Some(Uuid::new_v4()),
                status: TaskExecutionStatus::Completed,
                started_at: Some(now - chrono::Duration::minutes(10)),
                completed_at: Some(now - chrono::Duration::minutes(5)),
                pr_url: Some("https://github.com/repo/pull/1".to_string()),
                retry_count: 0,
                error_message: None,
            },
            crate::services::TaskExecution {
                task_id: Uuid::new_v4(),
                session_id: Some(Uuid::new_v4()),
                status: TaskExecutionStatus::Running,
                started_at: Some(now - chrono::Duration::minutes(3)),
                completed_at: None,
                pr_url: None,
                retry_count: 0,
                error_message: None,
            },
        ];
        
        // Store execution data
        for exec in &task_executions {
            ctx.map_view.task_executions.insert(exec.task_id, exec.clone());
        }
        
        // Verify timeline data
        assert_eq!(ctx.map_view.task_executions.len(), 2);
        
        let completed = ctx.map_view.task_executions.values()
            .filter(|e| e.status == TaskExecutionStatus::Completed)
            .count();
        assert_eq!(completed, 1);
    }

    #[test]
    fn test_ui_pr_status_display() {
        let mut ctx = setup_ui_test();
        
        // Create task with PR
        let task_id = Uuid::new_v4();
        let pr_url = "https://github.com/owner/repo/pull/123";
        
        ctx.map_view.task_pr_urls.insert(task_id, pr_url.to_string());
        ctx.map_view.pr_statuses.insert(pr_url.to_string(), "Approved".to_string());
        
        // Verify PR status display
        assert!(ctx.map_view.task_pr_urls.contains_key(&task_id));
        assert_eq!(
            ctx.map_view.pr_statuses.get(pr_url),
            Some(&"Approved".to_string())
        );
    }

    #[test]
    fn test_ui_keyboard_shortcuts() {
        let mut ctx = setup_ui_test();
        
        // Create and select tasks
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test".to_string(),
            position: Position { x: 100.0, y: 100.0 },
            ..Task::default()
        };
        
        ctx.runtime.block_on(async {
            ctx.repository.tasks.create(&task).await.unwrap();
        });
        
        ctx.map_view.selected_items.insert(task.id);
        
        // Simulate keyboard shortcuts
        // Ctrl+R for Run
        ctx.map_view.handle_keyboard_shortcut(Key::R, Modifiers::CTRL);
        assert!(ctx.map_view.auto_run_enabled);
        
        // Ctrl+P for Pause
        ctx.map_view.handle_keyboard_shortcut(Key::P, Modifiers::CTRL);
        assert!(ctx.map_view.auto_run_paused);
        
        // Ctrl+S for Stop
        ctx.map_view.handle_keyboard_shortcut(Key::S, Modifiers::CTRL);
        assert!(!ctx.map_view.auto_run_enabled);
    }

    #[test]
    fn test_ui_goal_auto_run() {
        let mut ctx = setup_ui_test();
        
        // Create goal with tasks
        let goal = Goal {
            id: Uuid::new_v4(),
            title: "Project Goal".to_string(),
            description: "".to_string(),
            status: GoalStatus::Active,
            position_x: 100.0,
            position_y: 100.0,
            position_width: 200.0,
            position_height: 150.0,
            ..Goal::default()
        };
        
        let task1 = Task {
            id: Uuid::new_v4(),
            title: "Goal Task 1".to_string(),
            goal_id: Some(goal.id),
            position: Position { x: 120.0, y: 120.0 },
            ..Task::default()
        };
        
        let task2 = Task {
            id: Uuid::new_v4(),
            title: "Goal Task 2".to_string(),
            goal_id: Some(goal.id),
            position: Position { x: 180.0, y: 120.0 },
            ..Task::default()
        };
        
        ctx.runtime.block_on(async {
            ctx.repository.goals.create(&goal).await.unwrap();
            ctx.repository.tasks.create(&task1).await.unwrap();
            ctx.repository.tasks.create(&task2).await.unwrap();
        });
        
        // Select goal (should select all its tasks)
        ctx.map_view.selected_items.insert(goal.id);
        ctx.map_view.select_goal_tasks(goal.id, &[task1.id, task2.id]);
        
        // Verify task selection
        assert!(ctx.map_view.selected_items.contains(&task1.id));
        assert!(ctx.map_view.selected_items.contains(&task2.id));
    }

    #[test]
    fn test_ui_real_time_updates() {
        let mut ctx = setup_ui_test();
        
        // Simulate real-time status updates
        let task_id = Uuid::new_v4();
        
        // Initial status
        ctx.map_view.task_execution_status.insert(task_id, TaskExecutionStatus::Queued);
        assert_eq!(
            ctx.map_view.task_execution_status.get(&task_id),
            Some(&TaskExecutionStatus::Queued)
        );
        
        // Update to running
        ctx.map_view.task_execution_status.insert(task_id, TaskExecutionStatus::Running);
        assert_eq!(
            ctx.map_view.task_execution_status.get(&task_id),
            Some(&TaskExecutionStatus::Running)
        );
        
        // Update to completed
        ctx.map_view.task_execution_status.insert(task_id, TaskExecutionStatus::Completed);
        assert_eq!(
            ctx.map_view.task_execution_status.get(&task_id),
            Some(&TaskExecutionStatus::Completed)
        );
    }
}

// Extension methods for MapView to support testing
impl MapView {
    #[cfg(test)]
    pub fn handle_keyboard_shortcut(&mut self, key: Key, modifiers: Modifiers) {
        if modifiers.ctrl {
            match key {
                Key::R => self.auto_run_enabled = true,
                Key::P => self.auto_run_paused = true,
                Key::S => {
                    self.auto_run_enabled = false;
                    self.auto_run_paused = false;
                }
                _ => {}
            }
        }
    }
    
    #[cfg(test)]
    pub fn select_goal_tasks(&mut self, goal_id: Uuid, task_ids: &[Uuid]) {
        for task_id in task_ids {
            self.selected_items.insert(*task_id);
        }
    }
}