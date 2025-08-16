use plon::repository::database::init_test_database;
use plon::repository::Repository;
use plon::services::RecurringService;
use plon::domain::recurring::{RecurringTaskTemplate, RecurrenceRule, RecurrencePattern};
use plon::domain::task::{Task, TaskStatus, Priority};
use chrono::{NaiveTime, Weekday, Utc, Duration};
use std::sync::Arc;
use uuid::Uuid;
use sqlx;

#[tokio::test]
async fn test_create_recurring_template_through_service() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    let template = service.create_recurring_template(
        "Weekly Review".to_string(),
        "Review weekly progress".to_string(),
        RecurrencePattern::Weekly,
        1,
        Some(vec![Weekday::Fri]),
        None,
        None,
        NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    assert_eq!(template.title, "Weekly Review");
    assert_eq!(template.recurrence_rule.pattern, RecurrencePattern::Weekly);
    assert_eq!(template.recurrence_rule.days_of_week, vec![Weekday::Fri]);
    assert!(template.active);
    assert!(template.next_occurrence.is_some());
}

#[tokio::test]
async fn test_generate_tasks_from_templates() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    // Create a daily template that should generate immediately
    let template = service.create_recurring_template(
        "Daily Standup".to_string(),
        "Team standup meeting".to_string(),
        RecurrencePattern::Daily,
        1,
        None,
        None,
        None,
        NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    // Generate tasks from all active templates
    let generated_tasks = service.generate_due_tasks().await.unwrap();
    
    assert_eq!(generated_tasks.len(), 1);
    assert_eq!(generated_tasks[0].title, "Daily Standup");
    assert_eq!(generated_tasks[0].description, "Team standup meeting");
    assert_eq!(generated_tasks[0].status, TaskStatus::Todo);
    
    // Verify the template was updated
    let updated_template = service.get_template(template.id).await.unwrap().unwrap();
    assert_eq!(updated_template.recurrence_rule.occurrences_count, 1);
    assert!(updated_template.last_generated.is_some());
}

#[tokio::test]
async fn test_update_template() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    let mut template = service.create_recurring_template(
        "Monthly Report".to_string(),
        "Submit monthly report".to_string(),
        RecurrencePattern::Monthly,
        1,
        None,
        Some(15),
        None,
        NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    // Update the template
    template.title = "Updated Monthly Report".to_string();
    template.priority = Priority::High;
    template.estimated_hours = Some(3.0);
    
    service.update_template(&template).await.unwrap();
    
    let fetched = service.get_template(template.id).await.unwrap().unwrap();
    assert_eq!(fetched.title, "Updated Monthly Report");
    assert_eq!(fetched.priority, Priority::High);
    assert_eq!(fetched.estimated_hours, Some(3.0));
}

#[tokio::test]
async fn test_deactivate_and_reactivate_template() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    let template = service.create_recurring_template(
        "Weekly Meeting".to_string(),
        "Team sync".to_string(),
        RecurrencePattern::Weekly,
        1,
        Some(vec![Weekday::Mon]),
        None,
        None,
        NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    // Deactivate the template
    service.deactivate_template(template.id).await.unwrap();
    
    let deactivated = service.get_template(template.id).await.unwrap().unwrap();
    assert!(!deactivated.active);
    
    // Verify it doesn't generate tasks when inactive
    let generated = service.generate_due_tasks().await.unwrap();
    assert_eq!(generated.len(), 0);
    
    // Reactivate the template
    service.reactivate_template(template.id).await.unwrap();
    
    let reactivated = service.get_template(template.id).await.unwrap().unwrap();
    assert!(reactivated.active);
    assert!(reactivated.next_occurrence.is_some());
}

#[tokio::test]
async fn test_delete_template() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    let template = service.create_recurring_template(
        "Temporary Task".to_string(),
        "To be deleted".to_string(),
        RecurrencePattern::Daily,
        1,
        None,
        None,
        None,
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    // Delete the template
    let deleted = service.delete_template(template.id).await.unwrap();
    assert!(deleted);
    
    // Verify it's gone
    let fetched = service.get_template(template.id).await.unwrap();
    assert!(fetched.is_none());
}

