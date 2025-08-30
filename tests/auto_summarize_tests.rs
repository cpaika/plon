use eframe::egui::{self};
use plon::domain::goal::Goal;
use plon::domain::task::{SubTask, Task};
use plon::services::summarization::{SummarizationLevel, SummarizationService, SummaryCache};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

#[tokio::test]
async fn test_zoom_level_to_abstraction_mapping() {
    // Test zoom level to summarization level mappings
    let test_cases = vec![
        (0.1, SummarizationLevel::HighLevel),  // Very zoomed out
        (0.25, SummarizationLevel::HighLevel), // Zoomed out
        (0.4, SummarizationLevel::MidLevel),   // Mid-high
        (0.7, SummarizationLevel::MidLevel),   // Mid
        (1.0, SummarizationLevel::LowLevel),   // Normal
        (1.5, SummarizationLevel::LowLevel),   // Slightly zoomed in
        (2.5, SummarizationLevel::Detailed),   // Zoomed in
        (4.0, SummarizationLevel::Detailed),   // Very zoomed in
    ];

    for (zoom, expected_level) in test_cases {
        // Match the actual implementation in MapView
        let detail_level = match zoom {
            z if z < 0.3 => DetailLevel::Overview,
            z if z < 0.6 => DetailLevel::Summary,
            z if z < 1.5 => DetailLevel::Standard,
            _ => DetailLevel::Detailed,
        };

        let level = match detail_level {
            DetailLevel::Overview => SummarizationLevel::HighLevel,
            DetailLevel::Summary => SummarizationLevel::MidLevel,
            DetailLevel::Standard => SummarizationLevel::LowLevel,
            DetailLevel::Detailed => SummarizationLevel::Detailed,
        };

        assert_eq!(
            level, expected_level,
            "Zoom {} should map to {:?}",
            zoom, expected_level
        );
    }
}

#[derive(Debug, PartialEq)]
enum DetailLevel {
    Overview,
    Summary,
    Standard,
    Detailed,
}

#[tokio::test]
async fn test_dynamic_summarization_on_zoom() {
    let summarization_service = SummarizationService::new();
    let mut cache = SummaryCache::new(100);

    // Create sample tasks
    let tasks = vec![
        create_task_with_details(
            "Implement user authentication",
            "Security",
            vec![
                "Set up JWT tokens",
                "Create login endpoint",
                "Add password hashing",
                "Implement session management",
            ],
        ),
        create_task_with_details(
            "Design database schema",
            "Architecture",
            vec![
                "Define user table",
                "Create relations",
                "Add indexes",
                "Set up migrations",
            ],
        ),
        create_task_with_details(
            "Build UI components",
            "Frontend",
            vec![
                "Create button component",
                "Design form layouts",
                "Add responsive styles",
                "Implement animations",
            ],
        ),
    ];

    // Test at different zoom levels
    let zoom_levels = vec![0.2, 0.5, 1.0, 2.0, 3.0];

    for zoom in zoom_levels {
        let level = match zoom {
            z if z < 0.3 => SummarizationLevel::HighLevel,
            z if z < 0.6 => SummarizationLevel::MidLevel,
            z if z < 1.5 => SummarizationLevel::LowLevel,
            _ => SummarizationLevel::Detailed,
        };

        let mut summaries = Vec::new();
        for task in &tasks {
            let summary = summarization_service
                .summarize_with_cache(
                    &mut cache,
                    task.id,
                    &format!("{}: {}", task.title, task.description),
                    level,
                )
                .await;
            summaries.push(summary);
        }

        // Verify summary length varies with zoom
        if zoom < 0.5 {
            // High-level summary should be shorter
            for summary in &summaries {
                assert!(summary.len() < 200, "High-level summary should be concise");
            }
        } else if zoom > 2.0 {
            // Detailed view should show full information
            for (i, summary) in summaries.iter().enumerate() {
                assert!(summary.len() > 10, "Detailed view should have content");
            }
        }
    }
}

#[tokio::test]
async fn test_summarization_caching() {
    let mut cache = SummaryCache::new(100); // 100 item cache
    let service = SummarizationService::new();

    let task_id = Uuid::new_v4();
    let content = "Long task description with many details...";
    let level = SummarizationLevel::MidLevel;

    // First request - should hit the service
    let start = std::time::Instant::now();
    let summary1 = service
        .summarize_with_cache(&mut cache, task_id, content, level)
        .await;
    let first_duration = start.elapsed();

    // Second request - should hit cache
    let start = std::time::Instant::now();
    let summary2 = service
        .summarize_with_cache(&mut cache, task_id, content, level)
        .await;
    let cached_duration = start.elapsed();

    // Cache should be faster
    assert!(
        cached_duration < first_duration / 10,
        "Cached request should be much faster"
    );
    assert_eq!(summary1, summary2, "Cached summary should be identical");
}

