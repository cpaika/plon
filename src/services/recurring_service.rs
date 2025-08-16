use anyhow::Result;
use chrono::{NaiveDate, NaiveTime, Weekday};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::recurring::{RecurringTaskTemplate, RecurrenceRule, RecurrencePattern};
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
        if let Some(mut template) = self.repository.recurring.get(template_id).await? {
            if let Some(task) = template.generate_task() {
                // Save the generated task to the database
                self.repository.tasks.create(&task).await?;
                
                // Update the template
                self.repository.recurring.update(&template).await?;
                
                return Ok(Some(task));
            }
        }
        Ok(None)
    }

    pub async fn get_upcoming_occurrences(&self, days_ahead: i64) -> Result<Vec<RecurringTaskTemplate>> {
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