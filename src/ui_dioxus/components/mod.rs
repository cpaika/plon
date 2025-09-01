pub mod appearance_settings;
pub mod claude_config_admin;
pub mod general_settings;
pub mod workspace_settings;
// pub mod execution_monitor;  // Uses non-existent task_execution module
// pub mod execution_modal;     // Uses non-existent task_execution module
pub mod task_editor;

#[cfg(test)]
mod settings_context_test;

// TODO: Fix these components - they need PartialEq on Repository which is complex
// pub use execution_monitor::{ExecutionMonitor, ExecutionHistoryPanel};
// pub use execution_modal::ExecutionDetailsModal;

pub use appearance_settings::AppearanceSettings;
pub use claude_config_admin::ClaudeConfigAdmin;
pub use general_settings::GeneralSettings;
pub use task_editor::TaskEditor;
pub use workspace_settings::WorkspaceSettings;