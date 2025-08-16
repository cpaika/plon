use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfiguration {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub metadata_schema: MetadataSchema,
    pub state_machine: StateMachine,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachine {
    pub states: HashMap<String, StateDefinition>,
    pub initial_state: String,
    pub transitions: Vec<StateTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDefinition {
    pub name: String,
    pub display_name: String,
    pub color: String,
    pub description: String,
    pub is_final: bool,
    pub auto_actions: Vec<AutoAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: String,
    pub to_state: String,
    pub action_name: String,
    pub conditions: Vec<TransitionCondition>,
    pub effects: Vec<TransitionEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCondition {
    RequireMetadataField { field: String, value: Option<String> },
    RequireApproval { role: String },
    RequireAllSubtasksComplete,
    CustomValidation { script: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionEffect {
    SetMetadataField { field: String, value: String },
    NotifyResource { message_template: String },
    TriggerWebhook { url: String },
    CreateSubtask { template: SubtaskTemplate },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskTemplate {
    pub title: String,
    pub description: String,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoAction {
    SetDueDate { days_from_now: i32 },
    AssignToResource { resource_role: String },
    AddTag { tag: String },
    SendNotification { template: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetadataFieldConfig {
    pub name: String,
    pub display_name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub options: Vec<FieldOption>,
    pub default_value: Option<String>,
    pub validation_rules: Vec<ValidationRule>,
    pub help_text: String,
    pub show_in_list: bool,
    pub show_in_card: bool,
    pub sortable: bool,
    pub searchable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldOption {
    pub value: String,
    pub label: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    Text,
    LongText,
    Number,
    Decimal,
    Date,
    DateTime,
    Select,
    MultiSelect,
    Boolean,
    Url,
    Email,
    Phone,
    Currency,
    Percentage,
    Duration,
    User,
    Attachment,
    Formula,
    Relation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationRule {
    MinLength(usize),
    MaxLength(usize),
    MinValue(f64),
    MaxValue(f64),
    Regex(String),
    DateRange { min: Option<String>, max: Option<String> },
    UniqueValue,
    CustomValidation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct MetadataSchema {
    pub fields: HashMap<String, MetadataFieldConfig>,
    pub field_groups: Vec<FieldGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldGroup {
    pub name: String,
    pub display_name: String,
    pub fields: Vec<String>,
    pub collapsed_by_default: bool,
}

impl TaskConfiguration {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: String::new(),
            metadata_schema: MetadataSchema::default(),
            state_machine: StateMachine::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_metadata_field(&mut self, field: MetadataFieldConfig) {
        self.metadata_schema.fields.insert(field.name.clone(), field);
        self.updated_at = chrono::Utc::now();
    }

    pub fn add_state(&mut self, state: StateDefinition) {
        self.state_machine.states.insert(state.name.clone(), state);
        self.updated_at = chrono::Utc::now();
    }

    pub fn add_transition(&mut self, transition: StateTransition) {
        self.state_machine.transitions.push(transition);
        self.updated_at = chrono::Utc::now();
    }

    pub fn validate_metadata(&self, metadata: &HashMap<String, String>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for (name, field) in &self.metadata_schema.fields {
            if field.required && !metadata.contains_key(name) {
                errors.push(format!("Required field '{}' is missing", field.display_name));
            }

            if let Some(value) = metadata.get(name) {
                for rule in &field.validation_rules {
                    if let Err(e) = self.validate_field_rule(field, value, rule) {
                        errors.push(e);
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_field_rule(&self, field: &MetadataFieldConfig, value: &str, rule: &ValidationRule) -> Result<(), String> {
        match rule {
            ValidationRule::MinLength(min) => {
                if value.len() < *min {
                    return Err(format!("{} must be at least {} characters", field.display_name, min));
                }
            }
            ValidationRule::MaxLength(max) => {
                if value.len() > *max {
                    return Err(format!("{} must be at most {} characters", field.display_name, max));
                }
            }
            ValidationRule::MinValue(min) => {
                if let Ok(num) = value.parse::<f64>() {
                    if num < *min {
                        return Err(format!("{} must be at least {}", field.display_name, min));
                    }
                } else {
                    return Err(format!("{} must be a valid number", field.display_name));
                }
            }
            ValidationRule::MaxValue(max) => {
                if let Ok(num) = value.parse::<f64>() {
                    if num > *max {
                        return Err(format!("{} must be at most {}", field.display_name, max));
                    }
                } else {
                    return Err(format!("{} must be a valid number", field.display_name));
                }
            }
            ValidationRule::Regex(pattern) => {
                if let Ok(re) = regex::Regex::new(pattern)
                    && !re.is_match(value) {
                        return Err(format!("{} has invalid format", field.display_name));
                    }
            }
            ValidationRule::UniqueValue => {
                // This would need to be checked against the database
            }
            _ => {}
        }
        Ok(())
    }

    pub fn get_available_transitions(&self, current_state: &str) -> Vec<&StateTransition> {
        self.state_machine.transitions
            .iter()
            .filter(|t| t.from_state == current_state)
            .collect()
    }

    pub fn can_transition(&self, from_state: &str, to_state: &str, context: &TransitionContext) -> Result<(), String> {
        let transition = self.state_machine.transitions
            .iter()
            .find(|t| t.from_state == from_state && t.to_state == to_state)
            .ok_or_else(|| format!("No transition from {} to {}", from_state, to_state))?;

        for condition in &transition.conditions {
            self.check_condition(condition, context)?;
        }

        Ok(())
    }

    fn check_condition(&self, condition: &TransitionCondition, context: &TransitionContext) -> Result<(), String> {
        match condition {
            TransitionCondition::RequireMetadataField { field, value } => {
                let field_value = context.metadata.get(field)
                    .ok_or_else(|| format!("Required field {} is missing", field))?;
                
                if let Some(expected) = value
                    && field_value != expected {
                        return Err(format!("Field {} must be {}", field, expected));
                    }
            }
            TransitionCondition::RequireAllSubtasksComplete => {
                if !context.all_subtasks_complete {
                    return Err("All subtasks must be completed".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TransitionContext {
    pub metadata: HashMap<String, String>,
    pub all_subtasks_complete: bool,
    pub user_role: Option<String>,
}


impl Default for StateMachine {
    fn default() -> Self {
        let mut states = HashMap::new();
        
        states.insert("todo".to_string(), StateDefinition {
            name: "todo".to_string(),
            display_name: "To Do".to_string(),
            color: "#808080".to_string(),
            description: "Task is waiting to be started".to_string(),
            is_final: false,
            auto_actions: vec![],
        });
        
        states.insert("in_progress".to_string(), StateDefinition {
            name: "in_progress".to_string(),
            display_name: "In Progress".to_string(),
            color: "#3b82f6".to_string(),
            description: "Task is being worked on".to_string(),
            is_final: false,
            auto_actions: vec![],
        });
        
        states.insert("done".to_string(), StateDefinition {
            name: "done".to_string(),
            display_name: "Done".to_string(),
            color: "#10b981".to_string(),
            description: "Task is completed".to_string(),
            is_final: true,
            auto_actions: vec![],
        });

        let transitions = vec![
            StateTransition {
                from_state: "todo".to_string(),
                to_state: "in_progress".to_string(),
                action_name: "Start".to_string(),
                conditions: vec![],
                effects: vec![],
            },
            StateTransition {
                from_state: "in_progress".to_string(),
                to_state: "done".to_string(),
                action_name: "Complete".to_string(),
                conditions: vec![],
                effects: vec![],
            },
            StateTransition {
                from_state: "in_progress".to_string(),
                to_state: "todo".to_string(),
                action_name: "Stop".to_string(),
                conditions: vec![],
                effects: vec![],
            },
        ];

        Self {
            states,
            initial_state: "todo".to_string(),
            transitions,
        }
    }
}

pub fn create_software_development_config() -> TaskConfiguration {
    let mut config = TaskConfiguration::new("Software Development".to_string());
    config.description = "Configuration for software development tasks".to_string();

    config.add_metadata_field(MetadataFieldConfig {
        name: "story_points".to_string(),
        display_name: "Story Points".to_string(),
        field_type: FieldType::Select,
        required: false,
        options: vec![
            FieldOption { value: "1".to_string(), label: "1 - Trivial".to_string(), color: Some("#10b981".to_string()), icon: None },
            FieldOption { value: "2".to_string(), label: "2 - Easy".to_string(), color: Some("#3b82f6".to_string()), icon: None },
            FieldOption { value: "3".to_string(), label: "3 - Medium".to_string(), color: Some("#f59e0b".to_string()), icon: None },
            FieldOption { value: "5".to_string(), label: "5 - Hard".to_string(), color: Some("#ef4444".to_string()), icon: None },
            FieldOption { value: "8".to_string(), label: "8 - Complex".to_string(), color: Some("#dc2626".to_string()), icon: None },
        ],
        default_value: Some("3".to_string()),
        validation_rules: vec![],
        help_text: "Estimate the complexity of this task".to_string(),
        show_in_list: true,
        show_in_card: true,
        sortable: true,
        searchable: false,
    });

    config.add_metadata_field(MetadataFieldConfig {
        name: "sprint".to_string(),
        display_name: "Sprint".to_string(),
        field_type: FieldType::Text,
        required: false,
        options: vec![],
        default_value: None,
        validation_rules: vec![ValidationRule::MaxLength(50)],
        help_text: "Sprint or iteration this task belongs to".to_string(),
        show_in_list: true,
        show_in_card: true,
        sortable: true,
        searchable: true,
    });

    config.add_metadata_field(MetadataFieldConfig {
        name: "pr_url".to_string(),
        display_name: "Pull Request URL".to_string(),
        field_type: FieldType::Url,
        required: false,
        options: vec![],
        default_value: None,
        validation_rules: vec![],
        help_text: "Link to the pull request".to_string(),
        show_in_list: false,
        show_in_card: true,
        sortable: false,
        searchable: false,
    });

    let review_state = StateDefinition {
        name: "review".to_string(),
        display_name: "In Review".to_string(),
        color: "#8b5cf6".to_string(),
        description: "Task is being reviewed".to_string(),
        is_final: false,
        auto_actions: vec![
            AutoAction::AddTag { tag: "needs-review".to_string() },
        ],
    };

    config.state_machine.states.insert("review".to_string(), review_state);

    config.state_machine.transitions.push(StateTransition {
        from_state: "in_progress".to_string(),
        to_state: "review".to_string(),
        action_name: "Submit for Review".to_string(),
        conditions: vec![
            TransitionCondition::RequireMetadataField { 
                field: "pr_url".to_string(), 
                value: None 
            },
        ],
        effects: vec![
            TransitionEffect::NotifyResource { 
                message_template: "Task '{title}' is ready for review".to_string() 
            },
        ],
    });

    config.state_machine.transitions.push(StateTransition {
        from_state: "review".to_string(),
        to_state: "done".to_string(),
        action_name: "Approve".to_string(),
        conditions: vec![],
        effects: vec![],
    });

    config.state_machine.transitions.push(StateTransition {
        from_state: "review".to_string(),
        to_state: "in_progress".to_string(),
        action_name: "Request Changes".to_string(),
        conditions: vec![],
        effects: vec![],
    });

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_configuration() {
        let mut config = TaskConfiguration::new("Test Config".to_string());
        
        let field = MetadataFieldConfig {
            name: "priority".to_string(),
            display_name: "Priority".to_string(),
            field_type: FieldType::Select,
            required: true,
            options: vec![
                FieldOption { value: "low".to_string(), label: "Low".to_string(), color: None, icon: None },
                FieldOption { value: "high".to_string(), label: "High".to_string(), color: None, icon: None },
            ],
            default_value: Some("low".to_string()),
            validation_rules: vec![],
            help_text: "Task priority".to_string(),
            show_in_list: true,
            show_in_card: true,
            sortable: true,
            searchable: false,
        };
        
        config.add_metadata_field(field);
        assert_eq!(config.metadata_schema.fields.len(), 1);
    }

    #[test]
    fn test_state_machine() {
        let config = TaskConfiguration::new("Test".to_string());
        let transitions = config.get_available_transitions("todo");
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].to_state, "in_progress");
    }

    #[test]
    fn test_metadata_validation() {
        let mut config = TaskConfiguration::new("Test".to_string());
        
        config.add_metadata_field(MetadataFieldConfig {
            name: "required_field".to_string(),
            display_name: "Required Field".to_string(),
            field_type: FieldType::Text,
            required: true,
            options: vec![],
            default_value: None,
            validation_rules: vec![ValidationRule::MinLength(5)],
            help_text: String::new(),
            show_in_list: false,
            show_in_card: false,
            sortable: false,
            searchable: false,
        });

        let mut metadata = HashMap::new();
        let result = config.validate_metadata(&metadata);
        assert!(result.is_err());

        metadata.insert("required_field".to_string(), "test".to_string());
        let result = config.validate_metadata(&metadata);
        assert!(result.is_err()); // Too short

        metadata.insert("required_field".to_string(), "test123".to_string());
        let result = config.validate_metadata(&metadata);
        assert!(result.is_ok());
    }
}