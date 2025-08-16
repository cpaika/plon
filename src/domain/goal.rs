use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use crate::domain::task::Task;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Goal {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: GoalStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub target_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub task_ids: HashSet<Uuid>,
    pub subgoal_ids: HashSet<Uuid>,
    pub parent_goal_id: Option<Uuid>,
    pub estimated_hours: Option<f32>,
    pub progress: f32,
    pub position_x: f64,
    pub position_y: f64,
    pub position_width: f64,
    pub position_height: f64,
    pub color: String, // For UI visualization
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoalPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GoalStatus {
    NotStarted,
    Active,
    InProgress,
    OnHold,
    AtRisk,
    Completed,
    Cancelled,
}

impl Goal {
    pub fn new(title: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: GoalStatus::Active,
            created_at: now,
            updated_at: now,
            target_date: None,
            completed_at: None,
            task_ids: HashSet::new(),
            subgoal_ids: HashSet::new(),
            parent_goal_id: None,
            estimated_hours: None,
            progress: 0.0,
            position_x: 0.0,
            position_y: 0.0,
            position_width: 200.0,
            position_height: 150.0,
            color: "#4A90E2".to_string(),
        }
    }

    pub fn add_task(&mut self, task_id: Uuid) {
        self.task_ids.insert(task_id);
        self.updated_at = Utc::now();
    }

    pub fn remove_task(&mut self, task_id: &Uuid) -> bool {
        let removed = self.task_ids.remove(task_id);
        if removed {
            self.updated_at = Utc::now();
        }
        removed
    }

    pub fn update_status(&mut self, status: GoalStatus) {
        self.status = status;
        self.updated_at = Utc::now();
        
        if status == GoalStatus::Completed {
            self.completed_at = Some(Utc::now());
        } else {
            self.completed_at = None;
        }
    }

    pub fn add_subgoal(&mut self, subgoal_id: Uuid) {
        self.subgoal_ids.insert(subgoal_id);
        self.updated_at = Utc::now();
    }
    
    pub fn update_progress(&mut self, tasks: &[Task]) {
        if self.task_ids.is_empty() {
            self.progress = 0.0;
            return;
        }
        
        let mut completed_weight = 0.0;
        let mut total_weight = 0.0;
        
        for task in tasks {
            if self.task_ids.contains(&task.id) {
                let weight = 1.0; // Could be based on estimated hours or priority
                total_weight += weight;
                
                if task.status == crate::domain::task::TaskStatus::Done {
                    completed_weight += weight;
                } else if !task.subtasks.is_empty() {
                    let (completed, total) = task.subtask_progress();
                    let partial_progress = completed as f32 / total as f32;
                    completed_weight += weight * partial_progress;
                }
            }
        }
        
        if total_weight > 0.0 {
            self.progress = (completed_weight / total_weight * 100.0).round() * 100.0 / 100.0;
        } else {
            self.progress = 0.0;
        }
    }
    
    pub fn calculate_hours(&self, tasks: &[Task]) -> (f32, f32) {
        let mut estimated_total = 0.0;
        let mut actual_total = 0.0;
        
        for task in tasks {
            if self.task_ids.contains(&task.id) {
                if let Some(est) = task.estimated_hours {
                    estimated_total += est;
                }
                if let Some(act) = task.actual_hours {
                    actual_total += act;
                }
            }
        }
        
        (estimated_total, actual_total)
    }
    
    pub fn is_complete(&self) -> bool {
        self.status == GoalStatus::Completed
    }
    
    pub fn is_overdue(&self) -> bool {
        if let Some(target) = self.target_date {
            target < Utc::now() && self.status != GoalStatus::Completed
        } else {
            false
        }
    }
    
    pub fn set_position(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.position_x = x;
        self.position_y = y;
        self.position_width = width;
        self.position_height = height;
        self.updated_at = Utc::now();
    }

    pub fn is_at_risk(&self) -> bool {
        if let Some(target) = self.target_date {
            let days_remaining = (target - Utc::now()).num_days();
            days_remaining < 7 && self.status != GoalStatus::Completed
        } else {
            false
        }
    }

    pub fn calculate_progress(&self, task_statuses: &[(Uuid, bool)]) -> f32 {
        if self.task_ids.is_empty() {
            return 0.0;
        }

        let completed_count = task_statuses
            .iter()
            .filter(|(id, completed)| self.task_ids.contains(id) && *completed)
            .count();

        (completed_count as f32 / self.task_ids.len() as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_goal() {
        let goal = Goal::new("Q1 Goals".to_string(), "Goals for Q1".to_string());
        assert_eq!(goal.title, "Q1 Goals");
        assert_eq!(goal.description, "Goals for Q1");
        assert_eq!(goal.status, GoalStatus::Active);
        assert!(goal.task_ids.is_empty());
        assert_eq!(goal.color, "#4A90E2");
    }

    #[test]
    fn test_add_remove_tasks() {
        let mut goal = Goal::new("Goal".to_string(), "".to_string());
        let task_id1 = Uuid::new_v4();
        let task_id2 = Uuid::new_v4();
        
        goal.add_task(task_id1);
        assert!(goal.task_ids.contains(&task_id1));
        assert_eq!(goal.status, GoalStatus::Active);
        
        goal.add_task(task_id2);
        assert_eq!(goal.task_ids.len(), 2);
        
        assert!(goal.remove_task(&task_id1));
        assert!(!goal.task_ids.contains(&task_id1));
        assert_eq!(goal.task_ids.len(), 1);
        
        assert!(!goal.remove_task(&task_id1));
    }

    #[test]
    fn test_update_status() {
        let mut goal = Goal::new("Goal".to_string(), "".to_string());
        assert_eq!(goal.status, GoalStatus::Active);
        assert!(goal.completed_at.is_none());
        
        goal.update_status(GoalStatus::InProgress);
        assert_eq!(goal.status, GoalStatus::InProgress);
        
        goal.update_status(GoalStatus::Completed);
        assert_eq!(goal.status, GoalStatus::Completed);
        assert!(goal.completed_at.is_some());
        
        goal.update_status(GoalStatus::AtRisk);
        assert_eq!(goal.status, GoalStatus::AtRisk);
        assert!(goal.completed_at.is_none());
    }

    #[test]
    fn test_is_at_risk() {
        let mut goal = Goal::new("Goal".to_string(), "".to_string());
        assert!(!goal.is_at_risk());
        
        goal.target_date = Some(Utc::now() + chrono::Duration::days(5));
        assert!(goal.is_at_risk());
        
        goal.target_date = Some(Utc::now() + chrono::Duration::days(10));
        assert!(!goal.is_at_risk());
        
        goal.target_date = Some(Utc::now() + chrono::Duration::days(5));
        goal.update_status(GoalStatus::Completed);
        assert!(!goal.is_at_risk());
    }

    #[test]
    fn test_calculate_progress() {
        let mut goal = Goal::new("Goal".to_string(), "".to_string());
        let task_id1 = Uuid::new_v4();
        let task_id2 = Uuid::new_v4();
        let task_id3 = Uuid::new_v4();
        
        goal.add_task(task_id1);
        goal.add_task(task_id2);
        goal.add_task(task_id3);
        
        let statuses = vec![
            (task_id1, true),
            (task_id2, false),
            (task_id3, true),
        ];
        
        let progress = goal.calculate_progress(&statuses);
        assert!((progress - 66.66667).abs() < 0.001);
        
        let all_complete = vec![
            (task_id1, true),
            (task_id2, true),
            (task_id3, true),
        ];
        assert_eq!(goal.calculate_progress(&all_complete), 100.0);
        
        let none_complete = vec![
            (task_id1, false),
            (task_id2, false),
            (task_id3, false),
        ];
        assert_eq!(goal.calculate_progress(&none_complete), 0.0);
    }

    #[test]
    fn test_set_position() {
        let mut goal = Goal::new("Goal".to_string(), "".to_string());
        goal.set_position(100.0, 200.0, 300.0, 150.0);
        
        assert_eq!(goal.position_x, 100.0);
        assert_eq!(goal.position_y, 200.0);
        assert_eq!(goal.position_width, 300.0);
        assert_eq!(goal.position_height, 150.0);
    }
}