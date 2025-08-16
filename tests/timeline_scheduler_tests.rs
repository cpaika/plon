use plon::domain::{task::*, resource::*, dependency::*};
use plon::services::timeline_scheduler::*;
use chrono::{NaiveDate, Utc, DateTime};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[test]
fn test_schedule_single_task_no_dependencies() {
    let mut scheduler = TimelineScheduler::new();
    
    let task = Task::new("Task 1".to_string(), "Description".to_string());
    let task_id = task.id;
    let mut task = task;
    task.estimated_hours = Some(8.0);
    
    let resource = Resource::new("Developer".to_string(), "Dev".to_string(), 40.0);
    let resource_id = resource.id;
    task.assigned_resource_id = Some(resource_id);
    
    let mut tasks = HashMap::new();
    tasks.insert(task_id, task);
    
    let mut resources = HashMap::new();
    resources.insert(resource_id, resource);
    
    let graph = DependencyGraph::new();
    
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let schedule = scheduler.calculate_schedule(&tasks, &resources, &graph, start_date);
    
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap();
    assert_eq!(schedule.task_schedules.len(), 1);
    
    let task_schedule = &schedule.task_schedules[&task_id];
    assert_eq!(task_schedule.start_date, start_date);
    assert_eq!(task_schedule.end_date, start_date); // 8 hours = 1 day
    assert_eq!(task_schedule.allocated_hours, 8.0);
}

#[test]
fn test_schedule_tasks_with_dependencies() {
    let mut scheduler = TimelineScheduler::new();
    
    // Create two tasks with dependency
    let task1 = Task::new("Task 1".to_string(), "First task".to_string());
    let task1_id = task1.id;
    let mut task1 = task1;
    task1.estimated_hours = Some(8.0);
    
    let task2 = Task::new("Task 2".to_string(), "Second task".to_string());
    let task2_id = task2.id;
    let mut task2 = task2;
    task2.estimated_hours = Some(16.0); // 2 days
    
    let resource = Resource::new("Developer".to_string(), "Dev".to_string(), 40.0);
    let resource_id = resource.id;
    task1.assigned_resource_id = Some(resource_id);
    task2.assigned_resource_id = Some(resource_id);
    
    let mut tasks = HashMap::new();
    tasks.insert(task1_id, task1);
    tasks.insert(task2_id, task2);
    
    let mut resources = HashMap::new();
    resources.insert(resource_id, resource);
    
    let mut graph = DependencyGraph::new();
    let dep = Dependency::new(task1_id, task2_id, DependencyType::FinishToStart);
    graph.add_dependency(&dep).unwrap();
    
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let schedule = scheduler.calculate_schedule(&tasks, &resources, &graph, start_date);
    
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap();
    
    let task1_schedule = &schedule.task_schedules[&task1_id];
    let task2_schedule = &schedule.task_schedules[&task2_id];
    
    assert_eq!(task1_schedule.start_date, start_date);
    assert_eq!(task1_schedule.end_date, start_date);
    
    // Task 2 should start after task 1 ends
    assert_eq!(task2_schedule.start_date, NaiveDate::from_ymd_opt(2024, 1, 2).unwrap());
    assert_eq!(task2_schedule.end_date, NaiveDate::from_ymd_opt(2024, 1, 3).unwrap());
}

