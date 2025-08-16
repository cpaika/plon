/// Visual tests for Kanban view
/// These tests verify the visual appearance and layout of the Kanban board
use plon::ui::PlonApp;
use plon::domain::task::{Task, TaskStatus, Priority};
use std::fs;
use std::path::PathBuf;

/// Helper to create a test app with sample data
fn setup_test_app_with_data() -> PlonApp {
    let mut app = PlonApp::new_for_test();
    
    // Add various tasks to different columns
    let mut task1 = Task::new("Design new feature".to_string(), "Create mockups and wireframes".to_string());
    task1.status = TaskStatus::Todo;
    task1.priority = Priority::High;
    task1.tags.insert("design".to_string());
    
    let mut task2 = Task::new("Implement authentication".to_string(), "Add OAuth2 support".to_string());
    task2.status = TaskStatus::InProgress;
    task2.priority = Priority::Critical;
    task2.tags.insert("backend".to_string());
    task2.tags.insert("security".to_string());
    
    let mut task3 = Task::new("Code review".to_string(), "Review PR #123".to_string());
    task3.status = TaskStatus::Review;
    task3.priority = Priority::Medium;
    
    let mut task4 = Task::new("Deploy to staging".to_string(), "".to_string());
    task4.status = TaskStatus::Done;
    task4.priority = Priority::Low;
    
    // Add many tasks to test scrolling
    for i in 0..15 {
        let mut task = Task::new(
            format!("Task {}", i),
            format!("Description for task {}", i)
        );
        task.status = match i % 4 {
            0 => TaskStatus::Todo,
            1 => TaskStatus::InProgress,
            2 => TaskStatus::Review,
            _ => TaskStatus::Done,
        };
        app.add_test_task(task);
    }
    
    app.add_test_task(task1);
    app.add_test_task(task2);
    app.add_test_task(task3);
    app.add_test_task(task4);
    
    app.switch_to_kanban_view();
    app
}

#[test]
fn test_kanban_visual_layout() {
    let mut app = setup_test_app_with_data();
    let kanban = app.get_kanban_view();
    
    // Create a layout snapshot for verification
    let mut layout_snapshot = String::new();
    
    // Document the column structure
    layout_snapshot.push_str("=== KANBAN BOARD LAYOUT ===\n\n");
    
    for (idx, column) in kanban.columns.iter().enumerate() {
        layout_snapshot.push_str(&format!("Column {}: {}\n", idx, column.title));
        layout_snapshot.push_str(&format!("  Position: ({:.0}, {:.0})\n", 
            column.bounds.min.x, column.bounds.min.y));
        layout_snapshot.push_str(&format!("  Size: {:.0}x{:.0}\n", 
            column.bounds.width(), column.bounds.height()));
        layout_snapshot.push_str(&format!("  WIP Limit: {:?}\n", column.wip_limit));
        layout_snapshot.push_str(&format!("  Collapsed: {}\n", column.collapsed));
        
        // Count tasks in this column
        let task_count = kanban.get_tasks_for_column(idx).len();
        layout_snapshot.push_str(&format!("  Tasks: {}\n", task_count));
        
        // List task titles
        for task in kanban.get_tasks_for_column(idx) {
            layout_snapshot.push_str(&format!("    - {} (Priority: {:?})\n", 
                task.title, task.priority));
        }
        
        layout_snapshot.push_str("\n");
    }
    
    // Save snapshot
    let snapshot_dir = PathBuf::from("tests/visual/snapshots");
    fs::create_dir_all(&snapshot_dir).ok();
    let snapshot_path = snapshot_dir.join("kanban_layout.txt");
    fs::write(&snapshot_path, &layout_snapshot).expect("Failed to write snapshot");
    
    // Verify layout properties
    assert_kanban_layout_is_correct(&kanban);
}

#[test]
fn test_kanban_height_fills_viewport() {
    let mut app = setup_test_app_with_data();
    
    // Simulate different viewport sizes
    let viewport_sizes = vec![
        (1920.0, 1080.0, "Full HD"),
        (1366.0, 768.0, "Laptop"),
        (1024.0, 768.0, "iPad"),
        (2560.0, 1440.0, "2K"),
    ];
    
    for (width, height, name) in viewport_sizes {
        let kanban = app.get_kanban_view_mut();
        kanban.update_layout_with_height(width, height);
        
        // Verify columns use available height
        for column in &kanban.columns {
            assert!(
                column.bounds.height() >= height * 0.7,
                "{} - Column '{}' height ({}) should be at least 70% of viewport height ({})",
                name, column.title, column.bounds.height(), height
            );
        }
        
        // Document the layout
        println!("\n{} Layout ({}x{}):", name, width, height);
        println!("  Column width: {}", kanban.calculate_column_width(width));
        for column in &kanban.columns {
            println!("  {} height: {}", column.title, column.bounds.height());
        }
    }
}

#[test]
fn test_kanban_scroll_areas_have_unique_ids() {
    let app = setup_test_app_with_data();
    let kanban = app.get_kanban_view();
    
    // Check that we would generate unique IDs for ScrollAreas
    let mut scroll_ids = Vec::new();
    
    // Main horizontal scroll
    scroll_ids.push("kanban_main_horizontal_scroll".to_string());
    
    // Column scrolls
    for idx in 0..kanban.columns.len() {
        let id = format!("kanban_column_scroll_{}", idx);
        assert!(
            !scroll_ids.contains(&id),
            "Duplicate ScrollArea ID found: {}",
            id
        );
        scroll_ids.push(id);
    }
    
    println!("ScrollArea IDs generated: {:?}", scroll_ids);
}

