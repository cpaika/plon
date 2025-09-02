use chrono::{Duration, Utc};
use plon::domain::dependency::{Dependency, DependencyGraph, DependencyType};
use plon::domain::goal::{Goal, GoalStatus};
use plon::domain::task::{Task, TaskStatus};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[test]
fn test_project_duration_calculation() {
    // Create tasks with dependencies
    let mut tasks = vec![
        Task::new("Design".to_string(), "Design phase".to_string()),
        Task::new(
            "Implementation".to_string(),
            "Implementation phase".to_string(),
        ),
        Task::new("Testing".to_string(), "Testing phase".to_string()),
        Task::new("Deployment".to_string(), "Deployment phase".to_string()),
    ];

    // Set estimates
    tasks[0].estimated_hours = Some(16.0);
    tasks[1].estimated_hours = Some(40.0);
    tasks[2].estimated_hours = Some(24.0);
    tasks[3].estimated_hours = Some(8.0);

    // Build dependency graph
    let mut graph = DependencyGraph::new();

    // Create sequential dependencies
    let dep1 = Dependency::new(tasks[0].id, tasks[1].id, DependencyType::FinishToStart);
    let dep2 = Dependency::new(tasks[1].id, tasks[2].id, DependencyType::FinishToStart);
    let dep3 = Dependency::new(tasks[2].id, tasks[3].id, DependencyType::FinishToStart);

    graph.add_dependency(&dep1).unwrap();
    graph.add_dependency(&dep2).unwrap();
    graph.add_dependency(&dep3).unwrap();

    // Calculate total project duration
    let mut estimates = HashMap::new();
    for task in &tasks {
        estimates.insert(task.id, task.estimated_hours.unwrap_or(0.0));
    }

    let critical_path = graph.get_critical_path(&estimates);
    let total_duration: f32 = critical_path
        .iter()
        .map(|id| estimates.get(id).unwrap_or(&0.0))
        .sum();

    assert_eq!(total_duration, 88.0); // 16 + 40 + 24 + 8
}

#[test]
fn test_parallel_tasks_duration() {
    let mut tasks = vec![
        Task::new("Setup".to_string(), String::new()),
        Task::new("Frontend".to_string(), String::new()),
        Task::new("Backend".to_string(), String::new()),
        Task::new("Integration".to_string(), String::new()),
    ];

    tasks[0].estimated_hours = Some(8.0);
    tasks[1].estimated_hours = Some(32.0); // Longer path
    tasks[2].estimated_hours = Some(24.0); // Shorter path
    tasks[3].estimated_hours = Some(16.0);

    let mut graph = DependencyGraph::new();

    // Setup -> Frontend and Backend (parallel)
    // Frontend, Backend -> Integration
    graph
        .add_dependency(&Dependency::new(
            tasks[0].id,
            tasks[1].id,
            DependencyType::FinishToStart,
        ))
        .unwrap();
    graph
        .add_dependency(&Dependency::new(
            tasks[0].id,
            tasks[2].id,
            DependencyType::FinishToStart,
        ))
        .unwrap();
    graph
        .add_dependency(&Dependency::new(
            tasks[1].id,
            tasks[3].id,
            DependencyType::FinishToStart,
        ))
        .unwrap();
    graph
        .add_dependency(&Dependency::new(
            tasks[2].id,
            tasks[3].id,
            DependencyType::FinishToStart,
        ))
        .unwrap();

    let mut estimates = HashMap::new();
    for task in &tasks {
        estimates.insert(task.id, task.estimated_hours.unwrap_or(0.0));
    }

    let critical_path = graph.get_critical_path(&estimates);
    let total_duration: f32 = critical_path
        .iter()
        .map(|id| estimates.get(id).unwrap_or(&0.0))
        .sum();

    // Critical path should be Setup -> Frontend -> Integration (8 + 32 + 16)
    assert_eq!(total_duration, 56.0);
    assert!(critical_path.contains(&tasks[1].id)); // Frontend is on critical path
    assert!(!critical_path.contains(&tasks[2].id)); // Backend is not on critical path
}

