use plon::domain::task::{Task, TaskStatus, Priority, SubTask};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::services::TaskService;
use plon::ui::views::kanban_view::{KanbanView, FilterOptions, QuickAddMetadata};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use chrono::{Utc, Duration};
use uuid::Uuid;

mod visual_tests {
    use super::*;

    #[tokio::test]
    async fn test_card_visual_hierarchy() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository);
        
        let mut critical_task = Task::new("Critical Bug".to_string(), "System down".to_string());
        critical_task.priority = Priority::Critical;
        critical_task.add_tag("bug".to_string());
        critical_task.add_tag("production".to_string());
        
        let mut normal_task = Task::new("Feature Request".to_string(), "".to_string());
        normal_task.priority = Priority::Medium;
        
        service.create(critical_task.clone()).await.unwrap();
        service.create(normal_task.clone()).await.unwrap();
        
        let view = KanbanView::new();
        
        assert!(view.get_card_style(&critical_task).border_color == (255, 0, 0, 255));
        assert!(view.get_card_style(&critical_task).border_width == 3.0);
        assert!(view.get_card_style(&normal_task).border_width == 1.0);
    }

    #[tokio::test]
    async fn test_card_color_coding_by_priority() {
        let view = KanbanView::new();
        
        let mut tasks = vec![
            (Priority::Critical, (255, 59, 48, 255)),
            (Priority::High, (255, 149, 0, 255)),
            (Priority::Medium, (52, 199, 89, 255)),
            (Priority::Low, (175, 175, 175, 255)),
        ];
        
        for (priority, expected_color) in tasks {
            let mut task = Task::new("Test".to_string(), "".to_string());
            task.priority = priority;
            let style = view.get_card_style(&task);
            assert_eq!(style.priority_indicator_color, expected_color);
        }
    }

    #[tokio::test]
    async fn test_card_shadow_and_hover_effects() {
        let mut view = KanbanView::new();
        let task = Task::new("Test Task".to_string(), "".to_string());
        
        let normal_style = view.get_card_style(&task);
        assert_eq!(normal_style.shadow_blur, 4.0);
        assert_eq!(normal_style.shadow_offset, (0.0, 2.0));
        
        view.set_hovered_card(Some(task.id));
        let hovered_style = view.get_card_style(&task);
        assert_eq!(hovered_style.shadow_blur, 12.0);
        assert_eq!(hovered_style.shadow_offset, (0.0, 4.0));
        assert_eq!(hovered_style.elevation, 2.0);
    }

    #[tokio::test]
    async fn test_progress_bar_visualization() {
        let view = KanbanView::new();
        let mut task = Task::new("Task with subtasks".to_string(), "".to_string());
        
        for i in 1..=5 {
            task.add_subtask(format!("Subtask {}", i));
        }
        
        task.complete_subtask(task.subtasks[0].id).unwrap();
        task.complete_subtask(task.subtasks[1].id).unwrap();
        
        let progress = view.calculate_progress_bar(&task);
        assert_eq!(progress.percentage, 40.0);
        assert_eq!(progress.completed_count, 2);
        assert_eq!(progress.total_count, 5);
        assert_eq!(progress.color, (52, 199, 89, 255));
    }

    #[tokio::test]
    async fn test_overdue_task_visual_indicators() {
        let view = KanbanView::new();
        let mut overdue_task = Task::new("Overdue Task".to_string(), "".to_string());
        overdue_task.due_date = Some(Utc::now() - Duration::days(3));
        
        let style = view.get_card_style(&overdue_task);
        assert!(style.show_overdue_badge);
        assert_eq!(style.overdue_badge_color, (255, 59, 48, 255));
        assert!(style.pulse_animation);
    }

    #[tokio::test]
    async fn test_blocked_task_visual_pattern() {
        let view = KanbanView::new();
        let mut blocked_task = Task::new("Blocked Task".to_string(), "".to_string());
        blocked_task.status = TaskStatus::Blocked;
        
        let style = view.get_card_style(&blocked_task);
        assert!(style.show_blocked_overlay);
        assert_eq!(style.blocked_pattern, "diagonal_stripes");
        assert_eq!(style.opacity, 0.8);
    }

    #[tokio::test]
    async fn test_tag_color_assignments() {
        let mut view = KanbanView::new();
        
        let tags = vec!["frontend", "backend", "bug", "feature", "documentation"];
        let colors = view.assign_tag_colors(&tags);
        
        assert_eq!(colors.len(), tags.len());
        
        for tag in &tags {
            assert!(colors.contains_key(&tag.to_string()));
            let color = colors.get(&tag.to_string()).unwrap();
            assert!(color.0 <= 255 && color.1 <= 255 && color.2 <= 255);
        }
        
        let duplicate_colors = view.assign_tag_colors(&tags);
        assert_eq!(colors, duplicate_colors);
    }
}

