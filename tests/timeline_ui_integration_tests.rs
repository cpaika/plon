use plon::ui::views::timeline_view::{TimelineView, TimelineViewMode, TimelineFilter, TimelineProcessedData};
use plon::ui::widgets::gantt_chart::GanttChart;
use plon::domain::{task::*, resource::*, dependency::*, goal::Goal};
use plon::services::timeline_scheduler::{TimelineScheduler, TimelineSchedule};
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_timeline_view_creation() {
    let view = TimelineView::new();
    assert!(view.show_gantt);
    assert!(view.show_resources);
    assert_eq!(view.selected_view, TimelineViewMode::Gantt);
}

#[test]
fn test_timeline_view_mode_switching() {
    let mut view = TimelineView::new();
    
    view.set_view_mode(TimelineViewMode::List);
    assert_eq!(view.selected_view, TimelineViewMode::List);
    
    view.set_view_mode(TimelineViewMode::Calendar);
    assert_eq!(view.selected_view, TimelineViewMode::Calendar);
    
    view.set_view_mode(TimelineViewMode::Gantt);
    assert_eq!(view.selected_view, TimelineViewMode::Gantt);
}

#[test]
fn test_timeline_view_with_tasks_and_resources() {
    let mut view = TimelineView::new();
    
    // Create test data
    let mut task1 = Task::new("Frontend Development".to_string(), "Build UI".to_string());
    task1.estimated_hours = Some(40.0);
    
    let mut task2 = Task::new("Backend API".to_string(), "Create REST endpoints".to_string());
    task2.estimated_hours = Some(60.0);
    
    let resource1 = Resource::new("Alice".to_string(), "Frontend Dev".to_string(), 40.0);
    let resource2 = Resource::new("Bob".to_string(), "Backend Dev".to_string(), 40.0);
    
    task1.assigned_resource_id = Some(resource1.id);
    task2.assigned_resource_id = Some(resource2.id);
    
    let mut tasks = HashMap::new();
    tasks.insert(task1.id, task1.clone());
    tasks.insert(task2.id, task2.clone());
    
    let mut resources = HashMap::new();
    resources.insert(resource1.id, resource1.clone());
    resources.insert(resource2.id, resource2.clone());
    
    // Process timeline
    let processed = view.process_timeline_data(&tasks, &resources);
    
    assert_eq!(processed.task_count, 2);
    assert_eq!(processed.resource_count, 2);
    assert_eq!(processed.unassigned_tasks, 0);
}

#[test]
fn test_timeline_filtering() {
    let mut view = TimelineView::new();
    
    let mut task1 = Task::new("Task 1".to_string(), "".to_string());
    task1.status = TaskStatus::InProgress;
    
    let mut task2 = Task::new("Task 2".to_string(), "".to_string());
    task2.status = TaskStatus::Done;
    
    let mut task3 = Task::new("Task 3".to_string(), "".to_string());
    task3.status = TaskStatus::Todo;
    
    let mut tasks = HashMap::new();
    tasks.insert(task1.id, task1);
    tasks.insert(task2.id, task2);
    tasks.insert(task3.id, task3);
    
    // Filter for in-progress tasks
    view.set_filter(TimelineFilter::InProgress);
    let filtered = view.apply_filters(&tasks);
    assert_eq!(filtered.len(), 1);
    
    // Filter for completed tasks
    view.set_filter(TimelineFilter::Completed);
    let filtered = view.apply_filters(&tasks);
    assert_eq!(filtered.len(), 1);
    
    // Show all tasks
    view.set_filter(TimelineFilter::All);
    let filtered = view.apply_filters(&tasks);
    assert_eq!(filtered.len(), 3);
}

#[test]
fn test_timeline_resource_assignment() {
    let mut view = TimelineView::new();
    
    let mut task = Task::new("Unassigned Task".to_string(), "".to_string());
    let resource = Resource::new("Developer".to_string(), "Dev".to_string(), 40.0);
    
    assert!(task.assigned_resource_id.is_none());
    
    view.assign_resource_to_task(&mut task, resource.id);
    
    assert_eq!(task.assigned_resource_id, Some(resource.id));
}

#[test]
fn test_timeline_dependency_creation() {
    let mut view = TimelineView::new();
    
    let task1 = Task::new("Task 1".to_string(), "".to_string());
    let task2 = Task::new("Task 2".to_string(), "".to_string());
    
    let mut graph = DependencyGraph::new();
    
    let success = view.create_dependency(
        task1.id,
        task2.id,
        DependencyType::FinishToStart,
        &mut graph
    );
    
    assert!(success);
    
    let dependencies = graph.get_dependencies(task2.id);
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].0, task1.id);
}

