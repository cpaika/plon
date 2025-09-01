-- Create app_settings table for storing application preferences
CREATE TABLE IF NOT EXISTS app_settings (
    id TEXT PRIMARY KEY NOT NULL,
    
    -- General Settings
    default_task_status TEXT NOT NULL DEFAULT '"Todo"',
    auto_save_interval_seconds INTEGER NOT NULL DEFAULT 30,
    enable_notifications INTEGER NOT NULL DEFAULT 1,
    notification_sound INTEGER NOT NULL DEFAULT 1,
    date_format TEXT NOT NULL DEFAULT 'MM/DD/YYYY',
    time_format TEXT NOT NULL DEFAULT '12h',
    week_starts_on TEXT NOT NULL DEFAULT 'Sunday',
    enable_time_tracking INTEGER NOT NULL DEFAULT 1,
    show_task_numbers INTEGER NOT NULL DEFAULT 0,
    
    -- Workspace Settings
    default_project_directory TEXT NOT NULL DEFAULT '~/plon-projects',
    database_path TEXT NOT NULL DEFAULT 'plon.db',
    enable_auto_backup INTEGER NOT NULL DEFAULT 1,
    backup_directory TEXT NOT NULL DEFAULT '~/plon-backups',
    backup_frequency_hours INTEGER NOT NULL DEFAULT 24,
    max_backups_to_keep INTEGER NOT NULL DEFAULT 7,
    enable_file_watching INTEGER NOT NULL DEFAULT 1,
    git_auto_commit INTEGER NOT NULL DEFAULT 0,
    task_template_directory TEXT,
    
    -- Appearance Settings
    theme TEXT NOT NULL DEFAULT '"Light"',
    accent_color TEXT NOT NULL DEFAULT '#3b82f6',
    font_size TEXT NOT NULL DEFAULT '"Medium"',
    ui_density TEXT NOT NULL DEFAULT '"Comfortable"',
    sidebar_position TEXT NOT NULL DEFAULT '"Left"',
    show_sidebar INTEGER NOT NULL DEFAULT 1,
    show_toolbar INTEGER NOT NULL DEFAULT 1,
    show_statusbar INTEGER NOT NULL DEFAULT 1,
    enable_animations INTEGER NOT NULL DEFAULT 1,
    
    -- Integration Settings
    enable_github_integration INTEGER NOT NULL DEFAULT 0,
    enable_slack_integration INTEGER NOT NULL DEFAULT 0,
    slack_webhook_url TEXT,
    enable_discord_integration INTEGER NOT NULL DEFAULT 0,
    discord_webhook_url TEXT,
    enable_calendar_sync INTEGER NOT NULL DEFAULT 0,
    calendar_provider TEXT,
    
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);