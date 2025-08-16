use plon::domain::task::{Task, TaskStatus, Priority, SubTask};
use plon::domain::goal::{Goal, GoalStatus};
use plon::domain::resource::Resource;
use plon::domain::dependency::{Dependency, DependencyGraph, DependencyType};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::services::{TaskService, GoalService, ResourceService};
use plon::ui::views::kanban_view::{KanbanView, FilterOptions, DragContext};
use plon::ui::views::timeline_view::TimelineView;
use plon::ui::views::map_view::MapView;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use chrono::{Utc, Duration, NaiveDate};
use uuid::Uuid;

// Helper function to create test repository
async fn create_test_repository() -> Arc<Repository> {
    let pool = init_test_database().await.unwrap();
    Arc::new(Repository::new(pool))
}

// Helper function to create sample tasks
fn create_sample_tasks(count: usize) -> Vec<Task> {
    (0..count).map(|i| {
        let mut task = Task::new(
            format!("Task {}", i),
            format!("Description for task {}", i)
        );
        task.priority = match i % 4 {
            0 => Priority::Critical,
            1 => Priority::High,
            2 => Priority::Medium,
            _ => Priority::Low,
        };
        task.status = match i % 5 {
            0 => TaskStatus::Todo,
            1 => TaskStatus::InProgress,
            2 => TaskStatus::Blocked,
            3 => TaskStatus::Review,
            _ => TaskStatus::Done,
        };
        if i % 3 == 0 {
            task.due_date = Some(Utc::now() + Duration::days((i as i64) - 5));
        }
        task
    }).collect()
}

mod kanban_tests {
    use super::*;
    use eframe::egui::{Pos2, Vec2};

    #[test]
    fn test_kanban_initialization() {
        let kanban = KanbanView::new();
        assert!(!kanban.columns.is_empty());
        assert_eq!(kanban.columns.len(), 5); // Todo, InProgress, Blocked, Review, Done
        assert!(kanban.selected_tasks.is_empty());
        assert!(kanban.drag_context.is_none());
    }

    #[test]
    fn test_drag_and_drop_single_task() {
        let mut kanban = KanbanView::new();
        let task = Task::new("Test Task".to_string(), "Description".to_string());
        kanban.tasks.push(task.clone());

        // Start drag
        kanban.start_drag(task.id, Pos2::new(100.0, 100.0));
        assert!(kanban.is_dragging());
        assert_eq!(kanban.drag_context.as_ref().unwrap().task_id, task.id);

        // Update drag position
        kanban.update_drag_position(Pos2::new(300.0, 100.0));
        assert_eq!(kanban.drag_context.as_ref().unwrap().current_position, Pos2::new(300.0, 100.0));

        // Drop in new column
        kanban.drop_task_at_column(1); // Drop in InProgress column
        assert!(kanban.drag_context.is_none());
        assert_eq!(kanban.tasks[0].status, TaskStatus::InProgress);
    }

    #[test]
    fn test_multi_select_drag_and_drop() {
        let mut kanban = KanbanView::new();
        let tasks = create_sample_tasks(3);
        for task in &tasks {
            kanban.tasks.push(task.clone());
            kanban.selected_tasks.insert(task.id);
        }

        // Start multi-drag
        kanban.start_drag(tasks[0].id, Pos2::new(100.0, 100.0));
        assert_eq!(kanban.drag_context.as_ref().unwrap().selected_tasks.len(), 3);

        // Drop all selected tasks
        kanban.drop_tasks_at_column(2); // Drop in Blocked column
        for task in &kanban.tasks {
            if kanban.selected_tasks.contains(&task.id) {
                assert_eq!(task.status, TaskStatus::Blocked);
            }
        }
    }

