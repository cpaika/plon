use plon::repository::database::init_test_database;
use plon::repository::recurring_repository::RecurringRepository;
use plon::domain::recurring::{RecurringTaskTemplate, RecurrenceRule, RecurrencePattern};
use plon::domain::task::Priority;
use chrono::{NaiveTime, Weekday, Utc};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_create_recurring_template() {
    let pool = Arc::new(init_test_database().await.unwrap());
    let repo = RecurringRepository::new(pool);
    
    let rule = RecurrenceRule {
        pattern: RecurrencePattern::Daily,
        interval: 1,
        days_of_week: vec![],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: None,
        occurrences_count: 0,
    };
    
    let template = RecurringTaskTemplate::new(
        "Daily Standup".to_string(),
        "Morning team standup".to_string(),
        rule,
    );
    
    let result = repo.create(&template).await;
    assert!(result.is_ok());
    
    // Verify the template was created
    let fetched = repo.get(template.id).await.unwrap();
    assert!(fetched.is_some());
    
    let fetched_template = fetched.unwrap();
    assert_eq!(fetched_template.title, "Daily Standup");
    assert_eq!(fetched_template.description, "Morning team standup");
    assert_eq!(fetched_template.recurrence_rule.pattern, RecurrencePattern::Daily);
    assert_eq!(fetched_template.active, true);
}

#[tokio::test]
async fn test_update_recurring_template() {
    let pool = Arc::new(init_test_database().await.unwrap());
    let repo = RecurringRepository::new(pool);
    
    let rule = RecurrenceRule {
        pattern: RecurrencePattern::Weekly,
        interval: 1,
        days_of_week: vec![Weekday::Mon, Weekday::Wed, Weekday::Fri],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: None,
        occurrences_count: 0,
    };
    
    let mut template = RecurringTaskTemplate::new(
        "Team Meeting".to_string(),
        "Weekly team sync".to_string(),
        rule,
    );
    
    repo.create(&template).await.unwrap();
    
    // Update the template
    template.title = "Updated Team Meeting".to_string();
    template.priority = Priority::High;
    template.estimated_hours = Some(1.5);
    
    let result = repo.update(&template).await;
    assert!(result.is_ok());
    
    // Verify the update
    let fetched = repo.get(template.id).await.unwrap().unwrap();
    assert_eq!(fetched.title, "Updated Team Meeting");
    assert_eq!(fetched.priority, Priority::High);
    assert_eq!(fetched.estimated_hours, Some(1.5));
}

#[tokio::test]
async fn test_delete_recurring_template() {
    let pool = Arc::new(init_test_database().await.unwrap());
    let repo = RecurringRepository::new(pool);
    
    let rule = RecurrenceRule {
        pattern: RecurrencePattern::Monthly,
        interval: 1,
        days_of_week: vec![],
        day_of_month: Some(15),
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: None,
        occurrences_count: 0,
    };
    
    let template = RecurringTaskTemplate::new(
        "Monthly Report".to_string(),
        "Submit monthly report".to_string(),
        rule,
    );
    
    repo.create(&template).await.unwrap();
    
    // Delete the template
    let deleted = repo.delete(template.id).await.unwrap();
    assert!(deleted);
    
    // Verify deletion
    let fetched = repo.get(template.id).await.unwrap();
    assert!(fetched.is_none());
    
    // Try to delete non-existent template
    let deleted_again = repo.delete(template.id).await.unwrap();
    assert!(!deleted_again);
}

#[tokio::test]
async fn test_list_active_templates() {
    let pool = Arc::new(init_test_database().await.unwrap());
    let repo = RecurringRepository::new(pool);
    
    // Create multiple templates with different states
    let rule1 = RecurrenceRule {
        pattern: RecurrencePattern::Daily,
        interval: 1,
        days_of_week: vec![],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: None,
        occurrences_count: 0,
    };
    
    let mut template1 = RecurringTaskTemplate::new(
        "Active Task 1".to_string(),
        "Description 1".to_string(),
        rule1,
    );
    
    let rule2 = RecurrenceRule {
        pattern: RecurrencePattern::Weekly,
        interval: 1,
        days_of_week: vec![Weekday::Tue],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: None,
        occurrences_count: 0,
    };
    
    let mut template2 = RecurringTaskTemplate::new(
        "Active Task 2".to_string(),
        "Description 2".to_string(),
        rule2,
    );
    
    let rule3 = RecurrenceRule {
        pattern: RecurrencePattern::Monthly,
        interval: 1,
        days_of_week: vec![],
        day_of_month: Some(1),
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: None,
        occurrences_count: 0,
    };
    
    let mut template3 = RecurringTaskTemplate::new(
        "Inactive Task".to_string(),
        "Description 3".to_string(),
        rule3,
    );
    template3.deactivate();
    
    repo.create(&template1).await.unwrap();
    repo.create(&template2).await.unwrap();
    repo.create(&template3).await.unwrap();
    
    // List active templates
    let active_templates = repo.list_active().await.unwrap();
    assert_eq!(active_templates.len(), 2);
    
    // Verify only active templates are returned
    let titles: Vec<String> = active_templates.iter().map(|t| t.title.clone()).collect();
    assert!(titles.contains(&"Active Task 1".to_string()));
    assert!(titles.contains(&"Active Task 2".to_string()));
    assert!(!titles.contains(&"Inactive Task".to_string()));
}

