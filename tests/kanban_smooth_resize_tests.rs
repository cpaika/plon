use plon::domain::task::{Task, TaskStatus};
use plon::ui::views::kanban_view_enhanced::{KanbanView, DragState, CardAnimation};
use eframe::egui::{Pos2, Vec2, Rect};
use uuid::Uuid;
use std::time::{Duration, Instant};

#[cfg(test)]
mod kanban_smooth_resize_tests {
    use super::*;

    fn create_test_kanban_with_animation() -> KanbanView {
        let mut kanban = KanbanView::new();
        kanban.enable_smooth_animations = true;
        kanban.animation_duration = Duration::from_millis(200);
        kanban.update_layout(1200.0);
        kanban
    }

    fn create_task_with_height(title: &str, height: f32) -> Task {
        let mut task = Task::new(title.to_string(), String::new());
        // Add content to affect height
        for i in 0..((height / 20.0) as usize) {
            task.subtasks.push(plon::domain::task::SubTask {
                id: Uuid::new_v4(),
                title: format!("Subtask {}", i),
                description: String::new(),
                completed: false,
                created_at: chrono::Utc::now(),
                completed_at: None,
            });
        }
        task
    }

    // Test 1: Cards smoothly resize when dragged over them
    #[test]
    fn test_cards_resize_smoothly_when_dragged_over() {
        let mut kanban = create_test_kanban_with_animation();
        
        // Add cards with different heights
        let card1 = create_task_with_height("Card 1", 80.0);
        let card2 = create_task_with_height("Card 2", 100.0);
        let card3 = create_task_with_height("Card 3", 120.0);
        let dragged = create_task_with_height("Dragged", 90.0);
        
        let card1_id = card1.id;
        let card2_id = card2.id;
        let card3_id = card3.id;
        let dragged_id = dragged.id;
        
        kanban.add_task(card1);
        kanban.add_task(card2);
        kanban.add_task(card3);
        kanban.add_task(dragged);
        
        // Start dragging
        kanban.start_drag_with_animation(dragged_id, Pos2::new(100.0, 200.0));
        
        // Hover between card1 and card2
        let hover_pos = kanban.calculate_insert_position(0, 1);
        kanban.update_drag_with_animation(hover_pos);
        
        // Check that space is being made between cards
        let gap_height = kanban.get_animated_gap_height(0, 1);
        assert!(gap_height > 0.0);
        assert!(gap_height <= 90.0); // Should animate towards dragged card height
    }

    // Test 2: Animation timing for resize
    #[test]
    fn test_resize_animation_timing() {
        let mut kanban = create_test_kanban_with_animation();
        
        let card1 = create_task_with_height("Card 1", 80.0);
        let card2 = create_task_with_height("Card 2", 80.0);
        let dragged = create_task_with_height("Dragged", 100.0);
        
        kanban.add_task(card1);
        kanban.add_task(card2);
        let dragged_id = dragged.id;
        kanban.add_task(dragged);
        
        // Start drag
        let start_time = Instant::now();
        kanban.start_drag_with_animation(dragged_id, Pos2::new(100.0, 200.0));
        
        // Update position to trigger animation
        kanban.update_drag_with_animation(Pos2::new(100.0, 250.0));
        kanban.set_hover_insert_position(0, 1);
        
        // Check animation progress over time
        let mut last_gap = 0.0;
        for _ in 0..10 {
            kanban.update_animations(start_time.elapsed());
            let current_gap = kanban.get_animated_gap_height(0, 1);
            
            // Gap should be increasing (opening up)
            assert!(current_gap >= last_gap);
            last_gap = current_gap;
            
            // Simulate frame update
            std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }
        
        // After animation completes, gap should be at target height
        kanban.update_animations(Duration::from_millis(300));
        let final_gap = kanban.get_animated_gap_height(0, 1);
        assert_eq!(final_gap, 100.0); // Height of dragged card
    }

