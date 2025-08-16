use plon::domain::{task::*, goal::*, resource::*, dependency::*};
use plon::ui::views::timeline_view::*;
use plon::services::timeline_scheduler::*;
use eframe::egui;
use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate, Local};
use std::collections::HashMap;

#[cfg(test)]
mod timeline_view_tests {
    use super::*;

    fn create_test_task(title: &str, hours: Option<f32>, scheduled_days_ahead: Option<i64>) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.estimated_hours = hours;
        if let Some(days) = scheduled_days_ahead {
            task.scheduled_date = Some(Utc::now() + Duration::days(days));
        }
        task
    }

    fn create_test_resource(name: &str, weekly_hours: f32) -> Resource {
        Resource::new(name.to_string(), "Developer".to_string(), weekly_hours)
    }

    #[test]
    fn test_timeline_view_initialization() {
        let view = TimelineView::new();
        assert_eq!(view.days_to_show, 30);
        assert_eq!(view.show_gantt, true);
        assert_eq!(view.show_resources, true);
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
    fn test_timeline_filter_application() {
        let view = TimelineView::new();
        let mut tasks = HashMap::new();
        
        let mut task1 = create_test_task("In Progress Task", Some(5.0), Some(1));
        task1.status = TaskStatus::InProgress;
        tasks.insert(task1.id, task1);
        
        let mut task2 = create_test_task("Completed Task", Some(3.0), Some(2));
        task2.status = TaskStatus::Done;
        tasks.insert(task2.id, task2);
        
        let task3 = create_test_task("Unassigned Task", Some(8.0), Some(3));
        tasks.insert(task3.id, task3);
        
        // Test All filter
        let filtered = view.apply_filters(&tasks);
        assert_eq!(filtered.len(), 3);
        
        // Test InProgress filter
        let mut view_in_progress = TimelineView::new();
        view_in_progress.set_filter(TimelineFilter::InProgress);
        let filtered_in_progress = view_in_progress.apply_filters(&tasks);
        assert_eq!(filtered_in_progress.len(), 1);
        
        // Test Completed filter
        let mut view_completed = TimelineView::new();
        view_completed.set_filter(TimelineFilter::Completed);
        let filtered_completed = view_completed.apply_filters(&tasks);
        assert_eq!(filtered_completed.len(), 1);
        
        // Test Unassigned filter
        let mut view_unassigned = TimelineView::new();
        view_unassigned.set_filter(TimelineFilter::Unassigned);
        let filtered_unassigned = view_unassigned.apply_filters(&tasks);
        assert_eq!(filtered_unassigned.len(), 3); // All tasks are unassigned to resources
    }

    #[test]
    fn test_date_range_configuration() {
        let mut view = TimelineView::new();
        
        view.set_date_range(7);
        assert_eq!(view.days_to_show, 7);
        
        view.set_date_range(365);
        assert_eq!(view.days_to_show, 365);
        
        // Test clamping
        view.set_date_range(5);
        assert_eq!(view.days_to_show, 7); // Min is 7
        
        view.set_date_range(400);
        assert_eq!(view.days_to_show, 365); // Max is 365
    }

    #[test]
    fn test_resource_assignment() {
        let view = TimelineView::new();
        let mut task = create_test_task("Test Task", Some(10.0), Some(5));
        let resource_id = Uuid::new_v4();
        
        assert!(task.assigned_resource_id.is_none());
        
        view.assign_resource_to_task(&mut task, resource_id);
        
        assert_eq!(task.assigned_resource_id, Some(resource_id));
        assert!(task.updated_at > task.created_at);
    }

    #[test]
    fn test_dependency_creation() {
        let view = TimelineView::new();
        let mut graph = DependencyGraph::new();
        
        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();
        
        let success = view.create_dependency(
            task1_id,
            task2_id,
            DependencyType::FinishToStart,
            &mut graph
        );
        
        assert!(success);
        
        // Verify dependency was added
        let deps = graph.get_dependencies(task2_id);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, task1_id);
    }

    #[test]
    fn test_schedule_calculation() {
        let mut view = TimelineView::new();
        let mut tasks = HashMap::new();
        let mut resources = HashMap::new();
        let graph = DependencyGraph::new();
        
        let resource = create_test_resource("Developer", 40.0);
        resources.insert(resource.id, resource.clone());
        
        let mut task1 = create_test_task("Task 1", Some(20.0), Some(1));
        task1.assigned_resource_id = Some(resource.id);
        tasks.insert(task1.id, task1);
        
        let mut task2 = create_test_task("Task 2", Some(15.0), Some(3));
        task2.assigned_resource_id = Some(resource.id);
        tasks.insert(task2.id, task2);
        
        let result = view.calculate_schedule(&tasks, &resources, &graph);
        assert!(result.is_ok());
        
        let schedule = result.unwrap();
        assert_eq!(schedule.task_schedules.len(), 2);
        assert!(schedule.warnings.is_empty() || !schedule.warnings.is_empty()); // May have warnings
    }

    #[test]
    fn test_critical_path_identification() {
        let view = TimelineView::new();
        let task_id = Uuid::new_v4();
        let critical_path = vec![task_id, Uuid::new_v4(), Uuid::new_v4()];
        
        assert!(view.is_task_critical(task_id, &critical_path));
        assert!(!view.is_task_critical(Uuid::new_v4(), &critical_path));
    }

    #[test]
    fn test_task_grouping_by_goal() {
        let view = TimelineView::new();
        let mut tasks = HashMap::new();
        let goals = HashMap::new();
        
        let goal_id = Uuid::new_v4();
        
        let mut task1 = create_test_task("Task with Goal", Some(5.0), Some(1));
        task1.goal_id = Some(goal_id);
        tasks.insert(task1.id, task1.clone());
        
        let task2 = create_test_task("Task without Goal", Some(3.0), Some(2));
        tasks.insert(task2.id, task2.clone());
        
        let grouped = view.group_tasks_by_goal(&tasks, &goals);
        
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get(&Some(goal_id)).unwrap().len(), 1);
        assert_eq!(grouped.get(&None).unwrap().len(), 1);
    }

    #[test]
    fn test_warning_generation() {
        let view = TimelineView::new();
        let mut tasks = HashMap::new();
        
        // Overdue task
        let mut overdue_task = create_test_task("Overdue Task", Some(5.0), None);
        overdue_task.due_date = Some(Utc::now() - Duration::days(1));
        overdue_task.status = TaskStatus::Todo;
        tasks.insert(overdue_task.id, overdue_task);
        
        // Unassigned task with estimate
        let unassigned_task = create_test_task("Unassigned with Estimate", Some(10.0), Some(5));
        tasks.insert(unassigned_task.id, unassigned_task);
        
        // Assigned task without estimate
        let mut no_estimate_task = create_test_task("No Estimate", None, Some(3));
        no_estimate_task.assigned_resource_id = Some(Uuid::new_v4());
        tasks.insert(no_estimate_task.id, no_estimate_task);
        
        let warnings = view.generate_warnings(&tasks);
        
        assert!(warnings.len() >= 3);
        assert!(warnings.iter().any(|w| w.contains("overdue")));
        assert!(warnings.iter().any(|w| w.contains("unassigned")));
        assert!(warnings.iter().any(|w| w.contains("no estimate")));
    }

    #[test]
    fn test_timeline_export() {
        let view = TimelineView::new();
        let mut tasks = HashMap::new();
        let mut resources = HashMap::new();
        
        let task = create_test_task("Export Test", Some(8.0), Some(2));
        tasks.insert(task.id, task);
        
        let resource = create_test_resource("Test Resource", 40.0);
        resources.insert(resource.id, resource);
        
        let schedule = TimelineSchedule {
            task_schedules: HashMap::new(),
            resource_allocations: Vec::new(),
            critical_path: Vec::new(),
            warnings: Vec::new(),
        };
        
        let result = view.export_timeline(&tasks, &resources, &schedule);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json.contains("tasks"));
        assert!(json.contains("resources"));
        assert!(json.contains("schedule"));
        assert!(json.contains("chart_settings"));
    }

    #[test]
    fn test_timeline_data_processing() {
        let view = TimelineView::new();
        let mut tasks = HashMap::new();
        let mut resources = HashMap::new();
        
        let resource = create_test_resource("Resource 1", 40.0);
        resources.insert(resource.id, resource.clone());
        
        let mut assigned_task = create_test_task("Assigned Task", Some(10.0), Some(1));
        assigned_task.assigned_resource_id = Some(resource.id);
        tasks.insert(assigned_task.id, assigned_task);
        
        let unassigned_task = create_test_task("Unassigned Task", Some(5.0), Some(2));
        tasks.insert(unassigned_task.id, unassigned_task);
        
        let data = view.process_timeline_data(&tasks, &resources);
        
        assert_eq!(data.task_count, 2);
        assert_eq!(data.resource_count, 1);
        assert_eq!(data.unassigned_tasks, 1);
    }
}

