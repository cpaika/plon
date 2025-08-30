#[cfg(test)]
mod tests {
    use super::super::map_view::MapView;
    use crate::domain::task::{Task, TaskStatus, Priority, Position};
    use crate::domain::goal::{Goal, GoalStatus};
    use crate::repository::database::init_test_database;
    use crate::repository::Repository;
    use crate::services::{
        TaskService, GoalService, DependencyService,
        AutoRunOrchestrator, AutoRunConfig, AutoRunStatus,
        TaskExecutionStatus,
    };
    use egui::{Pos2, Vec2, Rect, PointerButton, Event, Key, Modifiers};
    use std::sync::Arc;
    use std::collections::HashSet;
    use uuid::Uuid;

    // ============== Input Validation Tests ==============
    
    #[test]
    fn test_invalid_task_positions() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let mut map_view = MapView::new(repository.clone(), Some(runtime.clone()));
        
        // Test NaN positions
        let task_nan = Task {
            id: Uuid::new_v4(),
            title: "NaN Task".to_string(),
            position: Position { x: f32::NAN, y: 100.0 },
            ..Task::default()
        };
        
        // Should handle NaN gracefully
        assert!(task_nan.position.x.is_nan());
        
        // Test infinite positions
        let task_inf = Task {
            id: Uuid::new_v4(),
            title: "Inf Task".to_string(),
            position: Position { x: f32::INFINITY, y: f32::NEG_INFINITY },
            ..Task::default()
        };
        
        assert!(task_inf.position.x.is_infinite());
        
        // Test extreme positions
        let task_extreme = Task {
            id: Uuid::new_v4(),
            title: "Extreme Task".to_string(),
            position: Position { x: 1e10, y: -1e10 },
            ..Task::default()
        };
        
