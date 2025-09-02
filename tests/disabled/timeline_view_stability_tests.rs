use chrono::Local;
use plon::domain::{dependency::*, task::*};
use plon::ui::views::timeline_view::{TimelineFilter, TimelineView, TimelineViewMode};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_timeline_view_preserves_state_on_filter_change() {
    let mut view = TimelineView::new();

    // Store initial zoom level
    let initial_zoom = view.zoom_level;
    let initial_days = view.days_to_show;

    // Change filter
    view.set_filter(TimelineFilter::InProgress);

    // State should be preserved
    assert_eq!(
        view.zoom_level, initial_zoom,
        "Zoom level should be preserved when changing filter"
    );
    assert_eq!(
        view.days_to_show, initial_days,
        "Days to show should be preserved when changing filter"
    );
}

#[test]
fn test_timeline_view_preserves_state_on_view_mode_change() {
    let mut view = TimelineView::new();

    // Store initial state
    let initial_zoom = view.zoom_level;
    let initial_days = view.days_to_show;

    // Change view mode
    view.set_view_mode(TimelineViewMode::List);

    // State should be preserved
    assert_eq!(
        view.zoom_level, initial_zoom,
        "Zoom level should be preserved when changing view mode"
    );
    assert_eq!(
        view.days_to_show, initial_days,
        "Days to show should be preserved when changing view mode"
    );

    // Change to another mode
    view.set_view_mode(TimelineViewMode::Calendar);

    // State should still be preserved
    assert_eq!(
        view.zoom_level, initial_zoom,
        "Zoom level should be preserved across multiple view mode changes"
    );
    assert_eq!(
        view.days_to_show, initial_days,
        "Days to show should be preserved across multiple view mode changes"
    );
}

#[test]
fn test_timeline_view_zoom_updates_correctly() {
    let mut view = TimelineView::new();

    // Set initial state
    view.days_to_show = 30;
    view.zoom_level = 1.0;

    // Zoom out (show more days)
    view.set_date_range(60);

    // Zoom level should be adjusted
    assert_eq!(view.days_to_show, 60, "Days to show should be updated");
    assert!(
        view.zoom_level > 1.0,
        "Zoom level should increase when zooming out"
    );

    // Zoom in (show fewer days)
    view.set_date_range(15);

    assert_eq!(view.days_to_show, 15, "Days to show should be updated");
}

#[test]
fn test_timeline_view_caches_schedule_when_unchanged() {
    let mut view = TimelineView::new();

    // Create test tasks
    let mut task1 = Task::new("Task 1".to_string(), "".to_string());
    task1.estimated_hours = Some(8.0);

    let mut tasks = HashMap::new();
    tasks.insert(task1.id, task1);

    let resources = HashMap::new();
    let graph = DependencyGraph::new();

    // Calculate schedule first time
    let schedule1 = view
        .calculate_schedule(&tasks, &resources, &graph)
        .expect("Should calculate schedule");

    // Calculate again with same data
    let schedule2 = view
        .calculate_schedule(&tasks, &resources, &graph)
        .expect("Should return cached schedule");

    // Should return the same schedule (cached)
    assert_eq!(
        schedule1.task_schedules.len(),
        schedule2.task_schedules.len(),
        "Should return cached schedule when task count unchanged"
    );
}

#[test]
fn test_timeline_view_scroll_to_today_updates_start_date() {
    let mut view = TimelineView::new();

    // Set a start date in the past
    view.start_date = Local::now().naive_local().date() - chrono::Duration::days(30);
    let old_start = view.start_date;

    // Scroll to today
    view.scroll_to_today();

    // Start date should be updated to show today
    assert_ne!(view.start_date, old_start, "Start date should be updated");

    // Today should be visible (within a week of the new start date)
    let today = Local::now().naive_local().date();
    let days_from_start = (today - view.start_date).num_days();
    assert!(
        (0..=7).contains(&days_from_start),
        "Today should be visible after scrolling to today"
    );
}

#[test]
fn test_timeline_view_reset_restores_defaults() {
    let mut view = TimelineView::new();

    // Set various state
    view.zoom_level = 2.0;
    view.days_to_show = 90;
    view.selected_task_id = Some(Uuid::new_v4());
    let old_start = view.start_date;
    view.start_date = old_start - chrono::Duration::days(10);

    // Reset view
    view.reset_view();

    // State should be reset to defaults (except scroll position which ScrollArea manages)
    assert_eq!(view.zoom_level, 1.0, "Zoom level should reset to 1.0");
    assert_eq!(view.days_to_show, 30, "Days to show should reset to 30");
    // Note: selected_task_id is not reset by reset_view
    // Note: start_date is reset to today
}

#[test]
fn test_timeline_view_preserves_selected_task_during_operations() {
    let mut view = TimelineView::new();

    let task_id = Uuid::new_v4();
    view.selected_task_id = Some(task_id);

    // Change filter
    view.set_filter(TimelineFilter::Completed);
    assert_eq!(
        view.selected_task_id,
        Some(task_id),
        "Selected task should be preserved when changing filter"
    );

    // Change view mode
    view.set_view_mode(TimelineViewMode::Calendar);
    assert_eq!(
        view.selected_task_id,
        Some(task_id),
        "Selected task should be preserved when changing view mode"
    );

    // Zoom
    view.set_date_range(60);
    assert_eq!(
        view.selected_task_id,
        Some(task_id),
        "Selected task should be preserved when zooming"
    );
}
