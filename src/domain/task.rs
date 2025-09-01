use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String, // Markdown content
    pub status: TaskStatus,
    pub priority: Priority,
    pub metadata: HashMap<String, String>,
    pub tags: HashSet<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub assigned_resource_id: Option<Uuid>,
    pub goal_id: Option<Uuid>,
    pub parent_task_id: Option<Uuid>,
    pub position: Position, // For map view
    pub subtasks: Vec<SubTask>,
    pub is_archived: bool,
    pub assignee: Option<String>,
    pub configuration_id: Option<Uuid>, // Link to task configuration
    pub sort_order: i32, // For ordering within Kanban columns
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubTask {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Blocked,
    Review,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for Task {
    fn default() -> Self {
        Self::new("".to_string(), "".to_string())
    }
}

impl Task {
    pub fn new(title: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            metadata: HashMap::new(),
            tags: HashSet::new(),
            created_at: now,
            updated_at: now,
            due_date: None,
            scheduled_date: None,
            completed_at: None,
            estimated_hours: None,
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            position: Position { x: 0.0, y: 0.0 },
            subtasks: Vec::new(),
            is_archived: false,
            assignee: None,
            configuration_id: None,
            sort_order: 0,
        }
    }

    pub fn new_simple(title: String) -> Self {
        Self::new(title, String::new())
    }

    pub fn add_subtask(&mut self, description: String) -> Uuid {
        let subtask = SubTask {
            id: Uuid::new_v4(),
            title: description.clone(),
            description,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
        };
        let id = subtask.id;
        self.subtasks.push(subtask);
        self.updated_at = Utc::now();
        id
    }

    pub fn complete_subtask(&mut self, subtask_id: Uuid) -> Result<(), String> {
        if let Some(subtask) = self.subtasks.iter_mut().find(|s| s.id == subtask_id) {
            if !subtask.completed {
                subtask.completed = true;
                subtask.completed_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            } else {
                Err("Subtask already completed".to_string())
            }
        } else {
            Err("Subtask not found".to_string())
        }
    }

    pub fn uncomplete_subtask(&mut self, subtask_id: Uuid) -> Result<(), String> {
        if let Some(subtask) = self.subtasks.iter_mut().find(|s| s.id == subtask_id) {
            if subtask.completed {
                subtask.completed = false;
                subtask.completed_at = None;
                self.updated_at = Utc::now();
                Ok(())
            } else {
                Err("Subtask not completed".to_string())
            }
        } else {
            Err("Subtask not found".to_string())
        }
    }

    pub fn subtask_progress(&self) -> (usize, usize) {
        let total = self.subtasks.len();
        let completed = self.subtasks.iter().filter(|s| s.completed).count();
        (completed, total)
    }

    pub fn extract_subtasks_from_markdown(&mut self) {
        let regex = regex::Regex::new(r"(?m)^- \[ \] (.+)$").unwrap();
        let description = self.description.clone();
        for cap in regex.captures_iter(&description) {
            if let Some(desc) = cap.get(1) {
                self.add_subtask(desc.as_str().to_string());
            }
        }
    }

    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now();

        if status == TaskStatus::Done {
            self.completed_at = Some(Utc::now());
        } else {
            self.completed_at = None;
        }
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        self.position = Position { x, y };
        self.updated_at = Utc::now();
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    pub fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
        self.updated_at = Utc::now();
    }

    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let removed = self.tags.remove(tag);
        if removed {
            self.updated_at = Utc::now();
        }
        removed
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            due < Utc::now() && self.status != TaskStatus::Done
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task() {
        let task = Task::new("Test Task".to_string(), "Description".to_string());
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "Description");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, Priority::Medium);
        assert!(task.metadata.is_empty());
        assert!(task.tags.is_empty());
        assert!(task.subtasks.is_empty());
    }

    #[test]
    fn test_add_subtask() {
        let mut task = Task::new("Main Task".to_string(), "".to_string());
        let id = task.add_subtask("Subtask 1".to_string());

        assert_eq!(task.subtasks.len(), 1);
        assert_eq!(task.subtasks[0].description, "Subtask 1");
        assert!(!task.subtasks[0].completed);
        assert_eq!(task.subtasks[0].id, id);
    }

    #[test]
    fn test_complete_subtask() {
        let mut task = Task::new("Main Task".to_string(), "".to_string());
        let id = task.add_subtask("Subtask 1".to_string());

        assert!(task.complete_subtask(id).is_ok());
        assert!(task.subtasks[0].completed);
        assert!(task.subtasks[0].completed_at.is_some());

        // Completing again should error
        assert!(task.complete_subtask(id).is_err());
    }

    #[test]
    fn test_uncomplete_subtask() {
        let mut task = Task::new("Main Task".to_string(), "".to_string());
        let id = task.add_subtask("Subtask 1".to_string());
        task.complete_subtask(id).unwrap();

        assert!(task.uncomplete_subtask(id).is_ok());
        assert!(!task.subtasks[0].completed);
        assert!(task.subtasks[0].completed_at.is_none());
    }

    #[test]
    fn test_subtask_progress() {
        let mut task = Task::new("Main Task".to_string(), "".to_string());
        let id1 = task.add_subtask("Subtask 1".to_string());
        let id2 = task.add_subtask("Subtask 2".to_string());
        task.add_subtask("Subtask 3".to_string());

        assert_eq!(task.subtask_progress(), (0, 3));

        task.complete_subtask(id1).unwrap();
        assert_eq!(task.subtask_progress(), (1, 3));

        task.complete_subtask(id2).unwrap();
        assert_eq!(task.subtask_progress(), (2, 3));
    }

    #[test]
    fn test_extract_subtasks_from_markdown() {
        let mut task = Task::new(
            "Task".to_string(),
            "# Task\n- [ ] First subtask\n- [ ] Second subtask\n- [x] Already done\nSome text\n- [ ] Third subtask".to_string()
        );

        task.extract_subtasks_from_markdown();
        assert_eq!(task.subtasks.len(), 3);
        assert_eq!(task.subtasks[0].description, "First subtask");
        assert_eq!(task.subtasks[1].description, "Second subtask");
        assert_eq!(task.subtasks[2].description, "Third subtask");
    }

    #[test]
    fn test_update_status() {
        let mut task = Task::new("Task".to_string(), "".to_string());
        assert_eq!(task.status, TaskStatus::Todo);
        assert!(task.completed_at.is_none());

        task.update_status(TaskStatus::InProgress);
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.completed_at.is_none());

        task.update_status(TaskStatus::Done);
        assert_eq!(task.status, TaskStatus::Done);
        assert!(task.completed_at.is_some());

        task.update_status(TaskStatus::Todo);
        assert_eq!(task.status, TaskStatus::Todo);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_metadata_operations() {
        let mut task = Task::new("Task".to_string(), "".to_string());

        task.add_metadata("category".to_string(), "infrastructure".to_string());
        task.add_metadata("team".to_string(), "backend".to_string());

        assert_eq!(
            task.metadata.get("category"),
            Some(&"infrastructure".to_string())
        );
        assert_eq!(task.metadata.get("team"), Some(&"backend".to_string()));
    }

    #[test]
    fn test_tag_operations() {
        let mut task = Task::new("Task".to_string(), "".to_string());

        task.add_tag("urgent".to_string());
        task.add_tag("bug".to_string());

        assert!(task.tags.contains("urgent"));
        assert!(task.tags.contains("bug"));

        assert!(task.remove_tag("urgent"));
        assert!(!task.tags.contains("urgent"));
        assert!(!task.remove_tag("nonexistent"));
    }

    #[test]
    fn test_is_overdue() {
        let mut task = Task::new("Task".to_string(), "".to_string());
        assert!(!task.is_overdue());

        task.due_date = Some(Utc::now() - chrono::Duration::days(1));
        assert!(task.is_overdue());

        task.update_status(TaskStatus::Done);
        assert!(!task.is_overdue());
    }

    #[test]
    fn test_position() {
        let mut task = Task::new("Task".to_string(), "".to_string());
        assert_eq!(task.position.x, 0.0);
        assert_eq!(task.position.y, 0.0);

        task.set_position(100.5, 200.3);
        assert_eq!(task.position.x, 100.5);
        assert_eq!(task.position.y, 200.3);
    }
}
