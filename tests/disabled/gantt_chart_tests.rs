use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use eframe::egui::{self, Pos2, Vec2};
use plon::domain::{
    dependency::{Dependency, DependencyType},
    resource::Resource,
    task::{Priority, Task, TaskStatus},
};
use plon::ui::views::gantt_view::GanttView;
use plon::ui::widgets::gantt_chart::{
    DragOperation, GanttChart, GanttColor, InteractiveGanttChart, Milestone,
};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[cfg(test)]
mod gantt_chart_widget_tests {
    use super::*;

    #[test]
    fn test_gantt_chart_creation() {
        let chart = GanttChart::new();
        assert_eq!(chart.zoom_level, 1.0);
        assert_eq!(chart.days_to_show, 30);
        assert!(chart.show_dependencies);
        assert!(chart.show_resources);
    }

    #[test]
    fn test_gantt_chart_zoom() {
        let mut chart = GanttChart::new();

        // Test zoom in
        let initial_zoom = chart.zoom_level;
        chart.zoom_in();
        assert!(chart.zoom_level > initial_zoom);
        assert!(chart.zoom_level <= 3.0);

        // Test zoom out
        chart.zoom_out();
        assert_eq!(chart.zoom_level, initial_zoom);

        // Test zoom limits
        for _ in 0..10 {
            chart.zoom_in();
        }
        assert_eq!(chart.zoom_level, 3.0);

        for _ in 0..20 {
            chart.zoom_out();
        }
        assert_eq!(chart.zoom_level, 0.3);
    }

    #[test]
    fn test_gantt_chart_date_range() {
        let mut chart = GanttChart::new();

        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        chart.set_start_date(start_date);
        assert_eq!(chart.get_start_date(), start_date);

        chart.set_days_to_show(60);
        let expected_end = start_date + Duration::days(59);
        assert_eq!(chart.get_end_date(), expected_end);
    }
}

#[cfg(test)]
mod interactive_gantt_tests {
    use super::*;