#[test]
fn test_resource_allocation_conflict() {
    let mut scheduler = TimelineScheduler::new();
    
    // Create two tasks that would overlap without resource constraints
    let task1 = Task::new("Task 1".to_string(), "First task".to_string());
    let task1_id = task1.id;
    let mut task1 = task1;
    task1.estimated_hours = Some(40.0); // Full week
    
    let task2 = Task::new("Task 2".to_string(), "Second task".to_string());
    let task2_id = task2.id;
    let mut task2 = task2;
    task2.estimated_hours = Some(20.0);
    
    let resource = Resource::new("Developer".to_string(), "Dev".to_string(), 40.0);
    let resource_id = resource.id;
    task1.assigned_resource_id = Some(resource_id);
    task2.assigned_resource_id = Some(resource_id);
    
    let mut tasks = HashMap::new();
    tasks.insert(task1_id, task1);
    tasks.insert(task2_id, task2);
    
    let mut resources = HashMap::new();
    resources.insert(resource_id, resource);
    
    let graph = DependencyGraph::new(); // No dependencies
    
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(); // Monday
    let schedule = scheduler.calculate_schedule(&tasks, &resources, &graph, start_date);
    
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap();
    
    let task1_schedule = &schedule.task_schedules[&task1_id];
    let task2_schedule = &schedule.task_schedules[&task2_id];
    
    // Both tasks should be scheduled, but not overlapping
    // One task should take the full first week (40 hours)
    // The other should start after it
    
    // Check that they don't overlap
    let (first_task, second_task) = if task1_schedule.start_date <= task2_schedule.start_date {
        (task1_schedule, task2_schedule)
    } else {
        (task2_schedule, task1_schedule)
    };
    
    assert_eq!(first_task.start_date, start_date);
    // First task should end on Friday (5 days * 8 hours = 40 hours)
    assert_eq!(first_task.end_date, NaiveDate::from_ymd_opt(2024, 1, 5).unwrap());
    
    // Second task should start the next Monday
    assert_eq!(second_task.start_date, NaiveDate::from_ymd_opt(2024, 1, 8).unwrap());
}

#[test]
fn test_multiple_resources_parallel_execution() {
    let mut scheduler = TimelineScheduler::new();
    
    // Create two tasks that can run in parallel with different resources
    let task1 = Task::new("Frontend".to_string(), "UI work".to_string());
    let task1_id = task1.id;
    let mut task1 = task1;
    task1.estimated_hours = Some(16.0);
    
    let task2 = Task::new("Backend".to_string(), "API work".to_string());
    let task2_id = task2.id;
    let mut task2 = task2;
    task2.estimated_hours = Some(24.0);
    
    let frontend_dev = Resource::new("Alice".to_string(), "Frontend".to_string(), 40.0);
    let frontend_id = frontend_dev.id;
    let backend_dev = Resource::new("Bob".to_string(), "Backend".to_string(), 40.0);
    let backend_id = backend_dev.id;
    
    task1.assigned_resource_id = Some(frontend_id);
    task2.assigned_resource_id = Some(backend_id);
    
    let mut tasks = HashMap::new();
    tasks.insert(task1_id, task1);
    tasks.insert(task2_id, task2);
    
    let mut resources = HashMap::new();
    resources.insert(frontend_id, frontend_dev);
    resources.insert(backend_id, backend_dev);
    
    let graph = DependencyGraph::new();
    
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let schedule = scheduler.calculate_schedule(&tasks, &resources, &graph, start_date);
    
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap();
    
    let task1_schedule = &schedule.task_schedules[&task1_id];
    let task2_schedule = &schedule.task_schedules[&task2_id];
    
    // Both tasks should start on the same day (parallel execution)
    assert_eq!(task1_schedule.start_date, start_date);
    assert_eq!(task2_schedule.start_date, start_date);
}