mod drag_drop_tests {
    use super::*;

    #[tokio::test]
    async fn test_drag_initiation() {
        let mut view = KanbanView::new();
        let task = Task::new("Draggable".to_string(), "".to_string());
        
        view.start_drag(task.id, (100.0, 200.0));
        
        assert!(view.is_dragging());
        assert_eq!(view.get_drag_context().unwrap().task_id, task.id);
        assert_eq!(view.get_drag_context().unwrap().start_position, (100.0, 200.0));
    }

    #[tokio::test]
    async fn test_drag_preview_rendering() {
        let mut view = KanbanView::new();
        let task = Task::new("Task".to_string(), "Description".to_string());
        
        view.start_drag(task.id, (0.0, 0.0));
        view.update_drag_position((150.0, 250.0));
        
        let preview = view.get_drag_preview().unwrap();
        assert_eq!(preview.position, (150.0, 250.0));
        assert_eq!(preview.opacity, 0.7);
        assert!(preview.show_drop_indicator);
    }

    #[tokio::test]
    async fn test_drop_zone_detection() {
        let mut view = KanbanView::new();
        
        let drop_zones = vec![
            ("Todo", (50.0, 100.0), (250.0, 700.0)),
            ("In Progress", (300.0, 100.0), (250.0, 700.0)),
            ("Review", (550.0, 100.0), (250.0, 700.0)),
            ("Done", (800.0, 100.0), (250.0, 700.0)),
        ];
        
        view.set_drop_zones(drop_zones);
        
        assert_eq!(view.get_drop_zone_at((100.0, 300.0)), Some("Todo"));
        assert_eq!(view.get_drop_zone_at((400.0, 300.0)), Some("In Progress"));
        assert_eq!(view.get_drop_zone_at((1200.0, 300.0)), None);
    }

    #[tokio::test]
    async fn test_drag_auto_scroll() {
        let mut view = KanbanView::new();
        view.set_viewport_bounds((0.0, 0.0), (1000.0, 800.0));
        
        view.start_drag(Uuid::new_v4(), (500.0, 400.0));
        
        view.update_drag_position((50.0, 400.0));
        assert_eq!(view.get_auto_scroll_velocity(), (-5.0, 0.0));
        
        view.update_drag_position((950.0, 400.0));
        assert_eq!(view.get_auto_scroll_velocity(), (5.0, 0.0));
        
        view.update_drag_position((500.0, 750.0));
        assert_eq!(view.get_auto_scroll_velocity(), (0.0, 5.0));
    }

    #[tokio::test]
    async fn test_drag_cancel_on_escape() {
        let mut view = KanbanView::new();
        let task_id = Uuid::new_v4();
        
        view.start_drag(task_id, (100.0, 200.0));
        assert!(view.is_dragging());
        
        view.cancel_drag();
        assert!(!view.is_dragging());
        assert!(view.get_drag_context().is_none());
    }

    #[tokio::test]
    async fn test_multi_select_drag() {
        let mut view = KanbanView::new();
        
        let task_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        view.select_multiple_tasks(task_ids.clone());
        
        view.start_multi_drag(task_ids[0], (100.0, 200.0));
        
        let context = view.get_drag_context().unwrap();
        assert_eq!(context.selected_tasks.len(), 3);
        assert!(context.is_multi_drag);
    }
}

mod column_customization_tests {
    use super::*;

