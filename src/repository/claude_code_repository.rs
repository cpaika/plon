use crate::domain::claude_code::{
    ClaudeCodeConfig, ClaudeCodeSession, ClaudePromptTemplate, SessionStatus,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Row, Sqlite};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct ClaudeCodeRepository {
    pool: Pool<Sqlite>,
}

impl ClaudeCodeRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create_session(&self, session: &ClaudeCodeSession) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO claude_code_sessions (
                id, task_id, status, branch_name, pr_url, pr_number,
                session_log, error_message, started_at, completed_at,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(session.id.to_string())
        .bind(session.task_id.to_string())
        .bind(session.status.as_str())
        .bind(&session.branch_name)
        .bind(&session.pr_url)
        .bind(session.pr_number)
        .bind(&session.session_log)
        .bind(&session.error_message)
        .bind(session.started_at)
        .bind(session.completed_at)
        .bind(session.created_at)
        .bind(session.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_session(&self, session: &ClaudeCodeSession) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE claude_code_sessions
            SET status = ?, branch_name = ?, pr_url = ?, pr_number = ?,
                session_log = ?, error_message = ?, completed_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(session.status.as_str())
        .bind(&session.branch_name)
        .bind(&session.pr_url)
        .bind(session.pr_number)
        .bind(&session.session_log)
        .bind(&session.error_message)
        .bind(session.completed_at)
        .bind(session.updated_at)
        .bind(session.id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_session(&self, id: Uuid) -> Result<Option<ClaudeCodeSession>> {
        let row = sqlx::query(
            r#"
            SELECT id, task_id, status, branch_name, pr_url, pr_number,
                   session_log, error_message, started_at, completed_at,
                   created_at, updated_at
            FROM claude_code_sessions
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_session(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_sessions_by_task(&self, task_id: Uuid) -> Result<Vec<ClaudeCodeSession>> {
        let rows = sqlx::query(
            r#"
            SELECT id, task_id, status, branch_name, pr_url, pr_number,
                   session_log, error_message, started_at, completed_at,
                   created_at, updated_at
            FROM claude_code_sessions
            WHERE task_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(self.row_to_session(row)?);
        }

        Ok(sessions)
    }

    pub async fn get_active_sessions(&self) -> Result<Vec<ClaudeCodeSession>> {
        let rows = sqlx::query(
            r#"
            SELECT id, task_id, status, branch_name, pr_url, pr_number,
                   session_log, error_message, started_at, completed_at,
                   created_at, updated_at
            FROM claude_code_sessions
            WHERE status IN ('initializing', 'working', 'creating_pr')
            ORDER BY started_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(self.row_to_session(row)?);
        }

        Ok(sessions)
    }

    pub async fn cleanup_old_sessions(&self, before: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM claude_code_sessions
            WHERE completed_at < ? AND status IN ('completed', 'failed', 'cancelled')
            "#,
        )
        .bind(before)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_config(&self, config: &ClaudeCodeConfig) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO claude_code_config (
                id, github_repo, github_owner, github_token, claude_api_key,
                default_base_branch, auto_create_pr, working_directory,
                claude_model, max_session_duration_minutes, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(config.id.to_string())
        .bind(&config.github_repo)
        .bind(&config.github_owner)
        .bind(&config.github_token)
        .bind(&config.claude_api_key)
        .bind(&config.default_base_branch)
        .bind(config.auto_create_pr)
        .bind(&config.working_directory)
        .bind(&config.claude_model)
        .bind(config.max_session_duration_minutes)
        .bind(config.created_at)
        .bind(config.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_config(&self, config: &ClaudeCodeConfig) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE claude_code_config
            SET github_repo = ?, github_owner = ?, github_token = ?, claude_api_key = ?,
                default_base_branch = ?, auto_create_pr = ?, working_directory = ?,
                claude_model = ?, max_session_duration_minutes = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&config.github_repo)
        .bind(&config.github_owner)
        .bind(&config.github_token)
        .bind(&config.claude_api_key)
        .bind(&config.default_base_branch)
        .bind(config.auto_create_pr)
        .bind(&config.working_directory)
        .bind(&config.claude_model)
        .bind(config.max_session_duration_minutes)
        .bind(config.updated_at)
        .bind(config.id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_config(&self) -> Result<Option<ClaudeCodeConfig>> {
        let row = sqlx::query(
            r#"
            SELECT id, github_repo, github_owner, github_token, claude_api_key,
                   default_base_branch, auto_create_pr, working_directory,
                   claude_model, max_session_duration_minutes, created_at, updated_at
            FROM claude_code_config
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ClaudeCodeConfig {
                id: Uuid::parse_str(row.get("id"))?,
                github_repo: row.get("github_repo"),
                github_owner: row.get("github_owner"),
                github_token: row.get("github_token"),
                claude_api_key: row.get("claude_api_key"),
                default_base_branch: row.get("default_base_branch"),
                auto_create_pr: row.get("auto_create_pr"),
                working_directory: row.get("working_directory"),
                claude_model: row.get("claude_model"),
                max_session_duration_minutes: row.get("max_session_duration_minutes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn create_template(&self, template: &ClaudePromptTemplate) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO claude_prompt_templates (
                id, name, template, description, variables, is_default,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(template.id.to_string())
        .bind(&template.name)
        .bind(&template.template)
        .bind(&template.description)
        .bind(serde_json::to_string(&template.variables)?)
        .bind(template.is_default)
        .bind(template.created_at)
        .bind(template.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_template(&self, name: &str) -> Result<Option<ClaudePromptTemplate>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, template, description, variables, is_default,
                   created_at, updated_at
            FROM claude_prompt_templates
            WHERE name = ?
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let variables_json: String = row.get("variables");
            Ok(Some(ClaudePromptTemplate {
                id: Uuid::parse_str(row.get("id"))?,
                name: row.get("name"),
                template: row.get("template"),
                description: row.get("description"),
                variables: serde_json::from_str(&variables_json)?,
                is_default: row.get("is_default"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_default_template(&self) -> Result<Option<ClaudePromptTemplate>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, template, description, variables, is_default,
                   created_at, updated_at
            FROM claude_prompt_templates
            WHERE is_default = true
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let variables_json: String = row.get("variables");
            Ok(Some(ClaudePromptTemplate {
                id: Uuid::parse_str(row.get("id"))?,
                name: row.get("name"),
                template: row.get("template"),
                description: row.get("description"),
                variables: serde_json::from_str(&variables_json)?,
                is_default: row.get("is_default"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    fn row_to_session(&self, row: sqlx::sqlite::SqliteRow) -> Result<ClaudeCodeSession> {
        let status_str: String = row.get("status");
        let status = SessionStatus::from_str(&status_str)
            .map_err(|e| anyhow::anyhow!("Invalid session status: {}", e))?;

        Ok(ClaudeCodeSession {
            id: Uuid::parse_str(row.get("id"))?,
            task_id: Uuid::parse_str(row.get("task_id"))?,
            status,
            branch_name: row.get("branch_name"),
            pr_url: row.get("pr_url"),
            pr_number: row.get("pr_number"),
            session_log: row.get("session_log"),
            error_message: row.get("error_message"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}
