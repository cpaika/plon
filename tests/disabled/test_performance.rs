use plon::domain::task::{Task, TaskStatus, Priority};
use plon::repository::{Repository, database::init_database};
use plon::repository::task_repository::TaskFilters;
use tempfile::tempdir;
use chrono::Utc;
use std::time::Instant;

#[tokio::test]
async fn test_bulk_insert_performance() {
    // Test inserting a large number of tasks
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    let task_count = 1000;
    let start = Instant::now();
    
    for i in 0..task_count {
        let mut task = Task::new(
            format!("Task {}", i),
            format!("Description for task {} with some additional text to make it realistic", i)
        );
        task.status = match i % 5 {
            0 => TaskStatus::Todo,
            1 => TaskStatus::InProgress,
            2 => TaskStatus::Blocked,
            3 => TaskStatus::Review,
            _ => TaskStatus::Done,
        };
        task.priority = match i % 4 {
            0 => Priority::Critical,
            1 => Priority::High,
            2 => Priority::Medium,
            _ => Priority::Low,
        };
        task.set_position((i as f64) * 10.0, (i as f64) * 5.0);
        
        if i % 10 == 0 {
            task.tags.insert("important".to_string());
            task.tags.insert(format!("batch-{}", i / 100));
        }
        
        repo.tasks.create(&task).await.unwrap();
    }
    
    let insert_duration = start.elapsed();
    let per_task_ms = insert_duration.as_millis() as f64 / task_count as f64;
    
    println!("✅ Inserted {} tasks in {:.2}s", task_count, insert_duration.as_secs_f64());
    println!("   Average: {:.2}ms per task", per_task_ms);
    
    // Performance expectation: should be reasonably fast
    assert!(per_task_ms < 50.0, "Insert performance too slow: {:.2}ms per task", per_task_ms);
}

#[tokio::test]
async fn test_query_performance_with_large_dataset() {
    // Test query performance with many tasks
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Insert many tasks
    for i in 0..500 {
        let mut task = Task::new(format!("Task {}", i), format!("Description {}", i));
        task.status = if i % 2 == 0 { TaskStatus::Todo } else { TaskStatus::InProgress };
        task.set_position((i as f64) * 10.0, (i as f64) * 5.0);
        repo.tasks.create(&task).await.unwrap();
    }
    
    // Test unfiltered query
    let start = Instant::now();
    let all_tasks = repo.tasks.list(TaskFilters::default()).await.unwrap();
    let list_duration = start.elapsed();
    
    assert_eq!(all_tasks.len(), 500);
    println!("✅ Retrieved {} tasks in {:.2}ms", all_tasks.len(), list_duration.as_millis());
    assert!(list_duration.as_millis() < 500, "List all performance too slow");
    
    // Test filtered query
    let start = Instant::now();
    let mut filters = TaskFilters::default();
    filters.status = Some(TaskStatus::Todo);
    let filtered_tasks = repo.tasks.list(filters).await.unwrap();
    let filter_duration = start.elapsed();
    
    assert_eq!(filtered_tasks.len(), 250);
    println!("✅ Filtered {} tasks in {:.2}ms", filtered_tasks.len(), filter_duration.as_millis());
    assert!(filter_duration.as_millis() < 200, "Filter performance too slow");
    
    // Test limited query
    let start = Instant::now();
    let mut filters = TaskFilters::default();
    filters.limit = Some(10);
    let limited_tasks = repo.tasks.list(filters).await.unwrap();
    let limit_duration = start.elapsed();
    
    assert_eq!(limited_tasks.len(), 10);
    println!("✅ Limited query (10 tasks) in {:.2}ms", limit_duration.as_millis());
    assert!(limit_duration.as_millis() < 50, "Limited query performance too slow");
}

#[tokio::test]
async fn test_spatial_query_performance() {
    // Test spatial query performance
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create tasks in a grid pattern
    let grid_size = 50; // 50x50 = 2500 tasks
    for x in 0..grid_size {
        for y in 0..grid_size {
            let mut task = Task::new(
                format!("Task at {},{}", x, y),
                "".to_string()
            );
            task.set_position(x as f64 * 100.0, y as f64 * 100.0);
            repo.tasks.create(&task).await.unwrap();
        }
    }
    
    // Query a small area
    let start = Instant::now();
    let area_tasks = repo.tasks.find_in_area(0.0, 1000.0, 0.0, 1000.0).await.unwrap();
    let spatial_duration = start.elapsed();
    
    // Should find approximately 100 tasks (10x10 area)
    assert!(area_tasks.len() >= 100 && area_tasks.len() <= 121);
    println!("✅ Spatial query found {} tasks in {:.2}ms", area_tasks.len(), spatial_duration.as_millis());
    assert!(spatial_duration.as_millis() < 100, "Spatial query too slow");
    
    // Query a larger area
    let start = Instant::now();
    let large_area = repo.tasks.find_in_area(0.0, 2500.0, 0.0, 2500.0).await.unwrap();
    let large_spatial_duration = start.elapsed();
    
    assert!(large_area.len() >= 625); // At least 25x25
    println!("✅ Large spatial query found {} tasks in {:.2}ms", 
             large_area.len(), large_spatial_duration.as_millis());
    assert!(large_spatial_duration.as_millis() < 200, "Large spatial query too slow");
}

