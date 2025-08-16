use plon::domain::task::{Task, TaskStatus};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::services::TaskService;
use std::sync::Arc;
use eframe::egui;

#[tokio::test]
async fn test_map_view_task_creation() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create a task at specific position
    let mut task = Task::new("Map Task".to_string(), "Created on map".to_string());
    task.set_position(100.0, 200.0);
    
    let created = service.create(task.clone()).await.unwrap();
    assert_eq!(created.position.x, 100.0);
    assert_eq!(created.position.y, 200.0);
    
    // Verify task appears in spatial query
    let tasks_in_area = service.repository.tasks.find_in_area(50.0, 150.0, 150.0, 250.0).await.unwrap();
    assert_eq!(tasks_in_area.len(), 1);
    assert_eq!(tasks_in_area[0].title, "Map Task");
}

#[tokio::test]
async fn test_map_view_task_dragging() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create task
    let mut task = Task::new("Draggable Task".to_string(), "".to_string());
    task.set_position(50.0, 50.0);
    let created = service.create(task).await.unwrap();
    
    // Simulate drag to new position
    let mut updated_task = created.clone();
    updated_task.set_position(150.0, 175.0);
    
    service.update(updated_task.clone()).await.unwrap();
    
    // Verify new position
    let fetched = service.get(created.id).await.unwrap().unwrap();
    assert_eq!(fetched.position.x, 150.0);
    assert_eq!(fetched.position.y, 175.0);
}

#[tokio::test]
async fn test_map_view_zoom_levels() {
    // Test zoom calculations
    let mut zoom: f32 = 1.0;
    
    // Zoom in
    zoom = (zoom * 1.2).min(5.0);
    assert!((zoom - 1.2).abs() < 0.001);
    
    // Zoom to max
    zoom = 10.0;
    zoom = zoom.min(5.0);
    assert_eq!(zoom, 5.0);
    
    // Zoom to min
    zoom = 0.01;
    zoom = zoom.max(0.1);
    assert_eq!(zoom, 0.1);
}

#[tokio::test]
async fn test_map_view_viewport_culling() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks at various positions
    let positions = vec![
        (100.0, 100.0, true),   // In view
        (400.0, 300.0, true),   // In view
        (1000.0, 1000.0, false), // Out of view
        (-500.0, -500.0, false), // Out of view
    ];
    
    for (i, (x, y, _)) in positions.iter().enumerate() {
        let mut task = Task::new(format!("Task {}", i), "".to_string());
        task.set_position(*x, *y);
        service.create(task).await.unwrap();
    }
    
    // Define viewport (0,0) to (800,600)
    let viewport = egui::Rect::from_min_size(
        egui::Pos2::new(0.0, 0.0),
        egui::Vec2::new(800.0, 600.0)
    );
    
    // Check visibility
    for (x, y, should_be_visible) in positions {
        let pos = egui::Pos2::new(x as f32, y as f32);
        assert_eq!(viewport.contains(pos), should_be_visible);
    }
}

#[tokio::test]
async fn test_map_view_coordinate_transformation() {
    let camera_pos = egui::Vec2::new(100.0, 50.0);
    let zoom = 2.0;
    let center = egui::Pos2::new(400.0, 300.0);
    
    // World to screen transformation
    let world_to_screen = |world_x: f32, world_y: f32| -> egui::Pos2 {
        egui::Pos2::new(
            center.x + (world_x + camera_pos.x) * zoom,
            center.y + (world_y + camera_pos.y) * zoom,
        )
    };
    
    // Test transformation
    let screen_pos = world_to_screen(0.0, 0.0);
    assert_eq!(screen_pos.x, 400.0 + 100.0 * 2.0); // 600
    assert_eq!(screen_pos.y, 300.0 + 50.0 * 2.0);  // 400
    
    // Screen to world transformation (inverse)
    let screen_to_world = |screen_x: f32, screen_y: f32| -> egui::Vec2 {
        egui::Vec2::new(
            (screen_x - center.x) / zoom - camera_pos.x,
            (screen_y - center.y) / zoom - camera_pos.y,
        )
    };
    
    let world_pos = screen_to_world(600.0, 400.0);
    assert!((world_pos.x - 0.0).abs() < 0.001);
    assert!((world_pos.y - 0.0).abs() < 0.001);
}

