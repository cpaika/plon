use crate::domain::task::{Priority, Task};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecurringTaskTemplate {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub metadata: HashMap<String, String>,
    pub assigned_resource_id: Option<Uuid>,
    pub estimated_hours: Option<f32>,
    pub recurrence_rule: RecurrenceRule,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_generated: Option<DateTime<Utc>>,
    pub next_occurrence: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecurrenceRule {
    pub pattern: RecurrencePattern,
    pub interval: u32, // e.g., every 2 weeks
    pub days_of_week: Vec<Weekday>,
    pub day_of_month: Option<u32>,
    pub month_of_year: Option<u32>,
    pub time_of_day: NaiveTime,
    pub end_date: Option<NaiveDate>,
    pub max_occurrences: Option<u32>,
    pub occurrences_count: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RecurrencePattern {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Custom,
}

impl RecurringTaskTemplate {
    pub fn new(title: String, description: String, recurrence_rule: RecurrenceRule) -> Self {
        let now = Utc::now();

        // Check if template should be active based on end date
        let active = if let Some(end_date) = recurrence_rule.end_date {
            now.date_naive() <= end_date
        } else {
            true
        };

        let mut template = Self {
            id: Uuid::new_v4(),
            title,
            description,
            priority: Priority::Medium,
            metadata: HashMap::new(),
            assigned_resource_id: None,
            estimated_hours: None,
            recurrence_rule,
            active,
            created_at: now,
            updated_at: now,
            last_generated: None,
            next_occurrence: None,
        };
        // For new templates, set next occurrence to now to allow immediate generation
        // but only if the template is active
        if template.active {
            template.next_occurrence = Some(now);
        }
        template
    }

    pub fn generate_task(&mut self) -> Option<Task> {
        if !self.active {
            return None;
        }

        if let Some(max) = self.recurrence_rule.max_occurrences
            && self.recurrence_rule.occurrences_count >= max
        {
            self.active = false;
            return None;
        }

        if let Some(end_date) = self.recurrence_rule.end_date
            && Utc::now().date_naive() > end_date
        {
            self.active = false;
            return None;
        }

        let mut task = Task::new(self.title.clone(), self.description.clone());

        task.priority = self.priority;
        task.metadata = self.metadata.clone();
        task.assigned_resource_id = self.assigned_resource_id;
        task.estimated_hours = self.estimated_hours;
        task.configuration_id = None; // Recurring tasks don't have a specific configuration

        if let Some(next) = self.next_occurrence {
            task.scheduled_date = Some(next);
        }

        self.last_generated = Some(Utc::now());
        self.recurrence_rule.occurrences_count += 1;
        self.next_occurrence = Some(self.calculate_next_occurrence());
        self.updated_at = Utc::now();

        Some(task)
    }

    pub fn calculate_next_occurrence(&self) -> DateTime<Utc> {
        let base = self.last_generated.unwrap_or_else(Utc::now);

        match self.recurrence_rule.pattern {
            RecurrencePattern::Daily => base + Duration::days(self.recurrence_rule.interval as i64),
            RecurrencePattern::Weekly => {
                let mut next = base + Duration::weeks(self.recurrence_rule.interval as i64);

                // Find next matching day of week
                if !self.recurrence_rule.days_of_week.is_empty() {
                    while !self.recurrence_rule.days_of_week.contains(&next.weekday()) {
                        next += Duration::days(1);
                    }
                }
                next
            }
            RecurrencePattern::Monthly => {
                let mut next = base;
                for _ in 0..self.recurrence_rule.interval {
                    next = add_months(next, 1);
                }

                // Adjust to specific day of month if set
                if let Some(day) = self.recurrence_rule.day_of_month {
                    next = set_day_of_month(next, day);
                }
                next
            }
            RecurrencePattern::Yearly => {
                let mut next = base;
                for _ in 0..self.recurrence_rule.interval {
                    next = add_years(next, 1);
                }

                // Adjust to specific month and day if set
                if let Some(month) = self.recurrence_rule.month_of_year {
                    next = set_month(next, month);
                }
                if let Some(day) = self.recurrence_rule.day_of_month {
                    next = set_day_of_month(next, day);
                }
                next
            }
            RecurrencePattern::Custom => {
                // For custom patterns, use more complex logic
                base + Duration::days(self.recurrence_rule.interval as i64)
            }
        }
    }

    pub fn should_generate_now(&self) -> bool {
        if !self.active {
            return false;
        }

        // Check if max occurrences has been reached
        if let Some(max) = self.recurrence_rule.max_occurrences
            && self.recurrence_rule.occurrences_count >= max
        {
            return false;
        }

        if let Some(next) = self.next_occurrence {
            next <= Utc::now()
        } else {
            true
        }
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }

    pub fn reactivate(&mut self) {
        self.active = true;
        self.next_occurrence = Some(self.calculate_next_occurrence());
        self.updated_at = Utc::now();
    }
}

// Helper functions for date manipulation
fn add_months(dt: DateTime<Utc>, months: u32) -> DateTime<Utc> {
    let naive = dt.naive_utc();
    let year = naive.year() + (naive.month() + months - 1) as i32 / 12;
    let month = ((naive.month() + months - 1) % 12) + 1;
    let day = naive.day().min(days_in_month(year, month));

    DateTime::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_time(naive.time()),
        Utc,
    )
}

fn add_years(dt: DateTime<Utc>, years: u32) -> DateTime<Utc> {
    let naive = dt.naive_utc();
    let year = naive.year() + years as i32;
    let day = if naive.month() == 2 && naive.day() == 29 && !is_leap_year(year) {
        28
    } else {
        naive.day()
    };

    DateTime::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(year, naive.month(), day)
            .unwrap()
            .and_time(naive.time()),
        Utc,
    )
}