    #[test]
    fn test_wip_limit_enforcement() {
        let mut kanban = KanbanView::new();
        kanban.columns[1].wip_limit = Some(2); // Set WIP limit for InProgress
        
        let tasks = create_sample_tasks(5);
        for mut task in tasks {
            task.status = TaskStatus::InProgress;
            kanban.tasks.push(task);
        }

        // Check WIP limit exceeded
        let in_progress_tasks: Vec<_> = kanban.tasks.iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .collect();
        
        assert!(kanban.is_wip_limit_exceeded("In Progress", &kanban.tasks));
        assert!(kanban.get_wip_violation_message("In Progress", &kanban.tasks).is_some());
    }

    #[test]
    fn test_filter_and_search() {
        let mut kanban = KanbanView::new();
        let tasks = create_sample_tasks(20);
        kanban.tasks = tasks;

        // Test search filter
        let mut filter = FilterOptions::default();
        filter.search_text = Some("Task 1".to_string());
        let filtered = kanban.apply_filters(&kanban.tasks, &filter);
        assert!(filtered.iter().all(|t| t.title.contains("Task 1")));

        // Test priority filter
        filter.search_text = None;
        filter.priorities = vec![Priority::Critical].into_iter().collect();
        let filtered = kanban.apply_filters(&kanban.tasks, &filter);
        assert!(filtered.iter().all(|t| t.priority == Priority::Critical));

        // Test status filter
        filter.priorities.clear();
        filter.show_completed = false;
        let filtered = kanban.apply_filters(&kanban.tasks, &filter);
        assert!(filtered.iter().all(|t| t.status != TaskStatus::Done));
    }

    #[test]
    fn test_column_operations() {
        let mut kanban = KanbanView::new();
        
        // Test column collapse
        let column_title = "To Do";
        kanban.toggle_column_collapse(column_title);
        assert!(kanban.columns[0].collapsed);
        
        // Test column visibility
        kanban.set_column_visible("In Progress", false);
        assert!(!kanban.is_column_visible_ext("In Progress"));
        
        // Test add custom column
        kanban.add_custom_column_ext("Custom", TaskStatus::Todo, (255, 0, 0, 255));
        assert_eq!(kanban.columns.len(), 6);
        assert_eq!(kanban.columns.last().unwrap().title, "Custom");
    }

    #[test]
    fn test_card_animations() {
        let mut kanban = KanbanView::new();
        let task = Task::new("Animated Task".to_string(), "".to_string());
        kanban.tasks.push(task.clone());

        // Start hover animation
        kanban.set_hovered_card(Some(task.id));
        assert_eq!(kanban.hovered_card, Some(task.id));

        // Start card animation
        use plon::ui::views::kanban_view::AnimationType;
        kanban.start_card_animation(task.id, AnimationType::HoverIn);
        assert!(kanban.animations.card_animations.contains_key(&task.id));

        // Update animations
        kanban.update_animations(0.1);
        assert_eq!(kanban.animations.time, 0.1);
    }

    #[test]
    fn test_quick_add_functionality() {
        let mut kanban = KanbanView::new();
        
        // Show quick add for a column
        kanban.show_quick_add("To Do");
        assert!(kanban.quick_add_states.contains_key("To Do"));
        assert!(kanban.quick_add_states.get("To Do").unwrap().visible);
    }

    #[test]
    fn test_swimlanes() {
        let mut kanban = KanbanView::new();
        let mut tasks = create_sample_tasks(10);
        
        // Add tags to tasks
        for (i, task) in tasks.iter_mut().enumerate() {
            if i % 2 == 0 {
                task.tags.insert("frontend".to_string());
            } else {
                task.tags.insert("backend".to_string());
            }
        }
        kanban.tasks = tasks;

        // Organize by tags
        use plon::ui::views::kanban_view::SwimlaneType;
        kanban.swimlane_config.swimlane_type = SwimlaneType::Tag;
        let swimlanes = kanban.organize_into_swimlanes(&kanban.tasks);
        
        assert!(swimlanes.contains_key("frontend"));
        assert!(swimlanes.contains_key("backend"));
        assert_eq!(swimlanes.get("frontend").unwrap().len(), 5);
        assert_eq!(swimlanes.get("backend").unwrap().len(), 5);
    }

