use super::kanban_view::{KanbanView, CardStyle, AnimationType, EasingType};
use crate::domain::task::{Task, TaskStatus, Priority};
use eframe::egui::{Color32, Stroke, Rounding};
use uuid::Uuid;
use std::time::Duration;
use chrono::Utc;

impl KanbanView {
    // Card hover and selection
    pub fn set_hovered_card(&mut self, card_id: Option<Uuid>) {
        if let Some(id) = card_id {
            if !self.is_card_hovered(id) {
                self.start_card_animation(id, AnimationType::HoverIn);
            }
        } else if let Some(prev_id) = self.hovered_card {
            self.start_card_animation(prev_id, AnimationType::HoverOut);
        }
        self.hovered_card = card_id;
    }
    
    pub fn is_card_hovered(&self, task_id: Uuid) -> bool {
        self.hovered_card == Some(task_id)
    }
    
    pub fn is_card_selected(&self, task_id: Uuid) -> bool {
        self.selected_tasks.contains(&task_id)
    }
    
    pub fn handle_card_click(&mut self, task_id: Uuid) {
        if self.selected_tasks.contains(&task_id) {
            self.selected_tasks.remove(&task_id);
        } else {
            self.selected_tasks.insert(task_id);
        }
        self.start_card_animation(task_id, AnimationType::Click);
    }
    
    // Card borders based on priority
    pub fn get_card_border(&self, priority: Priority) -> Stroke {
        match priority {
            Priority::Critical => Stroke::new(2.0, Color32::from_rgb(239, 68, 68)),
            Priority::High => Stroke::new(1.5, Color32::from_rgb(251, 146, 60)),
            Priority::Medium => Stroke::new(1.5, Color32::from_rgb(251, 191, 36)),
            Priority::Low => Stroke::new(1.0, Color32::from_rgb(156, 163, 175)),
        }
    }
    
    // Progress badges
    pub fn should_show_progress_badge(&self, task_id: Uuid) -> bool {
        self.tasks.iter()
            .find(|t| t.id == task_id)
            .map(|t| !t.subtasks.is_empty())
            .unwrap_or(false)
    }
    
