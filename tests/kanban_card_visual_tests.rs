use plon::domain::task::{Priority, Task, TaskStatus};
use plon::ui::views::kanban_view::{KanbanView, CardStyle, CardAnimation};
use plon::ui::widgets::task_card::TaskCard;
use eframe::egui::{self, Color32, Context, Painter, Pos2, Rect, Rounding, Shadow, Stroke, Vec2};
use std::time::Duration;
use uuid::Uuid;

#[cfg(test)]
mod card_visual_tests {
    use super::*;

    fn create_test_task() -> Task {
        let mut task = Task::new_simple("Test Task".to_string());
        task.id = Uuid::new_v4();
        task.priority = Priority::High;
        task.status = TaskStatus::InProgress;
        task
    }

    #[test]
    fn test_card_shadow_rendering() {
        let task = create_test_task();
        let card_style = CardStyle::default();
        
        // Test default shadow
        assert_eq!(card_style.shadow.extrusion, 8.0);
        assert_eq!(card_style.shadow.color, Color32::from_black_alpha(40));
        
        // Test hover shadow (should be more prominent)
        let hover_style = CardStyle::hover();
        assert_eq!(hover_style.shadow.extrusion, 16.0);
        assert_eq!(hover_style.shadow.color, Color32::from_black_alpha(60));
        
        // Test dragging shadow (should be most prominent)
        let drag_style = CardStyle::dragging();
        assert_eq!(drag_style.shadow.extrusion, 24.0);
        assert_eq!(drag_style.shadow.color, Color32::from_black_alpha(80));
    }

    #[test]
    fn test_card_hover_effects() {
        let mut kanban = KanbanView::new();
        let task = create_test_task();
        kanban.tasks.push(task.clone());
        
        // Simulate hover
        kanban.set_hovered_card(Some(task.id));
        
        // Check hover state
        assert!(kanban.is_card_hovered(task.id));
        
        // Check hover animation started
        assert!(kanban.card_animations.contains_key(&task.id));
        
        let animation = &kanban.card_animations[&task.id];
        assert_eq!(animation.animation_type, AnimationType::HoverIn);
        assert!(animation.start_time.elapsed() < Duration::from_millis(100));
    }

    #[test]
    fn test_card_transition_animations() {
        let mut kanban = KanbanView::new();
        let task = create_test_task();
        kanban.tasks.push(task.clone());
        
        // Test scale animation on hover
        kanban.start_card_animation(task.id, AnimationType::HoverIn);
        
        // Check animation progress at different time points
        let progress_0 = kanban.get_animation_progress(task.id, 0.0);
        assert_eq!(progress_0, 0.0);
        
        let progress_50 = kanban.get_animation_progress(task.id, 0.5);
        assert!((progress_50 - 0.5).abs() < 0.01);
        
        let progress_100 = kanban.get_animation_progress(task.id, 1.0);
        assert_eq!(progress_100, 1.0);
        
        // Test easing function
        let eased_value = kanban.apply_easing(0.5, EasingType::EaseInOut);
        assert!(eased_value > 0.4 && eased_value < 0.6);
    }

    #[test]
    fn test_card_corner_radius() {
        let card_style = CardStyle::default();
        
        // Test default corner radius
        assert_eq!(card_style.corner_radius, Rounding::same(8.0));
        
        // Test modern card design with larger radius
        let modern_style = CardStyle::modern();
        assert_eq!(modern_style.corner_radius, Rounding::same(12.0));
    }

    #[test]
    fn test_card_border_styling() {
        let mut kanban = KanbanView::new();
        
        // Test different priority borders
        let high_priority_border = kanban.get_card_border(Priority::Critical);
        assert_eq!(high_priority_border.color, Color32::from_rgb(239, 68, 68)); // Red
        assert_eq!(high_priority_border.width, 2.0);
        
        let medium_priority_border = kanban.get_card_border(Priority::Medium);
        assert_eq!(medium_priority_border.color, Color32::from_rgb(251, 191, 36)); // Yellow
        assert_eq!(medium_priority_border.width, 1.5);
        
        let low_priority_border = kanban.get_card_border(Priority::Low);
        assert_eq!(low_priority_border.color, Color32::from_rgb(156, 163, 175)); // Gray
        assert_eq!(low_priority_border.width, 1.0);
    }

