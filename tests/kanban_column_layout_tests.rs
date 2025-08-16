use plon::ui::views::kanban_view::{KanbanColumn, KanbanView, ColumnLayout};
use plon::domain::task::{Task, TaskStatus};
use eframe::egui::{self, Pos2, Rect, Vec2};
use uuid::Uuid;

#[cfg(test)]
mod column_layout_tests {
    use super::*;

    fn create_test_kanban_with_columns() -> KanbanView {
        let mut kanban = KanbanView::new();
        
        // Ensure we have the standard columns
        kanban.columns = vec![
            KanbanColumn {
                id: Uuid::new_v4(),
                title: "To Do".to_string(),
                status: TaskStatus::Todo,
                color: egui::Color32::from_rgb(229, 231, 235),
                tasks: vec![],
                width: 300.0,
                min_width: 250.0,
                max_width: 500.0,
                is_collapsed: false,
                wip_limit: None,
                bounds: Rect::NOTHING,
                is_resizing: false,
                resize_handle_hovered: false,
            },
            KanbanColumn {
                id: Uuid::new_v4(),
                title: "In Progress".to_string(),
                status: TaskStatus::InProgress,
                color: egui::Color32::from_rgb(251, 191, 36),
                tasks: vec![],
                width: 300.0,
                min_width: 250.0,
                max_width: 500.0,
                is_collapsed: false,
                wip_limit: Some(3),
                bounds: Rect::NOTHING,
                is_resizing: false,
                resize_handle_hovered: false,
            },
            KanbanColumn {
                id: Uuid::new_v4(),
                title: "Review".to_string(),
                status: TaskStatus::Review,
                color: egui::Color32::from_rgb(147, 197, 253),
                tasks: vec![],
                width: 300.0,
                min_width: 250.0,
                max_width: 500.0,
                is_collapsed: false,
                wip_limit: Some(2),
                bounds: Rect::NOTHING,
                is_resizing: false,
                resize_handle_hovered: false,
            },
            KanbanColumn {
                id: Uuid::new_v4(),
                title: "Done".to_string(),
                status: TaskStatus::Done,
                color: egui::Color32::from_rgb(134, 239, 172),
                tasks: vec![],
                width: 300.0,
                min_width: 250.0,
                max_width: 500.0,
                is_collapsed: false,
                wip_limit: None,
                bounds: Rect::NOTHING,
                is_resizing: false,
                resize_handle_hovered: false,
            },
        ];
        
        kanban
    }

    #[test]
    fn test_responsive_column_width_calculation() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Set viewport width
        let viewport_width = 1200.0;
        kanban.calculate_responsive_column_widths(viewport_width);
        
        // Each column should get equal width minus gaps
        let gap = 16.0;
        let total_gaps = gap * (kanban.columns.len() - 1) as f32;
        let expected_width = (viewport_width - total_gaps) / kanban.columns.len() as f32;
        