    pub fn get_progress_badge_text(&self, task_id: Uuid) -> String {
        if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
            let completed = task.subtasks.iter().filter(|st| st.completed).count();
            let total = task.subtasks.len();
            format!("{}/{}", completed, total)
        } else {
            "".to_string()
        }
    }
    
    pub fn get_progress_badge_color(&self, completion_ratio: f32) -> Color32 {
        if completion_ratio >= 1.0 {
            Color32::from_rgb(34, 197, 94) // Green
        } else if completion_ratio >= 0.5 {
            Color32::from_rgb(251, 191, 36) // Yellow
        } else {
            Color32::from_rgb(156, 163, 175) // Gray
        }
    }
    
    // Avatar generation
    pub fn get_avatar_initials(&self, task: &Task) -> String {
        if let Some(assignee) = &task.assignee {
            assignee.split_whitespace()
                .take(2)
                .map(|word| word.chars().next().unwrap_or(' '))
                .collect::<String>()
                .to_uppercase()
        } else {
            "".to_string()
        }
    }
    
    pub fn get_avatar_color(&self, name: &str) -> Color32 {
        // Generate consistent color based on name
        let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        let hue = (hash % 360) as f32;
        
        // Convert HSV to RGB for consistent, pleasant colors
        let h = hue / 60.0;
        let c = 1.0;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        
        let (r, g, b) = if h < 1.0 {
            (c, x, 0.0)
        } else if h < 2.0 {
            (x, c, 0.0)
        } else if h < 3.0 {
            (0.0, c, x)
        } else if h < 4.0 {
            (0.0, x, c)
        } else if h < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        Color32::from_rgb(
            (r * 200.0 + 55.0) as u8,
            (g * 200.0 + 55.0) as u8,
            (b * 200.0 + 55.0) as u8,
        )
    }
    
    // Tag colors
    pub fn get_tag_colors(&self, tags: &[String]) -> Vec<Color32> {
        tags.iter().map(|tag| {
            self.tag_colors.get(tag).copied().unwrap_or_else(|| {
                // Generate color if not cached
                self.get_avatar_color(tag)
            })
        }).collect()
    }
    
    pub fn get_tag_pill_style(&self) -> PillStyle {
        PillStyle {
            border_radius: Rounding::same(12.0),
            padding: eframe::egui::Vec2::new(8.0, 4.0),
        }
    }
    
    // Card menu visibility
    pub fn should_show_card_menu(&self, task_id: Uuid) -> bool {
        self.is_card_hovered(task_id)
    }
    
    pub fn get_card_menu_opacity(&self, task_id: Uuid) -> f32 {
        if self.is_card_hovered(task_id) {
            1.0
        } else {
            0.0
        }
    }
    
    // Due date indicators
    pub fn is_task_overdue(&self, task_id: Uuid) -> bool {
        self.tasks.iter()
            .find(|t| t.id == task_id)
            .and_then(|t| t.due_date)
            .map(|due| due < Utc::now())
            .unwrap_or(false)
    }
    
    pub fn get_due_date_indicator_color(&self, task_id: Uuid) -> Color32 {
        if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
            if let Some(due) = task.due_date {
                let now = Utc::now();
                if due < now {
                    Color32::from_rgb(239, 68, 68) // Red for overdue
                } else if due < now + chrono::Duration::days(1) {
                    Color32::from_rgb(251, 191, 36) // Yellow for upcoming
                } else {
                    Color32::from_rgb(156, 163, 175) // Gray for future
                }
            } else {
                Color32::TRANSPARENT
            }
        } else {
            Color32::TRANSPARENT
        }
    }
    
    // Blocked overlay
    pub fn should_show_blocked_overlay(&self, task_id: Uuid) -> bool {
        self.tasks.iter()
            .find(|t| t.id == task_id)
            .map(|t| t.status == TaskStatus::Blocked)
            .unwrap_or(false)
    }
    
    pub fn get_blocked_overlay_style(&self) -> OverlayStyle {
        OverlayStyle {
            color: Color32::from_rgba(239, 68, 68, 20),
            show_icon: true,
            icon: "🚫".to_string(),
        }
    }
    
    // Staggered animations
    pub fn start_staggered_animation(&mut self) {
        for (index, task) in self.tasks.iter().enumerate() {
            let delay = Duration::from_millis(index as u64 * 50);
            // Store delay for each card
        }
    }
    
    pub fn get_card_animation_delay(&self, task_id: Uuid) -> Duration {
        self.tasks.iter().position(|t| t.id == task_id)
            .map(|index| Duration::from_millis(index as u64 * 50))
            .unwrap_or(Duration::ZERO)
    }
    
    // Focus ring
    pub fn set_focused_card(&mut self, card_id: Option<Uuid>) {
        self.focused_card = card_id;
    }
    
    pub fn should_show_focus_ring(&self, task_id: Uuid) -> bool {
        self.focused_card == Some(task_id)
    }
    
    pub fn get_focus_ring_style(&self) -> FocusRingStyle {
        FocusRingStyle {
            color: Color32::from_rgb(59, 130, 246),
            width: 2.0,
            offset: 2.0,
        }
    }
    
    // Compact mode
    pub fn set_compact_mode(&mut self, compact: bool) {
        // Store compact mode preference
    }
    
    pub fn get_card_style(&self) -> CardStyle {
        CardStyle::default()
    }
    
    // Task visibility
    pub fn is_task_visible(&self, task_id: Uuid) -> bool {
        if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
            if task.is_archived && !self.show_archived {
                return false;
            }
            // Apply other filters
            true
        } else {
            false
        }
    }
}

pub struct PillStyle {
    pub border_radius: Rounding,
    pub padding: eframe::egui::Vec2,
}

pub struct OverlayStyle {
    pub color: Color32,
    pub show_icon: bool,
    pub icon: String,
}

pub struct FocusRingStyle {
    pub color: Color32,
    pub width: f32,
    pub offset: f32,
}