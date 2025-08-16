use eframe::egui::{self, Ui};
use chrono::{NaiveTime, Weekday, NaiveDate, Utc, Timelike};
use crate::domain::recurring::{RecurringTaskTemplate, RecurrenceRule, RecurrencePattern};
use crate::domain::task::Priority;
use std::collections::HashMap;

pub struct RecurringEditor {
    pub title: String,
    pub description: String,
    pub pattern: RecurrencePattern,
    pub interval: u32,
    pub selected_days: Vec<Weekday>,
    pub day_of_month: Option<u32>,
    pub month_of_year: Option<u32>,
    pub time: NaiveTime,
    pub end_date: Option<NaiveDate>,
    pub max_occurrences: Option<u32>,
    pub priority: Priority,
    pub estimated_hours: Option<f32>,
}

impl RecurringEditor {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            pattern: RecurrencePattern::Daily,
            interval: 1,
            selected_days: Vec::new(),
            day_of_month: None,
            month_of_year: None,
            time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_date: None,
            max_occurrences: None,
            priority: Priority::Medium,
            estimated_hours: None,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> bool {
        let mut should_save = false;
        
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Title:");
                ui.text_edit_singleline(&mut self.title);
            });
            
            ui.horizontal(|ui| {
                ui.label("Description:");
                ui.text_edit_multiline(&mut self.description);
            });
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Pattern:");
                egui::ComboBox::from_id_source("recurring_pattern_combo")
                    .selected_text(self.pattern_to_string(self.pattern))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.pattern, RecurrencePattern::Daily, "Daily");
                        ui.selectable_value(&mut self.pattern, RecurrencePattern::Weekly, "Weekly");
                        ui.selectable_value(&mut self.pattern, RecurrencePattern::Monthly, "Monthly");
                        ui.selectable_value(&mut self.pattern, RecurrencePattern::Yearly, "Yearly");
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Every");
                ui.push_id("interval_drag", |ui| {
                    ui.add(egui::DragValue::new(&mut self.interval).speed(1.0).clamp_range(1..=100));
                });
                ui.label(self.get_interval_label());
            });
            
            // Pattern-specific options
            match self.pattern {
                RecurrencePattern::Weekly => {
                    ui.label("Days of week:");
                    ui.horizontal(|ui| {
                        self.show_weekday_selector(ui);
                    });
                }
                RecurrencePattern::Monthly => {
                    ui.horizontal(|ui| {
                        ui.label("Day of month:");
                        let mut day = self.day_of_month.unwrap_or(1);
                        ui.push_id("monthly_day_drag", |ui| {
                            ui.add(egui::DragValue::new(&mut day).speed(1.0).clamp_range(1..=31));
                        });
                        self.day_of_month = Some(day);
                    });
                }
                RecurrencePattern::Yearly => {
                    ui.horizontal(|ui| {
                        ui.label("Month:");
                        let mut month = self.month_of_year.unwrap_or(1);
                        egui::ComboBox::from_id_source("yearly_month_combo")
                            .selected_text(self.month_name(month))
                            .show_ui(ui, |ui| {
                                for m in 1..=12 {
                                    ui.selectable_value(&mut month, m, self.month_name(m));
                                }
                            });
                        self.month_of_year = Some(month);
                        
                        ui.label("Day:");
                        let mut day = self.day_of_month.unwrap_or(1);
                        ui.push_id("yearly_day_drag", |ui| {
                            ui.add(egui::DragValue::new(&mut day).speed(1.0).clamp_range(1..=31));
                        });
                        self.day_of_month = Some(day);
                    });
                }
                _ => {}
            }
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Time of day:");
                let hour = self.time.hour();
                let minute = self.time.minute();
                
                let mut hour_val = hour;
                let mut minute_val = minute;
                
                ui.push_id("time_hour", |ui| {
                    ui.add(egui::DragValue::new(&mut hour_val).speed(1.0).clamp_range(0..=23));
                });
                ui.label(":");
                ui.push_id("time_minute", |ui| {
                    ui.add(egui::DragValue::new(&mut minute_val).speed(1.0).clamp_range(0..=59));
                });
                
                self.time = NaiveTime::from_hms_opt(hour_val, minute_val, 0).unwrap();
            });
            
            ui.horizontal(|ui| {
                ui.label("Priority:");
                egui::ComboBox::from_id_source("recurring_priority_combo")
                    .selected_text(format!("{:?}", self.priority))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.priority, Priority::Critical, "Critical");
                        ui.selectable_value(&mut self.priority, Priority::High, "High");
                        ui.selectable_value(&mut self.priority, Priority::Medium, "Medium");
                        ui.selectable_value(&mut self.priority, Priority::Low, "Low");
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Estimated hours:");
                if let Some(mut hours) = self.estimated_hours {
                    ui.push_id("estimated_hours_drag", |ui| {
                        ui.add(egui::DragValue::new(&mut hours).speed(0.1).clamp_range(0.0..=100.0));
                    });
                    self.estimated_hours = Some(hours);
                } else {
                    if ui.button("Set estimate").clicked() {
                        self.estimated_hours = Some(1.0);
                    }
                }
            });
            
            ui.separator();
            
            ui.label("Limits (optional):");
            
            ui.horizontal(|ui| {
                if let Some(mut max) = self.max_occurrences {
                    ui.checkbox(&mut false, "Max occurrences:");
                    ui.push_id("max_occurrences_drag", |ui| {
                        ui.add(egui::DragValue::new(&mut max).speed(1.0).clamp_range(1..=1000));
                    });
                    self.max_occurrences = Some(max);
                    if ui.button("Remove").clicked() {
                        self.max_occurrences = None;
                    }
                } else {
                    if ui.button("Add max occurrences").clicked() {
                        self.max_occurrences = Some(10);
                    }
                }
            });
            
            ui.separator();
            
            ui.label(format!("Preview: {}", self.get_frequency_description()));
            
            ui.separator();
            
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() && self.validate() {
                    should_save = true;
                }
                
                if ui.button("Cancel").clicked() {
                    self.reset();
                }
            });
        });
        
        should_save
    }

    fn show_weekday_selector(&mut self, ui: &mut Ui) {
        let weekdays = [
            (Weekday::Mon, "Mon"),
            (Weekday::Tue, "Tue"),
            (Weekday::Wed, "Wed"),
            (Weekday::Thu, "Thu"),
            (Weekday::Fri, "Fri"),
            (Weekday::Sat, "Sat"),
            (Weekday::Sun, "Sun"),
        ];
        
        for (day, label) in weekdays {
            let mut selected = self.selected_days.contains(&day);
            if ui.checkbox(&mut selected, label).changed() {
                if selected {
                    if !self.selected_days.contains(&day) {
                        self.selected_days.push(day);
                    }
                } else {
                    self.selected_days.retain(|&d| d != day);
                }
            }
        }
    }

    pub fn validate(&self) -> bool {
        if self.title.is_empty() {
            return false;
        }
        
        match self.pattern {
            RecurrencePattern::Weekly => !self.selected_days.is_empty(),
            RecurrencePattern::Monthly => {
                self.day_of_month.map_or(false, |d| d >= 1 && d <= 31)
            }
            RecurrencePattern::Yearly => {
                self.month_of_year.map_or(false, |m| m >= 1 && m <= 12) &&
                self.day_of_month.map_or(false, |d| d >= 1 && d <= 31)
            }
            _ => true,
        }
    }

    pub fn build_template(&self) -> RecurringTaskTemplate {
        let rule = RecurrenceRule {
            pattern: self.pattern,
            interval: self.interval,
            days_of_week: self.selected_days.clone(),
            day_of_month: self.day_of_month,
            month_of_year: self.month_of_year,
            time_of_day: self.time,
            end_date: self.end_date,
            max_occurrences: self.max_occurrences,
            occurrences_count: 0,
        };
        
        let mut template = RecurringTaskTemplate::new(
            self.title.clone(),
            self.description.clone(),
            rule,
        );
        
        template.priority = self.priority;
        template.estimated_hours = self.estimated_hours;
        
        template
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn pattern_to_string(&self, pattern: RecurrencePattern) -> &str {
        match pattern {
            RecurrencePattern::Daily => "Daily",
            RecurrencePattern::Weekly => "Weekly",
            RecurrencePattern::Monthly => "Monthly",
            RecurrencePattern::Yearly => "Yearly",
            RecurrencePattern::Custom => "Custom",
        }
    }

    fn get_interval_label(&self) -> &str {
        match self.pattern {
            RecurrencePattern::Daily => if self.interval == 1 { "day" } else { "days" },
            RecurrencePattern::Weekly => if self.interval == 1 { "week" } else { "weeks" },
            RecurrencePattern::Monthly => if self.interval == 1 { "month" } else { "months" },
            RecurrencePattern::Yearly => if self.interval == 1 { "year" } else { "years" },
            RecurrencePattern::Custom => "interval",
        }
    }

    pub fn get_frequency_description(&self) -> String {
        match self.pattern {
            RecurrencePattern::Daily => {
                if self.interval == 1 {
                    "Every day".to_string()
                } else {
                    format!("Every {} days", self.interval)
                }
            }
            RecurrencePattern::Weekly => {
                let days_str = self.selected_days.iter()
                    .map(|d| format!("{:?}", d).chars().take(3).collect::<String>())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                if self.interval == 1 {
                    format!("Every week on {}", days_str)
                } else {
                    format!("Every {} weeks on {}", self.interval, days_str)
                }
            }
            RecurrencePattern::Monthly => {
                let day = self.day_of_month.unwrap_or(1);
                let suffix = match day {
                    1 | 21 | 31 => "st",
                    2 | 22 => "nd",
                    3 | 23 => "rd",
                    _ => "th",
                };
                
                if self.interval == 1 {
                    format!("Every month on the {}{}", day, suffix)
                } else {
                    format!("Every {} months on the {}{}", self.interval, day, suffix)
                }
            }
            RecurrencePattern::Yearly => {
                let month = self.month_of_year.unwrap_or(1);
                let day = self.day_of_month.unwrap_or(1);
                
                if self.interval == 1 {
                    format!("Every year on {} {}", self.month_name(month), day)
                } else {
                    format!("Every {} years on {} {}", self.interval, self.month_name(month), day)
                }
            }
            RecurrencePattern::Custom => "Custom schedule".to_string(),
        }
    }

    fn month_name(&self, month: u32) -> &str {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Invalid",
        }
    }
}