        for column in &kanban.columns {
            assert!((column.width - expected_width).abs() < 1.0);
        }
    }

    #[test]
    fn test_column_min_max_width_constraints() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Test minimum width constraint
        kanban.resize_column(0, 100.0); // Try to set below minimum
        assert_eq!(kanban.columns[0].width, 250.0); // Should be clamped to min
        
        // Test maximum width constraint
        kanban.resize_column(0, 600.0); // Try to set above maximum
        assert_eq!(kanban.columns[0].width, 500.0); // Should be clamped to max
        
        // Test valid width
        kanban.resize_column(0, 350.0);
        assert_eq!(kanban.columns[0].width, 350.0);
    }

    #[test]
    fn test_column_resize_handle_detection() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Set column bounds
        kanban.columns[0].bounds = Rect::from_min_size(
            Pos2::new(0.0, 0.0),
            Vec2::new(300.0, 600.0)
        );
        
        // Test resize handle area (should be within 8px of right edge)
        assert!(kanban.is_over_resize_handle(Pos2::new(295.0, 300.0), 0));
        assert!(kanban.is_over_resize_handle(Pos2::new(298.0, 300.0), 0));
        assert!(!kanban.is_over_resize_handle(Pos2::new(280.0, 300.0), 0));
    }

    #[test]
    fn test_column_collapse_expand() {
        let mut kanban = create_test_kanban_with_columns();
        let original_width = kanban.columns[0].width;
        
        // Collapse column
        kanban.toggle_column_collapse(0);
        assert!(kanban.columns[0].is_collapsed);
        assert_eq!(kanban.columns[0].width, 50.0); // Collapsed width
        
        // Expand column
        kanban.toggle_column_collapse(0);
        assert!(!kanban.columns[0].is_collapsed);
        assert_eq!(kanban.columns[0].width, original_width); // Restored to original
    }

    #[test]
    fn test_fluid_layout_on_window_resize() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Test different viewport sizes
        let viewport_sizes = vec![1920.0, 1440.0, 1024.0, 768.0];
        
        for viewport_width in viewport_sizes {
            kanban.handle_viewport_resize(viewport_width);
            
            // Verify total width doesn't exceed viewport
            let total_width: f32 = kanban.columns.iter()
                .map(|c| c.width)
                .sum::<f32>() + (16.0 * (kanban.columns.len() - 1) as f32);
            
            assert!(total_width <= viewport_width);
            
            // Verify minimum widths are respected
            for column in &kanban.columns {
                assert!(column.width >= column.min_width);
            }
        }
    }

    #[test]
    fn test_column_reordering() {
        let mut kanban = create_test_kanban_with_columns();
        let first_id = kanban.columns[0].id;
        let second_id = kanban.columns[1].id;
        
        // Swap first two columns
        kanban.reorder_columns(0, 1);
        
        assert_eq!(kanban.columns[0].id, second_id);
        assert_eq!(kanban.columns[1].id, first_id);
    }

    #[test]
    fn test_column_header_sticky_scroll() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Set scroll position
        kanban.scroll_offset = Vec2::new(0.0, 100.0);
        
        // Headers should remain at top regardless of scroll
        for column in &kanban.columns {
            let header_pos = kanban.get_column_header_position(column.id);
            assert_eq!(header_pos.y, 0.0); // Headers stay at top
        }
    }

    #[test]
    fn test_column_auto_scroll_horizontal() {
        let mut kanban = create_test_kanban_with_columns();
        kanban.viewport_width = 1200.0;
        
        // Add many columns to require scrolling
        for i in 0..10 {
            kanban.add_custom_column(format!("Column {}", i));
        }
        
        // Test scroll to column
        kanban.scroll_to_column(8);
        
        // Verify column is visible
        assert!(kanban.is_column_visible(8));
    }

    #[test]
    fn test_column_gap_adjustment() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Test different gap sizes
        let gap_sizes = vec![8.0, 16.0, 24.0, 32.0];
        
        for gap in gap_sizes {
            kanban.set_column_gap(gap);
            kanban.calculate_column_positions();
            
            // Verify gaps between columns
            for i in 1..kanban.columns.len() {
                let prev_right = kanban.columns[i-1].bounds.max.x;
                let curr_left = kanban.columns[i].bounds.min.x;
                assert_eq!(curr_left - prev_right, gap);
            }
        }
    }

    #[test]
    fn test_column_equal_height() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Add different number of tasks to columns
        for i in 0..5 {
            let mut task = Task::new_simple(format!("Task {}", i));
            task.status = TaskStatus::Todo;
            kanban.tasks.push(task);
        }
        
        for i in 5..7 {
            let mut task = Task::new_simple(format!("Task {}", i));
            task.status = TaskStatus::InProgress;
            kanban.tasks.push(task);
        }
        
        kanban.equalize_column_heights();
        
        // All columns should have same height
        let first_height = kanban.columns[0].bounds.height();
        for column in &kanban.columns {
            assert_eq!(column.bounds.height(), first_height);
        }
    }

    #[test]
    fn test_column_hide_show() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Hide column
        kanban.hide_column(1);
        assert!(!kanban.is_column_visible(1));
        
        // Hidden column shouldn't take space
        kanban.calculate_column_positions();
        assert_eq!(kanban.columns[1].width, 0.0);
        
        // Show column
        kanban.show_column(1);
        assert!(kanban.is_column_visible(1));
        assert!(kanban.columns[1].width > 0.0);
    }

    #[test]
    fn test_column_fixed_vs_flexible_width() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Set first column as fixed width
        kanban.set_column_fixed_width(0, 400.0);
        
        // Set viewport and recalculate
        kanban.calculate_responsive_column_widths(1200.0);
        
        // First column should maintain fixed width
        assert_eq!(kanban.columns[0].width, 400.0);
        
        // Other columns should share remaining space
        let remaining_width = 1200.0 - 400.0 - (16.0 * 3.0); // minus gaps
        let flexible_width = remaining_width / 3.0;
        
        for i in 1..4 {
            assert!((kanban.columns[i].width - flexible_width).abs() < 1.0);
        }
    }

    #[test]
    fn test_column_responsive_breakpoints() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Test mobile breakpoint (< 768px)
        kanban.apply_responsive_layout(600.0);
        assert_eq!(kanban.get_layout_mode(), LayoutMode::Stacked);
        
        // Test tablet breakpoint (768-1024px)
        kanban.apply_responsive_layout(900.0);
        assert_eq!(kanban.get_layout_mode(), LayoutMode::Compact);
        
        // Test desktop breakpoint (> 1024px)
        kanban.apply_responsive_layout(1400.0);
        assert_eq!(kanban.get_layout_mode(), LayoutMode::Full);
    }

    #[test]
    fn test_column_content_overflow() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Add many tasks to a column
        for i in 0..20 {
            let mut task = Task::new_simple(format!("Task {}", i));
            task.status = TaskStatus::Todo;
            kanban.tasks.push(task);
        }
        
        // Check if column needs scrollbar
        assert!(kanban.column_needs_scrollbar(0));
        
        // Get scrollable height
        let content_height = kanban.get_column_content_height(0);
        let visible_height = kanban.columns[0].bounds.height();
        assert!(content_height > visible_height);
    }

    #[test]
    fn test_column_resize_neighbors() {
        let mut kanban = create_test_kanban_with_columns();
        kanban.viewport_width = 1200.0;
        
        // Enable neighbor resizing
        kanban.enable_neighbor_resize = true;
        
        let original_width_0 = kanban.columns[0].width;
        let original_width_1 = kanban.columns[1].width;
        
        // Resize first column larger
        kanban.resize_column_with_neighbor(0, 350.0);
        
        // First column should be larger
        assert_eq!(kanban.columns[0].width, 350.0);
        
        // Second column should be smaller by the same amount
        let diff = 350.0 - original_width_0;
        assert_eq!(kanban.columns[1].width, original_width_1 - diff);
    }

    #[test]
    fn test_column_drag_to_reorder() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Start dragging column
        kanban.start_column_drag(0, Pos2::new(150.0, 50.0));
        assert!(kanban.is_dragging_column());
        
        // Drag to new position
        kanban.update_column_drag(Pos2::new(450.0, 50.0));
        
        // Drop column
        let original_first = kanban.columns[0].id;
        kanban.drop_column();
        
        // Column should have moved
        assert_ne!(kanban.columns[0].id, original_first);
    }

    #[test]
    fn test_column_width_persistence() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Set custom widths
        kanban.resize_column(0, 350.0);
        kanban.resize_column(1, 280.0);
        
        // Save column widths
        let saved_widths = kanban.save_column_widths();
        
        // Reset to defaults
        kanban.reset_column_widths();
        
        // Restore saved widths
        kanban.restore_column_widths(saved_widths);
        
        assert_eq!(kanban.columns[0].width, 350.0);
        assert_eq!(kanban.columns[1].width, 280.0);
    }

    #[test]
    fn test_column_auto_balance() {
        let mut kanban = create_test_kanban_with_columns();
        
        // Set uneven widths
        kanban.columns[0].width = 400.0;
        kanban.columns[1].width = 250.0;
        kanban.columns[2].width = 350.0;
        kanban.columns[3].width = 200.0;
        
        // Auto-balance columns
        kanban.auto_balance_columns();
        
        // All columns should have equal width
        let expected_width = kanban.columns[0].width;
        for column in &kanban.columns {
            assert_eq!(column.width, expected_width);
        }
    }
}