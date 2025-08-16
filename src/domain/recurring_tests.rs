#[cfg(test)]
mod tests {
    use super::super::recurring::*;
    use crate::domain::task::{Task, Priority};
    use chrono::{Utc, Duration, NaiveTime, Weekday};
    use uuid::Uuid;

    #[test]
    fn test_create_recurring_template() {
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
            "Daily standup".to_string(),
            "Team sync meeting".to_string(),
            rule.clone(),
        );

        assert_eq!(template.title, "Daily standup");
        assert_eq!(template.recurrence_rule.pattern, RecurrencePattern::Daily);
        assert!(template.active);
        assert_eq!(template.recurrence_rule.occurrences_count, 0);
    }

    #[test]
    fn test_daily_recurrence_next_occurrence() {
        let rule = RecurrenceRule {
            pattern: RecurrencePattern::Daily,
            interval: 1,
            days_of_week: vec![],
            day_of_month: None,
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: None,
            occurrences_count: 0,
        };
        
        let template = RecurringTaskTemplate::new(
            "Daily task".to_string(),
            "".to_string(),
            rule,
        );

        let next = template.calculate_next_occurrence();
        let now = Utc::now();
        
        // Should be scheduled for today at 10am if before 10am, otherwise tomorrow
        if now.time() < NaiveTime::from_hms_opt(10, 0, 0).unwrap() {
            assert_eq!(next_date.date_naive(), now.date_naive());
        } else {
            assert_eq!(next_date.date_naive(), (now + Duration::days(1)).date_naive());
        }
    }

    #[test]
    fn test_weekly_recurrence_with_specific_days() {
        let mut template = RecurringTaskTemplate::new(
            "Weekly meeting".to_string(),
            "".to_string(),
            RecurrencePattern::Weekly,
            1,
            NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
        );
        
        template.days_of_week = Some(vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
        
        let next = template.calculate_next_occurrence();
        assert!(next.is_some());
        
        let next_date = next.unwrap();
        let weekday = next_date.weekday();
        
        assert!(
            weekday == Weekday::Mon || 
            weekday == Weekday::Wed || 
            weekday == Weekday::Fri
        );
    }

    #[test]
    fn test_monthly_recurrence() {
        let mut template = RecurringTaskTemplate::new(
            "Monthly report".to_string(),
            "".to_string(),
            RecurrencePattern::Monthly,
            1,
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        );
        
        template.day_of_month = Some(15);
        
        let next = template.calculate_next_occurrence();
        assert!(next.is_some());
        
        let next_date = next.unwrap();
        assert_eq!(next_date.day(), 15);
    }

    #[test]
    fn test_generate_task_from_template() {
        let template = RecurringTaskTemplate::new(
            "Review code".to_string(),
            "Weekly code review session".to_string(),
            RecurrencePattern::Weekly,
            1,
            NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
        );
        
        let task = template.generate_task();
        
        assert_eq!(task.title, "Review code");
        assert_eq!(task.description, "Weekly code review session");
        assert!(task.scheduled_date.is_some());
    }

    #[test]
    fn test_max_occurrences_limit() {
        let mut template = RecurringTaskTemplate::new(
            "Limited task".to_string(),
            "".to_string(),
            RecurrencePattern::Daily,
            1,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        );
        
        template.max_occurrences = Some(5);
        template.occurrences_count = 4;
        
        assert!(template.should_generate());
        
        template.occurrences_count = 5;
        assert!(!template.should_generate());
    }

    #[test]
    fn test_end_date_limit() {
        let mut template = RecurringTaskTemplate::new(
            "Time-limited task".to_string(),
            "".to_string(),
            RecurrencePattern::Daily,
            1,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        );
        
        template.end_date = Some(Utc::now() - Duration::days(1));
        assert!(!template.should_generate());
        
        template.end_date = Some(Utc::now() + Duration::days(30));
        assert!(template.should_generate());
    }

    #[test]
    fn test_deactivate_template() {
        let mut template = RecurringTaskTemplate::new(
            "Task".to_string(),
            "".to_string(),
            RecurrencePattern::Daily,
            1,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        );
        
        assert!(template.active);
        template.deactivate();
        assert!(!template.active);
        assert!(!template.should_generate());
    }
}