    #[test]
    fn test_keyboard_shortcuts() {
        let mut kanban = KanbanView::new();
        let task = Task::new("Test Task".to_string(), "".to_string());
        kanban.tasks.push(task.clone());
        kanban.selected_tasks.insert(task.id);

        // Test escape key
        kanban.start_drag(task.id, Pos2::new(100.0, 100.0));
        kanban.handle_escape_key();
        assert!(kanban.drag_context.is_none());
        assert!(kanban.selected_tasks.is_empty());
    }

    #[test]
    fn test_bulk_operations() {
        let mut kanban = KanbanView::new();
        let tasks = create_sample_tasks(5);
        for task in &tasks {
            kanban.tasks.push(task.clone());
            kanban.selected_tasks.insert(task.id);
        }

        // Bulk priority change
        kanban.set_bulk_priority(Priority::Critical);
        for task in &kanban.tasks {
            if kanban.selected_tasks.contains(&task.id) {
                assert_eq!(task.priority, Priority::Critical);
            }
        }
    }
}

mod timeline_tests {
    use super::*;

    #[test]
    fn test_timeline_initialization() {
        let timeline = TimelineView::new();
        assert_eq!(timeline.days_to_show, 30);
        assert!(timeline.show_gantt);
        assert!(timeline.show_resources);
    }

    #[test]
    fn test_schedule_calculation() {
        let mut timeline = TimelineView::new();
        let mut tasks = HashMap::new();
        let mut resources = HashMap::new();
        
        // Create tasks with estimates
        for i in 0..5 {
            let mut task = Task::new(format!("Task {}", i), "".to_string());
            task.estimated_hours = Some(8.0 * (i + 1) as f32);
            tasks.insert(task.id, task);
        }
        
        // Create resources
        let resource = Resource::new("Developer".to_string(), "dev".to_string());
        resources.insert(resource.id, resource);
        
        let graph = DependencyGraph::new();
        let schedule = timeline.calculate_schedule(&tasks, &resources, &graph);
        
        assert!(schedule.is_ok());
        let schedule = schedule.unwrap();
        assert_eq!(schedule.task_schedules.len(), 5);
    }

    #[test]
    fn test_critical_path_detection() {
        let mut timeline = TimelineView::new();
        let mut tasks = HashMap::new();
        let mut graph = DependencyGraph::new();
        
        // Create chain of dependent tasks
        let task1 = Task::new("Task 1".to_string(), "".to_string());
        let task2 = Task::new("Task 2".to_string(), "".to_string());
        let task3 = Task::new("Task 3".to_string(), "".to_string());
        
        tasks.insert(task1.id, task1.clone());
        tasks.insert(task2.id, task2.clone());
        tasks.insert(task3.id, task3.clone());
        
        // Add dependencies
        graph.add_dependency(&Dependency::new(task1.id, task2.id, DependencyType::FinishToStart)).unwrap();
        graph.add_dependency(&Dependency::new(task2.id, task3.id, DependencyType::FinishToStart)).unwrap();
        
        let estimates: HashMap<Uuid, f32> = tasks.iter()
            .map(|(id, _)| (*id, 8.0))
            .collect();
        
        let critical_path = graph.get_critical_path(&estimates);
        assert_eq!(critical_path.len(), 3);
    }

    #[test]
    fn test_filter_application() {
        let timeline = TimelineView::new();
        let mut tasks = HashMap::new();
        
        // Create tasks with different statuses
        for i in 0..10 {
            let mut task = Task::new(format!("Task {}", i), "".to_string());
            task.status = if i % 2 == 0 { TaskStatus::InProgress } else { TaskStatus::Done };
            if i % 3 == 0 {
                task.due_date = Some(Utc::now() - Duration::days(1)); // Overdue
            }
            tasks.insert(task.id, task);
        }
        
        // Test filter
        use plon::ui::views::timeline_view::TimelineFilter;
        let mut timeline = TimelineView::new();
        timeline.set_filter(TimelineFilter::InProgress);
        let filtered = timeline.apply_filters(&tasks);
        
        assert!(filtered.values().all(|t| t.status == TaskStatus::InProgress));
    }