#[tokio::test]
async fn test_list_active_templates() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    // Create multiple templates
    let template1 = service.create_recurring_template(
        "Daily Task".to_string(),
        "Every day".to_string(),
        RecurrencePattern::Daily,
        1,
        None,
        None,
        None,
        NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    let template2 = service.create_recurring_template(
        "Weekly Task".to_string(),
        "Every week".to_string(),
        RecurrencePattern::Weekly,
        1,
        Some(vec![Weekday::Wed]),
        None,
        None,
        NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    let template3 = service.create_recurring_template(
        "Monthly Task".to_string(),
        "Every month".to_string(),
        RecurrencePattern::Monthly,
        1,
        None,
        Some(1),
        None,
        NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    // Deactivate one template
    service.deactivate_template(template2.id).await.unwrap();
    
    // List active templates
    let active = service.list_active_templates().await.unwrap();
    assert_eq!(active.len(), 2);
    
    let titles: Vec<String> = active.iter().map(|t| t.title.clone()).collect();
    assert!(titles.contains(&"Daily Task".to_string()));
    assert!(titles.contains(&"Monthly Task".to_string()));
    assert!(!titles.contains(&"Weekly Task".to_string()));
}

#[tokio::test]
async fn test_max_occurrences_limit() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    // Create template with max 3 occurrences
    let template = service.create_recurring_template(
        "Limited Task".to_string(),
        "Only 3 times".to_string(),
        RecurrencePattern::Daily,
        1,
        None,
        None,
        None,
        NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
        None,
        Some(3),
    ).await.unwrap();
    
    // Generate first task
    let generated = service.generate_due_tasks().await.unwrap();
    assert_eq!(generated.len(), 1, "Failed on first generation");
    
    // Check occurrence count after first generation
    let template_check = service.get_template(template.id).await.unwrap().unwrap();
    assert_eq!(template_check.recurrence_rule.occurrences_count, 1);
    
    // Force template to be due again by directly updating the database
    sqlx::query("UPDATE recurring_templates SET next_occurrence = ? WHERE id = ?")
        .bind((chrono::Utc::now() - chrono::Duration::seconds(1)).to_rfc3339())
        .bind(template.id.to_string())
        .execute(&*repository.pool)
        .await.unwrap();
    
    // Generate second task  
    let generated = service.generate_due_tasks().await.unwrap();
    assert_eq!(generated.len(), 1, "Failed on second generation");
    
    // Check occurrence count after second generation
    let template_check = service.get_template(template.id).await.unwrap().unwrap();
    assert_eq!(template_check.recurrence_rule.occurrences_count, 2);
    
    // Force template to be due again
    sqlx::query("UPDATE recurring_templates SET next_occurrence = ? WHERE id = ?")
        .bind((chrono::Utc::now() - chrono::Duration::seconds(1)).to_rfc3339())
        .bind(template.id.to_string())
        .execute(&*repository.pool)
        .await.unwrap();
    
    // Generate third task
    let generated = service.generate_due_tasks().await.unwrap();
    assert_eq!(generated.len(), 1, "Failed on third generation");
    
    // Force one more check by setting next_occurrence to past - this should trigger deactivation
    sqlx::query("UPDATE recurring_templates SET next_occurrence = ? WHERE id = ?")
        .bind((chrono::Utc::now() - chrono::Duration::seconds(1)).to_rfc3339())
        .bind(template.id.to_string())
        .execute(&*repository.pool)
        .await.unwrap();
    
    // Fourth attempt should generate nothing (max occurrences reached) and deactivate the template
    let generated = service.generate_due_tasks().await.unwrap();
    assert_eq!(generated.len(), 0);
    
    // Template should be inactive
    let final_template = service.get_template(template.id).await.unwrap().unwrap();
    println!("Final template occurrences: {}, active: {}, max: {:?}", 
             final_template.recurrence_rule.occurrences_count, 
             final_template.active,
             final_template.recurrence_rule.max_occurrences);
    assert_eq!(final_template.recurrence_rule.occurrences_count, 3, "Expected 3 occurrences");
    assert!(!final_template.active, "Template should be inactive after reaching max occurrences");
}

#[tokio::test]
async fn test_end_date_limit() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    // Create template with end date in the past
    let past_date = (Utc::now() - Duration::days(1)).date_naive();
    
    let template = service.create_recurring_template(
        "Expired Task".to_string(),
        "Should not generate".to_string(),
        RecurrencePattern::Daily,
        1,
        None,
        None,
        None,
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        Some(past_date),
        None,
    ).await.unwrap();
    
    // Try to generate tasks - should get none
    let generated = service.generate_due_tasks().await.unwrap();
    assert_eq!(generated.len(), 0);
    
    // Template should be inactive
    let final_template = service.get_template(template.id).await.unwrap().unwrap();
    assert!(!final_template.active);
}

