use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
#[ignore] // Run with cargo test -- --ignored
fn test_app_starts_without_crash() {
    // Start the application
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "plon-desktop"])
        .spawn()
        .expect("Failed to start application");
    
    // Wait for app to initialize
    thread::sleep(Duration::from_secs(3));
    
    // Check if process is still running
    match child.try_wait() {
        Ok(None) => {
            // Process is still running - good!
            println!("✅ Application started successfully");
            
            // Kill it after test
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(Some(status)) => {
            panic!("❌ Application crashed with status: {:?}", status);
        }
        Err(e) => {
            panic!("❌ Error checking application status: {}", e);
        }
    }
}

#[test]
fn test_database_migrations_run() {
    // This test verifies that database migrations run successfully
    use plon::repository::database::init_test_database;
    
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let pool = init_test_database().await.expect("Failed to initialize test database");
        
        // Check that tables exist
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(&pool)
            .await
            .expect("Failed to query tables");
        
        assert!(result.len() > 0, "No tables found in database");
        
        // Check for specific tables
        let table_names: Vec<String> = result.iter()
            .filter_map(|row| {
                sqlx::Row::try_get::<String, _>(row, 0).ok()
            })
            .collect();
        
        assert!(table_names.contains(&"tasks".to_string()), "tasks table not found");
        assert!(table_names.contains(&"dependencies".to_string()), "dependencies table not found");
        
        println!("✅ Database migrations successful");
        println!("Found tables: {:?}", table_names);
    });
}