#[tokio::test]
async fn test_complex_recurrence_rules() {
    let pool = Arc::new(init_test_database().await.unwrap());
    let repo = RecurringRepository::new(pool);
    
    // Test yearly recurrence with specific month and day
    let yearly_rule = RecurrenceRule {
        pattern: RecurrencePattern::Yearly,
        interval: 1,
        days_of_week: vec![],
        day_of_month: Some(25),
        month_of_year: Some(12),
        time_of_day: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: Some(5),
        occurrences_count: 2,
    };
    
    let mut yearly_template = RecurringTaskTemplate::new(
        "Annual Review".to_string(),
        "End of year review".to_string(),
        yearly_rule,
    );
    yearly_template.priority = Priority::Critical;
    yearly_template.estimated_hours = Some(4.0);
    
    repo.create(&yearly_template).await.unwrap();
    
    // Test weekly recurrence with multiple days
    let weekly_rule = RecurrenceRule {
        pattern: RecurrencePattern::Weekly,
        interval: 2,
        days_of_week: vec![Weekday::Mon, Weekday::Wed, Weekday::Fri],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(15, 30, 0).unwrap(),
        end_date: Some(Utc::now().date_naive() + chrono::Duration::days(90)),
        max_occurrences: None,
        occurrences_count: 0,
    };
    
    let weekly_template = RecurringTaskTemplate::new(
        "Team Sync".to_string(),
        "Bi-weekly team synchronization".to_string(),
        weekly_rule,
    );
    
    repo.create(&weekly_template).await.unwrap();
    
    // Verify both templates are saved correctly
    let fetched_yearly = repo.get(yearly_template.id).await.unwrap().unwrap();
    assert_eq!(fetched_yearly.recurrence_rule.pattern, RecurrencePattern::Yearly);
    assert_eq!(fetched_yearly.recurrence_rule.month_of_year, Some(12));
    assert_eq!(fetched_yearly.recurrence_rule.day_of_month, Some(25));
    assert_eq!(fetched_yearly.recurrence_rule.max_occurrences, Some(5));
    assert_eq!(fetched_yearly.recurrence_rule.occurrences_count, 2);
    
    let fetched_weekly = repo.get(weekly_template.id).await.unwrap().unwrap();
    assert_eq!(fetched_weekly.recurrence_rule.pattern, RecurrencePattern::Weekly);
    assert_eq!(fetched_weekly.recurrence_rule.interval, 2);
    assert_eq!(fetched_weekly.recurrence_rule.days_of_week.len(), 3);
    assert!(fetched_weekly.recurrence_rule.end_date.is_some());
}

#[tokio::test]
async fn test_update_occurrence_tracking() {
    let pool = Arc::new(init_test_database().await.unwrap());
    let repo = RecurringRepository::new(pool);
    
    let rule = RecurrenceRule {
        pattern: RecurrencePattern::Daily,
        interval: 1,
        days_of_week: vec![],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: Some(10),
        occurrences_count: 0,
    };
    
    let mut template = RecurringTaskTemplate::new(
        "Daily Task".to_string(),
        "A daily recurring task".to_string(),
        rule,
    );
    
    repo.create(&template).await.unwrap();
    
    // Simulate generating tasks
    for i in 1..=5 {
        template.generate_task();
        repo.update(&template).await.unwrap();
        
        let fetched = repo.get(template.id).await.unwrap().unwrap();
        assert_eq!(fetched.recurrence_rule.occurrences_count, i);
        assert!(fetched.last_generated.is_some());
        assert!(fetched.next_occurrence.is_some());
    }
    
    // Verify the count persists
    let final_fetched = repo.get(template.id).await.unwrap().unwrap();
    assert_eq!(final_fetched.recurrence_rule.occurrences_count, 5);
}

#[tokio::test]
async fn test_metadata_and_resources() {
    let pool = Arc::new(init_test_database().await.unwrap());
    let repo = RecurringRepository::new(pool);
    
    let rule = RecurrenceRule {
        pattern: RecurrencePattern::Weekly,
        interval: 1,
        days_of_week: vec![Weekday::Thu],
        day_of_month: None,
        month_of_year: None,
        time_of_day: NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
        end_date: None,
        max_occurrences: None,
        occurrences_count: 0,
    };
    
    let mut template = RecurringTaskTemplate::new(
        "Code Review".to_string(),
        "Weekly code review session".to_string(),
        rule,
    );
    
    // Add metadata
    template.metadata.insert("project".to_string(), "backend".to_string());
    template.metadata.insert("team".to_string(), "engineering".to_string());
    template.metadata.insert("type".to_string(), "review".to_string());
    
    // Don't set resource ID to avoid foreign key constraint in test
    // In production, this would be a valid UUID from resources table
    template.estimated_hours = Some(2.5);
    
    repo.create(&template).await.unwrap();
    
    // Fetch and verify
    let fetched = repo.get(template.id).await.unwrap().unwrap();
    assert_eq!(fetched.metadata.len(), 3);
    assert_eq!(fetched.metadata.get("project"), Some(&"backend".to_string()));
    assert_eq!(fetched.metadata.get("team"), Some(&"engineering".to_string()));
    assert_eq!(fetched.metadata.get("type"), Some(&"review".to_string()));
    assert!(fetched.assigned_resource_id.is_none());
    assert_eq!(fetched.estimated_hours, Some(2.5));
}