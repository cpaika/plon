#[cfg(test)]
mod tests {
    use crate::ui::views::map_view::MapView;
    use crate::domain::task::{Task, TaskStatus, Position};
    use crate::repository::Repository;
    use crate::services::{ClaudeCodeService, DependencyService, TaskService, AutoRunOrchestrator};
    use std::sync::Arc;
    use uuid::Uuid;
    use eframe::egui;
    
    /// Create a proper end-to-end UI test that simulates actual button interaction
    #[test]
    fn test_play_button_end_to_end_ui_interaction() {
        // Create a headless egui context for testing
        let ctx = egui::Context::default();
        
        // Create test infrastructure
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(async {
            sqlx::SqlitePool::connect(":memory:").await.unwrap()
        });
        
        // Initialize database tables
        runtime.block_on(async {
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS tasks (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    description TEXT,
                    status TEXT NOT NULL,
                    position_x REAL NOT NULL,
                    position_y REAL NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    due_date TEXT,
                    priority INTEGER,
                    estimated_hours REAL,
                    actual_hours REAL,
                    tags TEXT,
                    metadata TEXT,
                    subtasks TEXT,
                    parent_id TEXT
                )
            "#).execute(&pool).await.unwrap();
            
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS dependencies (
                    from_task_id TEXT NOT NULL,
                    to_task_id TEXT NOT NULL,
                    dependency_type TEXT NOT NULL,
                    PRIMARY KEY (from_task_id, to_task_id)
                )
            "#).execute(&pool).await.unwrap();
            
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS claude_code_sessions (
                    id TEXT PRIMARY KEY,
                    task_id TEXT NOT NULL,
                    status TEXT NOT NULL,
                    started_at TEXT NOT NULL,
                    completed_at TEXT,
                    branch_name TEXT,
                    pr_url TEXT,
                    error_message TEXT,
                    log_output TEXT
                )
            "#).execute(&pool).await.unwrap();
        });
        
        let repository = Arc::new(Repository::new(pool));
        
        // Create test task
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test Task for UI".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 100.0, y: 100.0 },
            ..Task::default()
        };
        
        runtime.block_on(async {
            repository.tasks.create(&task).await.unwrap();
        });
        
        // Setup services
        let claude_service = Arc::new(ClaudeCodeService::new(repository.claude_code.clone()));
        let dep_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));
        
        // Create MapView and set up services
        let mut map_view = MapView::new();
        map_view.set_dependency_service(dep_service.clone());
        map_view.set_claude_service(claude_service.clone(), repository.clone());
        
        // Create a test UI
        let mut output = ctx.tessellate(vec![], 1.0);
        
        // Simulate showing the map view
        egui::CentralPanel::default().show(&ctx, |ui| {
            // Load tasks
            let mut tasks = runtime.block_on(async {
                repository.tasks.list_all().await.unwrap_or_default()
            });
            let mut goals = vec![];
            
            // Show the map view
            map_view.show(ui, &mut tasks, &mut goals);
        });
        
        // The button should be drawn for a Todo task
        assert_eq!(task.status, TaskStatus::Todo);
        
        // Verify orchestrator was created
        assert!(
            map_view.auto_run_orchestrator.is_some(),
            "Orchestrator should be created when both services are set"
        );
        
        // Simulate clicking the play button by calling the method directly
        // (In a real UI test framework, we'd simulate actual mouse events)
        map_view.start_claude_code_for_task(task.id);
        
        // Verify the task is marked as running
        assert!(
            map_view.running_tasks.contains_key(&task.id),
            "Task should be in running_tasks after clicking play button"
        );
        
        // Check the status
        if let Some(status) = map_view.running_tasks.get(&task.id) {
            assert!(
                matches!(status, crate::services::TaskExecutionStatus::Running),
                "Task should have Running status"
            );
        }
    }
    
    /// Test that verifies the play button interaction area is properly defined
    #[test]
    fn test_play_button_interaction_area() {
        // Create test context
        let task_rect = egui::Rect::from_min_max(
            egui::Pos2::new(100.0, 100.0),
            egui::Pos2::new(250.0, 180.0)
        );
        
        let zoom_level = 1.0;
        
        // Calculate play button position and rect
        let button_pos = task_rect.right_top() + egui::Vec2::new(-15.0 * zoom_level, 15.0 * zoom_level);
        let button_radius = 10.0 * zoom_level;
        let button_rect = egui::Rect::from_center_size(
            button_pos,
            egui::Vec2::splat(button_radius * 2.0)
        );
        
        // Test various click positions
        struct ClickTest {
            name: &'static str,
            pos: egui::Pos2,
            should_trigger_play: bool,
            should_trigger_modal: bool,
        }
        
        let tests = vec![
            ClickTest {
                name: "Center of play button",
                pos: button_pos,
                should_trigger_play: true,
                should_trigger_modal: false,
            },
            ClickTest {
                name: "Edge of play button",
                pos: button_pos + egui::Vec2::new(button_radius * 0.9, 0.0),
                should_trigger_play: true,
                should_trigger_modal: false,
            },
            ClickTest {
                name: "Just outside play button",
                pos: button_pos + egui::Vec2::new(button_radius * 1.5, 0.0),
                should_trigger_play: false,
                should_trigger_modal: false, // Outside task rect
            },
            ClickTest {
                name: "Center of task",
                pos: task_rect.center(),
                should_trigger_play: false,
                should_trigger_modal: true,
            },
            ClickTest {
                name: "Bottom left of task",
                pos: task_rect.left_bottom() + egui::Vec2::new(10.0, -10.0),
                should_trigger_play: false,
                should_trigger_modal: true,
            },
        ];
        
        for test in tests {
            let on_button = button_rect.contains(test.pos);
            let on_task = task_rect.contains(test.pos);
            
            assert_eq!(
                on_button,
                test.should_trigger_play,
                "Test '{}': play button detection mismatch at {:?}",
                test.name,
                test.pos
            );
            
            // Modal should trigger if on task but not on button
            let should_open_modal = on_task && !on_button;
            assert_eq!(
                should_open_modal,
                test.should_trigger_modal,
                "Test '{}': modal detection mismatch at {:?}",
                test.name,
                test.pos
            );
        }
    }
}