    fn create_test_task(id: Uuid, title: &str, start_date: NaiveDate, end_date: NaiveDate) -> Task {
        let now = Utc::now();
        Task {
            id,
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
            tags: HashSet::new(),
            metadata: HashMap::new(),
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

    #[test]
    fn test_drag_to_reschedule_initialization() {
        let mut interactive_chart = InteractiveGanttChart::new();

        assert!(interactive_chart.current_drag_operation.is_none());
        assert!(interactive_chart.hovered_task_id.is_none());
        assert!(interactive_chart.selected_task_id.is_none());
    }

    #[test]
    fn test_start_drag_to_reschedule() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task_id = Uuid::new_v4();
        let initial_pos = Pos2::new(100.0, 50.0);
        let initial_start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let initial_end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

        interactive_chart.start_drag(DragOperation::Reschedule {
            task_id,
            initial_start,
            initial_end,
            drag_start_pos: initial_pos,
        });

        assert!(interactive_chart.current_drag_operation.is_some());
        match &interactive_chart.current_drag_operation {
            Some(DragOperation::Reschedule {
                task_id: id,
                initial_start: start,
                initial_end: end,
                ..
            }) => {
                assert_eq!(*id, task_id);
                assert_eq!(*start, initial_start);
                assert_eq!(*end, initial_end);
            }
            _ => panic!("Expected Reschedule operation"),
        }
    }

    #[test]
    fn test_update_drag_position() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task_id = Uuid::new_v4();
        let initial_pos = Pos2::new(100.0, 50.0);
        let initial_start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let initial_end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

        interactive_chart.start_drag(DragOperation::Reschedule {
            task_id,
            initial_start,
            initial_end,
            drag_start_pos: initial_pos,
        });

        let new_pos = Pos2::new(130.0, 50.0); // 30 pixels to the right (assuming 30px = 1 day)
        let chart_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let column_width = 30.0;

        let (new_start, new_end) =
            interactive_chart.update_drag(new_pos, chart_start, column_width);

        // Should move 1 day forward
        assert_eq!(new_start, NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
        assert_eq!(new_end, NaiveDate::from_ymd_opt(2024, 1, 21).unwrap());
    }

    #[test]
    fn test_resize_task_from_start() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task_id = Uuid::new_v4();
        let initial_pos = Pos2::new(100.0, 50.0);
        let initial_start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let initial_end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

        interactive_chart.start_drag(DragOperation::ResizeStart {
            task_id,
            initial_start,
            initial_end,
            drag_start_pos: initial_pos,
        });

        let new_pos = Pos2::new(70.0, 50.0); // 30 pixels to the left
        let chart_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let column_width = 30.0;

        let (new_start, new_end) =
            interactive_chart.update_drag(new_pos, chart_start, column_width);

        // Start should move 1 day earlier, end stays the same
        assert_eq!(new_start, NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
        assert_eq!(new_end, NaiveDate::from_ymd_opt(2024, 1, 20).unwrap());
    }

    #[test]
    fn test_resize_task_from_end() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task_id = Uuid::new_v4();
        let initial_pos = Pos2::new(250.0, 50.0);
        let initial_start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let initial_end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

        interactive_chart.start_drag(DragOperation::ResizeEnd {
            task_id,
            initial_start,
            initial_end,
            drag_start_pos: initial_pos,
        });

        let new_pos = Pos2::new(280.0, 50.0); // 30 pixels to the right
        let chart_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let column_width = 30.0;

        let (new_start, new_end) =
            interactive_chart.update_drag(new_pos, chart_start, column_width);

        // End should move 1 day later, start stays the same
        assert_eq!(new_start, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(new_end, NaiveDate::from_ymd_opt(2024, 1, 21).unwrap());
    }

    #[test]
    fn test_resize_prevents_negative_duration() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task_id = Uuid::new_v4();
        let initial_pos = Pos2::new(100.0, 50.0);
        let initial_start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let initial_end = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(); // 2-day task

        interactive_chart.start_drag(DragOperation::ResizeStart {
            task_id,
            initial_start,
            initial_end,
            drag_start_pos: initial_pos,
        });

        // Try to drag start past end
        let new_pos = Pos2::new(250.0, 50.0); // Way past the end date
        let chart_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let column_width = 30.0;

        let (new_start, new_end) =
            interactive_chart.update_drag(new_pos, chart_start, column_width);

        // Should maintain minimum 1-day duration
        assert!(new_start <= new_end);
        let duration = (new_end - new_start).num_days();
        assert!(duration >= 0); // At least 1 day duration
    }

    #[test]
    fn test_complete_drag_operation() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task_id = Uuid::new_v4();
        let initial_pos = Pos2::new(100.0, 50.0);
        let initial_start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let initial_end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

        interactive_chart.start_drag(DragOperation::Reschedule {
            task_id,
            initial_start,
            initial_end,
            drag_start_pos: initial_pos,
        });

        let new_pos = Pos2::new(160.0, 50.0);
        let chart_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let column_width = 30.0;

        let result = interactive_chart.complete_drag(new_pos, chart_start, column_width);

        assert!(result.is_some());
        let (affected_task_id, new_start, new_end) = result.unwrap();
        assert_eq!(affected_task_id, task_id);
        assert_eq!(new_start, NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
        assert_eq!(new_end, NaiveDate::from_ymd_opt(2024, 1, 22).unwrap());

        // Drag operation should be cleared
        assert!(interactive_chart.current_drag_operation.is_none());
    }

    #[test]
    fn test_cancel_drag_operation() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task_id = Uuid::new_v4();
        let initial_pos = Pos2::new(100.0, 50.0);
        let initial_start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let initial_end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

        interactive_chart.start_drag(DragOperation::Reschedule {
            task_id,
            initial_start,
            initial_end,
            drag_start_pos: initial_pos,
        });

        interactive_chart.cancel_drag();

