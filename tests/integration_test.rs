use plon::domain::task::{Task, TaskStatus, Priority};
use plon::domain::goal::{Goal, GoalStatus};
use plon::domain::recurring::{RecurringTaskTemplate, RecurrenceRule, RecurrencePattern};
use plon::domain::dependency::{Dependency, DependencyType, DependencyGraph};
use chrono::{Utc, NaiveTime};
use uuid::Uuid;

#[test]
fn test_complete_workflow() {
    // Test 1: Create and manage tasks
    let mut task1 = Task::new("Design UI".to_string(), "Create mockups for the new dashboard".to_string());
    let mut task2 = Task::new("Implement Backend".to_string(), "Set up API endpoints".to_string());
    let mut task3 = Task::new("Write Tests".to_string(), "Create unit and integration tests".to_string());
    
    // Set task properties
    task1.priority = Priority::High;
    task1.estimated_hours = Some(8.0);
    task1.status = TaskStatus::InProgress;
    
    task2.priority = Priority::Critical;
    task2.estimated_hours = Some(16.0);
    
    task3.priority = Priority::Medium;
    task3.estimated_hours = Some(12.0);
    
    // Add subtasks
    task1.add_subtask("Create wireframes".to_string());
    task1.add_subtask("Design color scheme".to_string());
    task1.add_subtask("Create component library".to_string());
    
    // Complete a subtask
    task1.subtasks[0].completed = true;
    task1.subtasks[0].completed_at = Some(Utc::now());
    
    let (completed, total) = task1.subtask_progress();
    assert_eq!(completed, 1);
    assert_eq!(total, 3);
    
    // Test 2: Create and manage goals
    let mut goal = Goal::new(
        "Q1 Product Launch".to_string(),
        "Launch the new product features by end of Q1".to_string()
    );
    
    goal.add_task(task1.id);
    goal.add_task(task2.id);
    goal.add_task(task3.id);
    
    assert_eq!(goal.task_ids.len(), 3);
    
    // Update goal progress
    let tasks = vec![task1.clone(), task2.clone(), task3.clone()];
    goal.update_progress(&tasks);
    
    // Check goal status
    assert_eq!(goal.status, GoalStatus::Active);
    assert!(goal.progress >= 0.0 && goal.progress <= 100.0);
    
    // Test 3: Create dependencies between tasks
    let mut dep_graph = DependencyGraph::new();
    
    // Task2 depends on Task1 (Finish-to-Start)
    let dep1 = Dependency::new(task1.id, task2.id, DependencyType::FinishToStart);
    let _ = dep_graph.add_dependency(&dep1);
    
    // Task3 depends on Task2 (Finish-to-Start)
    let dep2 = Dependency::new(task2.id, task3.id, DependencyType::FinishToStart);
    let _ = dep_graph.add_dependency(&dep2);
    
    // Check dependency graph is valid (no cycles)
    assert!(!dep_graph.has_cycle());
    
    // Check for cycles (should not have any)
    assert!(!dep_graph.has_cycle());
    
    // Get dependencies for task2
    let task2_deps = dep_graph.get_dependencies_for(task2.id);
    assert_eq!(task2_deps.len(), 1);
    assert_eq!(task2_deps[0].from_task_id, task1.id);
    
    // Get dependents for task2
    let task2_dependents = dep_graph.get_dependents_for(task2.id);
    assert_eq!(task2_dependents.len(), 1);
    assert_eq!(task2_dependents[0].to_task_id, task3.id);
    
    // Test 4: Create recurring tasks
    let rule = RecurrenceRule {
        pattern: RecurrencePattern::Weekly,
        interval: 1,
        days_of_week: vec![chrono::Weekday::Mon, chrono::Weekday::Wed, chrono::Weekday::Fri],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: Some(10),
        occurrences_count: 0,
    };
    
    let mut recurring_template = RecurringTaskTemplate::new(
        "Team Standup".to_string(),
        "Daily team synchronization meeting".to_string(),
        rule,
    );
    
    // Generate a task from the template
    let generated_task = recurring_template.generate_task();
    assert!(generated_task.is_some());
    
    let generated = generated_task.unwrap();
    assert_eq!(generated.title, "Team Standup");
    assert_eq!(recurring_template.recurrence_rule.occurrences_count, 1);
    
    // Test 5: Task position and map view
    task1.position.x = 100.0;
    task1.position.y = 200.0;
    
    task2.position.x = 300.0;
    task2.position.y = 200.0;
    
    task3.position.x = 500.0;
    task3.position.y = 200.0;
    
    // Goal position
    goal.position_x = 300.0;
    goal.position_y = 50.0;
    goal.position_width = 400.0;
    goal.position_height = 100.0;
    
    // Test 6: Critical path calculation
    let mut task_durations = std::collections::HashMap::new();
    task_durations.insert(task1.id, 8.0);
    task_durations.insert(task2.id, 16.0);
    task_durations.insert(task3.id, 12.0);
    
    let critical_path = dep_graph.get_critical_path(&task_durations);
    assert!(!critical_path.is_empty());
    
    // The critical path should include all three tasks since they're sequential
    assert!(critical_path.contains(&task1.id));
    assert!(critical_path.contains(&task2.id));
    assert!(critical_path.contains(&task3.id));
    
    // Test 7: Task filtering and search
    let all_tasks = vec![task1.clone(), task2.clone(), task3.clone()];
    
    // Filter by status
    let in_progress: Vec<_> = all_tasks.iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .collect();
    assert_eq!(in_progress.len(), 1);
    
    // Filter by priority
    let high_priority: Vec<_> = all_tasks.iter()
        .filter(|t| t.priority == Priority::Critical || t.priority == Priority::High)
        .collect();
    assert_eq!(high_priority.len(), 2);
    
    // Test 8: Task completion workflow
    task1.status = TaskStatus::Done;
    task1.actual_hours = Some(7.5);
    
    // Since task1 is done, task2 can start
    task2.status = TaskStatus::InProgress;
    
    // Update goal progress again
    let updated_tasks = vec![
        {
            let mut t = task1.clone();
            t.status = TaskStatus::Done;
            t
        },
        task2.clone(),
        task3.clone()
    ];
    goal.update_progress(&updated_tasks);
    
    // Progress should be higher now
    assert!(goal.progress > 0.0);
    
    println!("All integration tests passed!");
}

