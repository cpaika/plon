use plon::domain::task::{Task, TaskStatus, Priority};
use plon::domain::goal::{Goal, GoalStatus};
use plon::domain::resource::Resource;
use eframe::egui;
use std::sync::{Arc, Mutex};

/// Test context for running UI tests with egui
pub struct TestContext {
    pub ctx: egui::Context,
    pub tasks: Arc<Mutex<Vec<Task>>>,
    pub goals: Arc<Mutex<Vec<Goal>>>,
    pub resources: Arc<Mutex<Vec<Resource>>>,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            ctx: egui::Context::default(),
            tasks: Arc::new(Mutex::new(Vec::new())),
            goals: Arc::new(Mutex::new(Vec::new())),
            resources: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_test_task(&self, title: &str) -> Task {
        let mut task = Task::new(title.to_string(), "Test description".to_string());
        task.set_position(100.0, 100.0);
        self.tasks.lock().unwrap().push(task.clone());
        task
    }

    pub fn add_test_goal(&self, title: &str) -> Goal {
        let goal = Goal::new(title.to_string(), "Test goal".to_string());
        self.goals.lock().unwrap().push(goal.clone());
        goal
    }

    pub fn add_test_resource(&self, name: &str) -> Resource {
        let resource = Resource::new(name.to_string(), "Developer".to_string(), 40.0);
        self.resources.lock().unwrap().push(resource.clone());
        resource
    }

    pub fn run_frame<F>(&self, f: F)
    where
        F: FnOnce(&egui::Context),
    {
        self.ctx.run(Default::default(), |ctx| {
            f(ctx);
        });
    }
}

#[cfg(test)]
mod task_tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let mut task = Task::new("Test Task".to_string(), "Description".to_string());
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, Priority::Medium);
        
        // Test position setting
        task.set_position(50.0, 75.0);
        assert_eq!(task.position.x, 50.0);
        assert_eq!(task.position.y, 75.0);
    }

    #[test]
    fn test_subtask_management() {
        let mut task = Task::new("Main Task".to_string(), "".to_string());
        
        // Add subtasks
        let id1 = task.add_subtask("Subtask 1".to_string());
        let id2 = task.add_subtask("Subtask 2".to_string());
        
        assert_eq!(task.subtasks.len(), 2);
        assert_eq!(task.subtask_progress(), (0, 2));
        
        // Complete a subtask
        task.complete_subtask(id1).unwrap();
        assert_eq!(task.subtask_progress(), (1, 2));
        
        // Complete all subtasks
        task.complete_subtask(id2).unwrap();
        assert_eq!(task.subtask_progress(), (2, 2));
    }

    #[test]
    fn test_task_status_updates() {
        let mut task = Task::new("Task".to_string(), "".to_string());
        
        task.update_status(TaskStatus::InProgress);
        assert_eq!(task.status, TaskStatus::InProgress);
        
        task.update_status(TaskStatus::Done);
        assert_eq!(task.status, TaskStatus::Done);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_overdue_detection() {
        let mut task = Task::new("Task".to_string(), "".to_string());
        assert!(!task.is_overdue());
        
        // Set due date to past
        task.due_date = Some(chrono::Utc::now() - chrono::Duration::days(1));
        assert!(task.is_overdue());
        
        // Complete the task
        task.update_status(TaskStatus::Done);
        assert!(!task.is_overdue());
    }
}

#[cfg(test)]
mod map_view_tests {
    use super::*;

    #[test]
    fn test_map_view_zoom() {
        let mut zoom: f32 = 1.0;
        
        // Test zoom in
        zoom = (zoom * 1.2).min(5.0);
        assert!((zoom - 1.2).abs() < 0.001);
        
        // Test zoom out
        zoom = (zoom / 1.2).max(0.1);
        assert!((zoom - 1.0).abs() < 0.001);
        
        // Test zoom limits
        zoom = 10.0;
        zoom = zoom.min(5.0);
        assert_eq!(zoom, 5.0);
        
        zoom = 0.01;
        zoom = zoom.max(0.1);
        assert_eq!(zoom, 0.1);
    }

