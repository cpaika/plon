use plon::domain::task::{Task, TaskStatus};
use plon::domain::dependency::{Dependency, DependencyType, DependencyGraph};
use plon::ui::MapView;
use eframe::egui::{self, Vec2, Pos2};
use uuid::Uuid;

#[test]
fn test_dependency_creation_state_machine() {
    // Test that the map view can enter and exit dependency creation mode
    let mut map_view = MapView::new();
    
    // Should start with no dependency creation in progress
    assert!(!map_view.is_creating_dependency());
    
    // Create two tasks
    let task1 = Task::new("Task 1".to_string(), String::new());
    let task2 = Task::new("Task 2".to_string(), String::new());
    
    // Start dependency creation from task1
    map_view.start_dependency_creation(task1.id);
    assert!(map_view.is_creating_dependency());
    assert_eq!(map_view.get_dependency_source(), Some(task1.id));
    
    // Complete dependency creation to task2
    let dependency = map_view.complete_dependency_creation(task2.id);
    assert!(dependency.is_some());
    assert!(!map_view.is_creating_dependency());
    
    let dep = dependency.unwrap();
    assert_eq!(dep.from_task_id, task1.id);
    assert_eq!(dep.to_task_id, task2.id);
    assert_eq!(dep.dependency_type, DependencyType::FinishToStart);
}

#[test]
fn test_dependency_creation_cancellation() {
    let mut map_view = MapView::new();
    let task1 = Task::new("Task 1".to_string(), String::new());
    
    // Start dependency creation
    map_view.start_dependency_creation(task1.id);
    assert!(map_view.is_creating_dependency());
    
    // Cancel dependency creation
    map_view.cancel_dependency_creation();
    assert!(!map_view.is_creating_dependency());
    assert_eq!(map_view.get_dependency_source(), None);
}

#[test]
fn test_dependency_arrow_preview() {
    let mut map_view = MapView::new();
    let task1_id = Uuid::new_v4();
    
    // Start dependency creation
    map_view.start_dependency_creation(task1_id);
    
    // Get preview arrow endpoints
    let start_pos = Vec2::new(100.0, 100.0);
    let mouse_pos = Vec2::new(200.0, 150.0);
    
    let preview = map_view.get_dependency_preview(start_pos, mouse_pos);
    assert!(preview.is_some());
    
    let (start, end) = preview.unwrap();
    assert_eq!(start, start_pos);
    assert_eq!(end, mouse_pos);
}

#[test]
fn test_dependency_visualization_data() {
    let mut graph = DependencyGraph::new();
    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();
    
    // Create dependencies
    let dep1 = Dependency::new(task1, task2, DependencyType::FinishToStart);
    let dep2 = Dependency::new(task2, task3, DependencyType::StartToStart);
    
    graph.add_dependency(&dep1).unwrap();
    graph.add_dependency(&dep2).unwrap();
    
    // Get all dependencies for visualization
    let all_deps = graph.get_all_dependencies();
    assert_eq!(all_deps.len(), 2);
    
    // Check dependency exists
    let deps_of_task2 = graph.get_dependencies(task2);
    assert_eq!(deps_of_task2.len(), 1);
    assert_eq!(deps_of_task2[0].0, task1);
}

#[test]
fn test_arrow_path_calculation() {
    use plon::ui::calculate_arrow_path;
    
    // Test straight arrow
    let start = Pos2::new(100.0, 100.0);
    let end = Pos2::new(200.0, 100.0);
    let path = calculate_arrow_path(start, end, DependencyType::FinishToStart);
    
    // Should have control points for a bezier curve
    assert!(path.len() >= 2);
    assert_eq!(path[0], start);
    assert_eq!(path[path.len() - 1], end);
    
    // Test different dependency types
    let path_ss = calculate_arrow_path(start, end, DependencyType::StartToStart);
    let path_ff = calculate_arrow_path(start, end, DependencyType::FinishToFinish);
    
    // Different types should produce different paths
    assert_ne!(path, path_ss);
    assert_ne!(path, path_ff);
}

