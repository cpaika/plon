use anyhow::Result;
use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{debug, error, info};

use crate::repository::Repository;
use crate::services::RecurringService;

#[allow(dead_code)]
pub struct RecurringTaskScheduler {
    recurring_service: RecurringService,
    interval_minutes: u64,
}

impl RecurringTaskScheduler {
    pub fn new(repository: Arc<Repository>, interval_minutes: u64) -> Self {
        Self {
            recurring_service: RecurringService::new(repository),
            interval_minutes,
        }
    }

    /// Start the scheduler that runs in the background
    pub async fn start(self) {
        let mut interval = interval(Duration::from_secs(self.interval_minutes * 60));

        info!(
            "Starting recurring task scheduler with {}min interval",
            self.interval_minutes
        );

        loop {
            interval.tick().await;

            if let Err(e) = self.check_and_generate_tasks().await {
                error!("Error generating recurring tasks: {}", e);
            }
        }
    }

    /// Run a single check and generation cycle
    pub async fn run_once(&self) -> Result<usize> {
        self.check_and_generate_tasks().await
    }

    async fn check_and_generate_tasks(&self) -> Result<usize> {
        debug!("Checking for recurring tasks to generate");

        let generated = self.recurring_service.generate_due_tasks().await?;
        let count = generated.len();

        if count > 0 {
            info!("Generated {} recurring tasks", count);
            for task in &generated {
                debug!("Generated task: {} ({})", task.title, task.id);
            }
        } else {
            debug!("No recurring tasks to generate at this time");
        }

        Ok(count)
    }

    /// Get upcoming recurring tasks for the next N days
    pub async fn get_upcoming(
        &self,
        days_ahead: i64,
    ) -> Result<Vec<(String, chrono::DateTime<chrono::Utc>)>> {
        let templates = self
            .recurring_service
            .get_upcoming_occurrences(days_ahead)
            .await?;

        let upcoming = templates
            .into_iter()
            .filter_map(|t| t.next_occurrence.map(|next| (t.title, next)))
            .collect();

        Ok(upcoming)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::recurring::RecurrencePattern;
    use crate::repository::database::init_test_database;
    use chrono::NaiveTime;

    #[tokio::test]
    async fn test_scheduler_run_once() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create a recurring template
        let recurring_service = RecurringService::new(repository.clone());
        recurring_service
            .create_recurring_template(
                "Test Task".to_string(),
                "Description".to_string(),
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

        // Create scheduler and run once
        let scheduler = RecurringTaskScheduler::new(repository, 5);
        let count = scheduler.run_once().await.unwrap();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_scheduler_no_tasks_due() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create scheduler without any templates
        let scheduler = RecurringTaskScheduler::new(repository, 5);
        let count = scheduler.run_once().await.unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_get_upcoming() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create recurring templates
        let recurring_service = RecurringService::new(repository.clone());

        recurring_service
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

        recurring_service
            .create_recurring_template(
                "Weekly Task".to_string(),
                "Every week".to_string(),
                RecurrencePattern::Weekly,
                1,
                Some(vec![chrono::Weekday::Mon]),
                None,
                None,
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                None,
                None,
            )
            .await
            .unwrap();

        // Get upcoming tasks for next 7 days
        let scheduler = RecurringTaskScheduler::new(repository, 5);
        let upcoming = scheduler.get_upcoming(7).await.unwrap();

        assert_eq!(upcoming.len(), 2);
        assert!(upcoming.iter().any(|(title, _)| title == "Daily Task"));
        assert!(upcoming.iter().any(|(title, _)| title == "Weekly Task"));
    }

    #[tokio::test]
    async fn test_scheduler_handles_errors_gracefully() {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));

        // Create a template with end date in the past (will be inactive)
        let recurring_service = RecurringService::new(repository.clone());
        let past_date = (chrono::Utc::now() - chrono::Duration::days(1)).date_naive();

        recurring_service
            .create_recurring_template(
                "Expired Task".to_string(),
                "Should not generate".to_string(),
                RecurrencePattern::Daily,
                1,
                None,
                None,
                None,
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                Some(past_date),
                None,
            )
            .await
            .unwrap();

        // Scheduler should handle this gracefully
        let scheduler = RecurringTaskScheduler::new(repository, 5);
        let count = scheduler.run_once().await.unwrap();

        assert_eq!(count, 0);
    }
}