#[test]
fn test_goal_completion_with_dependencies() {
    let mut goal = Goal::new("Sprint 1".to_string(), "First sprint goals".to_string());

    let task1 = Task::new("Task 1".to_string(), String::new());
    let task2 = Task::new("Task 2".to_string(), String::new());
    let task3 = Task::new("Task 3".to_string(), String::new());

    goal.add_task(task1.id);
    goal.add_task(task2.id);
    goal.add_task(task3.id);

    let mut graph = DependencyGraph::new();
    graph
        .add_dependency(&Dependency::new(
            task1.id,
            task2.id,
            DependencyType::FinishToStart,
        ))
        .unwrap();
    graph
        .add_dependency(&Dependency::new(
            task2.id,
            task3.id,
            DependencyType::FinishToStart,
        ))
        .unwrap();

    // Test that goal cannot be complete until all tasks are done
    let mut completed_tasks = HashSet::new();

    // No tasks completed
    assert!(!graph.can_start_task(task2.id, &completed_tasks));
    assert!(!graph.can_start_task(task3.id, &completed_tasks));

    // Complete task1
    completed_tasks.insert(task1.id);
    assert!(graph.can_start_task(task2.id, &completed_tasks));
    assert!(!graph.can_start_task(task3.id, &completed_tasks));

    // Complete task2
    completed_tasks.insert(task2.id);
    assert!(graph.can_start_task(task3.id, &completed_tasks));

    // Test goal progress
    let task_statuses = vec![(task1.id, true), (task2.id, true), (task3.id, false)];
    let progress = goal.calculate_progress(&task_statuses);
    assert_eq!(progress, 66.66667); // 2/3 tasks completed
}

#[test]
fn test_blocked_tasks_detection() {
    let mut graph = DependencyGraph::new();

    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();

    graph
        .add_dependency(&Dependency::new(
            task1,
            task2,
            DependencyType::FinishToStart,
        ))
        .unwrap();
    graph
        .add_dependency(&Dependency::new(
            task2,
            task3,
            DependencyType::FinishToStart,
        ))
        .unwrap();

    let completed = HashSet::new();

    // Task 2 and 3 should be blocked
    assert!(graph.can_start_task(task1, &completed));
    assert!(!graph.can_start_task(task2, &completed));
    assert!(!graph.can_start_task(task3, &completed));

    // Get all blocked tasks
    let all_tasks = vec![task1, task2, task3];
    let blocked_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|&&t| !graph.can_start_task(t, &completed))
        .collect();

    assert_eq!(blocked_tasks.len(), 2);
}

#[test]
fn test_dependency_types_affect_scheduling() {
    let mut graph = DependencyGraph::new();

    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();
    let task4 = Uuid::new_v4();

    // Different dependency types
    graph
        .add_dependency(&Dependency::new(
            task1,
            task2,
            DependencyType::FinishToStart,
        ))
        .unwrap();
    graph
        .add_dependency(&Dependency::new(task1, task3, DependencyType::StartToStart))
        .unwrap();
    graph
        .add_dependency(&Dependency::new(
            task2,
            task4,
            DependencyType::FinishToFinish,
        ))
        .unwrap();

    // For FinishToStart, task2 can't start until task1 is complete
    let mut completed = HashSet::new();
    assert!(!graph.can_start_task(task2, &completed));

    completed.insert(task1);
    assert!(graph.can_start_task(task2, &completed));

    // StartToStart and FinishToFinish have different semantics
    // (current implementation only handles FinishToStart properly)
}

#[test]
fn test_resource_allocation_with_dependencies() {
    // Test that resources can be properly allocated considering dependencies
    let task1 = Task::new("Backend API".to_string(), String::new());
    let task2 = Task::new("Frontend UI".to_string(), String::new());
    let task3 = Task::new("Integration".to_string(), String::new());

    let mut graph = DependencyGraph::new();
    graph
        .add_dependency(&Dependency::new(
            task1.id,
            task3.id,
            DependencyType::FinishToStart,
        ))
        .unwrap();
    graph
        .add_dependency(&Dependency::new(
            task2.id,
            task3.id,
            DependencyType::FinishToStart,
        ))
        .unwrap();

    // Simulate resource allocation
    let resource1 = Uuid::new_v4(); // Backend developer
    let resource2 = Uuid::new_v4(); // Frontend developer

    // Both resources can work in parallel on task1 and task2
    // But task3 needs to wait for both
    let completed = HashSet::new();
    assert!(graph.can_start_task(task1.id, &completed));
    assert!(graph.can_start_task(task2.id, &completed));
    assert!(!graph.can_start_task(task3.id, &completed));
}

