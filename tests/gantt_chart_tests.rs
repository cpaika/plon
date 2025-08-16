use plon::ui::widgets::gantt_chart::{GanttChart, GanttColor, Milestone};
use plon::ui::views::gantt_view::GanttView;
use plon::domain::{
    task::{Task, TaskStatus, Priority},
    resource::Resource,
    dependency::{Dependency, DependencyType},
};
use chrono::{NaiveDate, Duration, Local, Utc};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

#[cfg(test)]
mod gantt_chart_widget_tests {
    use super::*;

    #[test]
    fn test_gantt_chart_creation() {
        let chart = GanttChart::new();
        assert_eq!(chart.zoom_level, 1.0);
        assert_eq!(chart.days_to_show, 30);
        assert!(chart.show_dependencies);
        assert!(chart.show_resources);
    }

    #[test]
    fn test_gantt_chart_zoom() {
        let mut chart = GanttChart::new();
        
        // Test zoom in
        let initial_zoom = chart.zoom_level;
        chart.zoom_in();
        assert!(chart.zoom_level > initial_zoom);
        assert!(chart.zoom_level <= 3.0);
        
        // Test zoom out
        chart.zoom_out();
        assert_eq!(chart.zoom_level, initial_zoom);
        
        // Test zoom limits
        for _ in 0..10 {
            chart.zoom_in();
        }
        assert_eq!(chart.zoom_level, 3.0);
        
        for _ in 0..20 {
            chart.zoom_out();
        }
        assert_eq!(chart.zoom_level, 0.3);
    }

    #[test]
    fn test_gantt_chart_date_range() {
        let mut chart = GanttChart::new();
        
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        chart.set_start_date(start_date);
        assert_eq!(chart.get_start_date(), start_date);
        
        chart.set_days_to_show(60);
        let expected_end = start_date + Duration::days(59);
        assert_eq!(chart.get_end_date(), expected_end);
    }
}