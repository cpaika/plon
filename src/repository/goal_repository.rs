use crate::domain::goal::Goal;
use anyhow::Result;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct GoalRepository {
    pool: Arc<SqlitePool>,
}

impl GoalRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, goal: &Goal) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO goals (
                id, title, description, status, created_at, updated_at,
                target_date, completed_at, estimated_hours, progress,
                parent_goal_id, position_x, position_y, position_width, position_height, color
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(goal.id.to_string())
        .bind(&goal.title)
        .bind(&goal.description)
        .bind(format!("{:?}", goal.status))
        .bind(goal.created_at.to_rfc3339())
        .bind(goal.updated_at.to_rfc3339())
        .bind(goal.target_date.map(|d| d.to_rfc3339()))
        .bind(goal.completed_at.map(|d| d.to_rfc3339()))
        .bind(goal.estimated_hours)
        .bind(goal.progress)
        .bind(goal.parent_goal_id.map(|id| id.to_string()))
        .bind(goal.position_x)
        .bind(goal.position_y)
        .bind(goal.position_width)
        .bind(goal.position_height)
        .bind(&goal.color)
        .execute(self.pool.as_ref())
        .await?;

        // Add task associations
        for task_id in &goal.task_ids {
            sqlx::query("INSERT INTO goal_tasks (goal_id, task_id) VALUES (?, ?)")
                .bind(goal.id.to_string())
                .bind(task_id.to_string())
                .execute(self.pool.as_ref())
                .await?;
        }

        Ok(())
    }

    pub async fn update(&self, goal: &Goal) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE goals SET
                title = ?, description = ?, status = ?, updated_at = ?,
                target_date = ?, completed_at = ?, estimated_hours = ?, progress = ?,
                parent_goal_id = ?, position_x = ?, position_y = ?, 
                position_width = ?, position_height = ?, color = ?
            WHERE id = ?
            "#,
        )
        .bind(&goal.title)
        .bind(&goal.description)
        .bind(format!("{:?}", goal.status))
        .bind(goal.updated_at.to_rfc3339())
        .bind(goal.target_date.map(|d| d.to_rfc3339()))
        .bind(goal.completed_at.map(|d| d.to_rfc3339()))
        .bind(goal.estimated_hours)
        .bind(goal.progress)
        .bind(goal.parent_goal_id.map(|id| id.to_string()))
        .bind(goal.position_x)
        .bind(goal.position_y)
        .bind(goal.position_width)
        .bind(goal.position_height)
        .bind(&goal.color)
        .bind(goal.id.to_string())
        .execute(self.pool.as_ref())
        .await?;

        // Update task associations
        // First, delete existing associations
        sqlx::query("DELETE FROM goal_tasks WHERE goal_id = ?")
            .bind(goal.id.to_string())
            .execute(self.pool.as_ref())
            .await?;

        // Then add new associations
        for task_id in &goal.task_ids {
            sqlx::query("INSERT INTO goal_tasks (goal_id, task_id) VALUES (?, ?)")
                .bind(goal.id.to_string())
                .bind(task_id.to_string())
                .execute(self.pool.as_ref())
                .await?;
        }

        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Goal>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, description, status, created_at, updated_at,
                   target_date, completed_at, estimated_hours, progress,
                   parent_goal_id, position_x, position_y, position_width,
                   position_height, color
            FROM goals
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(self.pool.as_ref())
        .await?;

        if let Some(row) = row {
            use chrono::DateTime;
            use std::collections::HashSet;

            let goal_id: String = row.get("id");

            // Fetch associated task IDs
            let task_rows = sqlx::query("SELECT task_id FROM goal_tasks WHERE goal_id = ?")
                .bind(&goal_id)
                .fetch_all(self.pool.as_ref())
                .await?;

            let mut task_ids = HashSet::new();
            for task_row in task_rows {
                let task_id: String = task_row.get("task_id");
                if let Ok(uuid) = Uuid::parse_str(&task_id) {
                    task_ids.insert(uuid);
                }
            }

            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "NotStarted" => crate::domain::goal::GoalStatus::NotStarted,
                "Active" => crate::domain::goal::GoalStatus::Active,
                "InProgress" => crate::domain::goal::GoalStatus::InProgress,
                "OnHold" => crate::domain::goal::GoalStatus::OnHold,
                "AtRisk" => crate::domain::goal::GoalStatus::AtRisk,
                "Completed" => crate::domain::goal::GoalStatus::Completed,
                "Cancelled" => crate::domain::goal::GoalStatus::Cancelled,
                _ => crate::domain::goal::GoalStatus::NotStarted,
            };

            let goal = Goal {
                id: Uuid::parse_str(&goal_id)?,
                title: row.get("title"),
                description: row.get("description"),
                status,
                created_at: DateTime::parse_from_rfc3339(row.get("created_at"))?
                    .with_timezone(&chrono::Utc),
                updated_at: DateTime::parse_from_rfc3339(row.get("updated_at"))?
                    .with_timezone(&chrono::Utc),
                target_date: row
                    .get::<Option<String>, _>("target_date")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                completed_at: row
                    .get::<Option<String>, _>("completed_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                task_ids,
                subgoal_ids: HashSet::new(),
                parent_goal_id: row
                    .get::<Option<String>, _>("parent_goal_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                estimated_hours: row.get("estimated_hours"),
                progress: row.get("progress"),
                position_x: row.get("position_x"),
                position_y: row.get("position_y"),
                position_width: row.get("position_width"),
                position_height: row.get("position_height"),
                color: row.get("color"),
            };

            Ok(Some(goal))
        } else {
            Ok(None)
        }
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        // Delete task associations first
        sqlx::query("DELETE FROM goal_tasks WHERE goal_id = ?")
            .bind(id.to_string())
            .execute(self.pool.as_ref())
            .await?;

        // Delete the goal
        let result = sqlx::query("DELETE FROM goals WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool.as_ref())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_all(&self) -> Result<Vec<Goal>> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, description, status, created_at, updated_at,
                   target_date, completed_at, estimated_hours, progress,
                   parent_goal_id, position_x, position_y, position_width,
                   position_height, color
            FROM goals
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        let mut goals = Vec::new();

        for row in rows {
            use chrono::DateTime;
            use std::collections::HashSet;

            let goal_id: String = row.get("id");

            // Fetch associated task IDs
            let task_rows = sqlx::query("SELECT task_id FROM goal_tasks WHERE goal_id = ?")
                .bind(&goal_id)
                .fetch_all(self.pool.as_ref())
                .await?;

            let mut task_ids = HashSet::new();
            for task_row in task_rows {
                let task_id: String = task_row.get("task_id");
                if let Ok(uuid) = Uuid::parse_str(&task_id) {
                    task_ids.insert(uuid);
                }
            }

            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "NotStarted" => crate::domain::goal::GoalStatus::NotStarted,
                "Active" => crate::domain::goal::GoalStatus::Active,
                "InProgress" => crate::domain::goal::GoalStatus::InProgress,
                "OnHold" => crate::domain::goal::GoalStatus::OnHold,
                "AtRisk" => crate::domain::goal::GoalStatus::AtRisk,
                "Completed" => crate::domain::goal::GoalStatus::Completed,
                "Cancelled" => crate::domain::goal::GoalStatus::Cancelled,
                _ => crate::domain::goal::GoalStatus::NotStarted,
            };

            let goal = Goal {
                id: Uuid::parse_str(&goal_id)?,
                title: row.get("title"),
                description: row.get("description"),
                status,
                created_at: DateTime::parse_from_rfc3339(row.get("created_at"))?
                    .with_timezone(&chrono::Utc),
                updated_at: DateTime::parse_from_rfc3339(row.get("updated_at"))?
                    .with_timezone(&chrono::Utc),
                target_date: row
                    .get::<Option<String>, _>("target_date")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                completed_at: row
                    .get::<Option<String>, _>("completed_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                task_ids,
                subgoal_ids: HashSet::new(), // Will be populated if needed
                parent_goal_id: row
                    .get::<Option<String>, _>("parent_goal_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                estimated_hours: row.get("estimated_hours"),
                progress: row.get("progress"),
                position_x: row.get("position_x"),
                position_y: row.get("position_y"),
                position_width: row.get("position_width"),
                position_height: row.get("position_height"),
                color: row.get("color"),
            };

            goals.push(goal);
        }

        Ok(goals)
    }
}