    #[test]
    fn test_card_background_gradient() {
        let card_style = CardStyle::default();
        
        // Test gradient colors
        assert!(card_style.use_gradient);
        assert_eq!(card_style.gradient_start, Color32::from_rgb(255, 255, 255));
        assert_eq!(card_style.gradient_end, Color32::from_rgb(249, 250, 251));
        
        // Test dark mode gradient
        let dark_style = CardStyle::dark_mode();
        assert_eq!(dark_style.gradient_start, Color32::from_rgb(31, 41, 55));
        assert_eq!(dark_style.gradient_end, Color32::from_rgb(17, 24, 39));
    }

    #[test]
    fn test_card_content_spacing() {
        let card_style = CardStyle::default();
        
        // Test padding
        assert_eq!(card_style.padding, Vec2::new(16.0, 12.0));
        
        // Test content margins
        assert_eq!(card_style.title_margin_bottom, 8.0);
        assert_eq!(card_style.tag_spacing, 4.0);
        assert_eq!(card_style.footer_margin_top, 12.0);
    }

    #[test]
    fn test_card_interaction_feedback() {
        let mut kanban = KanbanView::new();
        let task = create_test_task();
        kanban.tasks.push(task.clone());
        
        // Test click feedback
        kanban.handle_card_click(task.id);
        assert!(kanban.is_card_selected(task.id));
        assert!(kanban.card_animations.contains_key(&task.id));
        
        let animation = &kanban.card_animations[&task.id];
        assert_eq!(animation.animation_type, AnimationType::Click);
        assert_eq!(animation.duration, Duration::from_millis(150));
    }

    #[test]
    fn test_card_loading_skeleton() {
        let card_style = CardStyle::skeleton();
        
        // Test skeleton animation
        assert!(card_style.is_skeleton);
        assert!(card_style.skeleton_shimmer);
        assert_eq!(card_style.skeleton_color, Color32::from_rgb(229, 231, 235));
        assert_eq!(card_style.shimmer_duration, Duration::from_millis(1500));
    }

    #[test]
    fn test_card_badge_rendering() {
        let mut kanban = KanbanView::new();
        let mut task = create_test_task();
        
        // Add subtasks to show progress badge
        task.subtasks = vec![
            SubTask { title: "Sub 1".to_string(), completed: true },
            SubTask { title: "Sub 2".to_string(), completed: false },
        ];
        
        kanban.tasks.push(task.clone());
        
        // Test badge visibility
        assert!(kanban.should_show_progress_badge(task.id));
        
        // Test badge content
        let badge_text = kanban.get_progress_badge_text(task.id);
        assert_eq!(badge_text, "1/2");
        
        // Test badge color based on completion
        let badge_color = kanban.get_progress_badge_color(0.5);
        assert_eq!(badge_color, Color32::from_rgb(251, 191, 36)); // Yellow for partial
    }

    #[test]
    fn test_card_avatar_display() {
        let mut kanban = KanbanView::new();
        let mut task = create_test_task();
        task.assignee = Some("John Doe".to_string());
        kanban.tasks.push(task.clone());
        
        // Test avatar generation
        let avatar_initials = kanban.get_avatar_initials(&task);
        assert_eq!(avatar_initials, "JD");
        
        // Test avatar color (should be consistent for same name)
        let avatar_color = kanban.get_avatar_color("John Doe");
        let avatar_color2 = kanban.get_avatar_color("John Doe");
        assert_eq!(avatar_color, avatar_color2);
    }

    #[test]
    fn test_card_tag_pills() {
        let mut kanban = KanbanView::new();
        let mut task = create_test_task();
        task.tags = vec!["urgent".to_string(), "bug".to_string(), "frontend".to_string()];
        kanban.tasks.push(task.clone());
        
        // Test tag pill colors
        let tag_colors = kanban.get_tag_colors(&task.tags);
        assert_eq!(tag_colors.len(), 3);
        
        // Each tag should have a unique color
        assert_ne!(tag_colors[0], tag_colors[1]);
        assert_ne!(tag_colors[1], tag_colors[2]);
        
        // Test tag pill styling
        let pill_style = kanban.get_tag_pill_style();
        assert_eq!(pill_style.border_radius, Rounding::same(12.0));
        assert_eq!(pill_style.padding, Vec2::new(8.0, 4.0));
    }

