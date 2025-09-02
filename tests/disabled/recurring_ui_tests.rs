use chrono::{NaiveTime, Weekday};
use plon::domain::recurring::{RecurrencePattern, RecurrenceRule, RecurringTaskTemplate};
use plon::repository::Repository;
use plon::repository::database::init_test_database;
use plon::services::RecurringService;
use plon::ui::views::recurring_view::RecurringView;
use plon::ui::widgets::recurring_editor::RecurringEditor;
use std::sync::Arc;

#[test]
fn test_recurring_view_creation() {
    let view = RecurringView::new();
    assert!(view.templates.is_empty());
    assert!(!view.show_editor);
}

#[test]
fn test_recurring_editor_creation() {
    let editor = RecurringEditor::new();
    assert_eq!(editor.title, "");
    assert_eq!(editor.description, "");
    assert_eq!(editor.pattern, RecurrencePattern::Daily);
    assert_eq!(editor.interval, 1);
    assert!(editor.selected_days.is_empty());
    assert!(editor.day_of_month.is_none());
    assert!(editor.month_of_year.is_none());
}

#[test]
fn test_recurring_editor_validation() {
    let mut editor = RecurringEditor::new();

    // Empty title should fail validation
    assert!(!editor.validate());

    // Valid title should pass
    editor.title = "Daily Standup".to_string();
    assert!(editor.validate());

    // Weekly pattern requires at least one day
    editor.pattern = RecurrencePattern::Weekly;
    assert!(!editor.validate());

    editor.selected_days.push(Weekday::Mon);
    assert!(editor.validate());

    // Monthly pattern with invalid day should fail
    editor.pattern = RecurrencePattern::Monthly;
    editor.day_of_month = Some(32);
    assert!(!editor.validate());

    editor.day_of_month = Some(15);
    assert!(editor.validate());

    // Yearly pattern with invalid month should fail
    editor.pattern = RecurrencePattern::Yearly;
    editor.month_of_year = Some(13);
    assert!(!editor.validate());

    editor.month_of_year = Some(12);
    assert!(editor.validate());
}

#[test]
fn test_recurring_editor_build_template() {
    let mut editor = RecurringEditor::new();
    editor.title = "Weekly Review".to_string();
    editor.description = "Review weekly progress".to_string();
    editor.pattern = RecurrencePattern::Weekly;
    editor.interval = 1;
    editor.selected_days = vec![Weekday::Fri];
    editor.time = NaiveTime::from_hms_opt(16, 0, 0).unwrap();

    let template = editor.build_template();
    assert_eq!(template.title, "Weekly Review");
    assert_eq!(template.description, "Review weekly progress");
    assert_eq!(template.recurrence_rule.pattern, RecurrencePattern::Weekly);
    assert_eq!(template.recurrence_rule.interval, 1);
    assert_eq!(template.recurrence_rule.days_of_week, vec![Weekday::Fri]);
    assert_eq!(
        template.recurrence_rule.time_of_day,
        NaiveTime::from_hms_opt(16, 0, 0).unwrap()
    );
}

#[test]
fn test_recurring_editor_reset() {
    let mut editor = RecurringEditor::new();
    editor.title = "Test".to_string();
    editor.description = "Test desc".to_string();
    editor.pattern = RecurrencePattern::Monthly;
    editor.interval = 2;
    editor.day_of_month = Some(15);

    editor.reset();

    assert_eq!(editor.title, "");
    assert_eq!(editor.description, "");
    assert_eq!(editor.pattern, RecurrencePattern::Daily);
    assert_eq!(editor.interval, 1);
    assert!(editor.day_of_month.is_none());
}

#[tokio::test]
async fn test_recurring_view_load_templates() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());

    // Create some templates
    let template1 = service
        .create_recurring_template(
            "Daily Task".to_string(),
            "Every day".to_string(),
            RecurrencePattern::Daily,
            1,
            None,
            None,
            None,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            None,
            None,
        )
        .await
        .unwrap();

    let template2 = service
        .create_recurring_template(
            "Weekly Task".to_string(),
            "Every week".to_string(),
            RecurrencePattern::Weekly,
            1,
            Some(vec![Weekday::Mon]),
            None,
            None,
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            None,
            None,
        )
        .await
        .unwrap();

    // Create view and load templates
    let mut view = RecurringView::new();
    view.load_templates(&service).await.unwrap();

    assert_eq!(view.templates.len(), 2);
    assert!(view.templates.iter().any(|t| t.title == "Daily Task"));
    assert!(view.templates.iter().any(|t| t.title == "Weekly Task"));
}

