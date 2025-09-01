use sqlx::{SqlitePool, FromRow};
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;
use crate::domain::task_execution::{TaskExecution, ExecutionStatus};
use chrono::{DateTime, Utc};

#[derive(FromRow)]
struct TaskExecutionRow {
    id: String,
    task_id: String,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    status: String,
    branch_name: String,
    pr_url: Option<String>,
    error_message: Option<String>,
    output_log: String,
}

#[derive(Clone)]
pub struct TaskExecutionRepository {
    pool: Arc<SqlitePool>,
}

impl TaskExecutionRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
    
    pub async fn create(&self, execution: &TaskExecution) -> Result<()> {
        let status_str = serde_json::to_string(&execution.status)?;
        let output_log_str = serde_json::to_string(&execution.output_log)?;
        
        sqlx::query(
            r#"
            INSERT INTO task_executions (
                id, task_id, started_at, completed_at, status,
                branch_name, pr_url, error_message, output_log
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(execution.id.to_string())
        .bind(execution.task_id.to_string())
        .bind(execution.started_at)
        .bind(execution.completed_at)
        .bind(status_str)
        .bind(execution.branch_name.clone())
        .bind(execution.pr_url.clone())
        .bind(execution.error_message.clone())
        .bind(output_log_str)
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn update(&self, execution: &TaskExecution) -> Result<()> {
        let status_str = serde_json::to_string(&execution.status)?;
        let output_log_str = serde_json::to_string(&execution.output_log)?;
        
        sqlx::query(
            r#"
            UPDATE task_executions
            SET completed_at = ?, status = ?, pr_url = ?, 
                error_message = ?, output_log = ?
            WHERE id = ?
            "#
        )
        .bind(execution.completed_at)
        .bind(status_str)
        .bind(execution.pr_url.clone())
        .bind(execution.error_message.clone())
        .bind(output_log_str)
        .bind(execution.id.to_string())
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get(&self, id: Uuid) -> Result<Option<TaskExecution>> {
        let row = sqlx::query_as::<_, TaskExecutionRow>(
            r#"
            SELECT id, task_id, started_at, completed_at, status,
                   branch_name, pr_url, error_message, output_log
            FROM task_executions
            WHERE id = ?
            "#
        )
        .bind(id.to_string())
        .fetch_optional(&*self.pool)
        .await?;
        
        match row {
            Some(r) => {
                let status: ExecutionStatus = serde_json::from_str(&r.status)?;
                let output_log: Vec<String> = serde_json::from_str(&r.output_log)?;
                let id = Uuid::parse_str(&r.id)?;
                let task_id = Uuid::parse_str(&r.task_id)?;
                
                Ok(Some(TaskExecution {
                    id,
                    task_id,
                    started_at: r.started_at,
                    completed_at: r.completed_at,
                    status,
                    branch_name: r.branch_name,
                    pr_url: r.pr_url,
                    error_message: r.error_message,
                    output_log,
                }))
            }
            None => Ok(None),
        }
    }
    
    pub async fn list_for_task(&self, task_id: Uuid) -> Result<Vec<TaskExecution>> {
        let rows = sqlx::query_as::<_, TaskExecutionRow>(
            r#"
            SELECT id, task_id, started_at, completed_at, status,
                   branch_name, pr_url, error_message, output_log
            FROM task_executions
            WHERE task_id = ?
            ORDER BY started_at DESC
            "#
        )
        .bind(task_id.to_string())
        .fetch_all(&*self.pool)
        .await?;
        
        let mut executions = Vec::new();
        for r in rows {
            let status: ExecutionStatus = serde_json::from_str(&r.status)?;
            let output_log: Vec<String> = serde_json::from_str(&r.output_log)?;
            let id = Uuid::parse_str(&r.id)?;
            let task_id = Uuid::parse_str(&r.task_id)?;
            
            executions.push(TaskExecution {
                id,
                task_id,
                started_at: r.started_at,
                completed_at: r.completed_at,
                status,
                branch_name: r.branch_name,
                pr_url: r.pr_url,
                error_message: r.error_message,
                output_log,
            });
        }
        
        Ok(executions)
    }
    
    pub async fn get_active_for_task(&self, task_id: Uuid) -> Result<Option<TaskExecution>> {
        let running_status = serde_json::to_string(&ExecutionStatus::Running)?;
        let pending_status = serde_json::to_string(&ExecutionStatus::PendingReview)?;
        
        let row = sqlx::query_as::<_, TaskExecutionRow>(
            r#"
            SELECT id, task_id, started_at, completed_at, status,
                   branch_name, pr_url, error_message, output_log
            FROM task_executions
            WHERE task_id = ? AND (status = ? OR status = ?)
            ORDER BY started_at DESC
            LIMIT 1
            "#
        )
        .bind(task_id.to_string())
        .bind(running_status)
        .bind(pending_status)
        .fetch_optional(&*self.pool)
        .await?;
        
        match row {
            Some(r) => {
                let status: ExecutionStatus = serde_json::from_str(&r.status)?;
                let output_log: Vec<String> = serde_json::from_str(&r.output_log)?;
                let id = Uuid::parse_str(&r.id)?;
                let task_id = Uuid::parse_str(&r.task_id)?;
                
                Ok(Some(TaskExecution {
                    id,
                    task_id,
                    started_at: r.started_at,
                    completed_at: r.completed_at,
                    status,
                    branch_name: r.branch_name,
                    pr_url: r.pr_url,
                    error_message: r.error_message,
                    output_log,
                }))
            }
            None => Ok(None),
        }
    }
    
    pub async fn list_recent(&self, limit: i32) -> Result<Vec<TaskExecution>> {
        let rows = sqlx::query_as::<_, TaskExecutionRow>(
            r#"
            SELECT id, task_id, started_at, completed_at, status,
                   branch_name, pr_url, error_message, output_log
            FROM task_executions
            ORDER BY started_at DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&*self.pool)
        .await?;
        
        let mut executions = Vec::new();
        for r in rows {
            let status: ExecutionStatus = serde_json::from_str(&r.status)?;
            let output_log: Vec<String> = serde_json::from_str(&r.output_log)?;
            let id = Uuid::parse_str(&r.id)?;
            let task_id = Uuid::parse_str(&r.task_id)?;
            
            executions.push(TaskExecution {
                id,
                task_id,
                started_at: r.started_at,
                completed_at: r.completed_at,
                status,
                branch_name: r.branch_name,
                pr_url: r.pr_url,
                error_message: r.error_message,
                output_log,
            });
        }
        
        Ok(executions)
    }
}