#[tokio::test]
async fn test_complex_weekly_recurrence() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    // Create bi-weekly template for Mon, Wed, Fri
    let template = service.create_recurring_template(
        "Team Sync".to_string(),
        "Bi-weekly sync".to_string(),
        RecurrencePattern::Weekly,
        2, // Every 2 weeks
        Some(vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]),
        None,
        None,
        NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    assert_eq!(template.recurrence_rule.interval, 2);
    assert_eq!(template.recurrence_rule.days_of_week.len(), 3);
    assert!(template.next_occurrence.is_some());
}

#[tokio::test]
async fn test_yearly_recurrence() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    // Create yearly template for December 25th
    let template = service.create_recurring_template(
        "Annual Celebration".to_string(),
        "Yearly event".to_string(),
        RecurrencePattern::Yearly,
        1,
        None,
        Some(25),
        Some(12),
        NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    assert_eq!(template.recurrence_rule.pattern, RecurrencePattern::Yearly);
    assert_eq!(template.recurrence_rule.day_of_month, Some(25));
    assert_eq!(template.recurrence_rule.month_of_year, Some(12));
}

#[tokio::test]
async fn test_template_with_metadata() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    let mut template = service.create_recurring_template(
        "Sprint Planning".to_string(),
        "Plan next sprint".to_string(),
        RecurrencePattern::Weekly,
        2,
        Some(vec![Weekday::Mon]),
        None,
        None,
        NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        None,
        None,
    ).await.unwrap();
    
    // Add metadata
    template.metadata.insert("project".to_string(), "backend".to_string());
    template.metadata.insert("sprint_length".to_string(), "2_weeks".to_string());
    template.estimated_hours = Some(2.0);
    
    service.update_template(&template).await.unwrap();
    
    // Generate a task and verify metadata is copied
    let generated = service.generate_due_tasks().await.unwrap();
    assert_eq!(generated.len(), 1);
    
    let task = &generated[0];
    assert_eq!(task.metadata.get("project"), Some(&"backend".to_string()));
    assert_eq!(task.metadata.get("sprint_length"), Some(&"2_weeks".to_string()));
    assert_eq!(task.estimated_hours, Some(2.0));
}

#[tokio::test]
async fn test_concurrent_task_generation() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());
    
    // Create multiple templates that should all generate tasks
    let templates = vec![
        ("Daily 1", RecurrencePattern::Daily),
        ("Daily 2", RecurrencePattern::Daily),
        ("Daily 3", RecurrencePattern::Daily),
    ];
    
    for (title, pattern) in templates {
        service.create_recurring_template(
            title.to_string(),
            format!("Description for {}", title),
            pattern,
            1,
            None,
            None,
            None,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            None,
            None,
        ).await.unwrap();
    }
    
    // Generate all tasks at once
    let generated = service.generate_due_tasks().await.unwrap();
    assert_eq!(generated.len(), 3);
    
    // Verify all tasks were created
    let titles: Vec<String> = generated.iter().map(|t| t.title.clone()).collect();
    assert!(titles.contains(&"Daily 1".to_string()));
    assert!(titles.contains(&"Daily 2".to_string()));
    assert!(titles.contains(&"Daily 3".to_string()));
}