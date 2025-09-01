use super::task::{Task, TaskStatus};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptyTitle,
    InvalidPosition { x: f64, y: f64 },
    InvalidStatusTransition { from: TaskStatus, to: TaskStatus },
    InconsistentDates { field: String, reason: String },
    InvalidCompletedState,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyTitle => write!(f, "Task title cannot be empty"),
            ValidationError::InvalidPosition { x, y } => {
                write!(f, "Invalid position: x={}, y={} (must be finite numbers)", x, y)
            }
            ValidationError::InvalidStatusTransition { from, to } => {
                write!(f, "Invalid status transition from {:?} to {:?}", from, to)
            }
            ValidationError::InconsistentDates { field, reason } => {
                write!(f, "Date validation failed for {}: {}", field, reason)
            }
            ValidationError::InvalidCompletedState => {
                write!(f, "completed_at should only be set when status is Done")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

pub struct TaskValidator;

impl TaskValidator {
    /// Validate a task before creation
    pub fn validate_new(task: &Task) -> Result<(), ValidationError> {
        Self::validate_title(task)?;
        Self::validate_position(task)?;
        Self::validate_dates_for_new(task)?;
        Self::validate_completed_state(task)?;
        Ok(())
    }
    
    /// Validate a task before update
    pub fn validate_update(old: &Task, new: &Task) -> Result<(), ValidationError> {
        Self::validate_title(new)?;
        Self::validate_position(new)?;
        Self::validate_status_transition(old, new)?;
        Self::validate_dates_for_update(old, new)?;
        Self::validate_completed_state(new)?;
        Ok(())
    }
    
    fn validate_title(task: &Task) -> Result<(), ValidationError> {
        if task.title.trim().is_empty() {
            return Err(ValidationError::EmptyTitle);
        }
        Ok(())
    }
    
    fn validate_position(task: &Task) -> Result<(), ValidationError> {
        if !task.position.x.is_finite() || !task.position.y.is_finite() {
            return Err(ValidationError::InvalidPosition {
                x: task.position.x,
                y: task.position.y,
            });
        }
        Ok(())
    }
    
    fn validate_status_transition(old: &Task, new: &Task) -> Result<(), ValidationError> {
        use TaskStatus::*;
        
        let valid_transition = match (old.status, new.status) {
            // Same status is always valid
            (from, to) if from == to => true,
            
            // From Todo - can go directly to Done for simple tasks
            (Todo, InProgress) | (Todo, Blocked) | (Todo, Done) | (Todo, Cancelled) => true,
            
            // From InProgress
            (InProgress, Todo) | (InProgress, Blocked) | (InProgress, Review) | 
            (InProgress, Done) | (InProgress, Cancelled) => true,
            
            // From Blocked
            (Blocked, Todo) | (Blocked, InProgress) | (Blocked, Cancelled) => true,
            
            // From Review
            (Review, InProgress) | (Review, Done) | (Review, Cancelled) => true,
            
            // From Done - allow reopening to Todo/InProgress, or moving to Review/Cancelled
            (Done, Todo) | (Done, InProgress) | (Done, Review) | (Done, Cancelled) => true,
            
            // From Cancelled - no transitions allowed
            (Cancelled, _) => false,
            
            // All other transitions are invalid
            _ => false,
        };
        
        if !valid_transition {
            return Err(ValidationError::InvalidStatusTransition {
                from: old.status,
                to: new.status,
            });
        }
        
        Ok(())
    }
    
    fn validate_dates_for_new(task: &Task) -> Result<(), ValidationError> {
        // Check if completed_at is set without Done status
        if task.completed_at.is_some() && task.status != TaskStatus::Done {
            return Err(ValidationError::InvalidCompletedState);
        }
        
        // Check if completed_at is before created_at
        if let Some(completed) = task.completed_at {
            if completed < task.created_at {
                return Err(ValidationError::InconsistentDates {
                    field: "completed_at".to_string(),
                    reason: "Cannot be before created_at".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_dates_for_update(old: &Task, new: &Task) -> Result<(), ValidationError> {
        // Don't allow changing created_at
        if new.created_at != old.created_at {
            return Err(ValidationError::InconsistentDates {
                field: "created_at".to_string(),
                reason: "Cannot modify creation date".to_string(),
            });
        }
        
        Self::validate_dates_for_new(new)?;
        
        // When transitioning to Done, ensure completed_at is set
        if old.status != TaskStatus::Done && new.status == TaskStatus::Done {
            if new.completed_at.is_none() {
                // We'll set it automatically in the normalize function
            }
        }
        
        Ok(())
    }
    
    fn validate_completed_state(task: &Task) -> Result<(), ValidationError> {
        match (task.status, task.completed_at) {
            (TaskStatus::Done, None) => {
                // Done tasks should have completed_at, but we can set it automatically
                Ok(())
            }
            (TaskStatus::Done, Some(_)) => Ok(()),
            (_, Some(_)) => Err(ValidationError::InvalidCompletedState),
            (_, None) => Ok(()),
        }
    }
    
    /// Normalize a task to ensure consistency
    pub fn normalize(task: &mut Task) {
        // Ensure finite positions
        if !task.position.x.is_finite() {
            task.position.x = 0.0;
        }
        if !task.position.y.is_finite() {
            task.position.y = 0.0;
        }
        
        // Trim whitespace from strings
        task.title = task.title.trim().to_string();
        task.description = task.description.trim().to_string();
        
        // Set completed_at when status becomes Done
        if task.status == TaskStatus::Done && task.completed_at.is_none() {
            task.completed_at = Some(Utc::now());
        }
        
        // Clear completed_at if status is not Done
        if task.status != TaskStatus::Done && task.completed_at.is_some() {
            task.completed_at = None;
        }
        
        // Update updated_at
        task.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Priority, Position};
    
    #[test]
    fn test_empty_title_validation() {
        let task = Task::new("".to_string(), "Description".to_string());
        let result = TaskValidator::validate_new(&task);
        assert!(matches!(result, Err(ValidationError::EmptyTitle)));
        
        let task_with_spaces = Task::new("   ".to_string(), "Description".to_string());
        let result = TaskValidator::validate_new(&task_with_spaces);
        assert!(matches!(result, Err(ValidationError::EmptyTitle)));
    }
    
    #[test]
    fn test_invalid_position_validation() {
        let mut task = Task::new("Valid Title".to_string(), "".to_string());
        
        task.position.x = f64::INFINITY;
        let result = TaskValidator::validate_new(&task);
        assert!(matches!(result, Err(ValidationError::InvalidPosition { .. })));
        
        task.position.x = 0.0;
        task.position.y = f64::NAN;
        let result = TaskValidator::validate_new(&task);
        assert!(matches!(result, Err(ValidationError::InvalidPosition { .. })));
    }
    
    #[test]
    fn test_status_transition_validation() {
        let mut old_task = Task::new("Task".to_string(), "".to_string());
        old_task.status = TaskStatus::Cancelled;
        
        let mut new_task = old_task.clone();
        new_task.status = TaskStatus::InProgress;
        
        let result = TaskValidator::validate_update(&old_task, &new_task);
        assert!(matches!(result, Err(ValidationError::InvalidStatusTransition { .. })));
        
        // Valid transition
        old_task.status = TaskStatus::Todo;
        new_task.status = TaskStatus::InProgress;
        let result = TaskValidator::validate_update(&old_task, &new_task);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_normalize() {
        let mut task = Task::new("  Title with spaces  ".to_string(), "  Description  ".to_string());
        task.position.x = f64::INFINITY;
        task.position.y = f64::NAN;
        task.status = TaskStatus::Done;
        
        TaskValidator::normalize(&mut task);
        
        assert_eq!(task.title, "Title with spaces");
        assert_eq!(task.description, "Description");
        assert_eq!(task.position.x, 0.0);
        assert_eq!(task.position.y, 0.0);
        assert!(task.completed_at.is_some());
    }
}