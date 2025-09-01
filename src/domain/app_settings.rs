use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::domain::task::TaskStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub id: Uuid,
    
    // General Settings
    pub default_task_status: TaskStatus,
    pub auto_save_interval_seconds: i32,
    pub enable_notifications: bool,
    pub notification_sound: bool,
    pub date_format: String,  // e.g., "MM/DD/YYYY" or "DD/MM/YYYY"
    pub time_format: String,  // e.g., "12h" or "24h"
    pub week_starts_on: String, // "Sunday" or "Monday"
    pub enable_time_tracking: bool,
    pub show_task_numbers: bool,
    
    // Workspace Settings
    pub default_project_directory: String,
    pub database_path: String,
    pub enable_auto_backup: bool,
    pub backup_directory: String,
    pub backup_frequency_hours: i32,
    pub max_backups_to_keep: i32,
    pub enable_file_watching: bool,
    pub git_auto_commit: bool,
    pub task_template_directory: Option<String>,
    
    // Appearance Settings
    pub theme: Theme,
    pub accent_color: String,  // Hex color
    pub font_size: FontSize,
    pub ui_density: UiDensity,
    pub sidebar_position: SidebarPosition,
    pub show_sidebar: bool,
    pub show_toolbar: bool,
    pub show_statusbar: bool,
    pub enable_animations: bool,
    
    // Integration Settings (non-sensitive)
    pub enable_github_integration: bool,
    pub enable_slack_integration: bool,
    pub slack_webhook_url: Option<String>,
    pub enable_discord_integration: bool,
    pub discord_webhook_url: Option<String>,
    pub enable_calendar_sync: bool,
    pub calendar_provider: Option<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FontSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum UiDensity {
    Compact,
    Comfortable,
    Spacious,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SidebarPosition {
    Left,
    Right,
}

impl Default for AppSettings {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            
            // General Settings
            default_task_status: TaskStatus::Todo,
            auto_save_interval_seconds: 30,
            enable_notifications: true,
            notification_sound: true,
            date_format: "MM/DD/YYYY".to_string(),
            time_format: "12h".to_string(),
            week_starts_on: "Sunday".to_string(),
            enable_time_tracking: true,
            show_task_numbers: false,
            
            // Workspace Settings
            default_project_directory: "~/plon-projects".to_string(),
            database_path: "plon.db".to_string(),
            enable_auto_backup: true,
            backup_directory: "~/plon-backups".to_string(),
            backup_frequency_hours: 24,
            max_backups_to_keep: 7,
            enable_file_watching: true,
            git_auto_commit: false,
            task_template_directory: None,
            
            // Appearance Settings
            theme: Theme::Light,
            accent_color: "#3b82f6".to_string(),
            font_size: FontSize::Medium,
            ui_density: UiDensity::Comfortable,
            sidebar_position: SidebarPosition::Left,
            show_sidebar: true,
            show_toolbar: true,
            show_statusbar: true,
            enable_animations: true,
            
            // Integration Settings
            enable_github_integration: false,
            enable_slack_integration: false,
            slack_webhook_url: None,
            enable_discord_integration: false,
            discord_webhook_url: None,
            enable_calendar_sync: false,
            calendar_provider: None,
            
            created_at: now,
            updated_at: now,
        }
    }
}

impl AppSettings {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update_timestamp(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light => write!(f, "Light"),
            Theme::Dark => write!(f, "Dark"),
            Theme::Auto => write!(f, "Auto"),
        }
    }
}

impl std::fmt::Display for FontSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontSize::Small => write!(f, "Small"),
            FontSize::Medium => write!(f, "Medium"),
            FontSize::Large => write!(f, "Large"),
            FontSize::ExtraLarge => write!(f, "Extra Large"),
        }
    }
}

impl std::fmt::Display for UiDensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiDensity::Compact => write!(f, "Compact"),
            UiDensity::Comfortable => write!(f, "Comfortable"),
            UiDensity::Spacious => write!(f, "Spacious"),
        }
    }
}