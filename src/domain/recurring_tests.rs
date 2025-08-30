#[cfg(test)]
mod tests {
    use super::super::recurring::*;
    use chrono::{Datelike, Duration, NaiveTime, Utc, Weekday};

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
            rule,
        );

        assert_eq!(template.title, "Daily standup");
        assert_eq!(template.description, "Team sync meeting");
        assert!(template.active);
    }

    #[test]
    fn test_daily_recurrence() {
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

        let template = RecurringTaskTemplate::new("Daily task".to_string(), "".to_string(), rule);

        let next = template.calculate_next_occurrence();
        let now = Utc::now();

        // Should be scheduled for today at 10am if before 10am, otherwise tomorrow
        if now.time() < NaiveTime::from_hms_opt(10, 0, 0).unwrap() {
            assert_eq!(next.date_naive(), now.date_naive());
        } else {
            assert_eq!(next.date_naive(), (now + Duration::days(1)).date_naive());
        }
    }

    #[test]
    fn test_weekly_recurrence_with_specific_days() {
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

        let template =
            RecurringTaskTemplate::new("Weekly meeting".to_string(), "".to_string(), rule);

        let next_date = template.calculate_next_occurrence();

        // Should be one of the specified weekdays
        let weekday = next_date.weekday();
        assert!(
            weekday == Weekday::Mon || weekday == Weekday::Wed || weekday == Weekday::Fri,
            "Next occurrence should be on Mon, Wed, or Fri, got {:?}",
            weekday
        );
    }

    #[test]
    fn test_monthly_recurrence() {
        let rule = RecurrenceRule {
            pattern: RecurrencePattern::Monthly,
            interval: 1,
            days_of_week: vec![],
            day_of_month: Some(15),
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: None,
            occurrences_count: 0,
        };

        let template =
            RecurringTaskTemplate::new("Monthly report".to_string(), "".to_string(), rule);

        let next_date = template.calculate_next_occurrence();
        assert_eq!(next_date.date_naive().day(), 15);
    }

    #[test]
    fn test_generate_task_from_template() {
        let rule = RecurrenceRule {
            pattern: RecurrencePattern::Weekly,
            interval: 1,
            days_of_week: vec![],
            day_of_month: None,
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: None,
            occurrences_count: 0,
        };

        let mut template = RecurringTaskTemplate::new(
            "Review code".to_string(),
            "Weekly code review session".to_string(),
            rule,
        );

        let task = template.generate_task();
        assert!(task.is_some());

        let task = task.unwrap();
        assert_eq!(task.title, "Review code");
        assert_eq!(task.description, "Weekly code review session");
        assert!(task.scheduled_date.is_some());
    }

    #[test]
    fn test_max_occurrences_limit() {
        let rule1 = RecurrenceRule {
            pattern: RecurrencePattern::Daily,
            interval: 1,
            days_of_week: vec![],
            day_of_month: None,
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: Some(5),
            occurrences_count: 4,
        };

        let template1 =
            RecurringTaskTemplate::new("Limited task".to_string(), "".to_string(), rule1);

        assert!(template1.should_generate_now());

        let rule2 = RecurrenceRule {
            pattern: RecurrencePattern::Daily,
            interval: 1,
            days_of_week: vec![],
            day_of_month: None,
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: Some(5),
            occurrences_count: 5,
        };

        let template2 =
            RecurringTaskTemplate::new("Limited task".to_string(), "".to_string(), rule2);
        assert!(!template2.should_generate_now());
    }

    #[test]
    fn test_end_date_check() {
        let rule1 = RecurrenceRule {
            pattern: RecurrencePattern::Daily,
            interval: 1,
            days_of_week: vec![],
            day_of_month: None,
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_date: Some((Utc::now() - Duration::days(1)).date_naive()),
            max_occurrences: None,
            occurrences_count: 0,
        };

        let template1 =
            RecurringTaskTemplate::new("Expired task".to_string(), "".to_string(), rule1);
        assert!(!template1.should_generate_now());

        let rule2 = RecurrenceRule {
            pattern: RecurrencePattern::Daily,
            interval: 1,
            days_of_week: vec![],
            day_of_month: None,
            month_of_year: None,
            time_of_day: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_date: Some((Utc::now() + Duration::days(30)).date_naive()),
            max_occurrences: None,
            occurrences_count: 0,
        };

        let template2 =
            RecurringTaskTemplate::new("Active task".to_string(), "".to_string(), rule2);
        assert!(template2.should_generate_now());
    }

    #[test]
    fn test_deactivate_template() {
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

        let mut template = RecurringTaskTemplate::new("Task".to_string(), "".to_string(), rule);

        assert!(template.active);
        template.deactivate();
        assert!(!template.active);
        assert!(!template.should_generate_now());
    }
}