    // Test 3: Multiple cards shift smoothly
    #[test]
    fn test_multiple_cards_shift_smoothly() {
        let mut kanban = create_test_kanban_with_animation();
        
        // Create column with 5 cards
        let mut cards = Vec::new();
        for i in 0..5 {
            let card = create_task_with_height(&format!("Card {}", i), 80.0);
            cards.push(card.id);
            kanban.add_task(card);
        }
        
        let dragged = create_task_with_height("Dragged", 100.0);
        let dragged_id = dragged.id;
        kanban.add_task(dragged);
        
        // Start dragging from bottom
        kanban.start_drag_with_animation(dragged_id, Pos2::new(100.0, 500.0));
        
        // Move to top of column
        kanban.update_drag_with_animation(Pos2::new(100.0, 100.0));
        kanban.set_hover_insert_position(0, 0);
        
        // All cards below should shift down smoothly
        kanban.update_animations(Duration::from_millis(100));
        
        for (idx, _) in cards.iter().enumerate() {
            let offset = kanban.get_card_animation_offset(0, idx);
            assert!(offset.y > 0.0); // Cards should be shifting down
        }
    }

    // Test 4: Gap closes smoothly when drag leaves
    #[test]
    fn test_gap_closes_when_drag_leaves() {
        let mut kanban = create_test_kanban_with_animation();
        
        let card1 = create_task_with_height("Card 1", 80.0);
        let card2 = create_task_with_height("Card 2", 80.0);
        let dragged = create_task_with_height("Dragged", 100.0);
        
        kanban.add_task(card1);
        kanban.add_task(card2);
        let dragged_id = dragged.id;
        kanban.add_task(dragged);
        
        // Start drag and create gap
        kanban.start_drag_with_animation(dragged_id, Pos2::new(100.0, 200.0));
        kanban.update_drag_with_animation(Pos2::new(100.0, 250.0));
        kanban.set_hover_insert_position(0, 1);
        
        // Animate gap opening
        kanban.update_animations(Duration::from_millis(200));
        let gap_open = kanban.get_animated_gap_height(0, 1);
        assert_eq!(gap_open, 100.0);
        
        // Move drag away from column
        kanban.update_drag_with_animation(Pos2::new(500.0, 250.0));
        kanban.clear_hover_insert_position();
        
        // Gap should start closing
        for i in 0..10 {
            kanban.update_animations(Duration::from_millis(200 + (i * 16)));
            let current_gap = kanban.get_animated_gap_height(0, 1);
            assert!(current_gap <= gap_open);
        }
        
        // Gap should be fully closed
        kanban.update_animations(Duration::from_millis(500));
        let gap_closed = kanban.get_animated_gap_height(0, 1);
        assert_eq!(gap_closed, 0.0);
    }

    // Test 5: Smooth transition between hover positions
    #[test]
    fn test_smooth_transition_between_positions() {
        let mut kanban = create_test_kanban_with_animation();
        
        // Create column with 4 cards
        for i in 0..4 {
            let card = create_task_with_height(&format!("Card {}", i), 80.0);
            kanban.add_task(card);
        }
        
        let dragged = create_task_with_height("Dragged", 100.0);
        let dragged_id = dragged.id;
        kanban.add_task(dragged);
        
        // Start drag
        kanban.start_drag_with_animation(dragged_id, Pos2::new(100.0, 200.0));
        
        // Hover at position 1
        kanban.set_hover_insert_position(0, 1);
        kanban.update_animations(Duration::from_millis(200));
        let gap1 = kanban.get_animated_gap_height(0, 1);
        
        // Move to position 3
        kanban.set_hover_insert_position(0, 3);
        
        // During transition, old gap should close and new gap should open
        kanban.update_animations(Duration::from_millis(250));
        let gap1_closing = kanban.get_animated_gap_height(0, 1);
        let gap3_opening = kanban.get_animated_gap_height(0, 3);
        
        assert!(gap1_closing < gap1); // Old gap closing
        assert!(gap3_opening > 0.0); // New gap opening
        
        // After full animation
        kanban.update_animations(Duration::from_millis(400));
        assert_eq!(kanban.get_animated_gap_height(0, 1), 0.0);
        assert_eq!(kanban.get_animated_gap_height(0, 3), 100.0);
    }

    // Test 6: Card opacity during drag
    #[test]
    fn test_dragged_card_visual_feedback() {
        let mut kanban = create_test_kanban_with_animation();
        
        let card = create_task_with_height("Card", 80.0);
        let card_id = card.id;
        kanban.add_task(card);
        
        // Before drag - full opacity
        let initial_opacity = kanban.get_card_opacity(card_id);
        assert_eq!(initial_opacity, 1.0);
        
        // During drag - the dragged card should be semi-transparent
        kanban.start_drag_with_animation(card_id, Pos2::new(100.0, 200.0));
        let drag_opacity = kanban.get_dragged_card_opacity();
        assert!(drag_opacity < 1.0);
        assert!(drag_opacity > 0.5); // Should still be visible
        
        // After drag completes
        kanban.complete_drag(1);
        let final_opacity = kanban.get_card_opacity(card_id);
        assert_eq!(final_opacity, 1.0);
    }