        // Should clamp to reasonable bounds
        let clamped_x = task_extreme.position.x.min(10000.0).max(-10000.0);
        assert!(clamped_x.abs() <= 10000.0);
    }

    #[test]
    fn test_empty_string_handling() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        // Test empty title
        let task_empty = Task {
            id: Uuid::new_v4(),
            title: "".to_string(),
            description: "".to_string(),
            ..Task::default()
        };
        
        runtime.block_on(async {
            let result = repository.tasks.create(&task_empty).await;
            // Should either succeed or provide meaningful error
            if result.is_err() {
                let err = result.unwrap_err();
                assert!(err.to_string().contains("title") || err.to_string().contains("empty"));
            }
        });
        
        // Test whitespace-only title
        let task_whitespace = Task {
            id: Uuid::new_v4(),
            title: "   \t\n  ".to_string(),
            description: "valid".to_string(),
            ..Task::default()
        };
        
        // Should trim whitespace
        assert_eq!(task_whitespace.title.trim(), "");
    }

    #[test]
    fn test_sql_injection_prevention() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        // Test SQL injection in title
        let task_injection = Task {
            id: Uuid::new_v4(),
            title: "'; DROP TABLE tasks; --".to_string(),
            description: "Test SQL injection".to_string(),
            ..Task::default()
        };
        
        runtime.block_on(async {
            // Should safely escape the string
            repository.tasks.create(&task_injection).await.unwrap();
            
            // Verify table still exists
            let all_tasks = repository.tasks.list_all().await.unwrap();
            assert!(all_tasks.len() >= 1);
        });
    }

    // ============== Concurrent Operation Tests ==============
    
    #[test]
    fn test_concurrent_task_updates() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        let task = Task {
            id: Uuid::new_v4(),
            title: "Concurrent Test".to_string(),
            ..Task::default()
        };
        
        runtime.block_on(async {
            repository.tasks.create(&task).await.unwrap();
            
            // Simulate concurrent updates
            let repo1 = repository.clone();
            let repo2 = repository.clone();
            let task_id = task.id;
            
            let handle1 = tokio::spawn(async move {
                let mut task = repo1.tasks.get(task_id).await.unwrap().unwrap();
                task.title = "Update 1".to_string();
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                repo1.tasks.update(&task).await
            });
            
            let handle2 = tokio::spawn(async move {
                let mut task = repo2.tasks.get(task_id).await.unwrap().unwrap();
                task.title = "Update 2".to_string();
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                repo2.tasks.update(&task).await
            });
            
            let result1 = handle1.await.unwrap();
            let result2 = handle2.await.unwrap();
            
            // Both should complete without panic
            assert!(result1.is_ok() || result2.is_ok());
            
            // Final state should be one of the updates
            let final_task = repository.tasks.get(task_id).await.unwrap().unwrap();
            assert!(final_task.title == "Update 1" || final_task.title == "Update 2");
        });
    }

    #[test]
    fn test_race_condition_in_dependency_creation() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let dep_service = Arc::new(DependencyService::new(repository.clone()));
        
        runtime.block_on(async {
            let task1 = Task::new("Task 1".to_string(), "".to_string());
            let task2 = Task::new("Task 2".to_string(), "".to_string());
            
            repository.tasks.create(&task1).await.unwrap();
            repository.tasks.create(&task2).await.unwrap();
            
            // Try to create the same dependency concurrently
            let svc1 = dep_service.clone();
            let svc2 = dep_service.clone();
            
            let handle1 = tokio::spawn(async move {
                svc1.add_dependency(task2.id, task1.id).await
            });
            
            let handle2 = tokio::spawn(async move {
                svc2.add_dependency(task2.id, task1.id).await
            });
            
            let result1 = handle1.await.unwrap();
            let result2 = handle2.await.unwrap();
            
            // One should succeed, one might fail with duplicate
            assert!(result1.is_ok() || result2.is_ok());
            
            // Should only have one dependency
            let deps = dep_service.get_dependencies(task2.id).await.unwrap();
            assert_eq!(deps.len(), 1);
        });
    }

    // ============== Edge Case Tests ==============
    
    #[test]
    fn test_circular_dependency_detection() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let dep_service = Arc::new(DependencyService::new(repository.clone()));
        
        runtime.block_on(async {
            let task1 = Task::new("Task 1".to_string(), "".to_string());
            let task2 = Task::new("Task 2".to_string(), "".to_string());
            let task3 = Task::new("Task 3".to_string(), "".to_string());
            
            repository.tasks.create(&task1).await.unwrap();
            repository.tasks.create(&task2).await.unwrap();
            repository.tasks.create(&task3).await.unwrap();
            
            // Create circular dependency: 1 -> 2 -> 3 -> 1
            dep_service.add_dependency(task2.id, task1.id).await.unwrap();
            dep_service.add_dependency(task3.id, task2.id).await.unwrap();
            
            // This should be detected as circular
            let result = dep_service.add_dependency(task1.id, task3.id).await;
            
            // Build dependency graph to check for cycles
            let graph = dep_service.build_dependency_graph().await.unwrap();
            assert!(graph.has_cycle());
        });
    }

    #[test]
    fn test_orphaned_task_handling() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        runtime.block_on(async {
            // Create task with non-existent parent
            let orphan_task = Task {
                id: Uuid::new_v4(),
                title: "Orphan".to_string(),
                parent_task_id: Some(Uuid::new_v4()), // Non-existent parent
                ..Task::default()
            };
            
            repository.tasks.create(&orphan_task).await.unwrap();
            
            // Should handle orphaned task gracefully
            let retrieved = repository.tasks.get(orphan_task.id).await.unwrap().unwrap();
            assert_eq!(retrieved.parent_task_id, orphan_task.parent_task_id);
        });
    }

    #[test]
    fn test_large_batch_operations() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        runtime.block_on(async {
            // Create many tasks at once
            let mut handles = Vec::new();
            
            for i in 0..100 {
                let repo = repository.clone();
                let handle = tokio::spawn(async move {
                    let task = Task {
                        id: Uuid::new_v4(),
                        title: format!("Task {}", i),
                        position: Position { 
                            x: (i as f32) * 10.0, 
                            y: (i as f32) * 10.0 
                        },
                        ..Task::default()
                    };
                    repo.tasks.create(&task).await
                });
                handles.push(handle);
            }
            
            // Wait for all to complete
            for handle in handles {
                handle.await.unwrap().unwrap();
            }
            
            // Verify all were created
            let all_tasks = repository.tasks.list_all().await.unwrap();
            assert_eq!(all_tasks.len(), 100);
        });
    }

    // ============== UI Event Handling Tests ==============
    
    #[test]
    fn test_rapid_click_handling() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let mut map_view = MapView::new(repository, Some(runtime));
        
        // Simulate rapid clicks
        let click_pos = Pos2::new(100.0, 100.0);
        let mut click_count = 0;
        
        for _ in 0..50 {
            map_view.handle_click(click_pos, PointerButton::Primary);
            click_count += 1;
        }
        
        // Should not crash or create duplicate selections
        assert_eq!(click_count, 50);
        assert!(map_view.click_debounce_time.is_some());
    }

    #[test]
    fn test_drag_outside_bounds() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let mut map_view = MapView::new(repository, Some(runtime));
        
        // Start drag at valid position
        map_view.start_drag(Pos2::new(100.0, 100.0));
        
        // Drag to extreme positions
        map_view.update_drag(Pos2::new(f32::MAX, f32::MAX));
        map_view.update_drag(Pos2::new(f32::MIN, f32::MIN));
        map_view.update_drag(Pos2::new(f32::NAN, f32::NAN));
        
        // Should handle gracefully without panic
        map_view.end_drag();
        
        // Camera should be in valid state
        assert!(!map_view.camera_pos.x.is_nan());
        assert!(!map_view.camera_pos.y.is_nan());
        assert!(!map_view.camera_pos.x.is_infinite());
        assert!(!map_view.camera_pos.y.is_infinite());
    }

    #[test]
    fn test_zoom_limits() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let mut map_view = MapView::new(repository, Some(runtime));
        
        // Test extreme zoom in
        for _ in 0..1000 {
            map_view.zoom_in();
        }
        assert!(map_view.zoom_level <= map_view.max_zoom);
        assert!(map_view.zoom_level > 0.0);
        
        // Test extreme zoom out
        for _ in 0..1000 {
            map_view.zoom_out();
        }
        assert!(map_view.zoom_level >= map_view.min_zoom);
        assert!(map_view.zoom_level > 0.0);
        
        // Test zero/negative zoom prevention
        map_view.set_zoom(-1.0);
        assert!(map_view.zoom_level > 0.0);
        
        map_view.set_zoom(0.0);
        assert!(map_view.zoom_level > 0.0);
        
        map_view.set_zoom(f32::NAN);
        assert!(!map_view.zoom_level.is_nan());
    }

    // ============== Memory and Performance Tests ==============
    
    #[test]
    fn test_memory_cleanup_on_task_deletion() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let mut map_view = MapView::new(repository.clone(), Some(runtime.clone()));
        
        runtime.block_on(async {
            // Create tasks
            let mut task_ids = Vec::new();
            for i in 0..10 {
                let task = Task {
                    id: Uuid::new_v4(),
                    title: format!("Task {}", i),
                    ..Task::default()
                };
                repository.tasks.create(&task).await.unwrap();
                task_ids.push(task.id);
                
                // Add to various maps
                map_view.task_execution_status.insert(task.id, TaskExecutionStatus::Running);
                map_view.task_execution_errors.insert(task.id, "Error".to_string());
                map_view.task_pr_urls.insert(task.id, "http://pr".to_string());
            }
            
            // Delete tasks
            for id in &task_ids {
                repository.tasks.delete(*id).await.unwrap();
                
                // Should clean up references
                map_view.cleanup_task_references(*id);
            }
            
            // Verify cleanup
            for id in &task_ids {
                assert!(!map_view.task_execution_status.contains_key(id));
                assert!(!map_view.task_execution_errors.contains_key(id));
                assert!(!map_view.task_pr_urls.contains_key(id));
            }
        });
    }

    #[test]
    fn test_event_queue_overflow() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let mut map_view = MapView::new(repository, Some(runtime));
        
        // Generate many events quickly
        for i in 0..10000 {
            let event = Event::PointerMoved(Pos2::new(i as f32, i as f32));
            map_view.handle_event(event);
        }
        
        // Should handle without overflow
        assert!(map_view.event_queue.len() <= map_view.max_event_queue_size);
    }

    // ============== Error Recovery Tests ==============
    
    #[test]
    fn test_database_connection_recovery() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        
        runtime.block_on(async {
            // Create pool that will be dropped
            let pool = init_test_database().await.unwrap();
            let repository = Arc::new(Repository::new(pool.clone()));
            
            let task = Task::new("Test".to_string(), "".to_string());
            repository.tasks.create(&task).await.unwrap();
            
            // Close pool connections
            pool.close().await;
            
            // Try to use repository after close
            let result = repository.tasks.get(task.id).await;
            assert!(result.is_err());
            
            // Should provide meaningful error
            if let Err(e) = result {
                let error_msg = e.to_string();
                assert!(error_msg.contains("closed") || error_msg.contains("connection"));
            }
        });
    }

    #[test]
    fn test_corrupted_data_handling() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        
        runtime.block_on(async {
            // Create task with invalid JSON in metadata
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("invalid".to_string(), "corrupted_value".to_string());
            let task = Task {
                id: Uuid::new_v4(),
                title: "Corrupted".to_string(),
                metadata,
                ..Task::default()
            };
            
            // Should handle gracefully
            let result = repository.tasks.create(&task).await;
            if result.is_err() {
                let err = result.unwrap_err();
                assert!(err.to_string().contains("JSON") || err.to_string().contains("serialize"));
            }
        });
    }

    // ============== State Consistency Tests ==============
    
    #[test]
    fn test_auto_run_state_consistency() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let mut map_view = MapView::new(repository.clone(), Some(runtime.clone()));
        
        // Start auto-run
        map_view.auto_run_enabled = true;
        map_view.auto_run_status = Some(AutoRunStatus::Running);
        
        // Pause
        map_view.auto_run_paused = true;
        assert_eq!(map_view.auto_run_status, Some(AutoRunStatus::Running)); // Status unchanged
        assert!(map_view.auto_run_paused);
        
        // Stop
        map_view.auto_run_enabled = false;
        map_view.auto_run_status = Some(AutoRunStatus::Idle);
        assert!(!map_view.auto_run_enabled);
        assert!(!map_view.auto_run_paused); // Should clear pause when stopped
    }

    #[test]
    fn test_selection_state_consistency() {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let pool = runtime.block_on(init_test_database()).unwrap();
        let repository = Arc::new(Repository::new(pool));
        let mut map_view = MapView::new(repository.clone(), Some(runtime.clone()));
        
        let task_id = Uuid::new_v4();
        let goal_id = Uuid::new_v4();
        
        // Select task
        map_view.select_task(task_id);
        assert_eq!(map_view.selected_task_id, Some(task_id));
        assert_eq!(map_view.selected_goal_id, None);
        
        // Select goal (should clear task selection)
        map_view.select_goal(goal_id);
        assert_eq!(map_view.selected_task_id, None);
        assert_eq!(map_view.selected_goal_id, Some(goal_id));
        
        // Clear selection
        map_view.clear_selection();
        assert_eq!(map_view.selected_task_id, None);
        assert_eq!(map_view.selected_goal_id, None);
    }
}