fn set_day_of_month(dt: DateTime<Utc>, day: u32) -> DateTime<Utc> {
    let naive = dt.naive_utc();
    let max_day = days_in_month(naive.year(), naive.month());
    let actual_day = day.min(max_day);

    DateTime::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(naive.year(), naive.month(), actual_day)
            .unwrap()
            .and_time(naive.time()),
        Utc,
    )
}

fn set_month(dt: DateTime<Utc>, month: u32) -> DateTime<Utc> {
    let naive = dt.naive_utc();
    let day = naive.day().min(days_in_month(naive.year(), month));

    DateTime::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(naive.year(), month, day)
            .unwrap()
            .and_time(naive.time()),
        Utc,
    )
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => panic!("Invalid month"),
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_daily_recurrence() {
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

        let mut template = RecurringTaskTemplate::new(
            "Daily Standup".to_string(),
            "Daily team standup meeting".to_string(),
            rule,
        );

        let task = template.generate_task();
        assert!(task.is_some());

        let task = task.unwrap();
        assert_eq!(task.title, "Daily Standup");
        assert!(template.next_occurrence.is_some());
    }

    #[test]
    fn test_weekly_recurrence() {
        let rule = RecurrenceRule {
            pattern: RecurrencePattern::Weekly,
            interval: 2,
            days_of_week: vec![Weekday::Mon, Weekday::Wed, Weekday::Fri],
            day_of_month: None,
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: None,
            occurrences_count: 0,
        };

        let template = RecurringTaskTemplate::new(
            "Team Meeting".to_string(),
            "Bi-weekly team meeting".to_string(),
            rule,
        );

        assert!(template.next_occurrence.is_some());
    }

    #[test]
    fn test_monthly_recurrence() {
        let rule = RecurrenceRule {
            pattern: RecurrencePattern::Monthly,
            interval: 1,
            days_of_week: vec![],
            day_of_month: Some(15),
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: None,
            occurrences_count: 0,
        };

        let mut template = RecurringTaskTemplate::new(
            "Monthly Report".to_string(),
            "Submit monthly report".to_string(),
            rule,
        );

        let task = template.generate_task();
        assert!(task.is_some());
        assert_eq!(template.recurrence_rule.occurrences_count, 1);
    }

    #[test]
    fn test_max_occurrences() {
        let rule = RecurrenceRule {
            pattern: RecurrencePattern::Daily,
            interval: 1,
            days_of_week: vec![],
            day_of_month: None,
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: Some(3),
            occurrences_count: 0,
        };

        let mut template = RecurringTaskTemplate::new(
            "Limited Task".to_string(),
            "Task with max occurrences".to_string(),
            rule,
        );

        assert!(template.generate_task().is_some());
        assert!(template.generate_task().is_some());
        assert!(template.generate_task().is_some());
        assert!(template.generate_task().is_none());
        assert!(!template.active);
    }

    #[test]
    fn test_end_date() {
        let rule = RecurrenceRule {
            pattern: RecurrencePattern::Daily,
            interval: 1,
            days_of_week: vec![],
            day_of_month: None,
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_date: Some(Utc::now().date_naive() - Duration::days(1)),
            max_occurrences: None,
            occurrences_count: 0,
        };

        let mut template = RecurringTaskTemplate::new(
            "Expired Task".to_string(),
            "Task with past end date".to_string(),
            rule,
        );

        assert!(template.generate_task().is_none());
        assert!(!template.active);
    }

    #[test]
    fn test_deactivate_reactivate() {
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

        let mut template =
            RecurringTaskTemplate::new("Task".to_string(), "Description".to_string(), rule);

        assert!(template.active);
        template.deactivate();
        assert!(!template.active);
        assert!(template.generate_task().is_none());

        template.reactivate();
        assert!(template.active);
        assert!(template.generate_task().is_some());
    }

    #[test]
    fn test_yearly_recurrence() {
        let rule = RecurrenceRule {
            pattern: RecurrencePattern::Yearly,
            interval: 1,
            days_of_week: vec![],
            day_of_month: Some(25),
            month_of_year: Some(12),
            time_of_day: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: None,
            occurrences_count: 0,
        };

        let template = RecurringTaskTemplate::new(
            "Annual Review".to_string(),
            "Yearly performance review".to_string(),
            rule,
        );

        assert!(template.next_occurrence.is_some());
    }

    #[test]
    fn test_date_helpers() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 31, 12, 0, 0).unwrap();

        // Test add_months with day overflow
        let next = add_months(dt, 1);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 29); // February has fewer days

        // Test add_years
        let next_year = add_years(dt, 1);
        assert_eq!(next_year.year(), 2025);
        assert_eq!(next_year.month(), 1);
        assert_eq!(next_year.day(), 31);

        // Test leap year handling
        let leap_day = Utc.with_ymd_and_hms(2024, 2, 29, 12, 0, 0).unwrap();
        let non_leap = add_years(leap_day, 1);
        assert_eq!(non_leap.year(), 2025);
        assert_eq!(non_leap.month(), 2);
        assert_eq!(non_leap.day(), 28);
    }
}
