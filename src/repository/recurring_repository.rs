use anyhow::Result;
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, NaiveTime, Utc, Weekday};
use crate::domain::recurring::{RecurringTaskTemplate, RecurrenceRule, RecurrencePattern};
use crate::domain::task::Priority;
use std::collections::HashMap;
use serde_json;

#[derive(Clone)]
pub struct RecurringRepository {
    pool: Arc<SqlitePool>,
}

impl RecurringRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, template: &RecurringTaskTemplate) -> Result<()> {
        let id = template.id.to_string();
        let metadata_json = serde_json::to_string(&template.metadata)?;
        let days_of_week_json = serde_json::to_string(&self.weekdays_to_strings(&template.recurrence_rule.days_of_week))?;
        let pattern = self.pattern_to_string(&template.recurrence_rule.pattern);
        let priority = self.priority_to_string(&template.priority);
        let time_of_day = template.recurrence_rule.time_of_day.format("%H:%M:%S").to_string();
        let assigned_resource_id = template.assigned_resource_id.map(|u| u.to_string());
        let end_date = template.recurrence_rule.end_date.map(|d| d.to_string());
        let last_generated = template.last_generated.map(|dt| dt.to_rfc3339());
        let next_occurrence = template.next_occurrence.map(|dt| dt.to_rfc3339());
        