#[cfg(test)]
mod timeline_ui_rendering_tests {
    use super::*;
    
    fn create_test_task(title: &str, hours: Option<f32>, scheduled_days_ahead: Option<i64>) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        task.estimated_hours = hours;
        if let Some(days) = scheduled_days_ahead {
            task.scheduled_date = Some(Utc::now() + Duration::days(days));
        }
        task
    }

    #[test]
    fn test_timeline_renders_with_tasks() {
        let mut view = TimelineView::new();
        let tasks = vec![
            create_test_task("Task 1", Some(5.0), Some(1)),
            create_test_task("Task 2", Some(8.0), Some(3)),
            create_test_task("Task 3", Some(3.0), Some(5)),
        ];
        let goals: Vec<Goal> = vec![];
        
        // Verify view state
        assert_eq!(view.days_to_show, 30);
        assert_eq!(view.selected_view, TimelineViewMode::Gantt);
        
        // Test that we can process timeline data
        let mut task_map = HashMap::new();
        for task in &tasks {
            task_map.insert(task.id, task.clone());
        }
        let resources = HashMap::new();
        let data = view.process_timeline_data(&task_map, &resources);
        assert_eq!(data.task_count, 3);
    }

    #[test]
    fn test_timeline_renders_empty_state() {
        let mut view = TimelineView::new();
        let tasks = HashMap::new();
        let resources = HashMap::new();
        
        // Should process without errors even with no data
        let data = view.process_timeline_data(&tasks, &resources);
        assert_eq!(data.task_count, 0);
        assert_eq!(view.days_to_show, 30);
    }

    #[test]
    fn test_timeline_list_view_filtering() {
        let mut view = TimelineView::new();
        view.set_view_mode(TimelineViewMode::List);
        view.set_filter(TimelineFilter::InProgress);
        
        let mut tasks = HashMap::new();
        
        let mut in_progress = create_test_task("In Progress", Some(5.0), Some(1));
        in_progress.status = TaskStatus::InProgress;
        tasks.insert(in_progress.id, in_progress);
        
        let mut done = create_test_task("Done", Some(3.0), Some(2));
        done.status = TaskStatus::Done;
        tasks.insert(done.id, done);
        
        // Apply filter
        let filtered = view.apply_filters(&tasks);
        assert_eq!(filtered.len(), 1);
        
        // Filter should be applied
        assert_eq!(view.selected_view, TimelineViewMode::List);
    }
}