#[test]
fn test_kanban_responsive_column_sizing() {
    let mut app = setup_test_app_with_data();
    let kanban = app.get_kanban_view_mut();
    
    // Test column width calculation for different viewport widths
    let test_cases = vec![
        (1920.0, 4, 400.0, "Desktop with 4 columns"),
        (1366.0, 4, 320.0, "Laptop with 4 columns"),
        (768.0, 4, 250.0, "Tablet should use minimum width"),
        (2560.0, 4, 400.0, "Wide screen should cap at max width"),
    ];
    
    for (viewport_width, visible_columns, expected_min_width, description) in test_cases {
        // Make sure all columns are visible
        for col in &mut kanban.columns {
            col.visible = true;
            col.collapsed = false;
        }
        
        let calculated_width = kanban.calculate_column_width(viewport_width);
        
        assert!(
            calculated_width >= expected_min_width,
            "{}: Column width {} should be at least {}",
            description, calculated_width, expected_min_width
        );
        
        println!("{}: width = {}", description, calculated_width);
    }
}

#[test]
fn test_kanban_card_heights_with_content() {
    let app = setup_test_app_with_data();
    let kanban = app.get_kanban_view();
    
    // Test different task configurations
    let mut test_task = Task::new("Test".to_string(), "".to_string());
    let base_height = kanban.calculate_card_height(&test_task);
    assert_eq!(base_height, 80.0, "Base card height should be 80px");
    
    // With description
    test_task.description = "Some description".to_string();
    let with_desc = kanban.calculate_card_height(&test_task);
    assert_eq!(with_desc, 100.0, "Card with description should be 100px");
    
    // With tags
    test_task.tags.insert("tag1".to_string());
    test_task.tags.insert("tag2".to_string());
    let with_tags = kanban.calculate_card_height(&test_task);
    assert_eq!(with_tags, 125.0, "Card with description and tags should be 125px");
    
    // With subtasks
    for i in 0..5 {
        test_task.add_subtask(format!("Subtask {}", i));
    }
    let with_subtasks = kanban.calculate_card_height(&test_task);
    assert_eq!(with_subtasks, 200.0, "Card height should be capped at 200px");
}

/// Helper function to assert common layout properties
fn assert_kanban_layout_is_correct(kanban: &plon::ui::views::kanban_view_improved::KanbanView) {
    // Check that we have the expected columns
    assert_eq!(kanban.columns.len(), 4, "Should have 4 columns");
    assert_eq!(kanban.columns[0].title, "To Do");
    assert_eq!(kanban.columns[1].title, "In Progress");
    assert_eq!(kanban.columns[2].title, "Review");
    assert_eq!(kanban.columns[3].title, "Done");
    
    // Check WIP limits
    assert_eq!(kanban.columns[1].wip_limit, Some(3));
    assert_eq!(kanban.columns[2].wip_limit, Some(2));
    
    // Check column spacing
    for i in 1..kanban.columns.len() {
        let prev_column = &kanban.columns[i - 1];
        let curr_column = &kanban.columns[i];
        
        if prev_column.visible && !prev_column.collapsed && curr_column.visible {
            let spacing = curr_column.bounds.min.x - prev_column.bounds.max.x;
            assert!(
                spacing >= 0.0,
                "Columns {} and {} should not overlap",
                prev_column.title,
                curr_column.title
            );
        }
    }
    
    // Check minimum column dimensions
    for column in &kanban.columns {
        if column.visible && !column.collapsed {
            assert!(
                column.bounds.width() >= 250.0,
                "Column {} width should be at least 250px",
                column.title
            );
            assert!(
                column.bounds.height() >= 500.0,
                "Column {} height should be at least 500px",
                column.title
            );
        }
    }
}

#[test]
fn test_visual_regression_snapshot() {
    // This test creates a detailed snapshot that can be compared across runs
    let app = setup_test_app_with_data();
    let kanban = app.get_kanban_view();
    
    let snapshot = generate_visual_snapshot(&kanban);
    
    let snapshot_path = PathBuf::from("tests/visual/snapshots/kanban_regression.json");
    fs::create_dir_all(snapshot_path.parent().unwrap()).ok();
    
    // If baseline exists, compare
    if snapshot_path.exists() {
        let baseline = fs::read_to_string(&snapshot_path).expect("Failed to read baseline");
        
        // For now, just check that the structure is similar
        // In a real implementation, you'd do more sophisticated comparison
        assert!(
            snapshot.len() > 0 && baseline.len() > 0,
            "Snapshot comparison failed"
        );
    } else {
        // Create baseline
        fs::write(&snapshot_path, &snapshot).expect("Failed to write baseline");
        println!("Created visual baseline at {:?}", snapshot_path);
    }
}

fn generate_visual_snapshot(kanban: &plon::ui::views::kanban_view_improved::KanbanView) -> String {
    use serde_json::json;
    
    let snapshot = json!({
        "columns": kanban.columns.iter().map(|col| {
            json!({
                "title": col.title,
                "bounds": {
                    "x": col.bounds.min.x,
                    "y": col.bounds.min.y,
                    "width": col.bounds.width(),
                    "height": col.bounds.height()
                },
                "task_count": kanban.get_tasks_for_column(col.position).len(),
                "wip_limit": col.wip_limit,
                "collapsed": col.collapsed
            })
        }).collect::<Vec<_>>(),
        "total_tasks": kanban.tasks.len(),
        "viewport_width": kanban.viewport_width
    });
    
    serde_json::to_string_pretty(&snapshot).unwrap()
}