#[tokio::test]
async fn test_update_performance() {
    // Test update performance with many tasks
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create tasks
    let task_count = 100;
    let mut task_ids = Vec::new();
    for i in 0..task_count {
        let task = Task::new(format!("Task {}", i), "".to_string());
        task_ids.push(task.id);
        repo.tasks.create(&task).await.unwrap();
    }
    
    // Update all tasks
    let start = Instant::now();
    for id in &task_ids {
        let mut task = repo.tasks.get(*id).await.unwrap().unwrap();
        task.status = TaskStatus::InProgress;
        task.priority = Priority::High;
        task.description = "Updated description with more content".to_string();
        repo.tasks.update(&task).await.unwrap();
    }
    let update_duration = start.elapsed();
    let per_update_ms = update_duration.as_millis() as f64 / task_count as f64;
    
    println!("✅ Updated {} tasks in {:.2}s", task_count, update_duration.as_secs_f64());
    println!("   Average: {:.2}ms per update", per_update_ms);
    assert!(per_update_ms < 100.0, "Update performance too slow");
}

#[tokio::test]
async fn test_delete_performance() {
    // Test delete performance
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create tasks
    let task_count = 100;
    let mut task_ids = Vec::new();
    for i in 0..task_count {
        let task = Task::new(format!("Task {}", i), "".to_string());
        task_ids.push(task.id);
        repo.tasks.create(&task).await.unwrap();
    }
    
    // Delete all tasks
    let start = Instant::now();
    for id in &task_ids {
        repo.tasks.delete(*id).await.unwrap();
    }
    let delete_duration = start.elapsed();
    let per_delete_ms = delete_duration.as_millis() as f64 / task_count as f64;
    
    println!("✅ Deleted {} tasks in {:.2}s", task_count, delete_duration.as_secs_f64());
    println!("   Average: {:.2}ms per delete", per_delete_ms);
    assert!(per_delete_ms < 50.0, "Delete performance too slow");
    
    // Verify all deleted
    let remaining = repo.tasks.list(TaskFilters::default()).await.unwrap();
    assert_eq!(remaining.len(), 0);
}

#[tokio::test]
async fn test_concurrent_operations_performance() {
    // Test performance with concurrent operations
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).await.unwrap();
    let repo = Repository::new(pool);
    
    // Create initial tasks
    for i in 0..50 {
        let task = Task::new(format!("Task {}", i), "".to_string());
        repo.tasks.create(&task).await.unwrap();
    }
    
    let start = Instant::now();
    
    // Launch concurrent operations
    let repo1 = repo.clone();
    let repo2 = repo.clone();
    let repo3 = repo.clone();
    
    let create_handle = tokio::spawn(async move {
        for i in 50..100 {
            let task = Task::new(format!("Concurrent Task {}", i), "".to_string());
            repo1.tasks.create(&task).await.unwrap();
        }
    });
    
    let read_handle = tokio::spawn(async move {
        for _ in 0..20 {
            let _ = repo2.tasks.list(TaskFilters::default()).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });
    
    let update_handle = tokio::spawn(async move {
        let all_tasks = repo3.tasks.list(TaskFilters::default()).await.unwrap();
        for task in all_tasks.iter().take(20) {
            let mut updated = task.clone();
            updated.description = "Concurrently updated".to_string();
            let _ = repo3.tasks.update(&updated).await;
        }
    });
    
    // Wait for all operations
    create_handle.await.unwrap();
    read_handle.await.unwrap();
    update_handle.await.unwrap();
    
    let concurrent_duration = start.elapsed();
    
    println!("✅ Concurrent operations completed in {:.2}s", concurrent_duration.as_secs_f64());
    assert!(concurrent_duration.as_secs() < 5, "Concurrent operations too slow");
    
    // Verify data integrity
    let final_tasks = repo.tasks.list(TaskFilters::default()).await.unwrap();
    assert!(final_tasks.len() >= 100, "Some tasks were lost during concurrent operations");
}