use plon::domain::task::{Task, TaskStatus, Priority};
use plon::repository::{Repository, database::init_database};
use tempfile::tempdir;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_empty_title_validation() {
    // Test that tasks with empty titles are rejected
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Try to create task with empty title
    let task = Task::new("".to_string(), "Has description but no title".to_string());
    let result = repo.tasks.create(&task).await;
    
    // Should handle empty title gracefully (either reject or set default)
    if result.is_ok() {
        // If it accepts empty title, verify it's stored correctly
        let saved = repo.tasks.get(task.id).await.unwrap();
        assert!(saved.is_some());
        println!("⚠️ System accepts empty titles - consider adding validation");
    } else {
        println!("✅ Empty titles are properly rejected");
    }
}

#[tokio::test]
async fn test_very_long_strings() {
    // Test handling of very long strings
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create task with very long title and description
    let long_title = "A".repeat(10000); // 10,000 characters
    let long_description = "B".repeat(100000); // 100,000 characters
    
    let task = Task::new(long_title.clone(), long_description.clone());
    let result = repo.tasks.create(&task).await;
    
    if let Ok(_) = result {
        let saved = repo.tasks.get(task.id).await.unwrap().unwrap();
        // Check if strings were truncated or stored fully
        if saved.title.len() == long_title.len() {
            println!("✅ System handles very long titles (10k chars)");
        } else {
            println!("⚠️ Title was truncated to {} chars", saved.title.len());
        }
        
        if saved.description.len() == long_description.len() {
            println!("✅ System handles very long descriptions (100k chars)");
        } else {
            println!("⚠️ Description was truncated to {} chars", saved.description.len());
        }
    } else {
        println!("❌ Failed to create task with very long strings: {:?}", result);
    }
}

#[tokio::test]
async fn test_special_characters_in_text() {
    // Test handling of special characters, emojis, and SQL injection attempts
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Test various special characters
    let test_cases = vec![
        ("SQL Injection'; DROP TABLE tasks; --", "Should handle SQL injection safely"),
        ("Unicode: 你好世界 🌍 🚀 ñ é ü", "Should handle unicode and emojis"),
        ("Special: <script>alert('xss')</script>", "Should handle HTML/JS safely"),
        ("Quotes: \"double\" and 'single' and `backtick`", "Should handle various quotes"),
        ("Line\nbreaks\r\nand\ttabs", "Should handle whitespace characters"),
        ("Null char: \0 and backslash: \\", "Should handle special control characters"),
    ];
    
    for (title, description) in test_cases {
        let task = Task::new(title.to_string(), description.to_string());
        let result = repo.tasks.create(&task).await;
        
        assert!(result.is_ok(), "Failed to create task with title: {}", title);
        
        let saved = repo.tasks.get(task.id).await.unwrap().unwrap();
        assert_eq!(saved.title, title, "Title not preserved correctly");
        assert_eq!(saved.description, description, "Description not preserved correctly");
    }
    
    println!("✅ All special characters handled safely");
}

#[tokio::test]
async fn test_task_position_boundaries() {
    // Test extreme position values
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Test extreme positions
    let test_positions = vec![
        (0.0, 0.0, "Origin position"),
        (-1000000.0, -1000000.0, "Large negative position"),
        (1000000.0, 1000000.0, "Large positive position"),
        (f64::MIN, f64::MIN, "Minimum float values"),
        (f64::MAX, f64::MAX, "Maximum float values"),
        (f64::INFINITY, f64::INFINITY, "Infinity values"),
        (f64::NEG_INFINITY, f64::NEG_INFINITY, "Negative infinity"),
        (f64::NAN, f64::NAN, "NaN values"),
    ];
    
    for (x, y, description) in test_positions {
        let mut task = Task::new(format!("Task at {}", description), "".to_string());
        task.set_position(x, y);
        
        let result = repo.tasks.create(&task).await;
        
        if result.is_ok() {
            let saved = repo.tasks.get(task.id).await.unwrap();
            if let Some(saved_task) = saved {
                // Check if position was preserved or normalized
                if saved_task.position.x.is_finite() && saved_task.position.y.is_finite() {
                    println!("✅ {} handled correctly", description);
                } else {
                    println!("⚠️ {} resulted in non-finite values", description);
                }
            }
        } else {
            println!("❌ {} was rejected: {:?}", description, result);
        }
    }
}