    #[test]
    fn test_resource_allocation() {
        let mut timeline = TimelineView::new();
        let mut tasks = HashMap::new();
        let mut resources = HashMap::new();
        
        // Create resource
        let mut resource = Resource::new("Developer".to_string(), "dev".to_string());
        resource.weekly_hours = 40.0;
        let resource_id = resource.id;
        resources.insert(resource.id, resource);
        
        // Create tasks assigned to resource
        for i in 0..3 {
            let mut task = Task::new(format!("Task {}", i), "".to_string());
            task.assigned_resource_id = Some(resource_id);
            task.estimated_hours = Some(16.0); // 2 days each
            tasks.insert(task.id, task);
        }
        
        let graph = DependencyGraph::new();
        let schedule = timeline.calculate_schedule(&tasks, &resources, &graph).unwrap();
        
        // Check resource allocations
        assert!(!schedule.resource_allocations.is_empty());
        let total_allocated: f32 = schedule.resource_allocations.iter()
            .filter(|a| a.resource_id == resource_id)
            .map(|a| a.hours)
            .sum();
        assert_eq!(total_allocated, 48.0); // 3 tasks * 16 hours
    }

    #[test]
    fn test_milestone_tracking() {
        let mut gantt = plon::ui::widgets::gantt_chart::GanttChart::new();
        
        use plon::ui::widgets::gantt_chart::{Milestone, GanttColor};
        let milestone = Milestone {
            id: Uuid::new_v4(),
            name: "Release v1.0".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            color: GanttColor::Red,
        };
        
        gantt.add_milestone(milestone.clone());
        assert_eq!(gantt.get_milestones().len(), 1);
        assert_eq!(gantt.get_milestones()[0].name, "Release v1.0");
    }

    #[test]
    fn test_weekend_handling() {
        let gantt = plon::ui::widgets::gantt_chart::GanttChart::new();
        let weekends = gantt.get_weekend_positions(700.0);
        
        // Should have weekends marked
        assert!(!weekends.is_empty());
        for weekend in weekends {
            assert!(weekend.day_of_week >= 5); // Saturday or Sunday
        }
    }

    #[test]
    fn test_export_functionality() {
        let gantt = plon::ui::widgets::gantt_chart::GanttChart::new();
        let tasks = HashMap::new();
        let resources = HashMap::new();
        let schedule = plon::services::timeline_scheduler::TimelineSchedule {
            task_schedules: HashMap::new(),
            resource_allocations: Vec::new(),
            critical_path: Vec::new(),
            warnings: Vec::new(),
        };
        
        let export_result = gantt.export_to_json(&tasks, &resources, &schedule);
        assert!(export_result.is_ok());
        
        let json = export_result.unwrap();
        assert!(json.contains("tasks"));
        assert!(json.contains("resources"));
        assert!(json.contains("schedule"));
    }
}

mod task_management_tests {
    use super::*;

    #[test]
    fn test_task_creation_validation() {
        let task = Task::new("".to_string(), "Description".to_string());
        assert_eq!(task.title, ""); // Should handle empty title
        
        let task = Task::new("A".repeat(1000), "Description".to_string());
        assert_eq!(task.title.len(), 1000); // Should handle long titles
    }