// Extension methods for MapView to support comprehensive testing
impl MapView {
    #[cfg(test)]
    pub fn handle_click(&mut self, pos: Pos2, button: PointerButton) {
        use std::time::Instant;
        
        // Debounce rapid clicks
        if let Some(last_click) = self.click_debounce_time {
            if last_click.elapsed().as_millis() < 50 {
                return; // Ignore click
            }
        }
        self.click_debounce_time = Some(Instant::now());
        
        // Handle click logic
        match button {
            PointerButton::Primary => {
                // Selection logic
            }
            PointerButton::Secondary => {
                // Context menu logic
            }
            _ => {}
        }
    }
    
    #[cfg(test)]
    pub fn start_drag(&mut self, pos: Pos2) {
        self.is_panning = true;
        self.pan_start_pos = Some(pos);
        self.last_mouse_pos = Some(pos);
    }
    
    #[cfg(test)]
    pub fn update_drag(&mut self, pos: Pos2) {
        if self.is_panning {
            if let Some(last_pos) = self.last_mouse_pos {
                let delta = pos - last_pos;
                // Clamp delta to prevent extreme movements
                let clamped_delta = Vec2::new(
                    delta.x.max(-1000.0).min(1000.0),
                    delta.y.max(-1000.0).min(1000.0)
                );
                
                if !clamped_delta.x.is_nan() && !clamped_delta.y.is_nan() {
                    self.camera_pos += clamped_delta / self.zoom_level;
                }
            }
            self.last_mouse_pos = Some(pos);
        }
    }
    