    // Test 7: Performance with many cards
    #[test]
    fn test_performance_with_many_cards() {
        let mut kanban = create_test_kanban_with_animation();
        
        // Add 50 cards to test performance
        let mut card_ids = Vec::new();
        for i in 0..50 {
            let card = create_task_with_height(&format!("Card {}", i), 80.0);
            card_ids.push(card.id);
            kanban.add_task(card);
        }
        
        let dragged = create_task_with_height("Dragged", 100.0);
        let dragged_id = dragged.id;
        kanban.add_task(dragged);
        
        let start = Instant::now();
        
        // Start drag
        kanban.start_drag_with_animation(dragged_id, Pos2::new(100.0, 200.0));
        
        // Move through multiple positions
        for i in 0..10 {
            let pos = i * 5;
            kanban.set_hover_insert_position(0, pos);
            kanban.update_animations(Duration::from_millis(16));
        }
        
        let elapsed = start.elapsed();
        // Animation updates should be fast even with many cards
        assert!(elapsed < Duration::from_millis(100));
    }

    // Test 8: Edge case - drag to empty column
    #[test]
    fn test_drag_to_empty_column_animation() {
        let mut kanban = create_test_kanban_with_animation();
        
        let card = create_task_with_height("Card", 80.0);
        let card_id = card.id;
        card.status = TaskStatus::Todo;
        kanban.add_task(card);
        
        // Start drag from Todo
        kanban.start_drag_with_animation(card_id, Pos2::new(100.0, 200.0));
        
        // Move to empty InProgress column
        kanban.update_drag_with_animation(Pos2::new(400.0, 200.0));
        
        // Should show drop indicator in empty column
        let drop_indicator = kanban.get_empty_column_drop_indicator(1);
        assert!(drop_indicator.visible);
        assert!(drop_indicator.opacity > 0.0);
    }

    // Test 9: Preserve animations during scroll
    #[test]
    fn test_animations_during_scroll() {
        let mut kanban = create_test_kanban_with_animation();
        
        // Add many cards to enable scrolling
        for i in 0..20 {
            let card = create_task_with_height(&format!("Card {}", i), 80.0);
            kanban.add_task(card);
        }
        
        let dragged = create_task_with_height("Dragged", 100.0);
        let dragged_id = dragged.id;
        kanban.add_task(dragged);
        
        // Start drag
        kanban.start_drag_with_animation(dragged_id, Pos2::new(100.0, 200.0));
        kanban.set_hover_insert_position(0, 5);
        
        // Simulate scroll
        kanban.handle_scroll_offset(Vec2::new(0.0, 100.0));
        
        // Animation should continue despite scroll
        let gap = kanban.get_animated_gap_height(0, 5);
        assert!(gap > 0.0);
    }

    // Test 10: Snap-to-position behavior
    #[test]
    fn test_snap_to_position_on_drop() {
        let mut kanban = create_test_kanban_with_animation();
        
        let card1 = create_task_with_height("Card 1", 80.0);
        let card2 = create_task_with_height("Card 2", 80.0);
        let dragged = create_task_with_height("Dragged", 100.0);
        
        kanban.add_task(card1);
        kanban.add_task(card2);
        let dragged_id = dragged.id;
        kanban.add_task(dragged);
        
        // Start drag
        kanban.start_drag_with_animation(dragged_id, Pos2::new(100.0, 400.0));
        
        // Move to between cards
        kanban.update_drag_with_animation(Pos2::new(100.0, 150.0));
        kanban.set_hover_insert_position(0, 1);
        
        // Drop the card
        kanban.complete_drag_with_animation(0, 1);
        
        // Card should animate to final position
        let animation_state = kanban.get_drop_animation_state(dragged_id);
        assert!(animation_state.is_animating);
        
        // After animation completes
        kanban.update_animations(Duration::from_millis(300));
        
        // Card should be in final position
        let final_pos = kanban.get_card_position(dragged_id);
        assert!(final_pos.is_some());
    }
}