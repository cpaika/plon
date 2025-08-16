use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub role: String,
    pub skills: HashSet<String>,
    pub metadata_filters: HashMap<String, String>, // e.g., "category" => "infrastructure"
    pub weekly_hours: f32,
    pub availability: Vec<Availability>,
    pub current_load: f32, // Current hours allocated
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Availability {
    pub date: NaiveDate,
    pub hours_available: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceAllocation {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub task_id: Uuid,
    pub hours_allocated: f32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

impl Resource {
    pub fn new(name: String, role: String, weekly_hours: f32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            email: None,
            role,
            skills: HashSet::new(),
            metadata_filters: HashMap::new(),
            weekly_hours,
            availability: Vec::new(),
            current_load: 0.0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_skill(&mut self, skill: String) {
        self.skills.insert(skill);
        self.updated_at = Utc::now();
    }

    pub fn add_metadata_filter(&mut self, key: String, value: String) {
        self.metadata_filters.insert(key, value);
        self.updated_at = Utc::now();
    }

    pub fn can_work_on_task(&self, task_metadata: &HashMap<String, String>) -> bool {
        if self.metadata_filters.is_empty() {
            return true; // No filters means can work on anything
        }

        for (key, value) in &self.metadata_filters {
            if let Some(task_value) = task_metadata.get(key) {
                if task_value == value {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_availability_for_week(&self, week_start: NaiveDate) -> f32 {
        let week_end = week_start + chrono::Duration::days(6);
        
        let custom_hours: f32 = self.availability
            .iter()
            .filter(|a| a.date >= week_start && a.date <= week_end)
            .map(|a| a.hours_available)
            .sum();

        if custom_hours > 0.0 {
            custom_hours
        } else {
            self.weekly_hours
        }
    }

    pub fn get_availability_for_date(&self, date: NaiveDate) -> f32 {
        self.availability
            .iter()
            .find(|a| a.date == date)
            .map(|a| a.hours_available)
            .unwrap_or_else(|| {
                // Default to weekly hours divided by 5 (working days)
                if date.weekday().num_days_from_monday() < 5 {
                    self.weekly_hours / 5.0
                } else {
                    0.0 // Weekends
                }
            })
    }

    pub fn set_availability(&mut self, date: NaiveDate, hours: f32) {
        if let Some(availability) = self.availability.iter_mut().find(|a| a.date == date) {
            availability.hours_available = hours;
        } else {
            self.availability.push(Availability {
                date,
                hours_available: hours,
            });
        }
        self.updated_at = Utc::now();
    }

    pub fn utilization_percentage(&self) -> f32 {
        if self.weekly_hours == 0.0 {
            return 0.0;
        }
        (self.current_load / self.weekly_hours) * 100.0
    }

    pub fn is_overloaded(&self) -> bool {
        self.current_load > self.weekly_hours
    }

    pub fn available_hours(&self) -> f32 {
        (self.weekly_hours - self.current_load).max(0.0)
    }
}

impl ResourceAllocation {
    pub fn new(resource_id: Uuid, task_id: Uuid, hours: f32, start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            resource_id,
            task_id,
            hours_allocated: hours,
            start_date: start,
            end_date: end,
            created_at: Utc::now(),
        }
    }

    pub fn duration_days(&self) -> i64 {
        (self.end_date - self.start_date).num_days() + 1
    }

    pub fn daily_hours(&self) -> f32 {
        self.hours_allocated / self.duration_days() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_resource() {
        let resource = Resource::new("John Doe".to_string(), "Developer".to_string(), 40.0);
        assert_eq!(resource.name, "John Doe");
        assert_eq!(resource.role, "Developer");
        assert_eq!(resource.weekly_hours, 40.0);
        assert_eq!(resource.current_load, 0.0);
        assert!(resource.skills.is_empty());
    }

    #[test]
    fn test_add_skill() {
        let mut resource = Resource::new("Jane".to_string(), "Engineer".to_string(), 40.0);
        resource.add_skill("Rust".to_string());
        resource.add_skill("Python".to_string());
        
        assert!(resource.skills.contains("Rust"));
        assert!(resource.skills.contains("Python"));
        assert_eq!(resource.skills.len(), 2);
    }

    #[test]
    fn test_metadata_filters() {
        let mut resource = Resource::new("Bob".to_string(), "DevOps".to_string(), 40.0);
        resource.add_metadata_filter("category".to_string(), "infrastructure".to_string());
        resource.add_metadata_filter("team".to_string(), "backend".to_string());
        
        let mut task_metadata = HashMap::new();
        task_metadata.insert("category".to_string(), "infrastructure".to_string());
        assert!(resource.can_work_on_task(&task_metadata));
        
        task_metadata.clear();
        task_metadata.insert("category".to_string(), "frontend".to_string());
        assert!(!resource.can_work_on_task(&task_metadata));
        
        task_metadata.insert("team".to_string(), "backend".to_string());
        assert!(resource.can_work_on_task(&task_metadata));
    }

    #[test]
    fn test_availability() {
        let mut resource = Resource::new("Alice".to_string(), "PM".to_string(), 40.0);
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        // Default availability for weekday
        assert_eq!(resource.get_availability_for_date(date), 8.0); // 40/5
        
        // Set custom availability
        resource.set_availability(date, 4.0);
        assert_eq!(resource.get_availability_for_date(date), 4.0);
        
        // Weekend should be 0 by default
        let weekend = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(); // Saturday
        assert_eq!(resource.get_availability_for_date(weekend), 0.0);
    }

    #[test]
    fn test_utilization() {
        let mut resource = Resource::new("Dev".to_string(), "Developer".to_string(), 40.0);
        
        assert_eq!(resource.utilization_percentage(), 0.0);
        assert_eq!(resource.available_hours(), 40.0);
        assert!(!resource.is_overloaded());
        
        resource.current_load = 30.0;
        assert_eq!(resource.utilization_percentage(), 75.0);
        assert_eq!(resource.available_hours(), 10.0);
        assert!(!resource.is_overloaded());
        
        resource.current_load = 50.0;
        assert_eq!(resource.utilization_percentage(), 125.0);
        assert_eq!(resource.available_hours(), 0.0);
        assert!(resource.is_overloaded());
    }

    #[test]
    fn test_resource_allocation() {
        let resource_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        
        let allocation = ResourceAllocation::new(resource_id, task_id, 20.0, start, end);
        
        assert_eq!(allocation.resource_id, resource_id);
        assert_eq!(allocation.task_id, task_id);
        assert_eq!(allocation.hours_allocated, 20.0);
        assert_eq!(allocation.duration_days(), 5);
        assert_eq!(allocation.daily_hours(), 4.0);
    }

    #[test]
    fn test_get_availability_for_week() {
        let mut resource = Resource::new("Dev".to_string(), "Developer".to_string(), 40.0);
        let week_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(); // Monday
        
        // Default weekly hours
        assert_eq!(resource.get_availability_for_week(week_start), 40.0);
        
        // Set custom hours for some days in the week
        resource.set_availability(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 6.0);
        resource.set_availability(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 6.0);
        resource.set_availability(NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 8.0);
        
        assert_eq!(resource.get_availability_for_week(week_start), 20.0);
    }
}