#[tokio::test]
async fn test_cluster_summarization() {
    let service = SummarizationService::new();

    // Create a cluster of related tasks
    let cluster_tasks = vec![
        create_task("Fix login bug", "Bug in authentication flow"),
        create_task("Update password validation", "Strengthen password rules"),
        create_task("Add 2FA support", "Two-factor authentication"),
        create_task("Refresh token implementation", "Auto-refresh JWT tokens"),
    ];

    // Get cluster summary at high abstraction
    let cluster_summary = service
        .summarize_cluster(&cluster_tasks, SummarizationLevel::HighLevel)
        .await;

    // Verify it creates a coherent summary
    assert!(
        cluster_summary.len() > 0,
        "Cluster summary should not be empty"
    );
    assert!(
        cluster_summary.len() < 300,
        "High-level cluster summary should be reasonably concise"
    );

    // Should identify the common theme
    let summary_lower = cluster_summary.to_lowercase();
    assert!(
        summary_lower.contains("auth") || summary_lower.contains("security"),
        "Should identify authentication/security theme"
    );
}

#[tokio::test]
async fn test_smooth_zoom_transitions() {
    let service = SummarizationService::new();
    let mut cache = SummaryCache::new(100);

    // Create sample tasks
    let tasks = create_sample_tasks(10);

    // Simulate smooth zoom from 0.5 to 2.0
    let zoom_steps = vec![0.5, 0.7, 0.9, 1.1, 1.3, 1.5, 1.7, 1.9, 2.0];
    let mut previous_summaries: Vec<String> = Vec::new();

    for zoom in zoom_steps {
        let level = match zoom {
            z if z < 0.3 => SummarizationLevel::HighLevel,
            z if z < 0.6 => SummarizationLevel::MidLevel,
            z if z < 1.5 => SummarizationLevel::LowLevel,
            _ => SummarizationLevel::Detailed,
        };

        let mut summaries = Vec::new();
        for task in &tasks {
            let summary = service
                .summarize_with_cache(
                    &mut cache,
                    task.id,
                    &format!("{}: {}", task.title, task.description),
                    level,
                )
                .await;
            summaries.push(summary);
        }

        if !previous_summaries.is_empty() {
            // Verify smooth transition - summaries should change gradually
            let change_ratio = calculate_summary_change(&previous_summaries, &summaries);
            assert!(
                change_ratio < 0.8,
                "Summary changes should be gradual during smooth zoom"
            );
        }

        previous_summaries = summaries;
    }
}

#[tokio::test]
async fn test_performance_with_many_tasks() {
    let service = SummarizationService::new();
    let mut cache = SummaryCache::new(500);

    // Create many tasks
    let tasks = create_sample_tasks(100);

    // Measure summarization time
    let start = std::time::Instant::now();

    for task in &tasks {
        service
            .summarize_with_cache(
                &mut cache,
                task.id,
                &task.description,
                SummarizationLevel::MidLevel,
            )
            .await;
    }

    let duration = start.elapsed();

    // Should complete within reasonable time (adjust threshold as needed)
    assert!(
        duration < Duration::from_secs(2),
        "Summarizing 100 tasks took {:?}, should be under 2s",
        duration
    );
}

#[tokio::test]
async fn test_mouse_wheel_zoom_summarization() {
    let service = SummarizationService::new();
    let mut cache = SummaryCache::new(100);
    let tasks = create_sample_tasks(5);

    // Simulate mouse wheel events affecting zoom
    let mut zoom = 1.0f32;
    let scroll_deltas = vec![
        120.0,  // Zoom in
        120.0,  // Zoom in more
        -120.0, // Zoom out
        -240.0, // Zoom out more
    ];

    for delta in scroll_deltas {
        // Update zoom based on scroll delta
        let zoom_factor = 1.0 + delta * 0.001;
        zoom = (zoom * zoom_factor).clamp(0.1, 5.0);

        // Determine summarization level from zoom
        let level = match zoom {
            z if z < 0.3 => SummarizationLevel::HighLevel,
            z if z < 0.6 => SummarizationLevel::MidLevel,
            z if z < 1.5 => SummarizationLevel::LowLevel,
            _ => SummarizationLevel::Detailed,
        };

        // Generate summaries
        for task in &tasks {
            let summary = service
                .summarize_with_cache(
                    &mut cache,
                    task.id,
                    &format!("{}: {}", task.title, task.description),
                    level,
                )
                .await;

            // Each summary should match the current level
            verify_summary_matches_level(&summary, level);
        }
    }
}