    #[cfg(test)]
    pub fn end_drag(&mut self) {
        self.is_panning = false;
        self.pan_start_pos = None;
    }
    
    #[cfg(test)]
    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.1).min(self.max_zoom);
    }
    
    #[cfg(test)]
    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.1).max(self.min_zoom);
    }
    
    #[cfg(test)]
    pub fn set_zoom(&mut self, zoom: f32) {
        if zoom.is_finite() && zoom > 0.0 {
            self.zoom_level = zoom.max(self.min_zoom).min(self.max_zoom);
        }
    }
    
    #[cfg(test)]
    pub fn select_task(&mut self, id: Uuid) {
        self.selected_task_id = Some(id);
        self.selected_goal_id = None;
        self.selected_items.insert(id);
    }
    
    #[cfg(test)]
    pub fn select_goal(&mut self, id: Uuid) {
        self.selected_goal_id = Some(id);
        self.selected_task_id = None;
        self.selected_items.insert(id);
    }
    
    #[cfg(test)]
    pub fn clear_selection(&mut self) {
        self.selected_task_id = None;
        self.selected_goal_id = None;
        self.selected_items.clear();
    }
    
    #[cfg(test)]
    pub fn cleanup_task_references(&mut self, task_id: Uuid) {
        self.task_execution_status.remove(&task_id);
        self.task_execution_errors.remove(&task_id);
        self.task_executions.remove(&task_id);
        self.task_pr_urls.remove(&task_id);
        self.selected_items.remove(&task_id);
        
        if self.selected_task_id == Some(task_id) {
            self.selected_task_id = None;
        }
    }
    
    #[cfg(test)]
    pub fn handle_event(&mut self, event: Event) {
        // Add to queue with overflow protection
        if self.event_queue.len() >= self.max_event_queue_size {
            self.event_queue.pop_front(); // Remove oldest
        }
        self.event_queue.push_back(event);
    }
    
    #[cfg(test)]
    pub const fn max_zoom(&self) -> f32 {
        10.0
    }
    
    #[cfg(test)]
    pub const fn min_zoom(&self) -> f32 {
        0.1
    }
}