#[tokio::test]
async fn test_recurring_view_create_template() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());

    let mut view = RecurringView::new();
    view.editor.title = "New Task".to_string();
    view.editor.description = "Description".to_string();
    view.editor.pattern = RecurrencePattern::Daily;
    view.editor.interval = 1;

    // Create template through view
    view.create_template(&service).await.unwrap();

    // Verify template was created
    view.load_templates(&service).await.unwrap();
    assert_eq!(view.templates.len(), 1);
    assert_eq!(view.templates[0].title, "New Task");
}

#[tokio::test]
async fn test_recurring_view_delete_template() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());

    // Create a template
    let template = service
        .create_recurring_template(
            "To Delete".to_string(),
            "Will be deleted".to_string(),
            RecurrencePattern::Daily,
            1,
            None,
            None,
            None,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            None,
            None,
        )
        .await
        .unwrap();

    let mut view = RecurringView::new();
    view.load_templates(&service).await.unwrap();
    assert_eq!(view.templates.len(), 1);

    // Delete the template
    view.delete_template(&service, template.id).await.unwrap();

    // Reload and verify deletion
    view.load_templates(&service).await.unwrap();
    assert_eq!(view.templates.len(), 0);
}

#[tokio::test]
async fn test_recurring_view_toggle_template() {
    let pool = init_test_database().await.unwrap();
    let repository = Arc::new(Repository::new(pool));
    let service = RecurringService::new(repository.clone());

    // Create an active template
    let template = service
        .create_recurring_template(
            "Toggle Task".to_string(),
            "Will be toggled".to_string(),
            RecurrencePattern::Daily,
            1,
            None,
            None,
            None,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            None,
            None,
        )
        .await
        .unwrap();

    let mut view = RecurringView::new();
    view.load_templates(&service).await.unwrap();
    assert!(view.templates[0].active);

    // Deactivate the template
    view.toggle_template(&service, template.id).await.unwrap();

    // After deactivation, it won't appear in active templates list
    view.load_templates(&service).await.unwrap();
    assert_eq!(view.templates.len(), 0);

    // Reactivate the template
    view.toggle_template(&service, template.id).await.unwrap();

    // After reactivation, it should appear again
    view.load_templates(&service).await.unwrap();
    assert_eq!(view.templates.len(), 1);
    assert!(view.templates[0].active);
}

#[test]
fn test_pattern_display_string() {
    let editor = RecurringEditor::new();

    assert_eq!(editor.pattern_to_string(RecurrencePattern::Daily), "Daily");
    assert_eq!(
        editor.pattern_to_string(RecurrencePattern::Weekly),
        "Weekly"
    );
    assert_eq!(
        editor.pattern_to_string(RecurrencePattern::Monthly),
        "Monthly"
    );
    assert_eq!(
        editor.pattern_to_string(RecurrencePattern::Yearly),
        "Yearly"
    );
    assert_eq!(
        editor.pattern_to_string(RecurrencePattern::Custom),
        "Custom"
    );
}

#[test]
fn test_frequency_description() {
    let mut editor = RecurringEditor::new();

    // Daily
    editor.pattern = RecurrencePattern::Daily;
    editor.interval = 1;
    assert_eq!(editor.get_frequency_description(), "Every day");

    editor.interval = 3;
    assert_eq!(editor.get_frequency_description(), "Every 3 days");

    // Weekly
    editor.pattern = RecurrencePattern::Weekly;
    editor.interval = 1;
    editor.selected_days = vec![Weekday::Mon, Weekday::Wed, Weekday::Fri];
    assert_eq!(
        editor.get_frequency_description(),
        "Every week on Mon, Wed, Fri"
    );

    editor.interval = 2;
    assert_eq!(
        editor.get_frequency_description(),
        "Every 2 weeks on Mon, Wed, Fri"
    );

    // Monthly
    editor.pattern = RecurrencePattern::Monthly;
    editor.interval = 1;
    editor.day_of_month = Some(15);
    assert_eq!(
        editor.get_frequency_description(),
        "Every month on the 15th"
    );

    // Yearly
    editor.pattern = RecurrencePattern::Yearly;
    editor.interval = 1;
    editor.month_of_year = Some(12);
    editor.day_of_month = Some(25);
    assert_eq!(
        editor.get_frequency_description(),
        "Every year on December 25"
    );
}
