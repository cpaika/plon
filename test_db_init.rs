use plon::repository::database;

#[tokio::main]
async fn main() {
    println!("Testing database initialization...");
    
    // Try to initialize the database
    match database::init_database("test_plon.db").await {
        Ok(pool) => {
            println!("✓ Database initialized successfully!");
            
            // Try to query tasks
            let result = sqlx::query("SELECT COUNT(*) as count FROM tasks")
                .fetch_one(&pool)
                .await;
                
            match result {
                Ok(row) => {
                    let count: i32 = sqlx::Row::get(&row, "count");
                    println!("✓ Found {} tasks in database", count);
                }
                Err(e) => println!("✗ Failed to query tasks: {}", e),
            }
            
            // Close pool
            pool.close().await;
        }
        Err(e) => println!("✗ Failed to initialize database: {}", e),
    }
    
    // Clean up test database
    std::fs::remove_file("test_plon.db").ok();
    println!("Cleaned up test database");
}