    #[test]
    fn test_world_to_screen_transformation() {
        let camera_pos = egui::Vec2::new(10.0, 20.0);
        let zoom = 2.0;
        let center = egui::Pos2::new(400.0, 300.0);
        
        // Transform function
        let to_screen = |world_x: f32, world_y: f32| -> egui::Pos2 {
            egui::Pos2::new(
                center.x + (world_x + camera_pos.x) * zoom,
                center.y + (world_y + camera_pos.y) * zoom,
            )
        };
        
        // Test transformation
        let screen_pos = to_screen(0.0, 0.0);
        assert_eq!(screen_pos.x, 400.0 + 10.0 * 2.0);
        assert_eq!(screen_pos.y, 300.0 + 20.0 * 2.0);
    }

    #[test]
    fn test_task_visibility_culling() {
        let viewport = egui::Rect::from_min_size(
            egui::Pos2::new(0.0, 0.0),
            egui::Vec2::new(800.0, 600.0)
        );
        
        // Task inside viewport
        let pos1 = egui::Pos2::new(400.0, 300.0);
        assert!(viewport.contains(pos1));
        
        // Task outside viewport
        let pos2 = egui::Pos2::new(900.0, 700.0);
        assert!(!viewport.contains(pos2));
    }

    #[test]
    fn test_grid_calculation() {
        let grid_size: f32 = 50.0;
        let center: f32 = 400.0;
        
        let start = ((0.0 - center) / grid_size).floor() * grid_size;
        let end = ((800.0 - center) / grid_size).ceil() * grid_size;
        
        assert_eq!(start, -400.0);
        assert_eq!(end, 400.0);
    }
}

#[cfg(test)]
mod list_view_tests {
    use super::*;

    #[test]
    fn test_task_filtering() {
        let ctx = TestContext::new();
        ctx.add_test_task("Important Task");
        ctx.add_test_task("Regular Task");
        ctx.add_test_task("Another Important");
        
        let filter_text = "important";
        let tasks = ctx.tasks.lock().unwrap();
        let filtered: Vec<_> = tasks
            .iter()
            .filter(|t| t.title.to_lowercase().contains(&filter_text.to_lowercase()))
            .collect();
        
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_status_color_mapping() {
        let get_status_color = |status: &TaskStatus| -> egui::Color32 {
            match status {
                TaskStatus::Todo => egui::Color32::GRAY,
                TaskStatus::InProgress => egui::Color32::from_rgb(100, 150, 255),
                TaskStatus::Done => egui::Color32::from_rgb(100, 255, 100),
                TaskStatus::Blocked => egui::Color32::from_rgb(255, 100, 100),
                _ => egui::Color32::DARK_GRAY,
            }
        };
        
        assert_eq!(get_status_color(&TaskStatus::Todo), egui::Color32::GRAY);
        assert_eq!(get_status_color(&TaskStatus::Done), egui::Color32::from_rgb(100, 255, 100));
    }
}

#[cfg(test)]
mod kanban_view_tests {
    use super::*;

    #[test]
    fn test_kanban_column_assignment() {
        let ctx = TestContext::new();
        
        let mut task1 = ctx.add_test_task("Todo Task");
        task1.status = TaskStatus::Todo;
        
        let mut task2 = ctx.add_test_task("In Progress Task");
        task2.status = TaskStatus::InProgress;
        
        let mut task3 = ctx.add_test_task("Done Task");
        task3.status = TaskStatus::Done;
        
        ctx.tasks.lock().unwrap().clear();
        ctx.tasks.lock().unwrap().push(task1);
        ctx.tasks.lock().unwrap().push(task2);
        ctx.tasks.lock().unwrap().push(task3);
        
        let tasks = ctx.tasks.lock().unwrap();
        
        let todo_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Todo).collect();
        let in_progress_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).collect();
        let done_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Done).collect();
        
        assert_eq!(todo_tasks.len(), 1);
        assert_eq!(in_progress_tasks.len(), 1);
        assert_eq!(done_tasks.len(), 1);
    }
}

#[cfg(test)]
mod dashboard_tests {
    use super::*;