#[test]
fn test_dependency_cycle_detection() {
    let mut dep_graph = DependencyGraph::new();
    
    let task1_id = Uuid::new_v4();
    let task2_id = Uuid::new_v4();
    let task3_id = Uuid::new_v4();
    
    // Create a cycle: 1 -> 2 -> 3 -> 1
    let _ = dep_graph.add_dependency(&Dependency::new(task1_id, task2_id, DependencyType::FinishToStart));
    let _ = dep_graph.add_dependency(&Dependency::new(task2_id, task3_id, DependencyType::FinishToStart));
    let _ = dep_graph.add_dependency(&Dependency::new(task3_id, task1_id, DependencyType::FinishToStart));
    
    // Should detect the cycle
    assert!(dep_graph.has_cycle());
}

#[test]
fn test_recurring_task_limits() {
    let rule = RecurrenceRule {
        pattern: RecurrencePattern::Daily,
        interval: 1,
        days_of_week: vec![],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: Some(3),
        occurrences_count: 0,
    };
    
    let mut template = RecurringTaskTemplate::new(
        "Limited Task".to_string(),
        "This task can only occur 3 times".to_string(),
        rule,
    );
    
    // Generate tasks up to the limit
    for i in 0..3 {
        let task = template.generate_task();
        assert!(task.is_some(), "Should generate task {}", i + 1);
    }
    
    // Should not generate more tasks
    let task = template.generate_task();
    assert!(task.is_none(), "Should not generate task beyond limit");
    assert!(!template.active, "Template should be deactivated");
}

#[test]
fn test_goal_hierarchy() {
    let mut parent_goal = Goal::new(
        "Annual Objectives".to_string(),
        "Company-wide goals for the year".to_string()
    );
    
    let q1_goal = Goal::new(
        "Q1 Objectives".to_string(),
        "First quarter goals".to_string()
    );
    
    let q2_goal = Goal::new(
        "Q2 Objectives".to_string(),
        "Second quarter goals".to_string()
    );
    
    parent_goal.add_subgoal(q1_goal.id);
    parent_goal.add_subgoal(q2_goal.id);
    
    assert_eq!(parent_goal.subgoal_ids.len(), 2);
    assert!(parent_goal.subgoal_ids.contains(&q1_goal.id));
    assert!(parent_goal.subgoal_ids.contains(&q2_goal.id));
}