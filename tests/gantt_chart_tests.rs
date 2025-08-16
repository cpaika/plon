use plon::ui::widgets::gantt_chart::*;
use plon::domain::{task::*, resource::*};
use plon::services::timeline_scheduler::*;
use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

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
    
    chart.zoom_in();
    assert!(chart.zoom_level > 1.0);
    
    chart.zoom_out();
    assert_eq!(chart.zoom_level, 1.0);
    
    chart.zoom_out();
    assert!(chart.zoom_level < 1.0);
    
    chart.reset_zoom();
    assert_eq!(chart.zoom_level, 1.0);
}

#[test]
fn test_gantt_chart_date_range() {
    let mut chart = GanttChart::new();
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    
    chart.set_start_date(start_date);
    assert_eq!(chart.get_start_date(), start_date);
    
    chart.set_days_to_show(60);
    assert_eq!(chart.days_to_show, 60);
    
    let end_date = chart.get_end_date();
    assert_eq!(end_date, start_date + chrono::Duration::days(59));
}

#[test]
fn test_gantt_bar_calculation() {
    let mut chart = GanttChart::new();
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    chart.set_start_date(start_date);
    
    let task_start = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
    let task_end = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
    
    let bar = chart.calculate_bar_position(task_start, task_end, 1000.0);
    
    assert!(bar.x > 0.0);
    assert!(bar.width > 0.0);
    assert_eq!(bar.duration_days, 6);
}

#[test]
fn test_gantt_task_grouping_by_resource() {
    let mut chart = GanttChart::new();
    
    let resource1_id = Uuid::new_v4();
    let resource2_id = Uuid::new_v4();
    
    let mut task1 = Task::new("Task 1".to_string(), "".to_string());
    task1.assigned_resource_id = Some(resource1_id);
    
    let mut task2 = Task::new("Task 2".to_string(), "".to_string());
    task2.assigned_resource_id = Some(resource2_id);
    
    let mut task3 = Task::new("Task 3".to_string(), "".to_string());
    task3.assigned_resource_id = Some(resource1_id);
    
    let mut tasks = HashMap::new();
    tasks.insert(task1.id, task1);
    tasks.insert(task2.id, task2);
    tasks.insert(task3.id, task3);
    
    let grouped = chart.group_tasks_by_resource(&tasks);
    
    assert_eq!(grouped.len(), 3); // 2 resources + 1 unassigned group
    assert_eq!(grouped.get(&Some(resource1_id)).unwrap().len(), 2);
    assert_eq!(grouped.get(&Some(resource2_id)).unwrap().len(), 1);
}

#[test]
fn test_gantt_critical_path_highlighting() {
    let mut chart = GanttChart::new();
    
    let task1_id = Uuid::new_v4();
    let task2_id = Uuid::new_v4();
    let task3_id = Uuid::new_v4();
    
    let critical_path = vec![task1_id, task3_id];
    chart.set_critical_path(critical_path.clone());
    
    assert!(chart.is_on_critical_path(task1_id));
    assert!(!chart.is_on_critical_path(task2_id));
    assert!(chart.is_on_critical_path(task3_id));
}

#[test]
fn test_gantt_dependency_lines() {
    let mut chart = GanttChart::new();
    chart.set_start_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    
    let task1_schedule = TaskSchedule {
        task_id: Uuid::new_v4(),
        resource_id: None,
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        allocated_hours: 40.0,
    };
    
    let task2_schedule = TaskSchedule {
        task_id: Uuid::new_v4(),
        resource_id: None,
        start_date: NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 12).unwrap(),
        allocated_hours: 40.0,
    };
    
    let dependency_line = chart.calculate_dependency_line(
        &task1_schedule,
        &task2_schedule,
        100.0,
        200.0,
        1000.0
    );
    
    assert!(dependency_line.start_x > 0.0);
    assert!(dependency_line.end_x > dependency_line.start_x);
    assert_eq!(dependency_line.start_y, 100.0);
    assert_eq!(dependency_line.end_y, 200.0);
}

#[test]
fn test_gantt_milestone_rendering() {
    let mut chart = GanttChart::new();
    
    let milestone_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let milestone = Milestone {
        id: Uuid::new_v4(),
        name: "Project Milestone".to_string(),
        date: milestone_date,
        color: GanttColor::Red,
    };
    
    chart.add_milestone(milestone.clone());
    
    assert_eq!(chart.get_milestones().len(), 1);
    assert_eq!(chart.get_milestones()[0].name, "Project Milestone");
}

#[test]
fn test_gantt_today_line() {
    let chart = GanttChart::new();
    let today = chrono::Local::now().naive_local().date();
    
    let today_position = chart.calculate_today_line_position(1000.0);
    
    if today >= chart.get_start_date() && today <= chart.get_end_date() {
        assert!(today_position.is_some());
        let pos = today_position.unwrap();
        assert!(pos >= 0.0 && pos <= 1000.0);
    } else {
        assert!(today_position.is_none());
    }
}

#[test]
fn test_gantt_resource_utilization_display() {
    let chart = GanttChart::new();
    
    let mut resource = Resource::new("Developer".to_string(), "Dev".to_string(), 40.0);
    resource.current_load = 30.0;
    
    let utilization = chart.calculate_resource_utilization(&resource);
    assert_eq!(utilization.percentage, 75.0);
    assert_eq!(utilization.color, GanttColor::Green); // 75% should be green (under 80%)
    
    resource.current_load = 45.0;
    let utilization = chart.calculate_resource_utilization(&resource);
    assert_eq!(utilization.percentage, 112.5);
    assert_eq!(utilization.color, GanttColor::Red); // Over 100% should be red
}

#[test]
fn test_gantt_weekend_highlighting() {
    let mut chart = GanttChart::new();
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(); // Monday
    
    chart.set_start_date(start_date);
    
    let weekends = chart.get_weekend_positions(1000.0);
    
    // Should have at least 4 weekends in 30 days
    assert!(weekends.len() >= 8); // 2 days per weekend
    
    // First weekend should be Saturday and Sunday
    // Note: Rust's weekday() returns 5 for Saturday, 6 for Sunday
    assert_eq!(weekends[0].day_of_week, 5); // Saturday
    assert_eq!(weekends[1].day_of_week, 6); // Sunday
}

#[test]
fn test_gantt_export_data() {
    let mut chart = GanttChart::new();
    
    let task = Task::new("Test Task".to_string(), "Description".to_string());
    let resource = Resource::new("Dev".to_string(), "Developer".to_string(), 40.0);
    
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
    
    let export_data = chart.export_to_json(&tasks, &resources, &schedule);
    assert!(export_data.is_ok());
    
    let json = export_data.unwrap();
    assert!(json.contains("tasks"));
    assert!(json.contains("resources"));
    assert!(json.contains("schedule"));
}