    #[test]
    fn test_subtask_operations() {
        let mut task = Task::new("Main Task".to_string(), "".to_string());
        
        // Add subtasks
        let subtask_ids: Vec<_> = (0..5).map(|i| {
            task.add_subtask(format!("Subtask {}", i))
        }).collect();
        
        assert_eq!(task.subtasks.len(), 5);
        
        // Complete subtasks
        for id in &subtask_ids[..3] {
            assert!(task.complete_subtask(*id).is_ok());
        }
        
        let (completed, total) = task.subtask_progress();
        assert_eq!(completed, 3);
        assert_eq!(total, 5);
        
        // Uncomplete subtask
        assert!(task.uncomplete_subtask(subtask_ids[0]).is_ok());
        let (completed, _) = task.subtask_progress();
        assert_eq!(completed, 2);
        
        // Try to complete non-existent subtask
        assert!(task.complete_subtask(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_task_status_transitions() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        // Valid transitions
        task.status = TaskStatus::Todo;
        task.status = TaskStatus::InProgress;
        assert_eq!(task.status, TaskStatus::InProgress);
        
        task.status = TaskStatus::Review;
        assert_eq!(task.status, TaskStatus::Review);
        
        task.status = TaskStatus::Done;
        assert_eq!(task.status, TaskStatus::Done);
        
        // Task should have completed_at when marked as done
        task.completed_at = Some(Utc::now());
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_overdue_detection() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        // Not overdue without due date
        assert!(!task.is_overdue());
        
        // Overdue with past due date
        task.due_date = Some(Utc::now() - Duration::days(1));
        assert!(task.is_overdue());
        
        // Not overdue if completed
        task.status = TaskStatus::Done;
        assert!(!task.is_overdue());
        
        // Not overdue with future due date
        task.status = TaskStatus::Todo;
        task.due_date = Some(Utc::now() + Duration::days(1));
        assert!(!task.is_overdue());
    }

    #[test]
    fn test_task_archiving() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        assert!(!task.is_archived);
        
        task.is_archived = true;
        assert!(task.is_archived);
        
        // Archived tasks should maintain their status
        task.status = TaskStatus::Done;
        assert_eq!(task.status, TaskStatus::Done);
        assert!(task.is_archived);
    }

    #[test]
    fn test_task_metadata() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        // Add metadata
        task.metadata.insert("category".to_string(), "bug".to_string());
        task.metadata.insert("severity".to_string(), "high".to_string());
        
        assert_eq!(task.metadata.get("category"), Some(&"bug".to_string()));
        assert_eq!(task.metadata.get("severity"), Some(&"high".to_string()));
        
        // Add tags
        task.tags.insert("urgent".to_string());
        task.tags.insert("customer".to_string());
        
        assert!(task.tags.contains("urgent"));
        assert!(task.tags.contains("customer"));
    }

    #[test]
    fn test_task_dependencies() {
        let task1 = Task::new("Task 1".to_string(), "".to_string());
        let task2 = Task::new("Task 2".to_string(), "".to_string());
        let task3 = Task::new("Task 3".to_string(), "".to_string());
        
        let mut graph = DependencyGraph::new();
        
        // Add valid dependencies
        assert!(graph.add_dependency(&Dependency::new(
            task1.id, 
            task2.id, 
            DependencyType::FinishToStart
        )).is_ok());
        
        assert!(graph.add_dependency(&Dependency::new(
            task2.id, 
            task3.id, 
            DependencyType::FinishToStart
        )).is_ok());
        
        // Try to add circular dependency
        assert!(graph.add_dependency(&Dependency::new(
            task3.id, 
            task1.id, 
            DependencyType::FinishToStart
        )).is_err());
        
        // Check dependencies
        let deps = graph.get_dependencies(task1.id);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, task2.id);
        
        let dependents = graph.get_dependents(task3.id);
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], task2.id);
    }
}

