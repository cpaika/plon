#[cfg(test)]
mod tests {
    use crate::domain::task::{Task, TaskStatus, Priority};
    use uuid::Uuid;
    use chrono::{Utc, Duration};
    use std::collections::HashSet;
    
    fn create_test_task(title: &str, priority: Priority, status: TaskStatus, days_offset: i64) -> Task {
        Task {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: "".to_string(),
            status,
            priority,
            created_at: Utc::now() + Duration::days(days_offset),
            updated_at: Utc::now(),
            due_date: if days_offset != 0 { 
                Some(Utc::now() + Duration::days(days_offset)) 
            } else { 
                None 
            },
            estimated_hours: None,
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            tags: HashSet::new(),
            assignee: None,
            position: crate::domain::task::Position { x: 0.0, y: 0.0 },
            scheduled_date: None,
            completed_at: None,
            metadata: std::collections::HashMap::new(),
            subtasks: vec![],
            is_archived: false,
            configuration_id: None,
            sort_order: 0,        }
    }
    
    #[test]
    fn test_sort_by_priority_descending() {
        let mut tasks = vec![
            create_test_task("Low task", Priority::Low, TaskStatus::Todo, 0),
            create_test_task("Critical task", Priority::Critical, TaskStatus::Todo, 0),
            create_test_task("Medium task", Priority::Medium, TaskStatus::Todo, 0),
            create_test_task("High task", Priority::High, TaskStatus::Todo, 0),
        ];
        
        // Sort by priority descending (Critical -> High -> Medium -> Low)
        tasks.sort_by(|a, b| {
            let a_priority = match a.priority {
                Priority::Critical => 4,
                Priority::High => 3,
                Priority::Medium => 2,
                Priority::Low => 1,
            };
            let b_priority = match b.priority {
                Priority::Critical => 4,
                Priority::High => 3,
                Priority::Medium => 2,
                Priority::Low => 1,
            };
            b_priority.cmp(&a_priority)
        });
        
        assert_eq!(tasks[0].title, "Critical task");
        assert_eq!(tasks[1].title, "High task");
        assert_eq!(tasks[2].title, "Medium task");
        assert_eq!(tasks[3].title, "Low task");
    }
    
    #[test]
    fn test_sort_by_due_date_ascending() {
        let mut tasks = vec![
            create_test_task("Task 3", Priority::Medium, TaskStatus::Todo, 3),
            create_test_task("Task 1", Priority::Medium, TaskStatus::Todo, 1),
            create_test_task("Task no due", Priority::Medium, TaskStatus::Todo, 0),
            create_test_task("Task 2", Priority::Medium, TaskStatus::Todo, 2),
        ];
        
        // Sort by due date ascending (earliest first, None last)
        tasks.sort_by(|a, b| {
            match (a.due_date, b.due_date) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(a_due), Some(b_due)) => a_due.cmp(&b_due),
            }
        });
        
        assert_eq!(tasks[0].title, "Task 1");
        assert_eq!(tasks[1].title, "Task 2");
        assert_eq!(tasks[2].title, "Task 3");
        assert_eq!(tasks[3].title, "Task no due");
    }
    
    #[test]
    fn test_sort_by_status() {
        let mut tasks = vec![
            create_test_task("Done task", Priority::Medium, TaskStatus::Done, 0),
            create_test_task("Todo task", Priority::Medium, TaskStatus::Todo, 0),
            create_test_task("Blocked task", Priority::Medium, TaskStatus::Blocked, 0),
            create_test_task("In Progress task", Priority::Medium, TaskStatus::InProgress, 0),
            create_test_task("Review task", Priority::Medium, TaskStatus::Review, 0),
        ];
        
        // Sort by status (Todo -> InProgress -> Review -> Blocked -> Done)
        tasks.sort_by(|a, b| {
            let a_status = match a.status {
                TaskStatus::Todo => 1,
                TaskStatus::InProgress => 2,
                TaskStatus::Review => 3,
                TaskStatus::Blocked => 4,
                TaskStatus::Done => 5,
                TaskStatus::Cancelled => 6,
            };
            let b_status = match b.status {
                TaskStatus::Todo => 1,
                TaskStatus::InProgress => 2,
                TaskStatus::Review => 3,
                TaskStatus::Blocked => 4,
                TaskStatus::Done => 5,
                TaskStatus::Cancelled => 6,
            };
            a_status.cmp(&b_status)
        });
        
        assert_eq!(tasks[0].title, "Todo task");
        assert_eq!(tasks[1].title, "In Progress task");
        assert_eq!(tasks[2].title, "Review task");
        assert_eq!(tasks[3].title, "Blocked task");
        assert_eq!(tasks[4].title, "Done task");
    }
    
    #[test]
    fn test_sort_by_title_alphabetical() {
        let mut tasks = vec![
            create_test_task("Zebra", Priority::Medium, TaskStatus::Todo, 0),
            create_test_task("Apple", Priority::Medium, TaskStatus::Todo, 0),
            create_test_task("banana", Priority::Medium, TaskStatus::Todo, 0),
            create_test_task("Cherry", Priority::Medium, TaskStatus::Todo, 0),
        ];
        
        // Sort alphabetically (case-insensitive)
        tasks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        
        assert_eq!(tasks[0].title, "Apple");
        assert_eq!(tasks[1].title, "banana");
        assert_eq!(tasks[2].title, "Cherry");
        assert_eq!(tasks[3].title, "Zebra");
    }
}