#[test]
fn test_arrow_hit_detection() {
    use plon::ui::is_point_near_arrow;
    
    let start = Pos2::new(100.0, 100.0);
    let end = Pos2::new(200.0, 100.0);
    
    // Point on the line should be detected
    let on_line = Pos2::new(150.0, 100.0);
    assert!(is_point_near_arrow(on_line, start, end, 5.0));
    
    // Point near the line should be detected
    let near_line = Pos2::new(150.0, 103.0);
    assert!(is_point_near_arrow(near_line, start, end, 5.0));
    
    // Point far from line should not be detected
    let far_from_line = Pos2::new(150.0, 110.0);
    assert!(!is_point_near_arrow(far_from_line, start, end, 5.0));
}

#[test]
fn test_dependency_cycle_prevention() {
    let mut graph = DependencyGraph::new();
    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();
    
    // Create valid chain
    let dep1 = Dependency::new(task1, task2, DependencyType::FinishToStart);
    let dep2 = Dependency::new(task2, task3, DependencyType::FinishToStart);
    
    assert!(graph.add_dependency(&dep1).is_ok());
    assert!(graph.add_dependency(&dep2).is_ok());
    
    // Try to create cycle
    let dep_cycle = Dependency::new(task3, task1, DependencyType::FinishToStart);
    assert!(graph.add_dependency(&dep_cycle).is_err());
}

#[test]
fn test_dependency_deletion() {
    let mut graph = DependencyGraph::new();
    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    
    // Add dependency
    let dep = Dependency::new(task1, task2, DependencyType::FinishToStart);
    graph.add_dependency(&dep).unwrap();
    
    // Verify it exists
    assert_eq!(graph.get_dependencies(task2).len(), 1);
    
    // Delete it
    assert!(graph.remove_dependency(task1, task2));
    
    // Verify it's gone
    assert_eq!(graph.get_dependencies(task2).len(), 0);
}

#[test]
fn test_critical_path_with_dependencies() {
    let mut graph = DependencyGraph::new();
    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();
    let task4 = Uuid::new_v4();
    
    // Create diamond shape dependencies
    graph.add_dependency(&Dependency::new(task1, task2, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(task1, task3, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(task2, task4, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(task3, task4, DependencyType::FinishToStart)).unwrap();
    
    // Set estimates - make path through task2 longer
    let mut estimates = std::collections::HashMap::new();
    estimates.insert(task1, 2.0);
    estimates.insert(task2, 10.0); // Longer
    estimates.insert(task3, 3.0);  // Shorter
    estimates.insert(task4, 1.0);
    
    let critical_path = graph.get_critical_path(&estimates);
    
    // Critical path should go through task2
    assert_eq!(critical_path.len(), 3);
    assert_eq!(critical_path[0], task1);
    assert_eq!(critical_path[1], task2);
    assert_eq!(critical_path[2], task4);
}

#[test]
fn test_total_project_duration_with_dependencies() {
    let mut graph = DependencyGraph::new();
    let task1 = Uuid::new_v4();
    let task2 = Uuid::new_v4();
    let task3 = Uuid::new_v4();
    
    // Create sequential dependencies
    graph.add_dependency(&Dependency::new(task1, task2, DependencyType::FinishToStart)).unwrap();
    graph.add_dependency(&Dependency::new(task2, task3, DependencyType::FinishToStart)).unwrap();
    
    let mut estimates = std::collections::HashMap::new();
    estimates.insert(task1, 5.0);
    estimates.insert(task2, 3.0);
    estimates.insert(task3, 2.0);
    
    // Total duration should be sum of all tasks in sequence
    let critical_path = graph.get_critical_path(&estimates);
    let total_duration: f32 = critical_path.iter()
        .map(|id| estimates.get(id).unwrap_or(&0.0))
        .sum();
    
    assert_eq!(total_duration, 10.0);
}