    #[tokio::test]
    async fn test_custom_column_creation() {
        let mut view = KanbanView::new();
        
        view.add_custom_column("Backlog", TaskStatus::Todo, (200, 200, 200, 255));
        view.add_custom_column("QA", TaskStatus::Review, (150, 100, 255, 255));
        
        let columns = view.get_columns();
        assert!(columns.iter().any(|c| c.title == "Backlog"));
        assert!(columns.iter().any(|c| c.title == "QA"));
    }

    #[tokio::test]
    async fn test_column_reordering() {
        let mut view = KanbanView::new();
        let initial_order = view.get_column_order();
        
        view.move_column(0, 2);
        let new_order = view.get_column_order();
        
        assert_ne!(initial_order, new_order);
        assert_eq!(new_order[2], initial_order[0]);
    }

    #[tokio::test]
    async fn test_column_visibility_toggle() {
        let mut view = KanbanView::new();
        
        view.set_column_visible("Review", false);
        assert!(!view.is_column_visible("Review"));
        
        view.set_column_visible("Review", true);
        assert!(view.is_column_visible("Review"));
    }

    #[tokio::test]
    async fn test_column_width_adjustment() {
        let mut view = KanbanView::new();
        
        view.set_column_width("Todo", 300.0);
        assert_eq!(view.get_column_width("Todo"), 300.0);
        
        view.set_column_width("Todo", 150.0);
        assert_eq!(view.get_column_width("Todo"), 200.0);
    }

    #[tokio::test]
    async fn test_column_collapse_expand() {
        let mut view = KanbanView::new();
        
        view.collapse_column("In Progress");
        assert!(view.is_column_collapsed("In Progress"));
        assert_eq!(view.get_column_width("In Progress"), 50.0);
        
        view.expand_column("In Progress");
        assert!(!view.is_column_collapsed("In Progress"));
        assert_eq!(view.get_column_width("In Progress"), 250.0);
    }
}

mod wip_limit_tests {
    use super::*;

    #[tokio::test]
    async fn test_wip_limit_enforcement() {
        let mut view = KanbanView::new();
        view.set_wip_limit("In Progress", 3);
        
        let mut tasks = Vec::new();
        for i in 0..5 {
            let mut task = Task::new(format!("Task {}", i), "".to_string());
            task.status = TaskStatus::InProgress;
            tasks.push(task);
        }
        
        assert!(view.is_wip_limit_exceeded("In Progress", &tasks));
        assert_eq!(view.get_wip_violation_message("In Progress", &tasks), 
                   Some("WIP limit exceeded: 5/3".to_string()));
    }

    #[tokio::test]
    async fn test_wip_limit_visual_warning() {
        let mut view = KanbanView::new();
        view.set_wip_limit("Review", 2);
        
        let column_style = view.get_column_style("Review", 3);
        assert!(column_style.show_wip_warning);
        assert_eq!(column_style.header_color, (255, 200, 0, 255));
        assert!(column_style.pulse_header);
    }

    #[tokio::test]
    async fn test_wip_limit_drag_prevention() {
        let mut view = KanbanView::new();
        view.set_wip_limit("In Progress", 2);
        
        let mut tasks = Vec::new();
        for i in 0..2 {
            let mut task = Task::new(format!("Task {}", i), "".to_string());
            task.status = TaskStatus::InProgress;
            tasks.push(task);
        }
        
        let new_task = Task::new("New Task".to_string(), "".to_string());
        assert!(!view.can_drop_in_column("In Progress", &new_task, &tasks));
    }
}

mod filtering_search_tests {
    use super::*;

