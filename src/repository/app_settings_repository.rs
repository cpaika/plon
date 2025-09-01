use anyhow::Result;
use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use crate::domain::app_settings::{AppSettings, Theme, FontSize, UiDensity, SidebarPosition};
use crate::domain::task::TaskStatus;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct AppSettingsRepository {
    pool: SqlitePool,
}

impl AppSettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    pub async fn get(&self) -> Result<Option<AppSettings>> {
        let record = sqlx::query(
            r#"
            SELECT * FROM app_settings 
            ORDER BY created_at DESC 
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = record {
            Ok(Some(AppSettings {
                id: Uuid::parse_str(row.get("id"))?,
                
                // General Settings
                default_task_status: serde_json::from_str(row.get("default_task_status")).unwrap_or(TaskStatus::Todo),
                auto_save_interval_seconds: row.get("auto_save_interval_seconds"),
                enable_notifications: row.get::<i32, _>("enable_notifications") != 0,
                notification_sound: row.get::<i32, _>("notification_sound") != 0,
                date_format: row.get("date_format"),
                time_format: row.get("time_format"),
                week_starts_on: row.get("week_starts_on"),
                enable_time_tracking: row.get::<i32, _>("enable_time_tracking") != 0,
                show_task_numbers: row.get::<i32, _>("show_task_numbers") != 0,
                
                // Workspace Settings
                default_project_directory: row.get("default_project_directory"),
                database_path: row.get("database_path"),
                enable_auto_backup: row.get::<i32, _>("enable_auto_backup") != 0,
                backup_directory: row.get("backup_directory"),
                backup_frequency_hours: row.get("backup_frequency_hours"),
                max_backups_to_keep: row.get("max_backups_to_keep"),
                enable_file_watching: row.get::<i32, _>("enable_file_watching") != 0,
                git_auto_commit: row.get::<i32, _>("git_auto_commit") != 0,
                task_template_directory: row.get("task_template_directory"),
                
                // Appearance Settings
                theme: serde_json::from_str(row.get("theme")).unwrap_or(Theme::Light),
                accent_color: row.get("accent_color"),
                font_size: serde_json::from_str(row.get("font_size")).unwrap_or(FontSize::Medium),
                ui_density: serde_json::from_str(row.get("ui_density")).unwrap_or(UiDensity::Comfortable),
                sidebar_position: serde_json::from_str(row.get("sidebar_position")).unwrap_or(SidebarPosition::Left),
                show_sidebar: row.get::<i32, _>("show_sidebar") != 0,
                show_toolbar: row.get::<i32, _>("show_toolbar") != 0,
                show_statusbar: row.get::<i32, _>("show_statusbar") != 0,
                enable_animations: row.get::<i32, _>("enable_animations") != 0,
                
                // Integration Settings
                enable_github_integration: row.get::<i32, _>("enable_github_integration") != 0,
                enable_slack_integration: row.get::<i32, _>("enable_slack_integration") != 0,
                slack_webhook_url: row.get("slack_webhook_url"),
                enable_discord_integration: row.get::<i32, _>("enable_discord_integration") != 0,
                discord_webhook_url: row.get("discord_webhook_url"),
                enable_calendar_sync: row.get::<i32, _>("enable_calendar_sync") != 0,
                calendar_provider: row.get("calendar_provider"),
                
                created_at: DateTime::parse_from_rfc3339(row.get("created_at"))?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row.get("updated_at"))?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn create(&self, settings: &AppSettings) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO app_settings (
                id, 
                default_task_status, auto_save_interval_seconds, enable_notifications,
                notification_sound, date_format, time_format, week_starts_on,
                enable_time_tracking, show_task_numbers,
                default_project_directory, database_path, enable_auto_backup,
                backup_directory, backup_frequency_hours, max_backups_to_keep,
                enable_file_watching, git_auto_commit, task_template_directory,
                theme, accent_color, font_size, ui_density, sidebar_position,
                show_sidebar, show_toolbar, show_statusbar, enable_animations,
                enable_github_integration, enable_slack_integration, slack_webhook_url,
                enable_discord_integration, discord_webhook_url,
                enable_calendar_sync, calendar_provider,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(settings.id.to_string())
        .bind(serde_json::to_string(&settings.default_task_status)?)
        .bind(settings.auto_save_interval_seconds)
        .bind(settings.enable_notifications as i32)
        .bind(settings.notification_sound as i32)
        .bind(&settings.date_format)
        .bind(&settings.time_format)
        .bind(&settings.week_starts_on)
        .bind(settings.enable_time_tracking as i32)
        .bind(settings.show_task_numbers as i32)
        .bind(&settings.default_project_directory)
        .bind(&settings.database_path)
        .bind(settings.enable_auto_backup as i32)
        .bind(&settings.backup_directory)
        .bind(settings.backup_frequency_hours)
        .bind(settings.max_backups_to_keep)
        .bind(settings.enable_file_watching as i32)
        .bind(settings.git_auto_commit as i32)
        .bind(&settings.task_template_directory)
        .bind(serde_json::to_string(&settings.theme)?)
        .bind(&settings.accent_color)
        .bind(serde_json::to_string(&settings.font_size)?)
        .bind(serde_json::to_string(&settings.ui_density)?)
        .bind(serde_json::to_string(&settings.sidebar_position)?)
        .bind(settings.show_sidebar as i32)
        .bind(settings.show_toolbar as i32)
        .bind(settings.show_statusbar as i32)
        .bind(settings.enable_animations as i32)
        .bind(settings.enable_github_integration as i32)
        .bind(settings.enable_slack_integration as i32)
        .bind(&settings.slack_webhook_url)
        .bind(settings.enable_discord_integration as i32)
        .bind(&settings.discord_webhook_url)
        .bind(settings.enable_calendar_sync as i32)
        .bind(&settings.calendar_provider)
        .bind(settings.created_at.to_rfc3339())
        .bind(settings.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn update(&self, settings: &AppSettings) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE app_settings SET
                default_task_status = ?, auto_save_interval_seconds = ?, enable_notifications = ?,
                notification_sound = ?, date_format = ?, time_format = ?, week_starts_on = ?,
                enable_time_tracking = ?, show_task_numbers = ?,
                default_project_directory = ?, database_path = ?, enable_auto_backup = ?,
                backup_directory = ?, backup_frequency_hours = ?, max_backups_to_keep = ?,
                enable_file_watching = ?, git_auto_commit = ?, task_template_directory = ?,
                theme = ?, accent_color = ?, font_size = ?, ui_density = ?, sidebar_position = ?,
                show_sidebar = ?, show_toolbar = ?, show_statusbar = ?, enable_animations = ?,
                enable_github_integration = ?, enable_slack_integration = ?, slack_webhook_url = ?,
                enable_discord_integration = ?, discord_webhook_url = ?,
                enable_calendar_sync = ?, calendar_provider = ?,
                updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(serde_json::to_string(&settings.default_task_status)?)
        .bind(settings.auto_save_interval_seconds)
        .bind(settings.enable_notifications as i32)
        .bind(settings.notification_sound as i32)
        .bind(&settings.date_format)
        .bind(&settings.time_format)
        .bind(&settings.week_starts_on)
        .bind(settings.enable_time_tracking as i32)
        .bind(settings.show_task_numbers as i32)
        .bind(&settings.default_project_directory)
        .bind(&settings.database_path)
        .bind(settings.enable_auto_backup as i32)
        .bind(&settings.backup_directory)
        .bind(settings.backup_frequency_hours)
        .bind(settings.max_backups_to_keep)
        .bind(settings.enable_file_watching as i32)
        .bind(settings.git_auto_commit as i32)
        .bind(&settings.task_template_directory)
        .bind(serde_json::to_string(&settings.theme)?)
        .bind(&settings.accent_color)
        .bind(serde_json::to_string(&settings.font_size)?)
        .bind(serde_json::to_string(&settings.ui_density)?)
        .bind(serde_json::to_string(&settings.sidebar_position)?)
        .bind(settings.show_sidebar as i32)
        .bind(settings.show_toolbar as i32)
        .bind(settings.show_statusbar as i32)
        .bind(settings.enable_animations as i32)
        .bind(settings.enable_github_integration as i32)
        .bind(settings.enable_slack_integration as i32)
        .bind(&settings.slack_webhook_url)
        .bind(settings.enable_discord_integration as i32)
        .bind(&settings.discord_webhook_url)
        .bind(settings.enable_calendar_sync as i32)
        .bind(&settings.calendar_provider)
        .bind(settings.updated_at.to_rfc3339())
        .bind(settings.id.to_string())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_or_create_default(&self) -> Result<AppSettings> {
        if let Some(settings) = self.get().await? {
            Ok(settings)
        } else {
            let settings = AppSettings::default();
            self.create(&settings).await?;
            Ok(settings)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    
    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        
        // Create the app_settings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_settings (
                id TEXT PRIMARY KEY NOT NULL,
                default_task_status TEXT NOT NULL DEFAULT '"Todo"',
                auto_save_interval_seconds INTEGER NOT NULL DEFAULT 30,
                enable_notifications INTEGER NOT NULL DEFAULT 1,
                notification_sound INTEGER NOT NULL DEFAULT 1,
                date_format TEXT NOT NULL DEFAULT 'MM/DD/YYYY',
                time_format TEXT NOT NULL DEFAULT '12h',
                week_starts_on TEXT NOT NULL DEFAULT 'Sunday',
                enable_time_tracking INTEGER NOT NULL DEFAULT 1,
                show_task_numbers INTEGER NOT NULL DEFAULT 0,
                default_project_directory TEXT NOT NULL DEFAULT '~/plon-projects',
                database_path TEXT NOT NULL DEFAULT 'plon.db',
                enable_auto_backup INTEGER NOT NULL DEFAULT 1,
                backup_directory TEXT NOT NULL DEFAULT '~/plon-backups',
                backup_frequency_hours INTEGER NOT NULL DEFAULT 24,
                max_backups_to_keep INTEGER NOT NULL DEFAULT 7,
                enable_file_watching INTEGER NOT NULL DEFAULT 1,
                git_auto_commit INTEGER NOT NULL DEFAULT 0,
                task_template_directory TEXT,
                theme TEXT NOT NULL DEFAULT '"Light"',
                accent_color TEXT NOT NULL DEFAULT '#3b82f6',
                font_size TEXT NOT NULL DEFAULT '"Medium"',
                ui_density TEXT NOT NULL DEFAULT '"Comfortable"',
                sidebar_position TEXT NOT NULL DEFAULT '"Left"',
                show_sidebar INTEGER NOT NULL DEFAULT 1,
                show_toolbar INTEGER NOT NULL DEFAULT 1,
                show_statusbar INTEGER NOT NULL DEFAULT 1,
                enable_animations INTEGER NOT NULL DEFAULT 1,
                enable_github_integration INTEGER NOT NULL DEFAULT 0,
                enable_slack_integration INTEGER NOT NULL DEFAULT 0,
                slack_webhook_url TEXT,
                enable_discord_integration INTEGER NOT NULL DEFAULT 0,
                discord_webhook_url TEXT,
                enable_calendar_sync INTEGER NOT NULL DEFAULT 0,
                calendar_provider TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();
        
        pool
    }
    
    #[tokio::test]
    async fn test_create_and_get_settings() {
        let pool = setup_test_db().await;
        let repo = AppSettingsRepository::new(pool);
        
        let settings = AppSettings::default();
        repo.create(&settings).await.unwrap();
        
        let loaded = repo.get().await.unwrap().unwrap();
        assert_eq!(loaded.id, settings.id);
        assert_eq!(loaded.theme, Theme::Light);
        assert_eq!(loaded.font_size, FontSize::Medium);
    }
    
    #[tokio::test]
    async fn test_update_settings() {
        let pool = setup_test_db().await;
        let repo = AppSettingsRepository::new(pool);
        
        let mut settings = AppSettings::default();
        repo.create(&settings).await.unwrap();
        
        settings.theme = Theme::Dark;
        settings.accent_color = "#ff0000".to_string();
        settings.enable_animations = false;
        settings.update_timestamp();
        
        repo.update(&settings).await.unwrap();
        
        let loaded = repo.get().await.unwrap().unwrap();
        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.accent_color, "#ff0000");
        assert_eq!(loaded.enable_animations, false);
    }
    
    #[tokio::test]
    async fn test_get_or_create_default() {
        let pool = setup_test_db().await;
        let repo = AppSettingsRepository::new(pool);
        
        // First call should create default
        let settings1 = repo.get_or_create_default().await.unwrap();
        assert_eq!(settings1.theme, Theme::Light);
        
        // Second call should get existing
        let settings2 = repo.get_or_create_default().await.unwrap();
        assert_eq!(settings1.id, settings2.id);
    }
}