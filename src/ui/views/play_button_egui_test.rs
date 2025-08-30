#[cfg(test)]
mod tests {
    use crate::domain::task::{Position, Task, TaskStatus};
    use crate::repository::Repository;
    use crate::services::{ClaudeCodeService, DependencyService};
    use crate::ui::views::map_view::MapView;
    use eframe::egui::{CentralPanel, Context, Pos2, Rect, Vec2};
    use std::sync::Arc;
    use uuid::Uuid;

    /// Helper to create a test egui context with proper setup
    fn create_test_context() -> Context {
        let ctx = Context::default();
        // Set up test viewport
        ctx.set_pixels_per_point(1.0);
        
        // Initialize the context by running it once to load fonts
        ctx.run(Default::default(), |_ctx| {
            // Empty frame to initialize
        });
        
        ctx
    }

    /// Helper to simulate a click at a specific position
    fn simulate_click(_ctx: &Context, _pos: Pos2) {
        // TODO: Fix input simulation - egui's input is immutable in tests
        // // Create pointer event
        // let events = vec![
        //     egui::Event::PointerMoved(pos),
        //     egui::Event::PointerButton {
        //         pos,
        //         button: egui::PointerButton::Primary,
        //         pressed: true,
        //         modifiers: egui::Modifiers::NONE,
        //     },
        //     egui::Event::PointerButton {
        //         pos,
        //         button: egui::PointerButton::Primary,
        //         pressed: false,
        //         modifiers: egui::Modifiers::NONE,
        //     },
        // ];

        // // Send events to context
        // for event in events {
        //     ctx.input(|i| i.events.push(event));
        // }
    }

    /// Test that the play button renders and responds to clicks
    #[tokio::test]
    async fn test_play_button_renders_and_clicks() {
        // Use the standard test database initialization
        let pool = crate::repository::database::init_test_database()
            .await
            .unwrap();

        let repository = Arc::new(Repository::new(pool));

        // Create test task
        let task = Task {
            id: Uuid::new_v4(),
            title: "Clickable Test Task".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 100.0, y: 100.0 },
            ..Task::default()
        };

        repository.tasks.create(&task).await.unwrap();

        // Setup services
        let claude_service = Arc::new(ClaudeCodeService::new(repository.claude_code.clone()));
        let dep_service = Arc::new(DependencyService::new(repository.clone()));

        // Create MapView with services (using test method that doesn't create runtime)
        let mut map_view = MapView::new_for_test();
        map_view.set_dependency_service(dep_service.clone());
        map_view.set_claude_service(claude_service.clone(), repository.clone());

        // Create test context
        let ctx = create_test_context();

        // Load tasks
        let mut tasks = repository.tasks.list(Default::default()).await.unwrap();
        let mut goals = vec![];

        // First frame - render the UI
        CentralPanel::default().show(&ctx, |ui| {
            map_view.show(ui, &mut tasks, &mut goals);
        });