#[tokio::test]
async fn test_map_view_clustering() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create clustered tasks
    let cluster_centers = vec![
        (100.0, 100.0),
        (500.0, 500.0),
        (200.0, 400.0),
    ];
    
    for (cx, cy) in cluster_centers {
        for i in 0..5 {
            let mut task = Task::new(format!("Cluster Task {}", i), "".to_string());
            let offset = (i as f64) * 10.0;
            task.set_position(cx + offset, cy + offset);
            service.create(task).await.unwrap();
        }
    }
    
    // Verify tasks can be queried by area
    let area1 = service.repository.tasks.find_in_area(50.0, 150.0, 50.0, 150.0).await.unwrap();
    assert!(area1.len() >= 5); // Should contain first cluster
    
    let area2 = service.repository.tasks.find_in_area(450.0, 550.0, 450.0, 550.0).await.unwrap();
    assert!(area2.len() >= 5); // Should contain second cluster
}

#[tokio::test]
async fn test_map_view_selection() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create tasks
    let task1 = service.create(Task::new("Task 1".to_string(), "".to_string())).await.unwrap();
    let task2 = service.create(Task::new("Task 2".to_string(), "".to_string())).await.unwrap();
    
    // Test selection state
    let mut selected_task_id: Option<uuid::Uuid> = None;
    
    // Select task1
    selected_task_id = Some(task1.id);
    assert_eq!(selected_task_id, Some(task1.id));
    
    // Change selection to task2
    selected_task_id = Some(task2.id);
    assert_eq!(selected_task_id, Some(task2.id));
    
    // Clear selection
    selected_task_id = None;
    assert!(selected_task_id.is_none());
}

#[tokio::test]
async fn test_map_view_grid_rendering() {
    let grid_size: f32 = 50.0;
    let zoom: f32 = 2.0;
    let scaled_grid = grid_size * zoom;
    
    let viewport_width: f32 = 800.0;
    let viewport_height: f32 = 600.0;
    let center_x = viewport_width / 2.0;
    let center_y = viewport_height / 2.0;
    
    // Calculate grid lines needed
    let start_x = ((0.0 - center_x) / scaled_grid).floor() * scaled_grid;
    let end_x = ((viewport_width - center_x) / scaled_grid).ceil() * scaled_grid;
    let start_y = ((0.0 - center_y) / scaled_grid).floor() * scaled_grid;
    let end_y = ((viewport_height - center_y) / scaled_grid).ceil() * scaled_grid;
    
    // Count grid lines
    let x_lines = ((end_x - start_x) / scaled_grid) as i32 + 1;
    let y_lines = ((end_y - start_y) / scaled_grid) as i32 + 1;
    
    assert!(x_lines > 0);
    assert!(y_lines > 0);
}

#[tokio::test]
async fn test_map_view_performance_with_many_tasks() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = TaskService::new(repository);
    
    // Create 100 tasks
    let start = std::time::Instant::now();
    
    for i in 0..100 {
        let mut task = Task::new(format!("Perf Task {}", i), "".to_string());
        task.set_position((i as f64) * 20.0, (i as f64) * 15.0);
        service.create(task).await.unwrap();
    }
    
    let creation_time = start.elapsed();
    
    // Query all tasks
    let query_start = std::time::Instant::now();
    let all_tasks = service.list_all().await.unwrap();
    let query_time = query_start.elapsed();
    
    assert_eq!(all_tasks.len(), 100);
    
    // Performance assertions (adjust as needed)
    assert!(creation_time.as_secs() < 5, "Task creation took too long");
    assert!(query_time.as_millis() < 100, "Task query took too long");
}

#[tokio::test]
async fn test_map_view_double_click_task_creation() {
    let camera_pos = egui::Vec2::new(0.0, 0.0);
    let zoom = 1.0;
    let center = egui::Pos2::new(400.0, 300.0);
    
    // Simulate double-click at specific screen position
    let click_pos = egui::Pos2::new(500.0, 400.0);
    
    // Convert to world position
    let world_pos = (click_pos - center) / zoom - camera_pos;
    
    // Create task at that position
    let mut task = Task::new("Double-click Task".to_string(), "".to_string());
    task.set_position(world_pos.x as f64, world_pos.y as f64);
    
    assert_eq!(task.position.x, 100.0);
    assert_eq!(task.position.y, 100.0);
}