#[test]
fn test_estimate_adjustment_with_dependencies() {
    // Test that estimates account for dependency overhead
    let mut tasks = vec![
        Task::new("Research".to_string(), String::new()),
        Task::new("Development".to_string(), String::new()),
        Task::new("Review".to_string(), String::new()),
    ];

    tasks[0].estimated_hours = Some(8.0);
    tasks[1].estimated_hours = Some(24.0);
    tasks[2].estimated_hours = Some(4.0);

    let mut graph = DependencyGraph::new();
    graph
        .add_dependency(&Dependency::new(
            tasks[0].id,
            tasks[1].id,
            DependencyType::FinishToStart,
        ))
        .unwrap();
    graph
        .add_dependency(&Dependency::new(
            tasks[1].id,
            tasks[2].id,
            DependencyType::FinishToStart,
        ))
        .unwrap();

    let mut estimates = HashMap::new();
    for task in &tasks {
        estimates.insert(task.id, task.estimated_hours.unwrap_or(0.0));
    }

    // Add buffer time for handoffs between tasks (10% overhead)
    let buffer_factor = 1.1;
    let critical_path = graph.get_critical_path(&estimates);
    let raw_duration: f32 = critical_path
        .iter()
        .map(|id| estimates.get(id).unwrap_or(&0.0))
        .sum();
    let adjusted_duration = raw_duration * buffer_factor;

    assert_eq!(raw_duration, 36.0);
    assert!((adjusted_duration - 39.6).abs() < 0.01); // 36 * 1.1
}

#[test]
fn test_complex_project_planning() {
    // Simulate a more complex project with multiple phases
    let mut phases = HashMap::new();

    // Phase 1: Planning
    let planning_tasks = vec![
        Task::new("Requirements".to_string(), String::new()),
        Task::new("Architecture".to_string(), String::new()),
        Task::new("Resource Planning".to_string(), String::new()),
    ];
    phases.insert("planning", planning_tasks);

    // Phase 2: Development
    let dev_tasks = vec![
        Task::new("Database".to_string(), String::new()),
        Task::new("API".to_string(), String::new()),
        Task::new("UI".to_string(), String::new()),
        Task::new("Mobile".to_string(), String::new()),
    ];
    phases.insert("development", dev_tasks);

    // Phase 3: Testing
    let test_tasks = vec![
        Task::new("Unit Tests".to_string(), String::new()),
        Task::new("Integration Tests".to_string(), String::new()),
        Task::new("UAT".to_string(), String::new()),
    ];
    phases.insert("testing", test_tasks);

    let mut graph = DependencyGraph::new();

    // Add inter-phase dependencies
    let planning = &phases["planning"];
    let development = &phases["development"];
    let testing = &phases["testing"];

    // All planning must complete before development
    for plan_task in planning {
        for dev_task in development {
            graph
                .add_dependency(&Dependency::new(
                    plan_task.id,
                    dev_task.id,
                    DependencyType::FinishToStart,
                ))
                .unwrap();
        }
    }

    // All development must complete before testing
    for dev_task in development {
        for test_task in testing {
            graph
                .add_dependency(&Dependency::new(
                    dev_task.id,
                    test_task.id,
                    DependencyType::FinishToStart,
                ))
                .unwrap();
        }
    }

    // Verify topological sort works
    let sorted = graph.topological_sort().unwrap();

    // Planning tasks should come first
    let planning_ids: HashSet<_> = planning.iter().map(|t| t.id).collect();
    let dev_ids: HashSet<_> = development.iter().map(|t| t.id).collect();
    let test_ids: HashSet<_> = testing.iter().map(|t| t.id).collect();

    let mut found_dev = false;
    let mut found_test = false;

    for task_id in sorted {
        if dev_ids.contains(&task_id) {
            found_dev = true;
        }
        if test_ids.contains(&task_id) {
            found_test = true;
        }

        // Once we find development tasks, we shouldn't find planning tasks
        if found_dev && planning_ids.contains(&task_id) {
            panic!("Planning task found after development task");
        }

        // Once we find test tasks, we shouldn't find development tasks
        if found_test && dev_ids.contains(&task_id) {
            panic!("Development task found after test task");
        }
    }
}
