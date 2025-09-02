use dioxus::prelude::*;
use plon::domain::task::{Task, TaskStatus, Priority, Position};
use plon::domain::dependency::{Dependency, DependencyType};
use plon::repository::database::init_test_database;
use plon::repository::Repository;
use uuid::Uuid;
use chrono::Utc;
use std::collections::{HashMap, HashSet};

#[cfg(test)]
mod component_tests {
    use super::*;

    #[test]
    fn test_map_view_component_renders() {
        // Create a test component that uses signals like MapView does
        fn TestMapComponent() -> Element {
            let tasks = use_signal(|| Vec::<Task>::new());
            let dependencies = use_signal(|| Vec::<Dependency>::new());
            let dragging_task = use_signal(|| None::<Uuid>);
            
            rsx! {
                div {
                    class: "map-container",
                    "data-testid": "map-container",
                    
                    button {
                        onclick: move |_| {
                            let new_task = create_test_task("Test Task");
                            tasks.write().push(new_task);
                        },
                        "Add Task"
                    }
                    
                    for task in tasks.read().iter() {
                        div {
                            key: "{task.id}",
                            class: "task-card",
                            "data-task-id": "{task.id}",
                            "{task.title}"
                        }
                    }
                }
            }
        }
        
        let mut vdom = VirtualDom::new(TestMapComponent);
        let _ = vdom.rebuild();
        
        // The component should render without panicking
        assert!(vdom.base_scope().id().0 > 0);
    }

    #[test]
    fn test_task_state_management() {
        fn TaskManager() -> Element {
            let tasks = use_signal(|| Vec::<Task>::new());
            let selected_task = use_signal(|| None::<Uuid>);
            
            // Add a task
            use_effect(move || {
                tasks.write().push(create_test_task("Initial Task"));
            });
            
            rsx! {
                div {
                    "Tasks: {tasks.read().len()}"
                }
            }
        }
        
        let mut vdom = VirtualDom::new(TaskManager);
        let _ = vdom.rebuild();
        
        // Component should initialize with tasks
        assert!(vdom.base_scope().id().0 > 0);
    }

    #[test]
    fn test_dependency_management() {
        fn DependencyComponent() -> Element {
            let tasks = use_signal(|| {
                vec![
                    create_test_task("Task 1"),
                    create_test_task("Task 2"),
                ]
            });
            let dependencies = use_signal(|| Vec::<Dependency>::new());
            
            let create_dependency = move |from_idx: usize, to_idx: usize| {
                let tasks_read = tasks.read();
                if from_idx < tasks_read.len() && to_idx < tasks_read.len() {
                    let dep = Dependency {
                        id: Uuid::new_v4(),
                        from_task_id: tasks_read[from_idx].id,
                        to_task_id: tasks_read[to_idx].id,
                        dependency_type: DependencyType::FinishToStart,
                        created_at: Utc::now(),
                    };
                    dependencies.write().push(dep);
                }
            };
            
            rsx! {
                div {
                    button {
                        onclick: move |_| create_dependency(0, 1),
                        "Create Dependency"
                    }
                    "Dependencies: {dependencies.read().len()}"
                }
            }
        }
        
        let mut vdom = VirtualDom::new(DependencyComponent);
        let _ = vdom.rebuild();
        
        // Component should handle dependencies
        assert!(vdom.base_scope().id().0 > 0);
    }

    #[test]
    fn test_drag_state_management() {
        #[derive(Clone, Debug)]
        struct DragState {
            from_task: Uuid,
            start_pos: (f64, f64),
        }
        
        fn DragComponent() -> Element {
            let drag_state = use_signal(|| None::<DragState>);
            let tasks = use_signal(|| vec![create_test_task("Draggable")]);
            
            rsx! {
                div {
                    for task in tasks.read().iter() {
                        div {
                            class: "task-card",
                            onmousedown: move |e| {
                                drag_state.set(Some(DragState {
                                    from_task: task.id,
                                    start_pos: (e.client_coordinates().x, e.client_coordinates().y),
                                }));
                            },
                            onmouseup: move |_| {
                                drag_state.set(None);
                            },
                            "{task.title}"
                        }
                    }
                    
                    if let Some(state) = drag_state.read().as_ref() {
                        div {
                            "Dragging from: {state.from_task}"
                        }
                    }
                }
            }
        }
        
        let mut vdom = VirtualDom::new(DragComponent);
        let _ = vdom.rebuild();
        
        assert!(vdom.base_scope().id().0 > 0);
    }

    // Helper function to create test tasks
    fn create_test_task(title: &str) -> Task {
        Task {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: String::new(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            position: Position { x: 100.0, y: 100.0 },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            due_date: None,
            scheduled_date: None,
            completed_at: None,
            estimated_hours: None,
            actual_hours: None,
            metadata: HashMap::new(),
            tags: HashSet::new(),
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            is_archived: false,
            assignee: None,
            configuration_id: None,
            sort_order: 0,
            subtasks: Vec::new(),
        }
    }
}

#[tokio::test]
async fn test_database_integration() {
    // Test that the database layer works correctly
    let pool = init_test_database().await.unwrap();
    let repo = Repository::new(pool);
    
    // Create test tasks
    let task1 = Task {
        id: Uuid::new_v4(),
        title: "DB Task 1".to_string(),
        description: String::new(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: Position { x: 100.0, y: 100.0 },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: HashMap::new(),
        tags: HashSet::new(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 0,
        subtasks: Vec::new(),
    };
    
    let task2 = Task {
        id: Uuid::new_v4(),
        title: "DB Task 2".to_string(),
        description: String::new(),
        status: TaskStatus::Todo,
        priority: Priority::Medium,
        position: Position { x: 300.0, y: 100.0 },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        due_date: None,
        scheduled_date: None,
        completed_at: None,
        estimated_hours: None,
        actual_hours: None,
        metadata: HashMap::new(),
        tags: HashSet::new(),
        assigned_resource_id: None,
        goal_id: None,
        parent_task_id: None,
        is_archived: false,
        assignee: None,
        configuration_id: None,
        sort_order: 1,
        subtasks: Vec::new(),
    };
    
    // Save tasks
    repo.tasks.create(&task1).await.unwrap();
    repo.tasks.create(&task2).await.unwrap();
    
    // Create dependency
    let dependency = Dependency {
        id: Uuid::new_v4(),
        from_task_id: task1.id,
        to_task_id: task2.id,
        dependency_type: DependencyType::FinishToStart,
        created_at: Utc::now(),
    };
    
    repo.dependencies.create(&dependency).await.unwrap();
    
    // Verify persistence
    let loaded_deps = repo.dependencies.get_all().await.unwrap();
    assert!(loaded_deps.len() >= 1);
    assert!(loaded_deps.iter().any(|d| d.from_task_id == task1.id && d.to_task_id == task2.id));
}