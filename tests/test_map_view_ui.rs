use dioxus::prelude::*;
use dioxus_desktop::DesktopContext;
use plon::ui_dioxus::views::map_final::MapView;
use plon::domain::task::{Task, TaskStatus, Priority};
use plon::domain::dependency::{Dependency, DependencyType};
use uuid::Uuid;
use chrono::Utc;

#[test]
fn test_map_view_renders() {
    // Create a test app with MapView
    let mut vdom = VirtualDom::new(MapView);
    
    // Render the component
    let _ = vdom.rebuild();
    
    // Check that it rendered without panicking
    assert!(vdom.base_scope().has_context::<Signal<Vec<Task>>>().is_some());
}

#[test]
fn test_task_creation() {
    let mut vdom = VirtualDom::new(|| {
        rsx! {
            MapView {}
        }
    });
    
    // Initial render
    let _ = vdom.rebuild();
    
    // Simulate clicking "Add Task" button
    // This would need to trigger the event handler
    // vdom.handle_event() can be used to simulate events
}

#[test]
fn test_dependency_creation_logic() {
    // Test the core logic without UI
    let task1 = Task {
        id: Uuid::new_v4(),
        title: "Task 1".to_string(),
        description: "".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: (100.0, 100.0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: Default::default(),
        tags: Default::default(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
    };
    
    let task2 = Task {
        id: Uuid::new_v4(),
        title: "Task 2".to_string(),
        description: "".to_string(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: (300.0, 100.0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: Default::default(),
        tags: Default::default(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
    };
    
    let dependency = Dependency {
        id: Uuid::new_v4(),
        from_task_id: task1.id,
        to_task_id: task2.id,
        dependency_type: DependencyType::FinishToStart,
        created_at: Utc::now(),
    };
    
    // Verify dependency is valid
    assert_ne!(dependency.from_task_id, dependency.to_task_id);
}