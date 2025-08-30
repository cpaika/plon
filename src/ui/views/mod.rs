pub mod claude_code_view;
pub mod dashboard_view;
pub mod gantt_view;
pub mod goal_view;
pub mod kanban_view_enhanced;
pub mod kanban_view_improved;
pub mod list_view;
pub mod map_view;
pub mod metadata_config_view;
pub mod recurring_view;
pub mod resource_view;
pub mod timeline_view;
// #[cfg(test)]
// mod map_view_comprehensive_tests;  // Temporarily disabled - needs fixes
#[cfg(test)]
mod arrow_flicker_test;
#[cfg(test)]
mod arrow_visibility_test;
#[cfg(test)]
mod claude_code_e2e_tests;
#[cfg(test)]
mod play_button_egui_test;
#[cfg(test)]
mod play_button_test;