    #[test]
    fn test_card_overflow_menu() {
        let mut kanban = KanbanView::new();
        let task = create_test_task();
        kanban.tasks.push(task.clone());
        
        // Test menu visibility on hover
        kanban.set_hovered_card(Some(task.id));
        assert!(kanban.should_show_card_menu(task.id));
        
        // Test menu icon opacity
        let menu_opacity = kanban.get_card_menu_opacity(task.id);
        assert_eq!(menu_opacity, 1.0); // Full opacity when hovered
        
        // Test menu not visible when not hovered
        kanban.set_hovered_card(None);
        let menu_opacity = kanban.get_card_menu_opacity(task.id);
        assert_eq!(menu_opacity, 0.0);
    }

    #[test]
    fn test_card_due_date_indicator() {
        let mut kanban = KanbanView::new();
        let mut task = create_test_task();
        
        // Set overdue date
        task.due_date = Some(chrono::Utc::now() - chrono::Duration::days(1));
        kanban.tasks.push(task.clone());
        
        // Test overdue indicator
        assert!(kanban.is_task_overdue(task.id));
        
        let indicator_color = kanban.get_due_date_indicator_color(task.id);
        assert_eq!(indicator_color, Color32::from_rgb(239, 68, 68)); // Red for overdue
        
        // Test upcoming due date
        let mut upcoming_task = create_test_task();
        upcoming_task.due_date = Some(chrono::Utc::now() + chrono::Duration::days(1));
        kanban.tasks.push(upcoming_task.clone());
        
        let indicator_color = kanban.get_due_date_indicator_color(upcoming_task.id);
        assert_eq!(indicator_color, Color32::from_rgb(251, 191, 36)); // Yellow for upcoming
    }

    #[test]
    fn test_card_blocked_overlay() {
        let mut kanban = KanbanView::new();
        let mut task = create_test_task();
        task.status = TaskStatus::Blocked;
        kanban.tasks.push(task.clone());
        
        // Test blocked overlay
        assert!(kanban.should_show_blocked_overlay(task.id));
        
        let overlay_style = kanban.get_blocked_overlay_style();
        assert_eq!(overlay_style.color, Color32::from_rgba(239, 68, 68, 20)); // Semi-transparent red
        assert!(overlay_style.show_icon);
        assert_eq!(overlay_style.icon, "🚫");
    }

    #[test]
    fn test_card_animation_staggering() {
        let mut kanban = KanbanView::new();
        
        // Add multiple tasks
        for i in 0..5 {
            let mut task = create_test_task();
            task.title = format!("Task {}", i);
            kanban.tasks.push(task);
        }
        
        // Start staggered animation
        kanban.start_staggered_animation();
        
        // Check each card has incrementally delayed animation
        for (index, task) in kanban.tasks.iter().enumerate() {
            let delay = kanban.get_card_animation_delay(task.id);
            assert_eq!(delay, Duration::from_millis(index as u64 * 50));
        }
    }

    #[test]
    fn test_card_focus_ring() {
        let mut kanban = KanbanView::new();
        let task = create_test_task();
        kanban.tasks.push(task.clone());
        
        // Test focus ring on keyboard navigation
        kanban.set_focused_card(Some(task.id));
        
        assert!(kanban.should_show_focus_ring(task.id));
        
        let focus_style = kanban.get_focus_ring_style();
        assert_eq!(focus_style.color, Color32::from_rgb(59, 130, 246)); // Blue focus ring
        assert_eq!(focus_style.width, 2.0);
        assert_eq!(focus_style.offset, 2.0);
    }

    #[test]
    fn test_card_compact_mode() {
        let mut kanban = KanbanView::new();
        kanban.set_compact_mode(true);
        
        let compact_style = kanban.get_card_style();
        let normal_style = CardStyle::default();
        
        // Compact cards should have less padding
        assert!(compact_style.padding.x < normal_style.padding.x);
        assert!(compact_style.padding.y < normal_style.padding.y);
        
        // Smaller font size in compact mode
        assert!(compact_style.title_font_size < normal_style.title_font_size);
    }
}