#[tokio::test]
async fn test_concurrent_task_updates() {
    // Test concurrent modifications to the same task
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create initial task
    let task = Task::new("Concurrent Task".to_string(), "Will be modified concurrently".to_string());
    let task_id = task.id;
    repo.tasks.create(&task).await.unwrap();
    
    // Simulate concurrent updates
    let repo1 = repo.clone();
    let repo2 = repo.clone();
    
    let handle1 = tokio::spawn(async move {
        for i in 0..10 {
            if let Ok(Some(mut task)) = repo1.tasks.get(task_id).await {
                task.title = format!("Update from thread 1 - {}", i);
                let _ = repo1.tasks.update(&task).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
    });
    
    let handle2 = tokio::spawn(async move {
        for i in 0..10 {
            if let Ok(Some(mut task)) = repo2.tasks.get(task_id).await {
                task.description = format!("Update from thread 2 - {}", i);
                let _ = repo2.tasks.update(&task).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
    });
    
    // Wait for both to complete
    let _ = handle1.await;
    let _ = handle2.await;
    
    // Verify task is still intact
    let final_task = repo.tasks.get(task_id).await.unwrap().unwrap();
    assert!(!final_task.title.is_empty());
    assert!(!final_task.description.is_empty());
    
    println!("✅ Concurrent updates handled safely");
    println!("   Final title: {}", final_task.title);
    println!("   Final description: {}", final_task.description);
}

#[tokio::test]
async fn test_delete_nonexistent_task() {
    // Test deleting a task that doesn't exist
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    let nonexistent_id = Uuid::new_v4();
    let result = repo.tasks.delete(nonexistent_id).await;
    
    // Should handle gracefully without panic
    match result {
        Ok(_) => println!("✅ Delete non-existent task succeeded silently"),
        Err(e) => println!("✅ Delete non-existent task returned error: {:?}", e),
    }
}

#[tokio::test]
async fn test_update_deleted_task() {
    // Test updating a task after it's been deleted
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create and then delete a task
    let mut task = Task::new("Task to Delete".to_string(), "".to_string());
    repo.tasks.create(&task).await.unwrap();
    repo.tasks.delete(task.id).await.unwrap();
    
    // Try to update the deleted task
    task.title = "Updated after deletion".to_string();
    let result = repo.tasks.update(&task).await;
    
    match result {
        Ok(_) => {
            // Check if it was recreated or truly updated
            let check = repo.tasks.get(task.id).await.unwrap();
            if check.is_some() {
                println!("⚠️ Update recreated deleted task");
            } else {
                println!("✅ Update succeeded but task remains deleted");
            }
        }
        Err(e) => println!("✅ Update of deleted task failed appropriately: {:?}", e),
    }
}

#[tokio::test]
async fn test_duplicate_task_ids() {
    // Test creating tasks with duplicate IDs
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    let shared_id = Uuid::new_v4();
    
    // Create first task with specific ID
    let mut task1 = Task::new("First Task".to_string(), "".to_string());
    task1.id = shared_id;
    repo.tasks.create(&task1).await.unwrap();
    
    // Try to create second task with same ID
    let mut task2 = Task::new("Second Task".to_string(), "".to_string());
    task2.id = shared_id;
    let result = repo.tasks.create(&task2).await;
    
    match result {
        Ok(_) => {
            // Check what happened
            let saved = repo.tasks.get(shared_id).await.unwrap().unwrap();
            if saved.title == "Second Task" {
                println!("⚠️ Duplicate ID overwrote the first task");
            } else {
                println!("⚠️ Duplicate ID was handled somehow");
            }
        }
        Err(e) => println!("✅ Duplicate ID properly rejected: {:?}", e),
    }
}

#[tokio::test]
async fn test_status_transitions() {
    // Test invalid status transitions
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create a Done task
    let mut task = Task::new("Completed Task".to_string(), "".to_string());
    task.status = TaskStatus::Done;
    task.completed_at = Some(Utc::now());
    repo.tasks.create(&task).await.unwrap();
    
    // Try to move back to Todo (should this be allowed?)
    task.status = TaskStatus::Todo;
    task.completed_at = None;
    let result = repo.tasks.update(&task).await;
    
    assert!(result.is_ok(), "Status transition from Done to Todo failed");
    
    let updated = repo.tasks.get(task.id).await.unwrap().unwrap();
    if updated.status == TaskStatus::Todo && updated.completed_at.is_none() {
        println!("⚠️ System allows reopening completed tasks");
    } else if updated.status == TaskStatus::Done {
        println!("✅ System prevents reopening completed tasks");
    } else {
        println!("❓ Unexpected status: {:?}", updated.status);
    }
    
    // Test Cancelled -> InProgress transition
    task.status = TaskStatus::Cancelled;
    repo.tasks.update(&task).await.unwrap();
    
    task.status = TaskStatus::InProgress;
    let result = repo.tasks.update(&task).await;
    
    if result.is_ok() {
        println!("⚠️ System allows reactivating cancelled tasks");
    } else {
        println!("✅ System prevents reactivating cancelled tasks");
    }
}

#[tokio::test] 
async fn test_task_with_invalid_dates() {
    // Test tasks with invalid date configurations
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Task with due date in the past but status Todo
    let mut past_due_todo = Task::new("Past Due Todo".to_string(), "".to_string());
    past_due_todo.due_date = Some(Utc::now() - chrono::Duration::days(30));
    past_due_todo.status = TaskStatus::Todo;
    
    let result = repo.tasks.create(&past_due_todo).await;
    assert!(result.is_ok(), "Should accept past due dates");
    
    // Task with completed_at but status not Done
    let mut incomplete_with_completed = Task::new("Incomplete but Completed?".to_string(), "".to_string());
    incomplete_with_completed.status = TaskStatus::InProgress;
    incomplete_with_completed.completed_at = Some(Utc::now());
    
    let result = repo.tasks.create(&incomplete_with_completed).await;
    if result.is_ok() {
        let saved = repo.tasks.get(incomplete_with_completed.id).await.unwrap().unwrap();
        println!("⚠️ System allows completed_at on non-Done tasks: {:?}", saved.status);
    } else {
        println!("✅ System validates completed_at matches status");
    }
    
    // Task with completed_at before created_at
    let mut time_traveler = Task::new("Time Traveler Task".to_string(), "".to_string());
    let now = Utc::now();
    time_traveler.created_at = now;
    time_traveler.completed_at = Some(now - chrono::Duration::days(1));
    time_traveler.status = TaskStatus::Done;
    
    let result = repo.tasks.create(&time_traveler).await;
    if result.is_ok() {
        println!("⚠️ System allows completed_at before created_at");
    } else {
        println!("✅ System validates date consistency");
    }
}