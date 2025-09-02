use crate::domain::task::{Task, TaskStatus};
use crate::repository::Repository;
use crate::repository::task_repository::TaskFilters;
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;

pub struct TaskDependencyService {
    repository: Arc<Repository>,
}

impl TaskDependencyService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }
    
    /// Check if a task can be completed based on its dependencies
    pub async fn can_complete_task(&self, task_id: Uuid) -> Result<bool> {
        let task = self.repository.tasks.get(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        
        // If task has a parent dependency, check if parent is complete
        if let Some(parent_id) = task.parent_task_id {
            let parent = self.repository.tasks.get(parent_id).await?
                .ok_or_else(|| anyhow::anyhow!("Parent task not found"))?;
            if parent.status != TaskStatus::Done && parent.status != TaskStatus::Cancelled {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Get all tasks that depend on a given task
    pub async fn get_dependent_tasks(&self, parent_task_id: Uuid) -> Result<Vec<Task>> {
        // Get all tasks
        let all_tasks = self.repository.tasks.list(TaskFilters {
            status: None,
            assigned_resource_id: None,
            goal_id: None,
            overdue: false,
            limit: None,
        }).await?;
        
        // Filter for tasks that depend on the parent
        let dependents: Vec<Task> = all_tasks
            .into_iter()
            .filter(|t| t.parent_task_id == Some(parent_task_id))
            .collect();
        
        Ok(dependents)
    }
    
    /// Unblock dependent tasks when a parent task is completed
    pub async fn unblock_dependent_tasks(&self, completed_task_id: Uuid) -> Result<Vec<Uuid>> {
        let dependent_tasks = self.get_dependent_tasks(completed_task_id).await?;
        let mut unblocked_ids = Vec::new();
        
        for mut task in dependent_tasks {
            if task.status == TaskStatus::Blocked {
                // Check if all other dependencies are also complete
                if self.can_complete_task(task.id).await? {
                    task.status = TaskStatus::Todo;
                    task.updated_at = chrono::Utc::now();
                    self.repository.tasks.update(&task).await?;
                    unblocked_ids.push(task.id);
                }
            }
        }
        
        Ok(unblocked_ids)
    }
    
    /// Set a task as blocked if it has incomplete dependencies
    pub async fn update_task_blocked_status(&self, task_id: Uuid) -> Result<bool> {
        let task = self.repository.tasks.get(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        
        if let Some(parent_id) = task.parent_task_id {
            let parent = self.repository.tasks.get(parent_id).await?
                .ok_or_else(|| anyhow::anyhow!("Parent task not found"))?;
            
            // If parent is not complete, task should be blocked
            if parent.status != TaskStatus::Done && parent.status != TaskStatus::Cancelled {
                if task.status != TaskStatus::Blocked {
                    let mut updated_task = task;
                    updated_task.status = TaskStatus::Blocked;
                    updated_task.updated_at = chrono::Utc::now();
                    self.repository.tasks.update(&updated_task).await?;
                    return Ok(true); // Task was blocked
                }
            }
        }
        
        Ok(false) // Task was not blocked
    }
    
    /// Create a dependency between two tasks
    pub async fn create_dependency(&self, dependent_task_id: Uuid, parent_task_id: Uuid) -> Result<()> {
        let mut dependent_task = self.repository.tasks.get(dependent_task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Dependent task not found"))?;
        
        // Prevent circular dependencies
        if self.would_create_cycle(dependent_task_id, parent_task_id).await? {
            return Err(anyhow::anyhow!("Cannot create circular dependency"));
        }
        
        dependent_task.parent_task_id = Some(parent_task_id);
        dependent_task.updated_at = chrono::Utc::now();
        
        // Check if task should be blocked
        let parent = self.repository.tasks.get(parent_task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Parent task not found"))?;
        if parent.status != TaskStatus::Done && parent.status != TaskStatus::Cancelled {
            dependent_task.status = TaskStatus::Blocked;
        }
        
        self.repository.tasks.update(&dependent_task).await?;
        Ok(())
    }
    
    /// Check if creating a dependency would create a cycle
    async fn would_create_cycle(&self, dependent_id: Uuid, parent_id: Uuid) -> Result<bool> {
        // Simple cycle detection: check if parent depends on dependent
        let mut current_id = Some(parent_id);
        let mut visited = std::collections::HashSet::new();
        
        while let Some(id) = current_id {
            if id == dependent_id {
                return Ok(true); // Cycle detected
            }
            
            if visited.contains(&id) {
                break; // Already visited, no cycle through this path
            }
            visited.insert(id);
            
            let task = self.repository.tasks.get(id).await?
                .ok_or_else(|| anyhow::anyhow!("Task not found in cycle check"))?;
            current_id = task.parent_task_id;
        }
        
        Ok(false)
    }
    
    /// Remove a dependency between tasks
    pub async fn remove_dependency(&self, task_id: Uuid) -> Result<()> {
        let mut task = self.repository.tasks.get(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        task.parent_task_id = None;
        
        // Unblock task if it was blocked only due to dependency
        if task.status == TaskStatus::Blocked {
            task.status = TaskStatus::Todo;
        }
        
        task.updated_at = chrono::Utc::now();
        self.repository.tasks.update(&task).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Task, Priority};
    use std::collections::HashSet;
    use sqlx::SqlitePool;
    
    async fn setup_test_service() -> TaskDependencyService {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let repo = Arc::new(Repository::new(pool));
        TaskDependencyService::new(repo)
    }
    
    #[tokio::test]
    async fn test_cycle_detection() {
        let service = setup_test_service().await;
        
        // Create three tasks
        let task_a = Task {
            id: Uuid::new_v4(),
            title: "Task A".to_string(),
            description: "".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            due_date: None,
            estimated_hours: None,
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            tags: HashSet::new(),
            assignee: None,
            position: crate::domain::task::Position { x: 0.0, y: 0.0 },
        };
        
        let task_b = Task {
            id: Uuid::new_v4(),
            title: "Task B".to_string(),
            parent_task_id: Some(task_a.id), // B depends on A
            ..task_a.clone()
        };
        
        service.repository.tasks.create(&task_a).await.unwrap();
        service.repository.tasks.create(&task_b).await.unwrap();
        
        // Try to make A depend on B (would create cycle)
        let result = service.create_dependency(task_a.id, task_b.id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("circular"));
    }
}