        sqlx::query(
            "INSERT INTO recurring_templates (
                id, title, description, priority, metadata,
                assigned_resource_id, estimated_hours,
                recurrence_pattern, recurrence_interval, days_of_week,
                day_of_month, month_of_year, time_of_day,
                end_date, max_occurrences, occurrences_count,
                active, created_at, updated_at,
                last_generated, next_occurrence
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&template.title)
        .bind(&template.description)
        .bind(&priority)
        .bind(&metadata_json)
        .bind(assigned_resource_id)
        .bind(template.estimated_hours)
        .bind(&pattern)
        .bind(template.recurrence_rule.interval as i32)
        .bind(&days_of_week_json)
        .bind(template.recurrence_rule.day_of_month.map(|d| d as i32))
        .bind(template.recurrence_rule.month_of_year.map(|m| m as i32))
        .bind(&time_of_day)
        .bind(end_date)
        .bind(template.recurrence_rule.max_occurrences.map(|m| m as i32))
        .bind(template.recurrence_rule.occurrences_count as i32)
        .bind(if template.active { 1 } else { 0 })
        .bind(template.created_at.to_rfc3339())
        .bind(template.updated_at.to_rfc3339())
        .bind(last_generated)
        .bind(next_occurrence)
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn update(&self, template: &RecurringTaskTemplate) -> Result<()> {
        let id = template.id.to_string();
        let metadata_json = serde_json::to_string(&template.metadata)?;
        let days_of_week_json = serde_json::to_string(&self.weekdays_to_strings(&template.recurrence_rule.days_of_week))?;
        let pattern = self.pattern_to_string(&template.recurrence_rule.pattern);
        let priority = self.priority_to_string(&template.priority);
        let time_of_day = template.recurrence_rule.time_of_day.format("%H:%M:%S").to_string();
        let assigned_resource_id = template.assigned_resource_id.map(|u| u.to_string());
        let end_date = template.recurrence_rule.end_date.map(|d| d.to_string());
        let last_generated = template.last_generated.map(|dt| dt.to_rfc3339());
        let next_occurrence = template.next_occurrence.map(|dt| dt.to_rfc3339());
        
        sqlx::query(
            "UPDATE recurring_templates SET
                title = ?, description = ?, priority = ?, metadata = ?,
                assigned_resource_id = ?, estimated_hours = ?,
                recurrence_pattern = ?, recurrence_interval = ?, days_of_week = ?,
                day_of_month = ?, month_of_year = ?, time_of_day = ?,
                end_date = ?, max_occurrences = ?, occurrences_count = ?,
                active = ?, updated_at = ?,
                last_generated = ?, next_occurrence = ?
            WHERE id = ?"
        )
        .bind(&template.title)
        .bind(&template.description)
        .bind(&priority)
        .bind(&metadata_json)
        .bind(assigned_resource_id)
        .bind(template.estimated_hours)
        .bind(&pattern)
        .bind(template.recurrence_rule.interval as i32)
        .bind(&days_of_week_json)
        .bind(template.recurrence_rule.day_of_month.map(|d| d as i32))
        .bind(template.recurrence_rule.month_of_year.map(|m| m as i32))
        .bind(&time_of_day)
        .bind(end_date)
        .bind(template.recurrence_rule.max_occurrences.map(|m| m as i32))
        .bind(template.recurrence_rule.occurrences_count as i32)
        .bind(if template.active { 1 } else { 0 })
        .bind(template.updated_at.to_rfc3339())
        .bind(last_generated)
        .bind(next_occurrence)
        .bind(&id)
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<RecurringTaskTemplate>> {
        let id_str = id.to_string();
        
        let row = sqlx::query(
            "SELECT * FROM recurring_templates WHERE id = ?"
        )
        .bind(&id_str)
        .fetch_optional(&*self.pool)
        .await?;
        
        match row {
            Some(row) => Ok(Some(self.row_to_template(row)?)),
            None => Ok(None),
        }
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let id_str = id.to_string();
        
        let result = sqlx::query(
            "DELETE FROM recurring_templates WHERE id = ?"
        )
        .bind(&id_str)
        .execute(&*self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }

    pub async fn list_active(&self) -> Result<Vec<RecurringTaskTemplate>> {
        let rows = sqlx::query(
            "SELECT * FROM recurring_templates WHERE active = 1"
        )
        .fetch_all(&*self.pool)
        .await?;
        
        let mut templates = Vec::new();
        for row in rows {
            templates.push(self.row_to_template(row)?);
        }
        
        Ok(templates)
    }
    
    fn row_to_template(&self, row: sqlx::sqlite::SqliteRow) -> Result<RecurringTaskTemplate> {
        let id: String = row.get("id");
        let title: String = row.get("title");
        let description: String = row.get("description");
        let priority_str: String = row.get("priority");
        let metadata_json: String = row.get("metadata");
        let assigned_resource_id: Option<String> = row.get("assigned_resource_id");
        let estimated_hours: Option<f32> = row.get("estimated_hours");
        
        let pattern_str: String = row.get("recurrence_pattern");
        let interval: i32 = row.get("recurrence_interval");
        let days_of_week_json: Option<String> = row.get("days_of_week");
        let day_of_month: Option<i32> = row.get("day_of_month");
        let month_of_year: Option<i32> = row.get("month_of_year");
        let time_of_day_str: String = row.get("time_of_day");
        let end_date_str: Option<String> = row.get("end_date");
        let max_occurrences: Option<i32> = row.get("max_occurrences");
        let occurrences_count: i32 = row.get("occurrences_count");
        
        let active: i32 = row.get("active");
        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");
        let last_generated_str: Option<String> = row.get("last_generated");
        let next_occurrence_str: Option<String> = row.get("next_occurrence");
        
        let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json)?;
        let days_of_week = if let Some(json) = days_of_week_json {
            self.strings_to_weekdays(&serde_json::from_str::<Vec<String>>(&json)?)?
        } else {
            Vec::new()
        };
        
        let time_of_day = NaiveTime::parse_from_str(&time_of_day_str, "%H:%M:%S")?;
        let end_date = end_date_str.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        
        let recurrence_rule = RecurrenceRule {
            pattern: self.string_to_pattern(&pattern_str)?,
            interval: interval as u32,
            days_of_week,
            day_of_month: day_of_month.map(|d| d as u32),
            month_of_year: month_of_year.map(|m| m as u32),
            time_of_day,
            end_date,
            max_occurrences: max_occurrences.map(|m| m as u32),
            occurrences_count: occurrences_count as u32,
        };
        
        Ok(RecurringTaskTemplate {
            id: Uuid::parse_str(&id)?,
            title,
            description,
            priority: self.string_to_priority(&priority_str)?,
            metadata,
            assigned_resource_id: assigned_resource_id.and_then(|s| Uuid::parse_str(&s).ok()),
            estimated_hours,
            recurrence_rule,
            active: active == 1,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc),
            last_generated: last_generated_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()).map(|dt| dt.with_timezone(&Utc)),
            next_occurrence: next_occurrence_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()).map(|dt| dt.with_timezone(&Utc)),
        })
    }
    
    fn pattern_to_string(&self, pattern: &RecurrencePattern) -> String {
        match pattern {
            RecurrencePattern::Daily => "Daily".to_string(),
            RecurrencePattern::Weekly => "Weekly".to_string(),
            RecurrencePattern::Monthly => "Monthly".to_string(),
            RecurrencePattern::Yearly => "Yearly".to_string(),
            RecurrencePattern::Custom => "Custom".to_string(),
        }
    }
    
    fn string_to_pattern(&self, s: &str) -> Result<RecurrencePattern> {
        match s {
            "Daily" => Ok(RecurrencePattern::Daily),
            "Weekly" => Ok(RecurrencePattern::Weekly),
            "Monthly" => Ok(RecurrencePattern::Monthly),
            "Yearly" => Ok(RecurrencePattern::Yearly),
            "Custom" => Ok(RecurrencePattern::Custom),
            _ => Err(anyhow::anyhow!("Invalid recurrence pattern: {}", s)),
        }
    }
    
    fn priority_to_string(&self, priority: &Priority) -> String {
        match priority {
            Priority::Critical => "Critical".to_string(),
            Priority::High => "High".to_string(),
            Priority::Medium => "Medium".to_string(),
            Priority::Low => "Low".to_string(),
        }
    }
    
    fn string_to_priority(&self, s: &str) -> Result<Priority> {
        match s {
            "Critical" => Ok(Priority::Critical),
            "High" => Ok(Priority::High),
            "Medium" => Ok(Priority::Medium),
            "Low" => Ok(Priority::Low),
            _ => Err(anyhow::anyhow!("Invalid priority: {}", s)),
        }
    }
    
    fn weekdays_to_strings(&self, weekdays: &[Weekday]) -> Vec<String> {
        weekdays.iter().map(|w| format!("{:?}", w)).collect()
    }
    
    fn strings_to_weekdays(&self, strings: &[String]) -> Result<Vec<Weekday>> {
        strings.iter().map(|s| self.string_to_weekday(s)).collect()
    }
    
    fn string_to_weekday(&self, s: &str) -> Result<Weekday> {
        match s {
            "Mon" => Ok(Weekday::Mon),
            "Tue" => Ok(Weekday::Tue),
            "Wed" => Ok(Weekday::Wed),
            "Thu" => Ok(Weekday::Thu),
            "Fri" => Ok(Weekday::Fri),
            "Sat" => Ok(Weekday::Sat),
            "Sun" => Ok(Weekday::Sun),
            _ => Err(anyhow::anyhow!("Invalid weekday: {}", s)),
        }
    }
}