#[test]
fn test_critical_path_calculation() {
    let mut scheduler = TimelineScheduler::new();
    
    // Create a diamond dependency pattern
    let start_task = Task::new("Start".to_string(), "".to_string());
    let start_id = start_task.id;
    let mut start_task = start_task;
    start_task.estimated_hours = Some(8.0);
    
    let path_a = Task::new("Path A".to_string(), "".to_string());
    let path_a_id = path_a.id;
    let mut path_a = path_a;
    path_a.estimated_hours = Some(40.0); // Longer path
    
    let path_b = Task::new("Path B".to_string(), "".to_string());
    let path_b_id = path_b.id;
    let mut path_b = path_b;
    path_b.estimated_hours = Some(16.0);
    
    let end_task = Task::new("End".to_string(), "".to_string());
    let end_id = end_task.id;
    let mut end_task = end_task;
    end_task.estimated_hours = Some(8.0);
    
    let resource = Resource::new("Dev".to_string(), "Dev".to_string(), 40.0);
    let resource_id = resource.id;
    
    start_task.assigned_resource_id = Some(resource_id);
    path_a.assigned_resource_id = Some(resource_id);
    path_b.assigned_resource_id = Some(resource_id);
    end_task.assigned_resource_id = Some(resource_id);
    
    let mut tasks = HashMap::new();
    tasks.insert(start_id, start_task);
    tasks.insert(path_a_id, path_a);
    tasks.insert(path_b_id, path_b);
    tasks.insert(end_id, end_task);
    
    let mut resources = HashMap::new();
    resources.insert(resource_id, resource);
    
    let mut graph = DependencyGraph::new();
    graph.add_dependency(&Dependency::new(start_id, path_a_id, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(start_id, path_b_id, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(path_a_id, end_id, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(path_b_id, end_id, DependencyType::FinishToStart)).unwrap();
    
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let schedule = scheduler.calculate_schedule(&tasks, &resources, &graph, start_date);
    
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap();
    
    // Critical path should be Start -> Path A -> End
    assert_eq!(schedule.critical_path.len(), 3);
    assert_eq!(schedule.critical_path[0], start_id);
    assert_eq!(schedule.critical_path[1], path_a_id);
    assert_eq!(schedule.critical_path[2], end_id);
    
    // Total duration should be sum of critical path
    let total_days = schedule.get_total_duration_days();
    assert!(total_days >= 7); // 1 + 5 + 1 days minimum
}

#[test]
fn test_resource_availability_constraints() {
    let mut scheduler = TimelineScheduler::new();
    
    let task = Task::new("Task".to_string(), "".to_string());
    let task_id = task.id;
    let mut task = task;
    task.estimated_hours = Some(20.0);
    
    let mut resource = Resource::new("Part-timer".to_string(), "Dev".to_string(), 20.0); // Half-time
    let resource_id = resource.id;
    
    // Set specific availability
    resource.set_availability(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 4.0);
    resource.set_availability(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 4.0);
    resource.set_availability(NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 0.0); // Not available
    resource.set_availability(NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(), 4.0);
    resource.set_availability(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(), 4.0);
    
    task.assigned_resource_id = Some(resource_id);
    
    let mut tasks = HashMap::new();
    tasks.insert(task_id, task);
    
    let mut resources = HashMap::new();
    resources.insert(resource_id, resource);
    
    let graph = DependencyGraph::new();
    
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let schedule = scheduler.calculate_schedule(&tasks, &resources, &graph, start_date);
    
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap();
    
    let task_schedule = &schedule.task_schedules[&task_id];
    // Should account for the day with 0 availability
    assert!(task_schedule.end_date >= NaiveDate::from_ymd_opt(2024, 1, 8).unwrap());
}

#[test]
fn test_unassigned_task_scheduling() {
    let mut scheduler = TimelineScheduler::new();
    
    let task = Task::new("Unassigned".to_string(), "".to_string());
    let task_id = task.id;
    let mut task = task;
    task.estimated_hours = Some(8.0);
    // No resource assigned
    
    let mut tasks = HashMap::new();
    tasks.insert(task_id, task);
    
    let resources = HashMap::new(); // No resources
    let graph = DependencyGraph::new();
    
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let schedule = scheduler.calculate_schedule(&tasks, &resources, &graph, start_date);
    
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap();
    
    // Unassigned task should still be scheduled (with warnings)
    assert_eq!(schedule.task_schedules.len(), 1);
    assert!(schedule.warnings.len() > 0);
    assert!(schedule.warnings[0].contains("unassigned"));
}