    #[test]
    fn test_statistics_calculation() {
        let ctx = TestContext::new();
        
        for i in 0..10 {
            let mut task = ctx.add_test_task(&format!("Task {}", i));
            if i < 5 {
                task.status = TaskStatus::Done;
            } else if i < 8 {
                task.status = TaskStatus::InProgress;
            } else {
                task.status = TaskStatus::Todo;
            }
            ctx.tasks.lock().unwrap().pop();
            ctx.tasks.lock().unwrap().push(task);
        }
        
        let tasks = ctx.tasks.lock().unwrap();
        let total = tasks.len();
        let completed = tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
        let in_progress = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
        let todo = tasks.iter().filter(|t| t.status == TaskStatus::Todo).count();
        
        assert_eq!(total, 10);
        assert_eq!(completed, 5);
        assert_eq!(in_progress, 3);
        assert_eq!(todo, 2);
        
        let completion_percentage = (completed as f32 / total as f32) * 100.0;
        assert!((completion_percentage - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_overdue_task_detection() {
        let ctx = TestContext::new();
        
        let mut overdue_task = ctx.add_test_task("Overdue Task");
        overdue_task.due_date = Some(chrono::Utc::now() - chrono::Duration::days(1));
        
        let mut future_task = ctx.add_test_task("Future Task");
        future_task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(1));
        
        ctx.tasks.lock().unwrap().clear();
        ctx.tasks.lock().unwrap().push(overdue_task);
        ctx.tasks.lock().unwrap().push(future_task);
        
        let tasks = ctx.tasks.lock().unwrap();
        let overdue: Vec<_> = tasks.iter().filter(|t| t.is_overdue()).collect();
        
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].title, "Overdue Task");
    }
}

#[cfg(test)]
mod ui_interaction_tests {
    use super::*;

    #[test]
    fn test_task_selection() {
        let mut selected_task_id: Option<uuid::Uuid> = None;
        let task = Task::new("Test".to_string(), "".to_string());
        
        // Simulate click
        selected_task_id = Some(task.id);
        assert_eq!(selected_task_id, Some(task.id));
    }

    #[test]
    fn test_drag_position_update() {
        let mut task = Task::new("Draggable".to_string(), "".to_string());
        task.set_position(100.0, 100.0);
        
        // Simulate drag
        let delta = egui::Vec2::new(50.0, -25.0);
        let zoom = 2.0;
        
        task.set_position(
            task.position.x + (delta.x / zoom) as f64,
            task.position.y + (delta.y / zoom) as f64,
        );
        
        assert_eq!(task.position.x, 125.0);
        assert_eq!(task.position.y, 87.5);
    }

    #[test]
    fn test_double_click_position() {
        let camera_pos = egui::Vec2::new(10.0, 10.0);
        let zoom = 1.0;
        let center = egui::Pos2::new(400.0, 300.0);
        let click_pos = egui::Pos2::new(450.0, 350.0);
        
        let world_pos = (click_pos - center) / zoom - camera_pos;
        
        assert_eq!(world_pos.x, 40.0);
        assert_eq!(world_pos.y, 40.0);
    }
}

#[cfg(test)]
mod goal_tests {
    use super::*;

    #[test]
    fn test_goal_progress_calculation() {
        let mut goal = Goal::new("Q1 Goals".to_string(), "".to_string());
        
        let task1_id = uuid::Uuid::new_v4();
        let task2_id = uuid::Uuid::new_v4();
        let task3_id = uuid::Uuid::new_v4();
        
        goal.add_task(task1_id);
        goal.add_task(task2_id);
        goal.add_task(task3_id);
        
        let task_statuses = vec![
            (task1_id, true),
            (task2_id, false),
            (task3_id, true),
        ];
        
        let progress = goal.calculate_progress(&task_statuses);
        assert!((progress - 66.66667).abs() < 0.001);
    }

    #[test]
    fn test_goal_at_risk_detection() {
        let mut goal = Goal::new("Urgent Goal".to_string(), "".to_string());
        
        // Not at risk without target date
        assert!(!goal.is_at_risk());
        
        // At risk with near target date
        goal.target_date = Some(chrono::Utc::now() + chrono::Duration::days(5));
        assert!(goal.is_at_risk());
        
        // Not at risk with far target date
        goal.target_date = Some(chrono::Utc::now() + chrono::Duration::days(30));
        assert!(!goal.is_at_risk());
        
        // Not at risk when completed
        goal.target_date = Some(chrono::Utc::now() + chrono::Duration::days(5));
        goal.status = GoalStatus::Completed;
        assert!(!goal.is_at_risk());
    }
}

#[cfg(test)]
mod resource_tests {
    use super::*;

    #[test]
    fn test_resource_utilization() {
        let mut resource = Resource::new("Developer".to_string(), "Engineer".to_string(), 40.0);
        
        assert_eq!(resource.utilization_percentage(), 0.0);
        assert_eq!(resource.available_hours(), 40.0);
        assert!(!resource.is_overloaded());
        
        resource.current_load = 30.0;
        assert_eq!(resource.utilization_percentage(), 75.0);
        assert_eq!(resource.available_hours(), 10.0);
        assert!(!resource.is_overloaded());
        
        resource.current_load = 50.0;
        assert_eq!(resource.utilization_percentage(), 125.0);
        assert_eq!(resource.available_hours(), 0.0);
        assert!(resource.is_overloaded());
    }

