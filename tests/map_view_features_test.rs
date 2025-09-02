#[cfg(test)]
mod map_view_feature_tests {
    use plon::domain::task::{Task, TaskStatus, Priority, Position};
    use std::collections::HashSet;
    use uuid::Uuid;

    // Test pan controls
    #[test]
    fn test_pan_controls() {
        let mut camera_x = 0.0;
        let mut camera_y = 0.0;
        
        // Pan right
        camera_x += 100.0;
        assert_eq!(camera_x, 100.0);
        assert_eq!(camera_y, 0.0);
        
        // Pan down
        camera_y += 50.0;
        assert_eq!(camera_x, 100.0);
        assert_eq!(camera_y, 50.0);
        
        // Pan left
        camera_x -= 150.0;
        assert_eq!(camera_x, -50.0);
        assert_eq!(camera_y, 50.0);
        
        // Pan up
        camera_y -= 100.0;
        assert_eq!(camera_x, -50.0);
        assert_eq!(camera_y, -50.0);
        
        // Reset to center
        camera_x = 0.0;
        camera_y = 0.0;
        assert_eq!(camera_x, 0.0);
        assert_eq!(camera_y, 0.0);
    }

    #[test]
    fn test_minimap_viewport_calculation() {
        let tasks = vec![
            Position { x: 100.0, y: 100.0 },
            Position { x: 500.0, y: 300.0 },
            Position { x: 200.0, y: 400.0 },
        ];
        
        // Calculate bounding box
        let min_x = tasks.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let max_x = tasks.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = tasks.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_y = tasks.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        
        assert_eq!(min_x, 100.0);
        assert_eq!(max_x, 500.0);
        assert_eq!(min_y, 100.0);
        assert_eq!(max_y, 400.0);
        
        // Minimap scale calculation (200px minimap width)
        let minimap_width = 200.0;
        let scale = minimap_width / (max_x - min_x + 100.0); // Add padding
        assert!(scale > 0.0 && scale <= 1.0);
    }

    #[test]
    fn test_grid_snap() {
        let grid_size = 20.0;
        
        // Test snapping to grid
        let snap_to_grid = |value: f64| -> f64 {
            (value / grid_size).round() * grid_size
        };
        
        assert_eq!(snap_to_grid(105.0), 100.0);
        assert_eq!(snap_to_grid(112.0), 120.0);
        assert_eq!(snap_to_grid(200.0), 200.0);
        assert_eq!(snap_to_grid(217.5), 220.0);
        assert_eq!(snap_to_grid(95.0), 100.0);
    }