mod resource_management_tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let mut resource = Resource::new("John Doe".to_string(), "Developer".to_string());
        
        assert_eq!(resource.name, "John Doe");
        assert_eq!(resource.role, "Developer");
        assert_eq!(resource.weekly_hours, 40.0);
        assert_eq!(resource.current_load, 0.0);
    }

    #[test]
    fn test_resource_skills() {
        let mut resource = Resource::new("Jane Doe".to_string(), "Designer".to_string());
        
        resource.add_skill("UI Design".to_string());
        resource.add_skill("Prototyping".to_string());
        resource.add_skill("User Research".to_string());
        
        assert_eq!(resource.skills.len(), 3);
        assert!(resource.has_skill("UI Design"));
        assert!(!resource.has_skill("Backend Development"));
    }

    #[test]
    fn test_resource_availability() {
        let mut resource = Resource::new("Test Resource".to_string(), "Tester".to_string());
        resource.weekly_hours = 40.0;
        
        // Test weekday availability
        let weekday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(); // Monday
        assert_eq!(resource.get_availability_for_date(weekday), 8.0);
        
        // Test weekend availability
        let weekend = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(); // Saturday
        assert_eq!(resource.get_availability_for_date(weekend), 0.0);
        
        // Test with custom availability
        resource.set_custom_availability(weekday, 4.0);
        assert_eq!(resource.get_availability_for_date(weekday), 4.0);
    }

    #[test]
    fn test_resource_utilization() {
        let mut resource = Resource::new("Test Resource".to_string(), "Developer".to_string());
        resource.weekly_hours = 40.0;
        resource.current_load = 32.0;
        
        assert_eq!(resource.utilization_percentage(), 80.0);
        assert_eq!(resource.available_hours(), 8.0);
        assert!(!resource.is_overallocated());
        
        // Test overallocation
        resource.current_load = 50.0;
        assert_eq!(resource.utilization_percentage(), 125.0);
        assert!(resource.is_overallocated());
    }

    #[test]
    fn test_resource_assignment() {
        let mut resource = Resource::new("Developer".to_string(), "dev".to_string());
        let mut task = Task::new("Feature Task".to_string(), "".to_string());
        
        // Assign resource to task
        task.assigned_resource_id = Some(resource.id);
        task.estimated_hours = Some(16.0);
        
        // Update resource load
        resource.current_load += task.estimated_hours.unwrap_or(0.0);
        
        assert_eq!(resource.current_load, 16.0);
        assert_eq!(task.assigned_resource_id, Some(resource.id));
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_states() {
        let kanban = KanbanView::new();
        assert!(kanban.tasks.is_empty());
        
        // Operations on empty state should not panic
        let filtered = kanban.apply_filters(&[], &FilterOptions::default());
        assert!(filtered.is_empty());
        
        let swimlanes = kanban.organize_into_swimlanes(&[]);
        assert!(swimlanes.is_empty() || swimlanes.values().all(|v| v.is_empty()));
    }

    #[test]
    fn test_maximum_limits() {
        let mut kanban = KanbanView::new();
        
        // Add maximum number of tasks
        let tasks = create_sample_tasks(1000);
        kanban.tasks = tasks;
        
        // Should handle large datasets
        let filter = FilterOptions::default();
        let filtered = kanban.apply_filters(&kanban.tasks, &filter);
        assert_eq!(filtered.len(), 1000);
        
        // Test with all tasks selected
        for task in &kanban.tasks {
            kanban.selected_tasks.insert(task.id);
        }
        assert_eq!(kanban.selected_tasks.len(), 1000);
    }

    #[test]
    fn test_unicode_handling() {
        let mut task = Task::new(
            "测试任务 🚀 тест τεστ".to_string(),
            "Description with émojis 😀😁😂 and special chars: <>&\"'".to_string()
        );
        
        task.tags.insert("中文标签".to_string());
        task.tags.insert("🏷️ emoji-tag".to_string());
        
        assert!(task.title.contains("🚀"));
        assert!(task.tags.contains("中文标签"));
        
        // Test in kanban view
        let mut kanban = KanbanView::new();
        kanban.tasks.push(task.clone());
        
        let mut filter = FilterOptions::default();
        filter.search_text = Some("测试".to_string());
        let filtered = kanban.apply_filters(&kanban.tasks, &filter);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_invalid_dates() {
        let mut task = Task::new("Test Task".to_string(), "".to_string());
        
        // Far future date
        task.due_date = Some(Utc::now() + Duration::days(365 * 100));
        assert!(!task.is_overdue());
        
        // Very old date
        task.due_date = Some(Utc::now() - Duration::days(365 * 50));
        task.status = TaskStatus::Todo;
        assert!(task.is_overdue());
    }

    #[test]
    fn test_concurrent_modifications() {
        let mut kanban = KanbanView::new();
        let task = Task::new("Concurrent Task".to_string(), "".to_string());
        kanban.tasks.push(task.clone());
        
        // Simulate concurrent modifications
        kanban.selected_tasks.insert(task.id);
        kanban.start_drag(task.id, Pos2::new(100.0, 100.0));
        
        // Try to modify while dragging
        if let Some(t) = kanban.tasks.iter_mut().find(|t| t.id == task.id) {
            t.title = "Modified Title".to_string();
        }
        
        // Should handle gracefully
        kanban.cancel_drag();
        assert!(kanban.drag_context.is_none());
    }

    #[test]
    fn test_recursive_dependencies() {
        let mut graph = DependencyGraph::new();
        let tasks: Vec<_> = (0..10).map(|i| {
            Task::new(format!("Task {}", i), "".to_string())
        }).collect();
        
        // Create a long chain of dependencies
        for i in 0..9 {
            assert!(graph.add_dependency(&Dependency::new(
                tasks[i].id,
                tasks[i + 1].id,
                DependencyType::FinishToStart
            )).is_ok());
        }
        
        // Should detect cycle if we try to close the loop
        assert!(graph.add_dependency(&Dependency::new(
            tasks[9].id,
            tasks[0].id,
            DependencyType::FinishToStart
        )).is_err());
        
        // Should handle topological sort
        let sorted = graph.topological_sort();
        assert!(sorted.is_ok());
    }

    #[test]
    fn test_memory_intensive_operations() {
        let mut kanban = KanbanView::new();
        
        // Create tasks with large descriptions
        for i in 0..100 {
            let mut task = Task::new(
                format!("Task {}", i),
                "A".repeat(10000) // 10KB description
            );
            
            // Add many tags
            for j in 0..50 {
                task.tags.insert(format!("tag-{}", j));
            }
            
            // Add many metadata entries
            for j in 0..50 {
                task.metadata.insert(format!("key-{}", j), format!("value-{}", j));
            }
            
            kanban.tasks.push(task);
        }
        
        // Should handle without issues
        assert_eq!(kanban.tasks.len(), 100);
        
        // Filter operations should still work
        let filter = FilterOptions::default();
        let filtered = kanban.apply_filters(&kanban.tasks, &filter);
        assert!(!filtered.is_empty());
    }
}

mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_large_dataset_filtering() {
        let mut kanban = KanbanView::new();
        let tasks = create_sample_tasks(5000);
        kanban.tasks = tasks;

        let start = Instant::now();
        let mut filter = FilterOptions::default();
        filter.search_text = Some("Task 1".to_string());
        let filtered = kanban.apply_filters(&kanban.tasks, &filter);
        let duration = start.elapsed();

        // Should complete within reasonable time
        assert!(duration.as_millis() < 100);
        assert!(!filtered.is_empty());
    }

    #[test]
    fn test_drag_performance() {
        let mut kanban = KanbanView::new();
        let tasks = create_sample_tasks(1000);
        for task in tasks {
            kanban.tasks.push(task);
        }

        let start = Instant::now();
        
        // Simulate rapid drag updates
        kanban.start_drag(kanban.tasks[0].id, Pos2::new(0.0, 0.0));
        for i in 0..100 {
            kanban.update_drag_position(Pos2::new(i as f32, i as f32));
        }
        kanban.drop_task_at_column(1);
        
        let duration = start.elapsed();
        
        // Should handle rapid updates efficiently
        assert!(duration.as_millis() < 50);
    }

    #[test]
    fn test_animation_performance() {
        let mut kanban = KanbanView::new();
        let tasks = create_sample_tasks(100);
        
        let start = Instant::now();
        
        // Start animations for all tasks
        for task in &tasks {
            use plon::ui::views::kanban_view::AnimationType;
            kanban.start_card_animation(task.id, AnimationType::HoverIn);
        }
        
        // Update animations multiple times
        for _ in 0..60 { // Simulate 60 FPS for 1 second
            kanban.update_animations(1.0 / 60.0);
        }
        
        let duration = start.elapsed();
        
        // Should handle 60 FPS updates
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_schedule_calculation_performance() {
        let mut timeline = TimelineView::new();
        let mut tasks = HashMap::new();
        let mut resources = HashMap::new();
        
        // Create many tasks
        for i in 0..100 {
            let mut task = Task::new(format!("Task {}", i), "".to_string());
            task.estimated_hours = Some(8.0 * ((i % 5) + 1) as f32);
            tasks.insert(task.id, task);
        }
        
        // Create resources
        for i in 0..10 {
            let resource = Resource::new(format!("Resource {}", i), "role".to_string());
            resources.insert(resource.id, resource);
        }
        
        let graph = DependencyGraph::new();
        
        let start = Instant::now();
        let schedule = timeline.calculate_schedule(&tasks, &resources, &graph);
        let duration = start.elapsed();
        
        assert!(schedule.is_ok());
        assert!(duration.as_millis() < 500);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        let repository = create_test_repository().await;
        let task_service = Arc::new(TaskService::new(repository.clone()));
        let goal_service = Arc::new(GoalService::new(repository.clone()));
        let resource_service = Arc::new(ResourceService::new(repository.clone()));
        
        // Create a goal
        let goal = Goal::new("Q1 Release".to_string(), "Release version 1.0".to_string());
        let goal_result = goal_service.create(goal.clone()).await;
        assert!(goal_result.is_ok());
        
        // Create resources
        let dev = Resource::new("Alice".to_string(), "Developer".to_string());
        let designer = Resource::new("Bob".to_string(), "Designer".to_string());
        
        let dev_result = resource_service.create(dev.clone()).await;
        let designer_result = resource_service.create(designer.clone()).await;
        assert!(dev_result.is_ok());
        assert!(designer_result.is_ok());
        
        // Create tasks
        let mut tasks = Vec::new();
        for i in 0..10 {
            let mut task = Task::new(
                format!("Feature {}", i),
                format!("Implement feature {}", i)
            );
            task.goal_id = Some(goal.id);
            task.assigned_resource_id = if i % 2 == 0 { Some(dev.id) } else { Some(designer.id) };
            task.estimated_hours = Some(8.0 * ((i % 3) + 1) as f32);
            
            let task_result = task_service.create(task.clone()).await;
            assert!(task_result.is_ok());
            tasks.push(task);
        }
        
        // Test UI components with real data
        let mut kanban = KanbanView::new();
        kanban.tasks = tasks.clone();
        
        // Test filtering
        let mut filter = FilterOptions::default();
        filter.assigned_to = Some(dev.id);
        let filtered = kanban.apply_filters(&kanban.tasks, &filter);
        assert_eq!(filtered.len(), 5);
        
        // Test timeline
        let mut timeline = TimelineView::new();
        let task_map: HashMap<_, _> = tasks.iter().map(|t| (t.id, t.clone())).collect();
        let resource_map = vec![(dev.id, dev), (designer.id, designer)].into_iter().collect();
        
        let graph = DependencyGraph::new();
        let schedule = timeline.calculate_schedule(&task_map, &resource_map, &graph);
        assert!(schedule.is_ok());
    }
}