#[tokio::test]
async fn test_button_zoom_summarization() {
    let service = SummarizationService::new();
    let mut cache = SummaryCache::new(100);
    let tasks = create_sample_tasks(3);

    let mut zoom = 1.0f32;

    // Test zoom in button
    for _ in 0..5 {
        zoom = (zoom * 1.2).min(5.0);

        let level = match zoom {
            z if z < 0.3 => SummarizationLevel::HighLevel,
            z if z < 0.6 => SummarizationLevel::MidLevel,
            z if z < 1.5 => SummarizationLevel::LowLevel,
            _ => SummarizationLevel::Detailed,
        };

        let mut summaries = Vec::new();
        for task in &tasks {
            let summary = service
                .summarize_with_cache(
                    &mut cache,
                    task.id,
                    &format!("{}: {}", task.title, task.description),
                    level,
                )
                .await;
            summaries.push(summary);
        }

        // As we zoom in, summaries should become more detailed
        let avg_length: usize =
            summaries.iter().map(|s| s.len()).sum::<usize>() / summaries.len().max(1);
        assert!(avg_length > 0, "Summaries should have content");
    }

    // Test zoom out button
    for _ in 0..5 {
        zoom = (zoom / 1.2).max(0.1);

        let level = match zoom {
            z if z < 0.3 => SummarizationLevel::HighLevel,
            z if z < 0.6 => SummarizationLevel::MidLevel,
            z if z < 1.5 => SummarizationLevel::LowLevel,
            _ => SummarizationLevel::Detailed,
        };

        let mut summaries = Vec::new();
        for task in &tasks {
            let summary = service
                .summarize_with_cache(
                    &mut cache,
                    task.id,
                    &format!("{}: {}", task.title, task.description),
                    level,
                )
                .await;
            summaries.push(summary);
        }

        // As we zoom out, summaries should become more concise
        let avg_length: usize =
            summaries.iter().map(|s| s.len()).sum::<usize>() / summaries.len().max(1);
        assert!(avg_length < 500, "Zoomed out summaries should be concise");
    }
}

