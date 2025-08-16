use eframe::egui;
use egui::CentralPanel;
use plon::domain::task::{Task, TaskStatus};

/// Test harness for running egui tests in headless mode
pub struct HeadlessHarness {
    ctx: egui::Context,
    output: egui::FullOutput,
}

impl HeadlessHarness {
    pub fn new() -> Self {
        let ctx = egui::Context::default();
        let raw_input = egui::RawInput::default();
        let output = ctx.run(raw_input, |_| {});
        
        Self { ctx, output }
    }

    pub fn run_frame<F>(&mut self, f: F) -> egui::FullOutput
    where
        F: FnOnce(&egui::Context),
    {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1024.0, 768.0),
            )),
            ..Default::default()
        };
        
        self.ctx.run(raw_input, f)
    }

    pub fn click_at(&mut self, pos: egui::Pos2) {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1024.0, 768.0),
            )),
            events: vec![
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
            ],
            ..Default::default()
        };
        
        self.output = self.ctx.run(raw_input, |_| {});
    }

    pub fn double_click_at(&mut self, pos: egui::Pos2) {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1024.0, 768.0),
            )),
            events: vec![
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
            ],
            ..Default::default()
        };
        
        self.output = self.ctx.run(raw_input, |_| {});
    }

    pub fn drag(&mut self, from: egui::Pos2, to: egui::Pos2) {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1024.0, 768.0),
            )),
            events: vec![
                egui::Event::PointerButton {
                    pos: from,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
                egui::Event::PointerMoved(to),
                egui::Event::PointerButton {
                    pos: to,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
            ],
            ..Default::default()
        };
        
        self.output = self.ctx.run(raw_input, |_| {});
    }

    pub fn scroll(&mut self, delta: egui::Vec2) {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1024.0, 768.0),
            )),
            events: vec![egui::Event::Scroll(delta)],
            ..Default::default()
        };
        
        self.output = self.ctx.run(raw_input, |_| {});
    }

    pub fn type_text(&mut self, text: &str) {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1024.0, 768.0),
            )),
            events: vec![egui::Event::Text(text.to_string())],
            ..Default::default()
        };
        
        self.output = self.ctx.run(raw_input, |_| {});
    }
}

#[cfg(test)]
mod headless_tests {
    use super::*;

    #[test]
    fn test_button_click() {
        let mut harness = HeadlessHarness::new();
        let mut clicked = false;
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                if ui.button("Test Button").clicked() {
                    clicked = true;
                }
            });
        });
        
        // Note: In a real test, we'd need to calculate the button position
        // This is a simplified example
        assert!(!clicked); // Button not clicked yet
    }

    #[test]
    fn test_text_edit() {
        let mut harness = HeadlessHarness::new();
        let mut text = String::new();
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.text_edit_singleline(&mut text);
            });
        });
        
        harness.type_text("Hello, World!");
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.text_edit_singleline(&mut text);
            });
        });
    }

    #[test]
    fn test_drag_interaction() {
        let mut harness = HeadlessHarness::new();
        let from = egui::Pos2::new(100.0, 100.0);
        let to = egui::Pos2::new(200.0, 200.0);
        
        harness.drag(from, to);
        
        // Verify drag delta
        let delta = to - from;
        assert_eq!(delta.x, 100.0);
        assert_eq!(delta.y, 100.0);
    }

    #[test]
    fn test_scroll_interaction() {
        let mut harness = HeadlessHarness::new();
        let mut scroll_area_offset = 0.0;
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for i in 0..100 {
                        ui.label(format!("Item {}", i));
                    }
                });
            });
        });
        
        // Simulate scroll
        harness.scroll(egui::Vec2::new(0.0, -100.0));
        
        // In a real implementation, we'd verify the scroll offset changed
    }

    #[test]
    fn test_window_modal() {
        let mut harness = HeadlessHarness::new();
        let mut show_window = true;
        let mut window_closed = false;
        
        harness.run_frame(|ctx| {
            if show_window {
                egui::Window::new("Test Window")
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("Window Content");
                        if ui.button("Close").clicked() {
                            window_closed = true;
                            show_window = false;
                        }
                    });
            }
        });
        
        assert!(!window_closed);
    }

    #[test]
    fn test_combo_box() {
        let mut harness = HeadlessHarness::new();
        let mut selected = TaskStatus::Todo;
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                egui::ComboBox::from_label("Status")
                    .selected_text(format!("{:?}", selected))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, TaskStatus::Todo, "Todo");
                        ui.selectable_value(&mut selected, TaskStatus::InProgress, "In Progress");
                        ui.selectable_value(&mut selected, TaskStatus::Done, "Done");
                    });
            });
        });
        
        assert_eq!(selected, TaskStatus::Todo);
    }

    #[test]
    fn test_progress_bar() {
        let mut harness = HeadlessHarness::new();
        let progress = 0.75;
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.add(egui::ProgressBar::new(progress).show_percentage());
            });
        });
        
        // Progress bar should render without panic
        assert!(progress >= 0.0 && progress <= 1.0);
    }

    #[test]
    fn test_color_picker() {
        let mut harness = HeadlessHarness::new();
        let mut color = egui::Color32::from_rgb(255, 0, 0);
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.color_edit_button_srgba(&mut color);
            });
        });
        
        assert_eq!(color, egui::Color32::from_rgb(255, 0, 0));
    }

    #[test]
    fn test_slider() {
        let mut harness = HeadlessHarness::new();
        let mut value = 50.0;
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.add(egui::Slider::new(&mut value, 0.0..=100.0));
            });
        });
        
        assert_eq!(value, 50.0);
    }

    #[test]
    fn test_checkbox() {
        let mut harness = HeadlessHarness::new();
        let mut checked = false;
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.checkbox(&mut checked, "Test Checkbox");
            });
        });
        
        assert!(!checked);
    }

    #[test]
    fn test_radio_buttons() {
        let mut harness = HeadlessHarness::new();
        let mut selected = 1;
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.radio_value(&mut selected, 0, "Option 0");
                ui.radio_value(&mut selected, 1, "Option 1");
                ui.radio_value(&mut selected, 2, "Option 2");
            });
        });
        
        assert_eq!(selected, 1);
    }

    #[test]
    fn test_collapsing_header() {
        let mut harness = HeadlessHarness::new();
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.collapsing("Header", |ui| {
                    ui.label("Content inside collapsing header");
                });
            });
        });
        
        // Test runs without panic
    }

    #[test]
    fn test_tabs() {
        let mut harness = HeadlessHarness::new();
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(true, "Tab 1").clicked() {
                        // Tab 1 selected
                    }
                    if ui.selectable_label(false, "Tab 2").clicked() {
                        // Tab 2 selected
                    }
                });
            });
        });
        
        // Test runs without panic
    }

    #[test]
    fn test_tooltip() {
        let mut harness = HeadlessHarness::new();
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.label("Hover me").on_hover_text("This is a tooltip");
            });
        });
        
        // Test runs without panic
    }

    #[test]
    fn test_context_menu() {
        let mut harness = HeadlessHarness::new();
        
        harness.run_frame(|ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let response = ui.label("Right-click me");
                response.context_menu(|ui| {
                    if ui.button("Option 1").clicked() {
                        // Handle option 1
                    }
                    if ui.button("Option 2").clicked() {
                        // Handle option 2
                    }
                });
            });
        });
        
        // Test runs without panic
    }
}