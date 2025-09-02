use crate::repository::Repository;
use crate::domain::task::TaskStatus;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TimeEntry {
    pub id: Uuid,
    pub task_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

pub struct TimeTrackingService {
    repository: Arc<Repository>,
    // In-memory storage for time entries (in production, this would be in the database)
    time_entries: std::sync::Mutex<HashMap<Uuid, TimeEntry>>,
    // Track active time entries per task
    active_entries: std::sync::Mutex<HashMap<Uuid, Uuid>>, // task_id -> entry_id
}

impl TimeTrackingService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self {
            repository,
            time_entries: std::sync::Mutex::new(HashMap::new()),
            active_entries: std::sync::Mutex::new(HashMap::new()),
        }
    }
    
    /// Start tracking time for a task
    pub async fn start_tracking(&self, task_id: Uuid, description: String) -> Result<Uuid> {
        // Check if task exists
        let task = self.repository.tasks.get(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        
        // Check if already tracking this task
        let active = self.active_entries.lock().unwrap();
        if active.contains_key(&task_id) {
            return Err(anyhow::anyhow!("Already tracking time for this task"));
        }
        drop(active);
        
        // Create new time entry
        let entry = TimeEntry {
            id: Uuid::new_v4(),
            task_id,
            start_time: Utc::now(),
            end_time: None,
            description,
            created_at: Utc::now(),
        };
        
        let entry_id = entry.id;
        
        // Store entry
        self.time_entries.lock().unwrap().insert(entry_id, entry);
        self.active_entries.lock().unwrap().insert(task_id, entry_id);
        
        // Update task status to InProgress if it was Todo
        if task.status == TaskStatus::Todo {
            let mut updated_task = task;
            updated_task.status = TaskStatus::InProgress;
            updated_task.updated_at = Utc::now();
            self.repository.tasks.update(&updated_task).await?;
        }
        
        Ok(entry_id)
    }
    
    /// Stop tracking time for a task
    pub async fn stop_tracking(&self, task_id: Uuid) -> Result<Duration> {
        // Find active entry
        let entry_id = {
            let mut active = self.active_entries.lock().unwrap();
            active.remove(&task_id).ok_or_else(|| {
                anyhow::anyhow!("No active time tracking for this task")
            })?
        };
        
        // Update entry with end time
        let mut entries = self.time_entries.lock().unwrap();
        let entry = entries.get_mut(&entry_id).ok_or_else(|| {
            anyhow::anyhow!("Time entry not found")
        })?;
        
        entry.end_time = Some(Utc::now());
        let duration = entry.end_time.unwrap() - entry.start_time;
        
        // Update task's actual hours
        let mut task = self.repository.tasks.get(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        let hours_tracked = duration.num_seconds() as f32 / 3600.0;
        task.actual_hours = Some(task.actual_hours.unwrap_or(0.0) + hours_tracked);
        task.updated_at = Utc::now();
        self.repository.tasks.update(&task).await?;
        
        Ok(duration)
    }
    
    /// Pause tracking (stop without updating task hours)
    pub fn pause_tracking(&self, task_id: Uuid) -> Result<()> {
        let entry_id = {
            let mut active = self.active_entries.lock().unwrap();
            active.remove(&task_id).ok_or_else(|| {
                anyhow::anyhow!("No active time tracking for this task")
            })?
        };
        
        let mut entries = self.time_entries.lock().unwrap();
        let entry = entries.get_mut(&entry_id).ok_or_else(|| {
            anyhow::anyhow!("Time entry not found")
        })?;
        
        entry.end_time = Some(Utc::now());
        Ok(())
    }
    
    /// Get total time tracked for a task
    pub fn get_total_time(&self, task_id: Uuid) -> Duration {
        let entries = self.time_entries.lock().unwrap();
        let mut total = Duration::zero();
        
        for entry in entries.values() {
            if entry.task_id == task_id {
                let end = entry.end_time.unwrap_or_else(Utc::now);
                total = total + (end - entry.start_time);
            }
        }
        
        total
    }
    
    /// Get all time entries for a task
    pub fn get_task_entries(&self, task_id: Uuid) -> Vec<TimeEntry> {
        let entries = self.time_entries.lock().unwrap();
        entries
            .values()
            .filter(|e| e.task_id == task_id)
            .cloned()
            .collect()
    }
    
    /// Check if a task is currently being tracked
    pub fn is_tracking(&self, task_id: Uuid) -> bool {
        self.active_entries.lock().unwrap().contains_key(&task_id)
    }
    
    /// Get the active time entry for a task
    pub fn get_active_entry(&self, task_id: Uuid) -> Option<TimeEntry> {
        let active = self.active_entries.lock().unwrap();
        if let Some(&entry_id) = active.get(&task_id) {
            let entries = self.time_entries.lock().unwrap();
            entries.get(&entry_id).cloned()
        } else {
            None
        }
    }
    
    /// Get a summary of time tracked in a date range
    pub fn get_time_summary(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> HashMap<Uuid, Duration> {
        let entries = self.time_entries.lock().unwrap();
        let mut summary: HashMap<Uuid, Duration> = HashMap::new();
        
        for entry in entries.values() {
            if entry.start_time >= start && entry.start_time <= end {
                let end_time = entry.end_time.unwrap_or_else(Utc::now);
                let duration = end_time - entry.start_time;
                
                *summary.entry(entry.task_id).or_insert(Duration::zero()) += duration;
            }
        }
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Task, Priority};
    use std::collections::HashSet;
    use sqlx::SqlitePool;
    
    async fn setup_test_service() -> (TimeTrackingService, Uuid) {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let repo = Arc::new(Repository::new(pool));
        
        // Create a test task
        let task = Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            due_date: None,
            estimated_hours: Some(5.0),
            actual_hours: None,
            assigned_resource_id: None,
            goal_id: None,
            parent_task_id: None,
            tags: HashSet::new(),
            assignee: None,
            position: crate::domain::task::Position { x: 0.0, y: 0.0 },
        };
        
        repo.tasks.create(&task).await.unwrap();
        
        (TimeTrackingService::new(repo), task.id)
    }
    
    #[tokio::test]
    async fn test_start_and_stop_tracking() {
        let (service, task_id) = setup_test_service().await;
        
        // Start tracking
        let entry_id = service.start_tracking(task_id, "Working on feature".to_string()).await.unwrap();
        assert!(service.is_tracking(task_id));
        
        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Stop tracking
        let duration = service.stop_tracking(task_id).await.unwrap();
        assert!(!service.is_tracking(task_id));
        assert!(duration.num_milliseconds() >= 100);
        
        // Verify task hours were updated
        let task = service.repository.tasks.get(task_id).await.unwrap().unwrap();
        assert!(task.actual_hours.is_some());
        assert!(task.actual_hours.unwrap() > 0.0);
    }
    
    #[tokio::test]
    async fn test_cannot_double_track() {
        let (service, task_id) = setup_test_service().await;
        
        // Start tracking
        service.start_tracking(task_id, "First".to_string()).await.unwrap();
        
        // Try to start again
        let result = service.start_tracking(task_id, "Second".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Already tracking"));
    }
    
    #[tokio::test]
    async fn test_pause_tracking() {
        let (service, task_id) = setup_test_service().await;
        
        // Start tracking
        service.start_tracking(task_id, "Working".to_string()).await.unwrap();
        
        // Pause
        service.pause_tracking(task_id).unwrap();
        assert!(!service.is_tracking(task_id));
        
        // Task hours should not be updated yet
        let task = service.repository.tasks.get(task_id).await.unwrap().unwrap();
        assert_eq!(task.actual_hours, None);
    }
    
    #[tokio::test]
    async fn test_total_time_calculation() {
        let (service, task_id) = setup_test_service().await;
        
        // Track multiple sessions
        for _ in 0..3 {
            service.start_tracking(task_id, "Session".to_string()).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            service.stop_tracking(task_id).await.unwrap();
        }
        
        let total = service.get_total_time(task_id);
        assert!(total.num_milliseconds() >= 150);
    }
}