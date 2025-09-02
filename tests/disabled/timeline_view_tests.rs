use chrono::{Duration, Local, NaiveDate, Utc};
use plon::domain::dependency::{Dependency, DependencyGraph, DependencyType};
use plon::domain::{goal::Goal, resource::Resource, task::Task};
use plon::ui::views::timeline_view::{TimelineFilter, TimelineView, TimelineViewMode};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(test)]
mod timeline_view_stability_tests {
    use super::*;

    fn create_test_tasks() -> Vec<Task> {
        let mut tasks = Vec::new();
        for i in 0..20 {
            let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
            task.scheduled_date = Some(Utc::now() + Duration::days(i * 2));
            task.due_date = Some(Utc::now() + Duration::days(i * 2 + 1));
            task.estimated_hours = Some(8.0);
            tasks.push(task);
        }
        tasks
    }

    #[test]
    fn test_timeline_view_initialization() {
        let view = TimelineView::new();
        assert_eq!(view.days_to_show, 30);
        assert!(view.show_gantt);
        assert!(view.show_resources);
        assert_eq!(view.selected_view, TimelineViewMode::Gantt);
    }

    #[test]
    fn test_timeline_view_date_range_stability() {
        let mut view = TimelineView::new();

        // Test that date range changes don't cause jumps
        let initial_days = view.days_to_show;
        view.set_date_range(60);
        assert_eq!(view.days_to_show, 60);

        // Test bounds
        view.set_date_range(5);
        assert_eq!(view.days_to_show, 7); // Should clamp to minimum

        view.set_date_range(400);
        assert_eq!(view.days_to_show, 365); // Should clamp to maximum
    }

    #[test]
    fn test_rapid_view_changes_cause_jumping() {
        // This test simulates rapid user interactions that cause jumping
        let mut view = TimelineView::new();
        let initial_start_date = view.start_date;

        // Simulate rapid clicking between view modes
        for _ in 0..20 {
            view.set_view_mode(TimelineViewMode::List);
            view.set_view_mode(TimelineViewMode::Gantt);
            view.set_view_mode(TimelineViewMode::Calendar);
        }

        // BUG DETECTED: The view should maintain its position but doesn't!
        // This test will fail, revealing the jumping issue
        assert_eq!(
            view.start_date, initial_start_date,
            "Start date changed after view switches!"
        );
        assert_eq!(
            view.days_to_show, 30,
            "Days to show changed after view switches!"
        );
    }
}