#[test]
fn test_timeline_schedule_calculation() {
    let mut view = TimelineView::new();
    
    let mut task1 = Task::new("Design".to_string(), "".to_string());
    task1.estimated_hours = Some(16.0);
    
    let mut task2 = Task::new("Implementation".to_string(), "".to_string());
    task2.estimated_hours = Some(40.0);
    
    let resource = Resource::new("Developer".to_string(), "Dev".to_string(), 40.0);
    
    task1.assigned_resource_id = Some(resource.id);
    task2.assigned_resource_id = Some(resource.id);
    
    let task1_id = task1.id;
    let task2_id = task2.id;
    
    let mut tasks = HashMap::new();
    tasks.insert(task1_id, task1);
    tasks.insert(task2_id, task2);
    
    let mut resources = HashMap::new();
    resources.insert(resource.id, resource);
    
    let mut graph = DependencyGraph::new();
    graph.add_dependency(&Dependency::new(task1_id, task2_id, DependencyType::FinishToStart)).unwrap();
    
    let schedule = view.calculate_schedule(&tasks, &resources, &graph);
    
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap();
    assert_eq!(schedule.task_schedules.len(), 2);
}

#[test]
fn test_timeline_critical_path_identification() {
    let view = TimelineView::new();
    
    let task1 = Task::new("Task 1".to_string(), "".to_string());
    let task2 = Task::new("Task 2".to_string(), "".to_string());
    let task3 = Task::new("Task 3".to_string(), "".to_string());
    
    let mut graph = DependencyGraph::new();
    graph.add_dependency(&Dependency::new(task1.id, task2.id, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(task2.id, task3.id, DependencyType::FinishToStart)).unwrap();
    
    let mut estimates = HashMap::new();
    estimates.insert(task1.id, 10.0);
    estimates.insert(task2.id, 20.0);
    estimates.insert(task3.id, 15.0);
    
    let critical_path = graph.get_critical_path(&estimates);
    
    assert_eq!(critical_path.len(), 3);
    assert!(view.is_task_critical(task1.id, &critical_path));
    assert!(view.is_task_critical(task2.id, &critical_path));
    assert!(view.is_task_critical(task3.id, &critical_path));
}

#[test]
fn test_timeline_date_range_adjustment() {
    let mut view = TimelineView::new();
    
    view.set_date_range(30);
    assert_eq!(view.days_to_show, 30);
    
    view.set_date_range(90);
    assert_eq!(view.days_to_show, 90);
    
    view.set_date_range(365);
    assert_eq!(view.days_to_show, 365);
    
    // Test boundaries
    view.set_date_range(5);
    assert_eq!(view.days_to_show, 7); // Minimum
    
    view.set_date_range(400);
    assert_eq!(view.days_to_show, 365); // Maximum
}

#[test]
fn test_timeline_export_functionality() {
    let view = TimelineView::new();
    
    let task = Task::new("Export Test".to_string(), "".to_string());
    let resource = Resource::new("Resource".to_string(), "Dev".to_string(), 40.0);
    
    let mut tasks = HashMap::new();
    tasks.insert(task.id, task);
    
    let mut resources = HashMap::new();
    resources.insert(resource.id, resource);
    
    let schedule = TimelineSchedule {
        task_schedules: HashMap::new(),
        resource_allocations: Vec::new(),
        critical_path: Vec::new(),
        warnings: Vec::new(),
    };
    
    let export_result = view.export_timeline(&tasks, &resources, &schedule);
    
    assert!(export_result.is_ok());
    let export_data = export_result.unwrap();
    assert!(export_data.contains("tasks"));
    assert!(export_data.contains("resources"));
}

#[test]
fn test_timeline_goal_integration() {
    let mut view = TimelineView::new();
    
    let mut goal = Goal::new("Q1 Release".to_string(), "Release version 1.0".to_string());
    goal.target_date = Some(Utc::now() + chrono::Duration::days(90));
    
    let mut task1 = Task::new("Feature A".to_string(), "".to_string());
    task1.goal_id = Some(goal.id);
    
    let mut task2 = Task::new("Feature B".to_string(), "".to_string());
    task2.goal_id = Some(goal.id);
    
    let mut tasks = HashMap::new();
    tasks.insert(task1.id, task1);
    tasks.insert(task2.id, task2);
    
    let mut goals = HashMap::new();
    goals.insert(goal.id, goal.clone());
    
    let grouped = view.group_tasks_by_goal(&tasks, &goals);
    
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped.get(&Some(goal.id)).unwrap().len(), 2);
}

#[test]
fn test_timeline_warning_generation() {
    let view = TimelineView::new();
    
    let mut task1 = Task::new("Overdue Task".to_string(), "".to_string());
    task1.due_date = Some(Utc::now() - chrono::Duration::days(5));
    task1.status = TaskStatus::InProgress;
    
    let mut task2 = Task::new("Unassigned Task".to_string(), "".to_string());
    task2.estimated_hours = Some(40.0);
    // No resource assigned
    
    let mut task3 = Task::new("No Estimate".to_string(), "".to_string());
    task3.assigned_resource_id = Some(Uuid::new_v4());
    // No estimated hours
    
    let mut tasks = HashMap::new();
    tasks.insert(task1.id, task1.clone());
    tasks.insert(task2.id, task2.clone());
    tasks.insert(task3.id, task3.clone());
    
    let warnings = view.generate_warnings(&tasks);
    
    assert!(warnings.len() >= 3);
    assert!(warnings.iter().any(|w| w.contains("overdue")));
    assert!(warnings.iter().any(|w| w.contains("unassigned")));
    assert!(warnings.iter().any(|w| w.contains("estimate")));
}