    #[tokio::test]
    async fn test_text_search_filter() {
        let view = KanbanView::new();
        
        let tasks = vec![
            Task::new("Fix login bug".to_string(), "Authentication issue".to_string()),
            Task::new("Add dashboard".to_string(), "New feature".to_string()),
            Task::new("Update documentation".to_string(), "API docs".to_string()),
        ];
        
        let filtered = view.filter_tasks(&tasks, "bug");
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].title.contains("bug"));
    }

    #[tokio::test]
    async fn test_tag_filter() {
        let view = KanbanView::new();
        
        let mut task1 = Task::new("Task 1".to_string(), "".to_string());
        task1.add_tag("frontend".to_string());
        
        let mut task2 = Task::new("Task 2".to_string(), "".to_string());
        task2.add_tag("backend".to_string());
        
        let mut task3 = Task::new("Task 3".to_string(), "".to_string());
        task3.add_tag("frontend".to_string());
        task3.add_tag("urgent".to_string());
        
        let tasks = vec![task1, task2, task3];
        
        let filter = FilterOptions {
            tags: vec!["frontend".to_string()],
            ..Default::default()
        };
        
        let filtered = view.apply_filters(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[tokio::test]
    async fn test_priority_filter() {
        let view = KanbanView::new();
        
        let mut high_task = Task::new("High".to_string(), "".to_string());
        high_task.priority = Priority::High;
        
        let mut critical_task = Task::new("Critical".to_string(), "".to_string());
        critical_task.priority = Priority::Critical;
        
        let mut low_task = Task::new("Low".to_string(), "".to_string());
        low_task.priority = Priority::Low;
        
        let tasks = vec![high_task, critical_task, low_task];
        
        let filter = FilterOptions {
            priorities: vec![Priority::High, Priority::Critical],
            ..Default::default()
        };
        
        let filtered = view.apply_filters(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[tokio::test]
    async fn test_assignee_filter() {
        let view = KanbanView::new();
        
        let resource_id = Uuid::new_v4();
        
        let mut assigned_task = Task::new("Assigned".to_string(), "".to_string());
        assigned_task.assigned_resource_id = Some(resource_id);
        
        let unassigned_task = Task::new("Unassigned".to_string(), "".to_string());
        
        let tasks = vec![assigned_task, unassigned_task];
        
        let filter = FilterOptions {
            assigned_to: Some(resource_id),
            ..Default::default()
        };
        
        let filtered = view.apply_filters(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
    }

    #[tokio::test]
    async fn test_date_range_filter() {
        let view = KanbanView::new();
        
        let mut task_today = Task::new("Today".to_string(), "".to_string());
        task_today.due_date = Some(Utc::now());
        
        let mut task_tomorrow = Task::new("Tomorrow".to_string(), "".to_string());
        task_tomorrow.due_date = Some(Utc::now() + Duration::days(1));
        
        let mut task_next_week = Task::new("Next Week".to_string(), "".to_string());
        task_next_week.due_date = Some(Utc::now() + Duration::days(7));
        
        let tasks = vec![task_today, task_tomorrow, task_next_week];
        
        let filter = FilterOptions {
            due_date_range: Some((Utc::now(), Utc::now() + Duration::days(3))),
            ..Default::default()
        };
        
        let filtered = view.apply_filters(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[tokio::test]
    async fn test_combined_filters() {
        let view = KanbanView::new();
        
        let mut task1 = Task::new("Frontend Bug".to_string(), "".to_string());
        task1.priority = Priority::High;
        task1.add_tag("frontend".to_string());
        task1.add_tag("bug".to_string());
        
        let mut task2 = Task::new("Backend Feature".to_string(), "".to_string());
        task2.priority = Priority::Medium;
        task2.add_tag("backend".to_string());
        
        let mut task3 = Task::new("Frontend Feature".to_string(), "".to_string());
        task3.priority = Priority::High;
        task3.add_tag("frontend".to_string());
        
        let tasks = vec![task1, task2, task3];
        
        let filter = FilterOptions {
            tags: vec!["frontend".to_string()],
            priorities: vec![Priority::High],
            ..Default::default()
        };
        
        let filtered = view.apply_filters(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
    }
}

mod swimlane_tests {
    use super::*;

    #[tokio::test]
    async fn test_swimlane_by_priority() {
        let mut view = KanbanView::new();
        view.enable_swimlanes_by_priority();
        
        let mut critical = Task::new("Critical".to_string(), "".to_string());
        critical.priority = Priority::Critical;
        
        let mut high = Task::new("High".to_string(), "".to_string());
        high.priority = Priority::High;
        
        let tasks = vec![critical, high];
        let swimlanes = view.organize_into_swimlanes(&tasks);
        
        assert_eq!(swimlanes.len(), 2);
        assert!(swimlanes.contains_key(&"Critical"));
        assert!(swimlanes.contains_key(&"High"));
    }

    #[tokio::test]
    async fn test_swimlane_by_assignee() {
        let mut view = KanbanView::new();
        
        let user1_id = Uuid::new_v4();
        let user2_id = Uuid::new_v4();
        
        let mut task1 = Task::new("Task 1".to_string(), "".to_string());
        task1.assigned_resource_id = Some(user1_id);
        
        let mut task2 = Task::new("Task 2".to_string(), "".to_string());
        task2.assigned_resource_id = Some(user2_id);
        
        let mut task3 = Task::new("Task 3".to_string(), "".to_string());
        task3.assigned_resource_id = Some(user1_id);
        
        let tasks = vec![task1, task2, task3];
        
        view.enable_swimlanes_by_assignee();
        let swimlanes = view.organize_into_swimlanes(&tasks);
        
        assert_eq!(swimlanes.get(&user1_id.to_string()).unwrap().len(), 2);
        assert_eq!(swimlanes.get(&user2_id.to_string()).unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_swimlane_collapse() {
        let mut view = KanbanView::new();
        view.enable_swimlanes_by_priority();
        
        view.collapse_swimlane("High");
        assert!(view.is_swimlane_collapsed("High"));
        
        view.expand_swimlane("High");
        assert!(!view.is_swimlane_collapsed("High"));
    }

    #[tokio::test]
    async fn test_swimlane_reordering() {
        let mut view = KanbanView::new();
        view.enable_swimlanes_by_priority();
        
        let initial_order = vec!["Critical", "High", "Medium", "Low"];
        view.set_swimlane_order(initial_order.clone());
        
        view.move_swimlane(3, 1);
        let new_order = view.get_swimlane_order();
        
        assert_eq!(new_order[1], "Low");
    }
}

mod quick_add_tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_add_in_column() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository);
        let mut view = KanbanView::new();
        
        view.show_quick_add("In Progress");
        assert!(view.is_quick_add_visible("In Progress"));
        
        let task_title = "Quick task";
        let created_task = view.create_quick_task("In Progress", task_title, &service).await;
        
        assert!(created_task.is_ok());
        let task = created_task.unwrap();
        assert_eq!(task.title, task_title);
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_quick_add_with_metadata() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository);
        let mut view = KanbanView::new();
        
        let metadata = QuickAddMetadata {
            title: "Task with metadata".to_string(),
            priority: Some(Priority::High),
            tags: vec!["urgent".to_string(), "bug".to_string()],
            due_date: Some(Utc::now() + Duration::days(2)),
            description: Some("Quick description".to_string()),
        };
        
        let task = view.create_quick_task_with_metadata("Todo", metadata, &service).await.unwrap();
        
        assert_eq!(task.priority, Priority::High);
        assert!(task.tags.contains(&"urgent".to_string()));
        assert!(task.tags.contains(&"bug".to_string()));
        assert!(task.due_date.is_some());
    }

    #[tokio::test]
    async fn test_quick_add_keyboard_shortcuts() {
        let mut view = KanbanView::new();
        
        view.handle_keyboard_shortcut("ctrl+n", Some("Todo"));
        assert!(view.is_quick_add_visible("Todo"));
        
        view.handle_keyboard_shortcut("escape", None);
        assert!(!view.is_quick_add_visible("Todo"));
    }
}

mod card_interaction_tests {
    use super::*;

    #[tokio::test]
    async fn test_card_double_click_edit() {
        let mut view = KanbanView::new();
        let task_id = Uuid::new_v4();
        
        view.handle_card_double_click(task_id);
        assert!(view.is_editing_task(task_id));
        assert!(view.get_edit_dialog().is_some());
    }

    #[tokio::test]
    async fn test_card_context_menu() {
        let mut view = KanbanView::new();
        let task_id = Uuid::new_v4();
        
        view.show_context_menu(task_id, (200.0, 300.0));
        
        let menu = view.get_context_menu().unwrap();
        assert_eq!(menu.task_id, task_id);
        assert_eq!(menu.position, (200.0, 300.0));
        assert!(menu.items.contains(&"Edit"));
        assert!(menu.items.contains(&"Delete"));
        assert!(menu.items.contains(&"Duplicate"));
        assert!(menu.items.contains(&"Move to"));
    }

    #[tokio::test]
    async fn test_card_inline_status_change() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository);
        
        let mut task = Task::new("Task".to_string(), "".to_string());
        task.status = TaskStatus::Todo;
        let created = service.create(task).await.unwrap();
        
        let mut view = KanbanView::new();
        view.quick_change_status(created.id, TaskStatus::InProgress, &service).await.unwrap();
        
        let updated = service.get(created.id).await.unwrap().unwrap();
        assert_eq!(updated.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_card_selection() {
        let mut view = KanbanView::new();
        
        let task_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        
        view.select_card(task_ids[0], false);
        assert_eq!(view.get_selected_cards().len(), 1);
        
        view.select_card(task_ids[1], true);
        assert_eq!(view.get_selected_cards().len(), 2);
        
        view.select_card(task_ids[2], false);
        assert_eq!(view.get_selected_cards().len(), 1);
        assert!(view.get_selected_cards().contains(&task_ids[2]));
    }

    #[tokio::test]
    async fn test_bulk_operations() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        let service = TaskService::new(repository);
        let mut view = KanbanView::new();
        
        let mut task_ids = Vec::new();
        for i in 0..3 {
            let task = Task::new(format!("Task {}", i), "".to_string());
            let created = service.create(task).await.unwrap();
            task_ids.push(created.id);
        }
        
        view.select_multiple_tasks(task_ids.clone());
        view.bulk_change_status(TaskStatus::Done, &service).await.unwrap();
        
        for id in task_ids {
            let task = service.get(id).await.unwrap().unwrap();
            assert_eq!(task.status, TaskStatus::Done);
        }
    }
}

mod animation_tests {
    use super::*;

    #[tokio::test]
    async fn test_card_move_animation() {
        let mut view = KanbanView::new();
        let task_id = Uuid::new_v4();
        
        view.start_card_animation(task_id, (100.0, 200.0), (300.0, 200.0));
        
        assert!(view.is_animating(task_id));
        
        view.update_animations(0.5);
        let position = view.get_animated_position(task_id).unwrap();
        assert_eq!(position, (200.0, 200.0));
        
        view.update_animations(0.5);
        assert!(!view.is_animating(task_id));
    }

    #[tokio::test]
    async fn test_column_expand_animation() {
        let mut view = KanbanView::new();
        
        view.start_column_expand_animation("Todo", 50.0, 250.0);
        
        for i in 0..10 {
            view.update_animations(0.1);
            let width = view.get_animated_column_width("Todo");
            assert!(width > 50.0 && width <= 250.0);
        }
        
        assert_eq!(view.get_column_width("Todo"), 250.0);
    }

    #[tokio::test]
    async fn test_new_card_slide_in() {
        let mut view = KanbanView::new();
        let task_id = Uuid::new_v4();
        
        view.add_card_with_animation(task_id, "Todo");
        
        let initial_opacity = view.get_card_opacity(task_id);
        assert_eq!(initial_opacity, 0.0);
        
        for _ in 0..10 {
            view.update_animations(0.1);
        }
        
        let final_opacity = view.get_card_opacity(task_id);
        assert_eq!(final_opacity, 1.0);
    }
}

mod persistence_tests {
    use super::*;

    #[tokio::test]
    async fn test_save_view_preferences() {
        let mut view = KanbanView::new();
        
        view.set_column_width("Todo", 300.0);
        view.set_wip_limit("In Progress", 5);
        view.enable_swimlanes_by_priority();
        view.set_column_visible("Done", false);
        
        let preferences = view.get_preferences();
        view.save_preferences(&preferences).await.unwrap();
        
        let mut new_view = KanbanView::new();
        new_view.load_preferences().await.unwrap();
        
        assert_eq!(new_view.get_column_width("Todo"), 300.0);
        assert_eq!(new_view.get_wip_limit("In Progress"), Some(5));
        assert!(new_view.are_swimlanes_enabled());
        assert!(!new_view.is_column_visible("Done"));
    }

    #[tokio::test]
    async fn test_restore_filter_state() {
        let mut view = KanbanView::new();
        
        let filter = FilterOptions {
            search_text: Some("bug".to_string()),
            tags: vec!["frontend".to_string()],
            priorities: vec![Priority::High, Priority::Critical],
            ..Default::default()
        };
        
        view.apply_filter(filter.clone());
        let saved = view.get_filter_state();
        
        let mut new_view = KanbanView::new();
        new_view.restore_filter_state(saved);
        
        let restored_filter = new_view.get_current_filter();
        assert_eq!(restored_filter.search_text, Some("bug".to_string()));
        assert_eq!(restored_filter.tags, vec!["frontend".to_string()]);
    }
}

mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_render_performance_with_many_cards() {
        let mut view = KanbanView::new();
        
        let mut tasks = Vec::new();
        for i in 0..500 {
            let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
            task.status = match i % 4 {
                0 => TaskStatus::Todo,
                1 => TaskStatus::InProgress,
                2 => TaskStatus::Review,
                _ => TaskStatus::Done,
            };
            tasks.push(task);
        }
        
        let start = Instant::now();
        view.prepare_render_data(&tasks);
        let elapsed = start.elapsed();
        
        assert!(elapsed.as_millis() < 100);
    }

    #[tokio::test]
    async fn test_virtual_scrolling() {
        let mut view = KanbanView::new();
        view.set_viewport_height(600.0);
        
        let mut tasks = Vec::new();
        for i in 0..100 {
            let mut task = Task::new(format!("Task {}", i), "".to_string());
            task.status = TaskStatus::Todo;
            tasks.push(task);
        }
        
        let visible_range = view.calculate_visible_range("Todo", &tasks);
        assert!(visible_range.1 - visible_range.0 < 20);
        
        view.scroll_column("Todo", 500.0);
        let new_range = view.calculate_visible_range("Todo", &tasks);
        assert!(new_range.0 > visible_range.0);
    }

    #[tokio::test]
    async fn test_incremental_search() {
        let view = KanbanView::new();
        
        let mut tasks = Vec::new();
        for i in 0..1000 {
            tasks.push(Task::new(format!("Task {}", i), format!("Description {}", i)));
        }
        
        let start = Instant::now();
        let results = view.search_tasks(&tasks, "Task 50");
        let elapsed = start.elapsed();
        
        assert!(elapsed.as_millis() < 50);
        assert!(results.iter().any(|t| t.title == "Task 50"));
    }
}

mod accessibility_tests {
    use super::*;

    #[tokio::test]
    async fn test_keyboard_navigation() {
        let mut view = KanbanView::new();
        let task_ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
        
        view.set_focusable_cards(task_ids.clone());
        
        view.handle_keyboard_navigation("ArrowDown");
        assert_eq!(view.get_focused_card(), Some(task_ids[0]));
        
        view.handle_keyboard_navigation("ArrowDown");
        assert_eq!(view.get_focused_card(), Some(task_ids[1]));
        
        view.handle_keyboard_navigation("ArrowUp");
        assert_eq!(view.get_focused_card(), Some(task_ids[0]));
        
        view.handle_keyboard_navigation("Enter");
        assert!(view.is_editing_task(task_ids[0]));
    }

    #[tokio::test]
    async fn test_screen_reader_labels() {
        let view = KanbanView::new();
        
        let mut task = Task::new("Important Task".to_string(), "Description".to_string());
        task.priority = Priority::High;
        task.status = TaskStatus::InProgress;
        task.add_tag("urgent".to_string());
        
        let aria_label = view.get_card_aria_label(&task);
        assert!(aria_label.contains("Important Task"));
        assert!(aria_label.contains("High priority"));
        assert!(aria_label.contains("In Progress"));
        assert!(aria_label.contains("urgent"));
    }

    #[tokio::test]
    async fn test_focus_trap_in_dialogs() {
        let mut view = KanbanView::new();
        let task_id = Uuid::new_v4();
        
        view.open_edit_dialog(task_id);
        assert!(view.is_focus_trapped());
        
        let focusable_elements = view.get_dialog_focusable_elements();
        assert!(!focusable_elements.is_empty());
        
        view.handle_tab_navigation(false);
        assert_eq!(view.get_focused_element(), focusable_elements[0]);
        
        view.close_edit_dialog();
        assert!(!view.is_focus_trapped());
    }
}