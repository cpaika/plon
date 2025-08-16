use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::task::{Position, Priority, SubTask, Task, TaskStatus};

#[derive(Clone)]
pub struct TaskRepository {
    pool: Arc<SqlitePool>,
}

impl TaskRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, task: &Task) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Insert main task
        sqlx::query(
            r#"
            INSERT INTO tasks (
                id, title, description, status, priority, metadata, tags,
                created_at, updated_at, due_date, scheduled_date, completed_at,
                estimated_hours, actual_hours, assigned_resource_id,
                goal_id, parent_task_id, position_x, position_y, configuration_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(task.id.to_string())
        .bind(&task.title)
        .bind(&task.description)
        .bind(status_to_string(&task.status))
        .bind(priority_to_string(&task.priority))
        .bind(serde_json::to_string(&task.metadata)?)
        .bind(serde_json::to_string(&task.tags)?)
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .bind(task.due_date.map(|d| d.to_rfc3339()))
        .bind(task.scheduled_date.map(|d| d.to_rfc3339()))
        .bind(task.completed_at.map(|d| d.to_rfc3339()))
        .bind(task.estimated_hours)
        .bind(task.actual_hours)
        .bind(task.assigned_resource_id.map(|id| id.to_string()))
        .bind(task.goal_id.map(|id| id.to_string()))
        .bind(task.parent_task_id.map(|id| id.to_string()))
        .bind(task.position.x)
        .bind(task.position.y)
        .bind(task.configuration_id.map(|id| id.to_string()))
        .execute(&mut *tx)
        .await?;

        // Insert subtasks
        for subtask in &task.subtasks {
            sqlx::query(
                r#"
                INSERT INTO subtasks (id, task_id, description, completed, created_at, completed_at)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(subtask.id.to_string())
            .bind(task.id.to_string())
            .bind(&subtask.description)
            .bind(subtask.completed as i32)
            .bind(subtask.created_at.to_rfc3339())
            .bind(subtask.completed_at.map(|d| d.to_rfc3339()))
            .execute(&mut *tx)
            .await?;
        }

        // Update spatial index (using rowid from the inserted task)
        let rowid: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
            .fetch_one(&mut *tx)
            .await?;
        
        sqlx::query(
            r#"
            INSERT INTO tasks_spatial (id, min_x, max_x, min_y, max_y)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(rowid)
        .bind(task.position.x)
        .bind(task.position.x)
        .bind(task.position.y)
        .bind(task.position.y)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update(&self, task: &Task) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Update main task
        sqlx::query(
            r#"
            UPDATE tasks SET
                title = ?, description = ?, status = ?, priority = ?,
                metadata = ?, tags = ?, updated_at = ?, due_date = ?,
                scheduled_date = ?, completed_at = ?, estimated_hours = ?,
                actual_hours = ?, assigned_resource_id = ?, goal_id = ?,
                parent_task_id = ?, position_x = ?, position_y = ?, configuration_id = ?
            WHERE id = ?
            "#,
        )
        .bind(&task.title)
        .bind(&task.description)
        .bind(status_to_string(&task.status))
        .bind(priority_to_string(&task.priority))
        .bind(serde_json::to_string(&task.metadata)?)
        .bind(serde_json::to_string(&task.tags)?)
        .bind(task.updated_at.to_rfc3339())
        .bind(task.due_date.map(|d| d.to_rfc3339()))
        .bind(task.scheduled_date.map(|d| d.to_rfc3339()))
        .bind(task.completed_at.map(|d| d.to_rfc3339()))
        .bind(task.estimated_hours)
        .bind(task.actual_hours)
        .bind(task.assigned_resource_id.map(|id| id.to_string()))
        .bind(task.goal_id.map(|id| id.to_string()))
        .bind(task.parent_task_id.map(|id| id.to_string()))
        .bind(task.position.x)
        .bind(task.position.y)
        .bind(task.configuration_id.map(|id| id.to_string()))
        .bind(task.id.to_string())
        .execute(&mut *tx)
        .await?;

        // Delete existing subtasks and insert new ones
        sqlx::query("DELETE FROM subtasks WHERE task_id = ?")
            .bind(task.id.to_string())
            .execute(&mut *tx)
            .await?;

        for subtask in &task.subtasks {
            sqlx::query(
                r#"
                INSERT INTO subtasks (id, task_id, description, completed, created_at, completed_at)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(subtask.id.to_string())
            .bind(task.id.to_string())
            .bind(&subtask.description)
            .bind(subtask.completed as i32)
            .bind(subtask.created_at.to_rfc3339())
            .bind(subtask.completed_at.map(|d| d.to_rfc3339()))
            .execute(&mut *tx)
            .await?;
        }

        // Update spatial index
        let rowid: Option<i64> = sqlx::query_scalar("SELECT rowid FROM tasks WHERE id = ?")
            .bind(task.id.to_string())
            .fetch_optional(&mut *tx)
            .await?;
        
        if let Some(rowid) = rowid {
            sqlx::query("DELETE FROM tasks_spatial WHERE id = ?")
                .bind(rowid)
                .execute(&mut *tx)
                .await?;

            sqlx::query(
                r#"
                INSERT INTO tasks_spatial (id, min_x, max_x, min_y, max_y)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(rowid)
            .bind(task.position.x)
            .bind(task.position.x)
            .bind(task.position.y)
            .bind(task.position.y)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Task>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, description, status, priority, metadata, tags,
                   created_at, updated_at, due_date, scheduled_date, completed_at,
                   estimated_hours, actual_hours, assigned_resource_id,
                   goal_id, parent_task_id, position_x, position_y, configuration_id
            FROM tasks WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(self.pool.as_ref())
        .await?;

        if let Some(row) = row {
            let task_id: String = row.get("id");
            let mut task = self.row_to_task(row)?;

            // Fetch subtasks
            let subtasks = sqlx::query(
                r#"
                SELECT id, description, completed, created_at, completed_at
                FROM subtasks WHERE task_id = ?
                ORDER BY created_at
                "#,
            )
            .bind(task_id)
            .fetch_all(self.pool.as_ref())
            .await?;

            for subtask_row in subtasks {
                task.subtasks.push(SubTask {
                    id: Uuid::parse_str(subtask_row.get("id"))?,
                    description: subtask_row.get("description"),
                    completed: subtask_row.get::<i32, _>("completed") != 0,
                    created_at: DateTime::parse_from_rfc3339(subtask_row.get("created_at"))?.with_timezone(&Utc),
                    completed_at: subtask_row.get::<Option<String>, _>("completed_at")
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                });
            }

            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let mut tx = self.pool.begin().await?;

        // Delete from spatial index
        let rowid: Option<i64> = sqlx::query_scalar("SELECT rowid FROM tasks WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&mut *tx)
            .await?;
        
        if let Some(rowid) = rowid {
            sqlx::query("DELETE FROM tasks_spatial WHERE id = ?")
                .bind(rowid)
                .execute(&mut *tx)
                .await?;
        }

        // Delete task (subtasks will cascade)
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn list(&self, filters: TaskFilters) -> Result<Vec<Task>> {
        let mut query = String::from(
            r#"
            SELECT DISTINCT t.id, t.title, t.description, t.status, t.priority,
                   t.metadata, t.tags, t.created_at, t.updated_at, t.due_date,
                   t.scheduled_date, t.completed_at, t.estimated_hours, t.actual_hours,
                   t.assigned_resource_id, t.goal_id, t.parent_task_id,
                   t.position_x, t.position_y, t.configuration_id
            FROM tasks t
            WHERE 1=1
            "#,
        );

        let mut conditions = Vec::new();

        if let Some(status) = &filters.status {
            conditions.push(format!("t.status = '{}'", status_to_string(status)));
        }

        if let Some(resource_id) = &filters.assigned_resource_id {
            conditions.push(format!("t.assigned_resource_id = '{}'", resource_id));
        }

        if let Some(goal_id) = &filters.goal_id {
            conditions.push(format!("t.goal_id = '{}'", goal_id));
        }

        if filters.overdue {
            conditions.push(format!(
                "t.due_date < '{}' AND t.status != 'Done'",
                Utc::now().to_rfc3339()
            ));
        }

        for condition in conditions {
            query.push_str(&format!(" AND {}", condition));
        }

        query.push_str(" ORDER BY t.created_at DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let rows = sqlx::query(&query).fetch_all(self.pool.as_ref()).await?;

        let mut tasks = Vec::new();
        for row in rows {
            let task_id: String = row.get("id");
            let mut task = self.row_to_task(row)?;

            // Fetch subtasks for each task
            let subtasks = sqlx::query(
                r#"
                SELECT id, description, completed, created_at, completed_at
                FROM subtasks WHERE task_id = ?
                ORDER BY created_at
                "#,
            )
            .bind(task_id)
            .fetch_all(self.pool.as_ref())
            .await?;

            for subtask_row in subtasks {
                task.subtasks.push(SubTask {
                    id: Uuid::parse_str(subtask_row.get("id"))?,
                    description: subtask_row.get("description"),
                    completed: subtask_row.get::<i32, _>("completed") != 0,
                    created_at: DateTime::parse_from_rfc3339(subtask_row.get("created_at"))?.with_timezone(&Utc),
                    completed_at: subtask_row.get::<Option<String>, _>("completed_at")
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                });
            }

            tasks.push(task);
        }

        Ok(tasks)
    }

    pub async fn find_in_area(&self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Result<Vec<Task>> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.title, t.description, t.status, t.priority,
                   t.metadata, t.tags, t.created_at, t.updated_at, t.due_date,
                   t.scheduled_date, t.completed_at, t.estimated_hours, t.actual_hours,
                   t.assigned_resource_id, t.goal_id, t.parent_task_id,
                   t.position_x, t.position_y, t.configuration_id
            FROM tasks t
            JOIN tasks_spatial s ON s.id = (SELECT rowid FROM tasks WHERE id = t.id)
            WHERE s.min_x <= ? AND s.max_x >= ?
              AND s.min_y <= ? AND s.max_y >= ?
            "#,
        )
        .bind(max_x)
        .bind(min_x)
        .bind(max_y)
        .bind(min_y)
        .fetch_all(self.pool.as_ref())
        .await?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(self.row_to_task(row)?);
        }

        Ok(tasks)
    }

    fn row_to_task(&self, row: sqlx::sqlite::SqliteRow) -> Result<Task> {
        Ok(Task {
            id: Uuid::parse_str(row.get("id"))?,
            title: row.get("title"),
            description: row.get("description"),
            status: string_to_status(row.get("status"))?,
            priority: string_to_priority(row.get("priority"))?,
            metadata: serde_json::from_str(row.get("metadata"))?,
            tags: serde_json::from_str(row.get("tags"))?,
            created_at: DateTime::parse_from_rfc3339(row.get("created_at"))?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(row.get("updated_at"))?.with_timezone(&Utc),
            due_date: row.get::<Option<String>, _>("due_date")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            scheduled_date: row.get::<Option<String>, _>("scheduled_date")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            completed_at: row.get::<Option<String>, _>("completed_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            estimated_hours: row.get("estimated_hours"),
            actual_hours: row.get("actual_hours"),
            assigned_resource_id: row.get::<Option<String>, _>("assigned_resource_id")
                .and_then(|s| Uuid::parse_str(&s).ok()),
            goal_id: row.get::<Option<String>, _>("goal_id")
                .and_then(|s| Uuid::parse_str(&s).ok()),
            parent_task_id: row.get::<Option<String>, _>("parent_task_id")
                .and_then(|s| Uuid::parse_str(&s).ok()),
            position: Position {
                x: row.get("position_x"),
                y: row.get("position_y"),
            },
            subtasks: Vec::new(), // Will be filled separately
            configuration_id: row.get::<Option<String>, _>("configuration_id")
                .and_then(|s| Uuid::parse_str(&s).ok()),
        })
    }
}

#[derive(Default)]
pub struct TaskFilters {
    pub status: Option<TaskStatus>,
    pub assigned_resource_id: Option<Uuid>,
    pub goal_id: Option<Uuid>,
    pub overdue: bool,
    pub limit: Option<u32>,
}

fn status_to_string(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "Todo",
        TaskStatus::InProgress => "InProgress",
        TaskStatus::Blocked => "Blocked",
        TaskStatus::Review => "Review",
        TaskStatus::Done => "Done",
        TaskStatus::Cancelled => "Cancelled",
    }
}

fn string_to_status(s: &str) -> Result<TaskStatus> {
    match s {
        "Todo" => Ok(TaskStatus::Todo),
        "InProgress" => Ok(TaskStatus::InProgress),
        "Blocked" => Ok(TaskStatus::Blocked),
        "Review" => Ok(TaskStatus::Review),
        "Done" => Ok(TaskStatus::Done),
        "Cancelled" => Ok(TaskStatus::Cancelled),
        _ => Err(anyhow::anyhow!("Invalid task status: {}", s)),
    }
}

fn priority_to_string(priority: &Priority) -> &'static str {
    match priority {
        Priority::Low => "Low",
        Priority::Medium => "Medium",
        Priority::High => "High",
        Priority::Critical => "Critical",
    }
}

fn string_to_priority(s: &str) -> Result<Priority> {
    match s {
        "Low" => Ok(Priority::Low),
        "Medium" => Ok(Priority::Medium),
        "High" => Ok(Priority::High),
        "Critical" => Ok(Priority::Critical),
        _ => Err(anyhow::anyhow!("Invalid priority: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::database::init_test_database;

    #[tokio::test]
    async fn test_task_crud() {
        let pool = init_test_database().await.unwrap();
        let repo = TaskRepository::new(Arc::new(pool));

        // Create task
        let mut task = Task::new("Test Task".to_string(), "Description".to_string());
        task.add_subtask("Subtask 1".to_string());
        
        repo.create(&task).await.unwrap();

        // Read task
        let fetched = repo.get(task.id).await.unwrap().unwrap();
        assert_eq!(fetched.title, "Test Task");
        assert_eq!(fetched.subtasks.len(), 1);

        // Update task
        let mut updated = fetched.clone();
        updated.title = "Updated Task".to_string();
        updated.update_status(TaskStatus::InProgress);
        repo.update(&updated).await.unwrap();

        let fetched = repo.get(task.id).await.unwrap().unwrap();
        assert_eq!(fetched.title, "Updated Task");
        assert_eq!(fetched.status, TaskStatus::InProgress);

        // Delete task
        assert!(repo.delete(task.id).await.unwrap());
        assert!(repo.get(task.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_task_list_filters() {
        let pool = init_test_database().await.unwrap();
        let repo = TaskRepository::new(Arc::new(pool));

        // Create multiple tasks
        let mut task1 = Task::new("Task 1".to_string(), "".to_string());
        task1.update_status(TaskStatus::Todo);
        
        let mut task2 = Task::new("Task 2".to_string(), "".to_string());
        task2.update_status(TaskStatus::InProgress);
        
        let mut task3 = Task::new("Task 3".to_string(), "".to_string());
        task3.update_status(TaskStatus::Done);

        repo.create(&task1).await.unwrap();
        repo.create(&task2).await.unwrap();
        repo.create(&task3).await.unwrap();

        // Test status filter
        let filters = TaskFilters {
            status: Some(TaskStatus::InProgress),
            ..Default::default()
        };
        let tasks = repo.list(filters).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Task 2");

        // Test no filters
        let tasks = repo.list(TaskFilters::default()).await.unwrap();
        assert_eq!(tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_spatial_query() {
        let pool = init_test_database().await.unwrap();
        let repo = TaskRepository::new(Arc::new(pool));

        let mut task1 = Task::new("Task 1".to_string(), "".to_string());
        task1.set_position(10.0, 10.0);
        
        let mut task2 = Task::new("Task 2".to_string(), "".to_string());
        task2.set_position(50.0, 50.0);
        
        let mut task3 = Task::new("Task 3".to_string(), "".to_string());
        task3.set_position(100.0, 100.0);

        repo.create(&task1).await.unwrap();
        repo.create(&task2).await.unwrap();
        repo.create(&task3).await.unwrap();

        // Query area that includes task1 and task2
        let tasks = repo.find_in_area(0.0, 60.0, 0.0, 60.0).await.unwrap();
        assert_eq!(tasks.len(), 2);
    }
}