#[tokio::test]
async fn test_goal_summarization_at_different_levels() {
    let service = SummarizationService::new();

    let goal = Goal::new(
        "Q1 Product Launch".to_string(),
        "Launch new product features for Q1 including user dashboard, analytics, and reporting"
            .to_string(),
    );

    let associated_tasks = vec![
        create_task("Design dashboard UI", "Create mockups and wireframes"),
        create_task("Implement analytics backend", "Set up data pipeline"),
        create_task("Build reporting module", "Generate PDF reports"),
        create_task("User testing", "Conduct beta testing"),
    ];

    // Test at different summarization levels
    let levels = vec![
        SummarizationLevel::HighLevel,
        SummarizationLevel::MidLevel,
        SummarizationLevel::LowLevel,
        SummarizationLevel::Detailed,
    ];

    for level in levels {
        let summary = service
            .summarize_goal(&goal, &associated_tasks, level)
            .await;

        match level {
            SummarizationLevel::HighLevel => {
                assert!(
                    summary.len() < 200,
                    "High-level goal summary should be concise"
                );
            }
            SummarizationLevel::Detailed => {
                assert!(
                    summary.len() > 50,
                    "Detailed summary should be comprehensive"
                );
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_real_time_summarization_updates() {
    let service = SummarizationService::new();
    let mut cache = SummaryCache::new(100);

    // Start with initial tasks
    let mut tasks = create_sample_tasks(3);

    // Get initial summaries at mid-high level
    let level = SummarizationLevel::MidLevel;
    let mut initial_summaries = Vec::new();
    for task in &tasks {
        let summary = service
            .summarize_with_cache(
                &mut cache,
                task.id,
                &format!("{}: {}", task.title, task.description),
                level,
            )
            .await;
        initial_summaries.push(summary);
    }

    // Modify a task
    tasks[0].title = "Updated Task Title".to_string();
    tasks[0].description = "Completely new description with different content".to_string();

    // Clear cache for the modified task to force re-summarization
    cache.clear();

    // Get updated summaries
    let mut updated_summaries = Vec::new();
    for task in &tasks {
        let summary = service
            .summarize_with_cache(
                &mut cache,
                task.id,
                &format!("{}: {}", task.title, task.description),
                level,
            )
            .await;
        updated_summaries.push(summary);
    }

    // Verify the summary for the modified task changed
    assert_ne!(
        initial_summaries[0], updated_summaries[0],
        "Summary should update when task content changes"
    );
}

#[tokio::test]
async fn test_viewport_based_summarization() {
    let service = SummarizationService::new();
    let mut cache = SummaryCache::new(100);

    // Create tasks spread across the map
    let mut tasks = Vec::new();
    for i in 0..20 {
        let mut task = create_task(&format!("Task {}", i), "Description");
        task.set_position((i as f64) * 200.0, (i as f64) * 150.0);
        tasks.push(task);
    }

    // Set viewport to show only some tasks
    let viewport =
        egui::Rect::from_min_size(egui::Pos2::new(0.0, 0.0), egui::Vec2::new(600.0, 400.0));

    // Get summaries only for visible tasks
    let mut visible_summaries = Vec::new();
    for task in &tasks {
        let pos = egui::Pos2::new(task.position.x as f32, task.position.y as f32);
        if viewport.contains(pos) {
            let summary = service
                .summarize_with_cache(
                    &mut cache,
                    task.id,
                    &format!("{}: {}", task.title, task.description),
                    SummarizationLevel::MidLevel,
                )
                .await;
            visible_summaries.push(summary);
        }
    }

    // Should have fewer summaries than total tasks
    assert!(
        visible_summaries.len() < tasks.len(),
        "Should only summarize visible tasks"
    );
}

#[tokio::test]
async fn test_language_model_integration() {
    let service = SummarizationService::new();

    // Test with real-world task description
    let complex_description = r#"
        This task involves refactoring the authentication module to support 
        OAuth 2.0, SAML, and traditional username/password authentication.
        We need to implement a strategy pattern for different auth providers,
        update the database schema to support multiple auth methods per user,
        create migration scripts, update the API endpoints, and ensure
        backward compatibility with existing sessions. Additionally, we need
        to add comprehensive logging, error handling, and security measures
        including rate limiting and CSRF protection.
    "#;

    // Test different summarization levels
    let high_summary = service
        .summarize(complex_description, SummarizationLevel::HighLevel)
        .await;
    let mid_summary = service
        .summarize(complex_description, SummarizationLevel::MidLevel)
        .await;
    let low_summary = service
        .summarize(complex_description, SummarizationLevel::LowLevel)
        .await;

    // Verify summaries are progressively more detailed
    assert!(
        high_summary.len() < mid_summary.len(),
        "High-level should be shortest"
    );
    assert!(
        mid_summary.len() < low_summary.len(),
        "Mid-level should be shorter than low-level"
    );

    // Verify key concepts are preserved
    assert!(
        high_summary.to_lowercase().contains("auth"),
        "Should mention authentication"
    );
}

#[tokio::test]
async fn test_concurrent_summarization_requests() {
    let service = SummarizationService::new();
    let tasks = create_sample_tasks(10);

    // Launch multiple concurrent summarization requests
    let summarization_futures: Vec<_> = tasks
        .iter()
        .map(|task| {
            let svc = service.clone();
            let desc = task.description.clone();
            async move { svc.summarize(&desc, SummarizationLevel::MidLevel).await }
        })
        .collect();

    // Wait for all with timeout
    let results = timeout(
        Duration::from_secs(5),
        futures::future::join_all(summarization_futures),
    )
    .await;

    assert!(
        results.is_ok(),
        "Concurrent requests should complete within timeout"
    );

    let summaries = results.unwrap();
    assert_eq!(
        summaries.len(),
        tasks.len(),
        "Should get summary for each task"
    );
}

// Helper functions

fn create_task(title: &str, description: &str) -> Task {
    let mut task = Task::new(title.to_string(), description.to_string());
    task.id = Uuid::new_v4();
    task
}

fn create_task_with_details(title: &str, category: &str, subtasks: Vec<&str>) -> Task {
    let mut task = create_task(title, &format!("{} implementation", category));
    task.metadata
        .insert("category".to_string(), category.to_string());

    for subtask_title in subtasks {
        let subtask = SubTask {
            id: Uuid::new_v4(),
            title: subtask_title.to_string(),
            description: subtask_title.to_string(),
            completed: false,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };
        task.subtasks.push(subtask);
    }

    task
}

fn create_sample_tasks(count: usize) -> Vec<Task> {
    (0..count)
        .map(|i| {
            create_task(
                &format!("Task {}", i),
                &format!("Description for task {}", i),
            )
        })
        .collect()
}

fn calculate_summary_change(prev: &[String], current: &[String]) -> f32 {
    if prev.len() != current.len() {
        return 1.0;
    }

    let mut total_change = 0.0;
    for (p, c) in prev.iter().zip(current.iter()) {
        let change = (p.len() as f32 - c.len() as f32).abs() / p.len().max(1) as f32;
        total_change += change;
    }

    total_change / prev.len() as f32
}

fn verify_summary_matches_level(summary: &str, level: SummarizationLevel) {
    match level {
        SummarizationLevel::HighLevel => {
            assert!(
                summary.len() < 60,
                "High-level summary should be under 60 chars"
            );
        }
        SummarizationLevel::MidLevel => {
            assert!(
                summary.len() < 150,
                "Mid-level summary should be under 150 chars"
            );
        }
        SummarizationLevel::LowLevel => {
            assert!(
                summary.len() < 300,
                "Low-level summary should be under 300 chars"
            );
        }
        SummarizationLevel::Detailed => {
            // Detailed can be any length
        }
    }
}