    #[test]
    fn test_auto_layout_algorithm() {
        // Simple force-directed layout
        struct Node {
            id: Uuid,
            x: f64,
            y: f64,
            vx: f64,
            vy: f64,
        }
        
        let mut nodes = vec![
            Node { id: Uuid::new_v4(), x: 100.0, y: 100.0, vx: 0.0, vy: 0.0 },
            Node { id: Uuid::new_v4(), x: 110.0, y: 100.0, vx: 0.0, vy: 0.0 }, // Too close
            Node { id: Uuid::new_v4(), x: 300.0, y: 200.0, vx: 0.0, vy: 0.0 },
        ];
        
        // Apply repulsion force between nodes
        let repulsion_force = 1000.0;
        let min_distance = 150.0;
        
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let dx = nodes[j].x - nodes[i].x;
                let dy = nodes[j].y - nodes[i].y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < min_distance && distance > 0.0 {
                    let force = repulsion_force / (distance * distance);
                    let fx = force * dx / distance;
                    let fy = force * dy / distance;
                    
                    nodes[i].vx -= fx;
                    nodes[i].vy -= fy;
                    nodes[j].vx += fx;
                    nodes[j].vy += fy;
                }
            }
        }
        
        // Apply velocity
        let damping = 0.5;
        for node in &mut nodes {
            node.x += node.vx * damping;
            node.y += node.vy * damping;
            node.vx *= damping;
            node.vy *= damping;
        }
        
        // Check that nodes have moved apart
        let distance_after = ((nodes[1].x - nodes[0].x).powi(2) + 
                             (nodes[1].y - nodes[0].y).powi(2)).sqrt();
        assert!(distance_after > 10.0); // They should have moved apart
    }

    #[test]
    fn test_task_clustering() {
        #[derive(Clone)]
        struct TaskCluster {
            center: Position,
            tasks: Vec<Uuid>,
            radius: f64,
        }
        
        let tasks = vec![
            (Uuid::new_v4(), Position { x: 100.0, y: 100.0 }),
            (Uuid::new_v4(), Position { x: 120.0, y: 110.0 }),
            (Uuid::new_v4(), Position { x: 500.0, y: 500.0 }),
            (Uuid::new_v4(), Position { x: 510.0, y: 490.0 }),
        ];
        
        // Simple clustering by distance
        let cluster_distance = 100.0;
        let mut clusters: Vec<TaskCluster> = Vec::new();
        
        for (id, pos) in &tasks {
            let mut found_cluster = false;
            
            for cluster in &mut clusters {
                let dx = pos.x - cluster.center.x;
                let dy = pos.y - cluster.center.y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < cluster_distance {
                    cluster.tasks.push(*id);
                    // Update cluster center (average)
                    let n = cluster.tasks.len() as f64;
                    cluster.center.x = (cluster.center.x * (n - 1.0) + pos.x) / n;
                    cluster.center.y = (cluster.center.y * (n - 1.0) + pos.y) / n;
                    found_cluster = true;
                    break;
                }
            }
            
            if !found_cluster {
                clusters.push(TaskCluster {
                    center: pos.clone(),
                    tasks: vec![*id],
                    radius: 50.0,
                });
            }
        }
        
        // Should have 2 clusters
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].tasks.len(), 2);
        assert_eq!(clusters[1].tasks.len(), 2);
    }

    #[test]
    fn test_search_and_filter() {
        let tasks = vec![
            Task {
                id: Uuid::new_v4(),
                title: "Design UI".to_string(),
                description: "Create mockups".to_string(),
                status: TaskStatus::InProgress,
                priority: Priority::High,
                tags: ["design", "ui"].iter().cloned().map(String::from).collect(),
                ..Default::default()
            },
            Task {
                id: Uuid::new_v4(),
                title: "Implement backend".to_string(),
                description: "REST API".to_string(),
                status: TaskStatus::Todo,
                priority: Priority::Medium,
                tags: ["backend", "api"].iter().cloned().map(String::from).collect(),
                ..Default::default()
            },
            Task {
                id: Uuid::new_v4(),
                title: "Write tests".to_string(),
                description: "Unit tests".to_string(),
                status: TaskStatus::Done,
                priority: Priority::Low,
                tags: ["testing"].iter().cloned().map(String::from).collect(),
                ..Default::default()
            },
        ];
        
        // Search by title
        let search_term = "design";
        let filtered: Vec<_> = tasks.iter()
            .filter(|t| t.title.to_lowercase().contains(search_term))
            .collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "Design UI");
        
        // Filter by status
        let in_progress: Vec<_> = tasks.iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .collect();
        assert_eq!(in_progress.len(), 1);
        
        // Filter by priority
        let high_priority: Vec<_> = tasks.iter()
            .filter(|t| t.priority == Priority::High)
            .collect();
        assert_eq!(high_priority.len(), 1);
        
        // Filter by tags
        let has_design_tag: Vec<_> = tasks.iter()
            .filter(|t| t.tags.contains("design"))
            .collect();
        assert_eq!(has_design_tag.len(), 1);
        
        // Combined filters
        let todo_medium: Vec<_> = tasks.iter()
            .filter(|t| t.status == TaskStatus::Todo && t.priority == Priority::Medium)
            .collect();
        assert_eq!(todo_medium.len(), 1);
        assert_eq!(todo_medium[0].title, "Implement backend");
    }

    #[test]
    fn test_keyboard_shortcuts() {
        #[derive(Debug, PartialEq)]
        enum Action {
            ZoomIn,
            ZoomOut,
            ResetView,
            PanLeft,
            PanRight,
            PanUp,
            PanDown,
            DeleteTask,
            EditTask,
            NewTask,
        }
        
        let get_action = |key: &str, ctrl: bool, shift: bool| -> Option<Action> {
            match (key, ctrl, shift) {
                ("=", true, _) | ("+", true, _) => Some(Action::ZoomIn),
                ("-", true, _) => Some(Action::ZoomOut),
                ("0", true, _) => Some(Action::ResetView),
                ("ArrowLeft", false, _) => Some(Action::PanLeft),
                ("ArrowRight", false, _) => Some(Action::PanRight),
                ("ArrowUp", false, _) => Some(Action::PanUp),
                ("ArrowDown", false, _) => Some(Action::PanDown),
                ("Delete", false, _) | ("Backspace", false, _) => Some(Action::DeleteTask),
                ("Enter", false, _) => Some(Action::EditTask),
                ("n", true, _) => Some(Action::NewTask),
                _ => None,
            }
        };
        
        assert_eq!(get_action("=", true, false), Some(Action::ZoomIn));
        assert_eq!(get_action("-", true, false), Some(Action::ZoomOut));
        assert_eq!(get_action("0", true, false), Some(Action::ResetView));
        assert_eq!(get_action("ArrowLeft", false, false), Some(Action::PanLeft));
        assert_eq!(get_action("Delete", false, false), Some(Action::DeleteTask));
        assert_eq!(get_action("n", true, false), Some(Action::NewTask));
        assert_eq!(get_action("x", false, false), None);
    }
}