        assert!(interactive_chart.current_drag_operation.is_none());
    }

    #[test]
    fn test_hover_detection() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task_id = Uuid::new_v4();
        let task_rect = egui::Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(150.0, 30.0));

        // Test hovering over task
        let hover_pos = Pos2::new(150.0, 60.0);
        let is_hovering = interactive_chart.update_hover(hover_pos, task_id, task_rect);
        assert!(is_hovering);
        assert_eq!(interactive_chart.hovered_task_id, Some(task_id));

        // Test hover cursor type for resize handles
        let resize_cursor = interactive_chart.get_hover_cursor(hover_pos, task_rect);
        assert!(resize_cursor.is_some());
    }

    #[test]
    fn test_resize_handle_detection() {
        let interactive_chart = InteractiveGanttChart::new();
        let task_rect = egui::Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(150.0, 30.0));

        // Test left resize handle
        let left_handle_pos = Pos2::new(102.0, 60.0);
        assert!(interactive_chart.is_near_left_handle(left_handle_pos, task_rect));

        // Test right resize handle
        let right_handle_pos = Pos2::new(248.0, 60.0);
        assert!(interactive_chart.is_near_right_handle(right_handle_pos, task_rect));

        // Test middle (no handle)
        let middle_pos = Pos2::new(175.0, 60.0);
        assert!(!interactive_chart.is_near_left_handle(middle_pos, task_rect));
        assert!(!interactive_chart.is_near_right_handle(middle_pos, task_rect));
    }

    #[test]
    fn test_visual_feedback_during_drag() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task_id = Uuid::new_v4();
        let initial_pos = Pos2::new(100.0, 50.0);
        let initial_start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let initial_end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();

        interactive_chart.start_drag(DragOperation::Reschedule {
            task_id,
            initial_start,
            initial_end,
            drag_start_pos: initial_pos,
        });

        // Should provide preview of new position
        assert!(interactive_chart.is_dragging());
        assert_eq!(interactive_chart.get_dragging_task_id(), Some(task_id));

        let preview_style = interactive_chart.get_drag_preview_style();
        assert!(preview_style.opacity < 1.0); // Should be semi-transparent
    }

    #[test]
    fn test_snap_to_grid() {
        let interactive_chart = InteractiveGanttChart::new();
        let column_width = 30.0;

        // Test snapping to nearest day
        let unsnapped_pos = Pos2::new(107.0, 50.0); // Not aligned to grid
        let snapped_pos = interactive_chart.snap_to_grid(unsnapped_pos, column_width);

        // Should snap to nearest column (90 or 120)
        assert!((snapped_pos.x - 90.0).abs() < 0.1 || (snapped_pos.x - 120.0).abs() < 0.1);
    }

    #[test]
    fn test_constrain_to_bounds() {
        let mut interactive_chart = InteractiveGanttChart::new();
        interactive_chart.set_min_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        interactive_chart.set_max_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

        // Test constraining dates within bounds
        let too_early = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
        let constrained_early = interactive_chart.constrain_date(too_early);
        assert_eq!(
            constrained_early,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );

        let too_late = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let constrained_late = interactive_chart.constrain_date(too_late);
        assert_eq!(
            constrained_late,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
        );
    }

    #[test]
    fn test_multi_select() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();

        // Select first task
        interactive_chart.select_task(task1, false);
        assert_eq!(interactive_chart.selected_tasks(), vec![task1]);

        // Add to selection with ctrl/cmd
        interactive_chart.select_task(task2, true);
        assert!(interactive_chart.selected_tasks().contains(&task1));
        assert!(interactive_chart.selected_tasks().contains(&task2));

        // Replace selection without modifier
        interactive_chart.select_task(task3, false);
        assert_eq!(interactive_chart.selected_tasks(), vec![task3]);
    }

    #[test]
    fn test_batch_reschedule() {
        let mut interactive_chart = InteractiveGanttChart::new();
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();

        // Select multiple tasks
        interactive_chart.select_task(task1, false);
        interactive_chart.select_task(task2, true);

        let offset_days = 5;
        let updates = interactive_chart.batch_reschedule(offset_days);

        assert_eq!(updates.len(), 2);
        assert!(updates.iter().any(|(id, _)| *id == task1));
        assert!(updates.iter().any(|(id, _)| *id == task2));
    }
}