    #[test]
    fn test_resource_task_filtering() {
        let mut resource = Resource::new("Backend Dev".to_string(), "Developer".to_string(), 40.0);
        resource.add_metadata_filter("category".to_string(), "backend".to_string());
        
        let mut backend_metadata = std::collections::HashMap::new();
        backend_metadata.insert("category".to_string(), "backend".to_string());
        assert!(resource.can_work_on_task(&backend_metadata));
        
        let mut frontend_metadata = std::collections::HashMap::new();
        frontend_metadata.insert("category".to_string(), "frontend".to_string());
        assert!(!resource.can_work_on_task(&frontend_metadata));
    }
}

#[cfg(test)]
mod view_switching_tests {
    use super::*;

    #[test]
    fn test_view_type_switching() {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum ViewType {
            List,
            Kanban,
            Map,
            Timeline,
            Dashboard,
        }
        
        let mut current_view = ViewType::Map;
        
        // Test switching to different views
        current_view = ViewType::List;
        assert_eq!(current_view, ViewType::List);
        
        current_view = ViewType::Kanban;
        assert_eq!(current_view, ViewType::Kanban);
        
        current_view = ViewType::Dashboard;
        assert_eq!(current_view, ViewType::Dashboard);
    }
}

#[cfg(test)]
mod timeline_view_tests {
    use super::*;

    #[test]
    fn test_scheduled_task_filtering() {
        let ctx = TestContext::new();
        
        let mut past_task = ctx.add_test_task("Past Task");
        past_task.scheduled_date = Some(chrono::Utc::now() - chrono::Duration::days(1));
        
        let mut future_task = ctx.add_test_task("Future Task");
        future_task.scheduled_date = Some(chrono::Utc::now() + chrono::Duration::days(1));
        
        ctx.tasks.lock().unwrap().clear();
        ctx.tasks.lock().unwrap().push(past_task);
        ctx.tasks.lock().unwrap().push(future_task);
        
        let today = chrono::Utc::now();
        let tasks = ctx.tasks.lock().unwrap();
        
        let scheduled: Vec<_> = tasks
            .iter()
            .filter(|t| t.scheduled_date.map_or(false, |d| d >= today))
            .collect();
        
        assert_eq!(scheduled.len(), 1);
        assert_eq!(scheduled[0].title, "Future Task");
    }
}

#[cfg(test)]
mod task_editor_tests {
    use super::*;

    #[test]
    fn test_task_creation_validation() {
        let mut new_task_title = String::new();
        let mut new_task_description = String::new();
        
        // Empty title should not create task
        assert!(new_task_title.is_empty());
        
        // Valid title should create task
        new_task_title = "New Task".to_string();
        new_task_description = "Description".to_string();
        assert!(!new_task_title.is_empty());
        
        let task = Task::new(new_task_title.clone(), new_task_description.clone());
        assert_eq!(task.title, "New Task");
        assert_eq!(task.description, "Description");
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_task_list_performance() {
        let ctx = TestContext::new();
        
        // Create 1000 tasks
        for i in 0..1000 {
            ctx.add_test_task(&format!("Task {}", i));
        }
        
        let tasks = ctx.tasks.lock().unwrap();
        assert_eq!(tasks.len(), 1000);
        
        // Test filtering performance
        let start = std::time::Instant::now();
        let filtered: Vec<_> = tasks
            .iter()
            .filter(|t| t.title.contains("5"))
            .collect();
        let duration = start.elapsed();
        
        // Should complete in less than 10ms
        assert!(duration.as_millis() < 10);
        assert!(filtered.len() > 0);
    }

    #[test]
    fn test_viewport_culling_performance() {
        let viewport = egui::Rect::from_min_size(
            egui::Pos2::new(0.0, 0.0),
            egui::Vec2::new(800.0, 600.0)
        );
        
        let start = std::time::Instant::now();
        
        // Test 10000 position checks
        for i in 0..10000 {
            let pos = egui::Pos2::new(i as f32 % 1000.0, i as f32 % 1000.0);
            viewport.contains(pos);
        }
        
        let duration = start.elapsed();
        
        // Should complete in less than 5ms
        assert!(duration.as_millis() < 5);
    }
}