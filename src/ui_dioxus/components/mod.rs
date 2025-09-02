pub mod appearance_settings;
pub mod claude_config_admin;
pub mod claude_output_modal_simple;
pub mod general_settings;
pub mod workspace_settings;
// pub mod execution_monitor;  // Uses non-existent task_execution module
// pub mod execution_modal;     // Uses non-existent task_execution module
pub mod task_editor;
pub mod task_edit_modal;
pub mod task_create_modal;
pub mod confirmation_dialog;
pub mod time_tracker;
pub mod export_button;

// Tests disabled - need dioxus_ssr crate
// #[cfg(test)]
// mod settings_context_test;
// #[cfg(test)]
// mod task_edit_modal_test;

// TODO: Fix these components - they need PartialEq on Repository which is complex
// pub use execution_monitor::{ExecutionMonitor, ExecutionHistoryPanel};
// pub use execution_modal::ExecutionDetailsModal;

pub use appearance_settings::AppearanceSettings;
pub use claude_config_admin::ClaudeConfigAdmin;
pub use claude_output_modal_simple::ClaudeOutputModal;
pub use general_settings::GeneralSettings;
pub use task_editor::TaskEditor;
pub use task_edit_modal::TaskEditModal;
pub use task_create_modal::TaskCreateModal;
pub use confirmation_dialog::ConfirmationDialog;
pub use workspace_settings::WorkspaceSettings;
pub use time_tracker::TimeTracker;
pub use export_button::ExportButton;