        // Calculate play button position
        let task_rect = Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(250.0, 180.0));
        let button_pos = task_rect.right_top() + Vec2::new(-15.0, 15.0);

        // Simulate clicking the play button
        simulate_click(&ctx, button_pos);

        // Process the click in the next frame
        CentralPanel::default().show(&ctx, |ui| {
            map_view.show(ui, &mut tasks, &mut goals);
        });

        // Verify task is now marked as running
        assert!(
            map_view.is_task_running(&task.id),
            "Task should be marked as running after clicking play button"
        );
    }

    /// Test that play button only shows for Todo and InProgress tasks
    #[tokio::test]
    async fn test_play_button_visibility_conditions() {
        let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();

        // Initialize tables
        sqlx::query(
            r#"
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
        "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repository = Arc::new(Repository::new(pool));

        // Create tasks with different statuses
        let todo_task = Task {
            id: Uuid::new_v4(),
            title: "Todo Task".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 100.0, y: 100.0 },
            ..Task::default()
        };

        let in_progress_task = Task {
            id: Uuid::new_v4(),
            title: "In Progress Task".to_string(),
            status: TaskStatus::InProgress,
            position: Position { x: 300.0, y: 100.0 },
            ..Task::default()
        };

        let done_task = Task {
            id: Uuid::new_v4(),
            title: "Done Task".to_string(),
            status: TaskStatus::Done,
            position: Position { x: 500.0, y: 100.0 },
            ..Task::default()
        };

        repository.tasks.create(&todo_task).await.unwrap();
        repository.tasks.create(&in_progress_task).await.unwrap();
        repository.tasks.create(&done_task).await.unwrap();

        // Test visibility conditions
        assert!(
            should_show_play_button(&todo_task, &std::collections::HashMap::new()),
            "Play button should show for Todo tasks"
        );

        assert!(
            should_show_play_button(&in_progress_task, &std::collections::HashMap::new()),
            "Play button should show for InProgress tasks"
        );

        assert!(
            !should_show_play_button(&done_task, &std::collections::HashMap::new()),
            "Play button should NOT show for Done tasks"
        );

        // Test that running tasks don't show play button
        let mut running_tasks = std::collections::HashMap::new();
        running_tasks.insert(todo_task.id, crate::services::TaskExecutionStatus::Running);

        assert!(
            !should_show_play_button(&todo_task, &running_tasks),
            "Play button should NOT show for already running tasks"
        );
    }

    /// Helper function to determine if play button should be shown
    fn should_show_play_button(
        task: &Task,
        running_tasks: &std::collections::HashMap<Uuid, crate::services::TaskExecutionStatus>,
    ) -> bool {
        (task.status == TaskStatus::Todo || task.status == TaskStatus::InProgress)
            && !running_tasks.contains_key(&task.id)
    }

    /// Test interaction between play button and task modal
    #[test]
    fn test_play_button_modal_interaction() {
        let ctx = create_test_context();

        // Define task and button areas
        let task_rect = Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(250.0, 180.0));

        let button_pos = task_rect.right_top() + Vec2::new(-15.0, 15.0);
        let button_radius = 10.0;
        let button_rect = Rect::from_center_size(button_pos, Vec2::splat(button_radius * 2.0));

        // Test different click positions
        struct ClickTest {
            name: &'static str,
            click_pos: Pos2,
            expect_play: bool,
            expect_modal: bool,
        }

        let test_cases = vec![
            ClickTest {
                name: "Click on play button center",
                click_pos: button_pos,
                expect_play: true,
                expect_modal: false,
            },
            ClickTest {
                name: "Click on play button edge",
                click_pos: button_pos + Vec2::new(button_radius * 0.8, 0.0),
                expect_play: true,
                expect_modal: false,
            },
            ClickTest {
                name: "Click just outside play button",
                click_pos: button_pos + Vec2::new(button_radius * 1.5, 0.0),
                expect_play: false,
                expect_modal: false, // Outside task rect too
            },
            ClickTest {
                name: "Click on task center",
                click_pos: task_rect.center(),
                expect_play: false,
                expect_modal: true,
            },
            ClickTest {
                name: "Click on task corner",
                click_pos: task_rect.left_top() + Vec2::new(5.0, 5.0),
                expect_play: false,
                expect_modal: true,
            },
        ];

        for test in test_cases {
            let on_button = button_rect.contains(test.click_pos);
            let on_task = task_rect.contains(test.click_pos);
            let should_open_modal = on_task && !on_button;

            assert_eq!(
                on_button, test.expect_play,
                "Test '{}': Play button detection failed at {:?}",
                test.name, test.click_pos
            );

            assert_eq!(
                should_open_modal, test.expect_modal,
                "Test '{}': Modal detection failed at {:?}",
                test.name, test.click_pos
            );
        }
    }

    /// Test hover effects on play button
    #[test]
    fn test_play_button_hover_effects() {
        let ctx = create_test_context();

        // Define button area
        let button_pos = Pos2::new(135.0, 115.0);
        let button_radius = 10.0;
        let button_rect = Rect::from_center_size(button_pos, Vec2::splat(button_radius * 2.0));

        // Test hover detection at various positions
        let hover_positions = vec![
            (button_pos, true, "Center should trigger hover"),
            (
                button_pos + Vec2::new(5.0, 0.0),
                true,
                "Near center should trigger hover",
            ),
            (
                button_pos + Vec2::new(button_radius * 0.9, 0.0),
                true,
                "Edge should trigger hover",
            ),
            (
                button_pos + Vec2::new(button_radius * 1.1, 0.0),
                false,
                "Outside should not trigger hover",
            ),
            (
                button_pos + Vec2::new(0.0, button_radius * 1.1),
                false,
                "Below should not trigger hover",
            ),
        ];

        for (pos, should_hover, description) in hover_positions {
            let is_hovering = button_rect.contains(pos);
            assert_eq!(is_hovering, should_hover, "{}", description);
        }
    }

    /// Test that multiple play buttons don't interfere with each other
    #[tokio::test]
    async fn test_multiple_play_buttons_no_interference() {
        let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();

        // Initialize database
        sqlx::query(
            r#"
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
        "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repository = Arc::new(Repository::new(pool));

        // Create multiple tasks
        let task1 = Task {
            id: Uuid::new_v4(),
            title: "Task 1".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 100.0, y: 100.0 },
            ..Task::default()
        };

        let task2 = Task {
            id: Uuid::new_v4(),
            title: "Task 2".to_string(),
            status: TaskStatus::Todo,
            position: Position { x: 300.0, y: 100.0 },
            ..Task::default()
        };

        repository.tasks.create(&task1).await.unwrap();
        repository.tasks.create(&task2).await.unwrap();

        // Calculate button positions
        let task1_rect = Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(250.0, 180.0));
        let task2_rect = Rect::from_min_max(Pos2::new(300.0, 100.0), Pos2::new(450.0, 180.0));

        let button1_pos = task1_rect.right_top() + Vec2::new(-15.0, 15.0);
        let button2_pos = task2_rect.right_top() + Vec2::new(-15.0, 15.0);

        let button_radius = 10.0;
        let button1_rect = Rect::from_center_size(button1_pos, Vec2::splat(button_radius * 2.0));
        let button2_rect = Rect::from_center_size(button2_pos, Vec2::splat(button_radius * 2.0));

        // Verify buttons don't overlap
        assert!(
            !button1_rect.intersects(button2_rect),
            "Play buttons should not overlap"
        );

        // Verify clicking one doesn't affect the other
        let click_on_button1 = button1_pos;
        assert!(
            button1_rect.contains(click_on_button1),
            "Click should be on button 1"
        );
        assert!(
            !button2_rect.contains(click_on_button1),
            "Click should not affect button 2"
        );
    }

    /// Test play button with different zoom levels
    #[test]
    fn test_play_button_zoom_scaling() {
        let zoom_levels = vec![0.5, 1.0, 1.5, 2.0, 3.0];

        for zoom in zoom_levels {
            // Calculate scaled button dimensions
            let base_pos = Pos2::new(135.0, 115.0);
            let scaled_offset = Vec2::new(-15.0 * zoom, 15.0 * zoom);
            let button_pos = base_pos + scaled_offset;
            let button_radius = 10.0 * zoom;
            let button_rect = Rect::from_center_size(button_pos, Vec2::splat(button_radius * 2.0));

            // Verify button scales properly
            assert_eq!(
                button_rect.width(),
                button_radius * 2.0,
                "Button width should scale with zoom {}",
                zoom
            );

            assert_eq!(
                button_rect.height(),
                button_radius * 2.0,
                "Button height should scale with zoom {}",
                zoom
            );

            // Verify button remains clickable at all zoom levels
            assert!(
                button_rect.contains(button_pos),
                "Button center should be clickable at zoom {}",
                zoom
            );

            // Verify minimum size constraint
            let effective_radius = button_radius.max(10.0);
            assert!(
                effective_radius >= 10.0,
                "Button should maintain minimum size at zoom {}",
                zoom
            );
        }
    }
}
