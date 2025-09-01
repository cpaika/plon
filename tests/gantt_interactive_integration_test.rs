use chrono::{DateTime, NaiveDate, Utc};
use eframe::egui::Pos2;
use plon::domain::dependency::Dependency;
use plon::domain::resource::Resource;
use plon::domain::task::{Priority, Task, TaskStatus};
use plon::ui::views::gantt_view::GanttView;
use plon::ui::widgets::gantt_chart::{DragOperation, InteractiveGanttChart};
use uuid::Uuid;

#[test]
fn test_gantt_interactive_integration() {
    // Create test data
    let mut tasks = vec![
        create_test_task(
            "Task 1",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
        ),
        create_test_task(
            "Task 2",
            NaiveDate::from_ymd_opt(2024, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
        ),
    ];

    let resources: Vec<Resource> = vec![];
    let dependencies: Vec<Dependency> = vec![];

    // Create interactive chart
    let mut interactive_chart = InteractiveGanttChart::new();

    // Test drag operation
    let task_id = tasks[0].id;
    let initial_start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let initial_end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

    // Start a drag operation
    interactive_chart.start_drag(DragOperation::Reschedule {
        task_id,
        initial_start,
        initial_end,
        drag_start_pos: Pos2::new(100.0, 50.0),
    });

    // Simulate dragging 3 days forward
    let new_pos = Pos2::new(190.0, 50.0); // 90 pixels = 3 days at 30px/day
    let chart_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let column_width = 30.0;

    let (new_start, new_end) = interactive_chart.update_drag(new_pos, chart_start, column_width);

    // Verify the dates moved correctly
    assert_eq!(new_start, NaiveDate::from_ymd_opt(2024, 1, 18).unwrap());
    assert_eq!(new_end, NaiveDate::from_ymd_opt(2024, 1, 23).unwrap());

    // Complete the drag
    let result = interactive_chart.complete_drag(new_pos, chart_start, column_width);
    assert!(result.is_some());

    let (updated_task_id, final_start, final_end) = result.unwrap();
    assert_eq!(updated_task_id, task_id);
    assert_eq!(final_start, new_start);
    assert_eq!(final_end, new_end);

    // Verify drag state is cleared
    assert!(!interactive_chart.is_dragging());
}

#[test]
fn test_resize_operation_integration() {
    let mut interactive_chart = InteractiveGanttChart::new();
    let task_id = Uuid::new_v4();

    // Test resizing from the end
    let initial_start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
    let initial_end = NaiveDate::from_ymd_opt(2024, 2, 5).unwrap();

    interactive_chart.start_drag(DragOperation::ResizeEnd {
        task_id,
        initial_start,
        initial_end,
        drag_start_pos: Pos2::new(150.0, 50.0),
    });

    // Extend by 2 days
    let new_pos = Pos2::new(210.0, 50.0); // 60 pixels = 2 days
    let chart_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let column_width = 30.0;

    let (new_start, new_end) = interactive_chart.update_drag(new_pos, chart_start, column_width);

    // Start should remain the same, end should extend
    assert_eq!(new_start, initial_start);
    assert_eq!(new_end, NaiveDate::from_ymd_opt(2024, 2, 7).unwrap());
}

#[test]
fn test_multi_select_and_batch_operations() {
    let mut interactive_chart = InteractiveGanttChart::new();

    let task1_id = Uuid::new_v4();
    let task2_id = Uuid::new_v4();
    let task3_id = Uuid::new_v4();

    // Select multiple tasks
    interactive_chart.select_task(task1_id, false);
    interactive_chart.select_task(task2_id, true); // Add to selection
    interactive_chart.select_task(task3_id, true); // Add to selection

    let selected = interactive_chart.selected_tasks();
    assert_eq!(selected.len(), 3);
    assert!(selected.contains(&task1_id));
    assert!(selected.contains(&task2_id));
    assert!(selected.contains(&task3_id));

    // Test batch reschedule
    let updates = interactive_chart.batch_reschedule(7);
    assert_eq!(updates.len(), 3);

    // Clear selection
    interactive_chart.select_task(task1_id, false); // Replace selection
    assert_eq!(interactive_chart.selected_tasks().len(), 1);
}

fn create_test_task(title: &str, start_date: NaiveDate, end_date: NaiveDate) -> Task {
    let now = Utc::now();
    Task {
        id: Uuid::new_v4(),
        title: title.to_string(),
        description: String::new(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        scheduled_date: Some(DateTime::from_naive_utc_and_offset(
            start_date.and_hms_opt(0, 0, 0).unwrap(),
            Utc,
        )),
        due_date: Some(DateTime::from_naive_utc_and_offset(
            end_date.and_hms_opt(23, 59, 59).unwrap(),
            Utc,
        )),
        estimated_hours: Some(8.0),
        actual_hours: Some(0.0),
        assigned_resource_id: None,
        tags: std::collections::HashSet::new(),
        metadata: std::collections::HashMap::new(),
        subtasks: vec![],
        completed_at: None,
        created_at: now,
        updated_at: now,
        parent_task_id: None,
        goal_id: None,
        position: plon::domain::task::Position { x: 0.0, y: 0.0 },
        is_archived: false,
        assignee: None,
        configuration_id: None,
    }
}
