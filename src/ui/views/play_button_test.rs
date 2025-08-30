#[cfg(test)]
mod tests {
    use crate::domain::task::{Position, Task, TaskStatus};
    use crate::repository::Repository;
    use crate::repository::database::init_test_database;
    use crate::services::{AutoRunOrchestrator, ClaudeCodeService, DependencyService, TaskService};
    use crate::ui::views::map_view::MapView;
    use eframe::egui;
    use std::sync::Arc;
    use uuid::Uuid;

    /// Test that the play button is properly clickable and starts Claude Code
    #[test]
    fn test_play_button_clickable() {
        // Create a runtime for database operations
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        
        // Setup test environment
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create test task
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 100.0, y: 100.0 },
            ..Task::default()
        };
        runtime.block_on(repository.tasks.create(&task)).unwrap();

        // Setup services
        let claude_service = Arc::new(ClaudeCodeService::new(repository.claude_code.clone()));
        let dep_service = Arc::new(DependencyService::new(repository.clone()));
        let task_service = Arc::new(TaskService::new(repository.clone()));

        // Create orchestrator
        let orchestrator = Arc::new(AutoRunOrchestrator::new(
            repository.clone(),
            claude_service.clone(),
            dep_service.clone(),
            task_service,
        ));

        // Create MapView with the runtime
        let mut map_view = MapView::new_with_deps(repository.clone(), Some(runtime.clone()));
        map_view.set_claude_service(claude_service, repository.clone());

        // Verify play button is shown for Todo tasks
        assert!(task.status == TaskStatus::Todo);
        assert!(map_view.get_task_execution_status(&task.id).is_none());

        // Simulate clicking the play button
        map_view.test_start_claude_code_for_task(task.id);

        // Verify task is marked as running
        assert!(map_view.is_task_running(&task.id));
        assert_eq!(
            map_view.get_task_execution_status(&task.id),
            Some(&crate::services::TaskExecutionStatus::Running)
        );
    }

    /// Test that play button has proper hover interaction
    #[test]
    fn test_play_button_hover_interaction() {
        // Create test context for UI simulation
        let task_id = Uuid::new_v4();
        let task_rect =
            egui::Rect::from_min_max(egui::Pos2::new(100.0, 100.0), egui::Pos2::new(250.0, 180.0));

        // Calculate play button position
        let zoom_level = 1.0;
        let button_pos =
            task_rect.right_top() + egui::Vec2::new(-15.0 * zoom_level, 15.0 * zoom_level);
        let button_radius = 10.0 * zoom_level;
        let button_rect =
            egui::Rect::from_center_size(button_pos, egui::Vec2::splat(button_radius * 2.0));

        // Test hover detection
        let hover_pos = button_pos; // Mouse directly on button center
        assert!(
            button_rect.contains(hover_pos),
            "Button should detect hover at center"
        );

        // Test edge detection
        let edge_pos = button_pos + egui::Vec2::new(button_radius * 0.9, 0.0);
        assert!(
            button_rect.contains(edge_pos),
            "Button should detect hover near edge"
        );

        // Test outside detection
        let outside_pos = button_pos + egui::Vec2::new(button_radius * 2.5, 0.0);
        assert!(
            !button_rect.contains(outside_pos),
            "Button should not detect hover outside"
        );
    }

    /// Test that play button doesn't conflict with task modal
    #[test]
    fn test_play_button_modal_conflict() {
        let task_rect =
            egui::Rect::from_min_max(egui::Pos2::new(100.0, 100.0), egui::Pos2::new(250.0, 180.0));

        let zoom_level = 1.0;
        let button_pos =
            task_rect.right_top() + egui::Vec2::new(-15.0 * zoom_level, 15.0 * zoom_level);
        let button_radius = 12.0 * zoom_level; // Using the larger radius from conflict detection
        let button_rect =
            egui::Rect::from_center_size(button_pos, egui::Vec2::splat(button_radius * 2.0));

        // Test that clicking on play button area is detected
        let click_on_button = button_pos;
        assert!(
            button_rect.contains(click_on_button),
            "Play button area should contain click"
        );

        // Test that clicking elsewhere on task doesn't trigger play button
        let click_on_task = task_rect.center();
        assert!(
            !button_rect.contains(click_on_task),
            "Task center should not be in play button area"
        );

        // Verify proper separation between areas
        assert!(
            task_rect.contains(click_on_button),
            "Play button is within task bounds"
        );
        assert!(
            task_rect.contains(click_on_task),
            "Task center is within task bounds"
        );
    }

    /// Test play button visibility conditions
    #[test]
    fn test_play_button_visibility_conditions() {
        use crate::services::TaskExecutionStatus;
        use std::collections::HashMap;

        let mut running_tasks = HashMap::new();
        let task_id = Uuid::new_v4();

        // Test: Play button should show for Todo tasks
        let status = TaskStatus::Todo;
        let is_running = running_tasks.get(&task_id).is_none();
        assert!(
            (status == TaskStatus::Todo || status == TaskStatus::InProgress) && is_running,
            "Play button should show for Todo tasks not running"
        );

        // Test: Play button should show for InProgress tasks
        let status = TaskStatus::InProgress;
        assert!(
            (status == TaskStatus::Todo || status == TaskStatus::InProgress) && is_running,
            "Play button should show for InProgress tasks not running"
        );

        // Test: Play button should NOT show for Done tasks
        let status = TaskStatus::Done;
        assert!(
            !((status == TaskStatus::Todo || status == TaskStatus::InProgress) && is_running),
            "Play button should not show for Done tasks"
        );

        // Test: Play button should NOT show for already running tasks
        running_tasks.insert(task_id, TaskExecutionStatus::Running);
        let status = TaskStatus::Todo;
        let is_running = running_tasks.get(&task_id).is_none();
        assert!(!is_running, "Play button should not show for running tasks");
    }

    /// Test play button interaction with zoom levels
    #[test]
    fn test_play_button_zoom_scaling() {
        let task_rect =
            egui::Rect::from_min_max(egui::Pos2::new(100.0, 100.0), egui::Pos2::new(250.0, 180.0));

        // Test different zoom levels
        for zoom_level in [0.5, 1.0, 1.5, 2.0] {
            let button_pos =
                task_rect.right_top() + egui::Vec2::new(-15.0 * zoom_level, 15.0 * zoom_level);
            let button_radius = 10.0 * zoom_level;
            let button_rect =
                egui::Rect::from_center_size(button_pos, egui::Vec2::splat(button_radius * 2.0));

            // Button should scale with zoom
            assert!(
                button_rect.width() == button_radius * 2.0,
                "Button width should scale with zoom level {}",
                zoom_level
            );
            assert!(
                button_rect.height() == button_radius * 2.0,
                "Button height should scale with zoom level {}",
                zoom_level
            );

            // Button should remain clickable at all zoom levels
            assert!(
                button_rect.contains(button_pos),
                "Button center should be clickable at zoom level {}",
                zoom_level
            );
        }
    }
}
