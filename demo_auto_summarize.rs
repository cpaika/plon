use plon::domain::task::Task;
use plon::domain::goal::Goal;
use plon::services::summarization::{SummarizationService, SummarizationLevel};

#[tokio::main]
async fn main() {
    println!("=== Auto-Summarize Feature Demo ===\n");
    
    // Create a summarization service
    let service = SummarizationService::new();
    
    // Create sample tasks
    let task1 = Task::new(
        "Implement OAuth2 Authentication".to_string(),
        "Integrate OAuth2 authentication with support for Google, GitHub, and Microsoft providers. Include proper token refresh, secure storage, and session management.".to_string()
    );
    
    let task2 = Task::new(
        "Database Performance Optimization".to_string(),
        "Analyze and optimize database queries, add appropriate indexes, implement query caching, and set up connection pooling for improved performance.".to_string()
    );
    
    // Demonstrate summarization at different zoom levels
    println!("Task: {}", task1.title);
    println!("Original description: {}\n", task1.description);
    
    let zoom_levels = vec![
        (0.2, "Very Zoomed Out", SummarizationLevel::HighLevel),
        (0.5, "Zoomed Out", SummarizationLevel::MidLevel),
        (1.0, "Normal View", SummarizationLevel::LowLevel),
        (2.5, "Zoomed In", SummarizationLevel::Detailed),
    ];
    
    for (zoom, label, level) in zoom_levels {
        println!("📍 Zoom Level: {} ({})", zoom, label);
        
        let summary = service.summarize(&task1.description, level).await;
        println!("   Summary: {}\n", summary);
    }
    
    // Demonstrate cluster summarization
    println!("\n=== Cluster Summarization ===");
    let cluster_tasks = vec![task1.clone(), task2.clone()];
    
    let cluster_summary = service.summarize_cluster(&cluster_tasks, SummarizationLevel::HighLevel).await;
    println!("Cluster of {} tasks:", cluster_tasks.len());
    println!("Summary: {}\n", cluster_summary);
    
    // Demonstrate goal summarization
    println!("=== Goal Summarization ===");
    let mut goal = Goal::new(
        "Q1 Technical Improvements".to_string(),
        "Improve system security and performance through authentication upgrades and database optimization".to_string()
    );
    
    let goal_summary = service.summarize_goal(&goal, &cluster_tasks, SummarizationLevel::MidLevel).await;
    println!("Goal: {}", goal.title);
    println!("Summary: {}", goal_summary);
    
    println!("\n✨ Auto-summarize feature demonstration complete!");
    println!("This feature dynamically adjusts content detail based on zoom level,");
    println!("providing a magical, fast experience for navigating complex project maps.");
}