use anyhow::Result;
use chrono::{NaiveDate, NaiveTime, Weekday};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::recurring::{RecurrencePattern, RecurrenceRule, RecurringTaskTemplate};
use crate::domain::task::Task;
use crate::repository::Repository;

#[derive(Clone)]
pub struct RecurringService {
    repository: Arc<Repository>,
}

impl RecurringService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn create_recurring_template(
        &self,
        title: String,
        description: String,
        pattern: RecurrencePattern,
        interval: u32,
        days_of_week: Option<Vec<Weekday>>,
        day_of_month: Option<u32>,
        month_of_year: Option<u32>,
        time_of_day: NaiveTime,
        end_date: Option<NaiveDate>,
        max_occurrences: Option<u32>,
    ) -> Result<RecurringTaskTemplate> {
        let rule = RecurrenceRule {
            pattern,
            interval,
            days_of_week: days_of_week.unwrap_or_default(),
            day_of_month,
            month_of_year,
            time_of_day,
            end_date,
            max_occurrences,
            occurrences_count: 0,
        };

        let template = RecurringTaskTemplate::new(title, description, rule);

        self.repository.recurring.create(&template).await?;

        Ok(template)
    }

    pub async fn update_template(&self, template: &RecurringTaskTemplate) -> Result<()> {
        self.repository.recurring.update(template).await
    }

    pub async fn get_template(&self, id: Uuid) -> Result<Option<RecurringTaskTemplate>> {
        self.repository.recurring.get(id).await
    }

    pub async fn delete_template(&self, id: Uuid) -> Result<bool> {
        self.repository.recurring.delete(id).await
    }

    pub async fn list_active_templates(&self) -> Result<Vec<RecurringTaskTemplate>> {
        self.repository.recurring.list_active().await
    }

    pub async fn deactivate_template(&self, id: Uuid) -> Result<()> {
        if let Some(mut template) = self.repository.recurring.get(id).await? {
            template.deactivate();
            self.repository.recurring.update(&template).await?;
        }
        Ok(())
    }

    pub async fn reactivate_template(&self, id: Uuid) -> Result<()> {
        if let Some(mut template) = self.repository.recurring.get(id).await? {
            template.reactivate();
            self.repository.recurring.update(&template).await?;
        }
        Ok(())
    }

    pub async fn generate_due_tasks(&self) -> Result<Vec<Task>> {
        let active_templates = self.repository.recurring.list_active().await?;
        let mut generated_tasks = Vec::new();

        for mut template in active_templates {
            if template.should_generate_now() {
                if let Some(task) = template.generate_task() {
                    // Save the generated task to the database
                    self.repository.tasks.create(&task).await?;
                    generated_tasks.push(task);

                    // Update the template with new occurrence count and next occurrence
                    self.repository.recurring.update(&template).await?;
                } else {
                    // Template returned None, which means it was deactivated
                    // We still need to update it in the database
                    self.repository.recurring.update(&template).await?;
                }
            }
        }

        Ok(generated_tasks)
    }

    pub async fn generate_tasks_for_template(&self, template_id: Uuid) -> Result<Option<Task>> {
        if let Some(mut template) = self.repository.recurring.get(template_id).await?
            && let Some(task) = template.generate_task()
        {
            // Save the generated task to the database
            self.repository.tasks.create(&task).await?;

            // Update the template
            self.repository.recurring.update(&template).await?;

            return Ok(Some(task));
        }
        Ok(None)
    }

    pub async fn get_upcoming_occurrences(
        &self,
        days_ahead: i64,
    ) -> Result<Vec<RecurringTaskTemplate>> {
        let templates = self.repository.recurring.list_active().await?;
        let cutoff = chrono::Utc::now() + chrono::Duration::days(days_ahead);

        let upcoming: Vec<RecurringTaskTemplate> = templates
            .into_iter()
            .filter(|t| {
                if let Some(next) = t.next_occurrence {
                    next <= cutoff
                } else {
                    false
                }
            })
            .collect();

        Ok(upcoming)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::database::init_test_database;
    use chrono::{Duration, Utc};

    async fn setup() -> RecurringService {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        RecurringService::new(repository)
    }

    #[tokio::test]
    async fn test_create_daily_recurring_template() {
        let service = setup().await;

        let template = service
            .create_recurring_template(
                "Daily Standup".to_string(),
                "Team sync meeting".to_string(),
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

        assert_eq!(template.title, "Daily Standup");
        assert_eq!(template.recurrence_rule.pattern, RecurrencePattern::Daily);
        assert_eq!(template.recurrence_rule.interval, 1);
    }

    #[tokio::test]
    async fn test_create_weekly_recurring_template() {
        let service = setup().await;

        let days = vec![Weekday::Mon, Weekday::Wed, Weekday::Fri];
        let template = service
            .create_recurring_template(
                "Team Meeting".to_string(),
                "Weekly team sync".to_string(),
                RecurrencePattern::Weekly,
                1,
                Some(days.clone()),
                None,
                None,
                NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(template.recurrence_rule.pattern, RecurrencePattern::Weekly);
        assert_eq!(template.recurrence_rule.days_of_week, days);
    }

    #[tokio::test]
    async fn test_monthly_recurring_template() {
        let service = setup().await;

        let template = service
            .create_recurring_template(
                "Monthly Report".to_string(),
                "Submit monthly report".to_string(),
                RecurrencePattern::Monthly,
                1,
                None,
                Some(15), // 15th of each month
                None,
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(template.recurrence_rule.pattern, RecurrencePattern::Monthly);
        assert_eq!(template.recurrence_rule.day_of_month, Some(15));
    }

    #[tokio::test]
    async fn test_template_with_max_occurrences() {
        let service = setup().await;

        let template = service
            .create_recurring_template(
                "Limited Task".to_string(),
                "Only 5 times".to_string(),
                RecurrencePattern::Daily,
                1,
                None,
                None,
                None,
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                None,
                Some(5),
            )
            .await
            .unwrap();

        assert_eq!(template.recurrence_rule.max_occurrences, Some(5));
        assert_eq!(template.recurrence_rule.occurrences_count, 0);
    }

    #[tokio::test]
    async fn test_template_with_end_date() {
        let service = setup().await;

        let end_date = (Utc::now() + Duration::days(30)).date_naive();
        let template = service
            .create_recurring_template(
                "Temporary Task".to_string(),
                "Ends in 30 days".to_string(),
                RecurrencePattern::Daily,
                1,
                None,
                None,
                None,
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                Some(end_date),
                None,
            )
            .await
            .unwrap();

        assert_eq!(template.recurrence_rule.end_date, Some(end_date));
    }

    #[tokio::test]
    async fn test_get_template() {
        let service = setup().await;

        let template = service
            .create_recurring_template(
                "Test Template".to_string(),
                "Description".to_string(),
                RecurrencePattern::Daily,
                1,
                None,
                None,
                None,
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                None,
                None,
            )
            .await
            .unwrap();

        let retrieved = service.get_template(template.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, template.id);
    }

    #[tokio::test]
    async fn test_deactivate_template() {
        let service = setup().await;

        let template = service
            .create_recurring_template(
                "To Deactivate".to_string(),
                "Will be deactivated".to_string(),
                RecurrencePattern::Daily,
                1,
                None,
                None,
                None,
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                None,
                None,
            )
            .await
            .unwrap();

        service.deactivate_template(template.id).await.unwrap();

        let retrieved = service.get_template(template.id).await.unwrap().unwrap();
        assert!(!retrieved.active);
    }

    #[tokio::test]
    async fn test_list_active_templates() {
        let service = setup().await;

        // Create active and inactive templates
        for i in 0..3 {
            let template = service
                .create_recurring_template(
                    format!("Template {}", i),
                    "".to_string(),
                    RecurrencePattern::Daily,
                    1,
                    None,
                    None,
                    None,
                    NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                    None,
                    None,
                )
                .await
                .unwrap();

            if i == 2 {
                service.deactivate_template(template.id).await.unwrap();
            }
        }

        let active = service.list_active_templates().await.unwrap();
        assert_eq!(active.len(), 2); // Only 2 should be active
    }
}
