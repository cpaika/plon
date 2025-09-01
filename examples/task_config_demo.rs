use plon::domain::task_config::{
    FieldOption, FieldType, MetadataFieldConfig, StateDefinition, StateTransition,
    TaskConfiguration, create_software_development_config,
};

fn main() {
    println!("Task Configuration Demo");
    println!("======================\n");

    // Create a basic configuration
    let mut config = TaskConfiguration::new("Project Management".to_string());
    config.description = "Configuration for project management tasks".to_string();

    // Add a metadata field for priority
    let priority_field = MetadataFieldConfig {
        name: "priority".to_string(),
        display_name: "Priority".to_string(),
        field_type: FieldType::Select,
        required: true,
        options: vec![
            FieldOption {
                value: "low".to_string(),
                label: "Low".to_string(),
                color: Some("#22c55e".to_string()),
                icon: None,
            },
            FieldOption {
                value: "medium".to_string(),
                label: "Medium".to_string(),
                color: Some("#eab308".to_string()),
                icon: None,
            },
            FieldOption {
                value: "high".to_string(),
                label: "High".to_string(),
                color: Some("#ef4444".to_string()),
                icon: None,
            },
        ],
        default_value: Some("medium".to_string()),
        validation_rules: vec![],
        help_text: "Select the priority level for this task".to_string(),
        show_in_list: true,
        show_in_card: true,
        sortable: true,
        searchable: false,
    };

    config.add_metadata_field(priority_field);

    // Add a custom state
    let review_state = StateDefinition {
        name: "in_review".to_string(),
        display_name: "In Review".to_string(),
        color: "#a855f7".to_string(),
        description: "Task is being reviewed".to_string(),
        is_final: false,
        auto_actions: vec![],
    };

    config.add_state(review_state);

    // Add a transition
    let transition = StateTransition {
        from_state: "in_progress".to_string(),
        to_state: "in_review".to_string(),
        action_name: "Submit for Review".to_string(),
        conditions: vec![],
        effects: vec![],
    };

    config.add_transition(transition);

    println!("Created configuration: {}", config.name);
    println!("Description: {}", config.description);
    println!("\nMetadata Fields:");
    for (name, field) in &config.metadata_schema.fields {
        println!(
            "  - {} ({}): {:?}",
            field.display_name, name, field.field_type
        );
    }

    println!("\nStates:");
    for (name, state) in &config.state_machine.states {
        println!(
            "  - {} ({}): {}",
            state.display_name, name, state.description
        );
    }

    println!("\nTransitions:");
    for transition in &config.state_machine.transitions {
        println!(
            "  - {} -> {} [{}]",
            transition.from_state, transition.to_state, transition.action_name
        );
    }

    // Test the software development preset
    println!("\n\nSoftware Development Preset");
    println!("============================\n");

    let dev_config = create_software_development_config();

    println!("Created configuration: {}", dev_config.name);
    println!("Description: {}", dev_config.description);

    println!("\nMetadata Fields:");
    for (name, field) in &dev_config.metadata_schema.fields {
        println!(
            "  - {} ({}): {:?}",
            field.display_name, name, field.field_type
        );
        if !field.options.is_empty() {
            println!("    Options:");
            for option in &field.options {
                println!("      * {} - {}", option.value, option.label);
            }
        }
    }

    println!("\nStates:");
    for (name, state) in &dev_config.state_machine.states {
        println!(
            "  - {} ({}): {}",
            state.display_name, name, state.description
        );
        if state.is_final {
            println!("    [FINAL STATE]");
        }
    }

    println!("\nTransitions:");
    for transition in &dev_config.state_machine.transitions {
        println!(
            "  - {} -> {} [{}]",
            transition.from_state, transition.to_state, transition.action_name
        );
        if !transition.conditions.is_empty() {
            println!(
                "    Conditions: {} condition(s)",
                transition.conditions.len()
            );
        }
        if !transition.effects.is_empty() {
            println!("    Effects: {} effect(s)", transition.effects.len());
        }
    }

    // Test metadata validation
    println!("\n\nMetadata Validation Test");
    println!("========================\n");

    let mut test_metadata = std::collections::HashMap::new();
    test_metadata.insert("story_points".to_string(), "3".to_string());
    test_metadata.insert("sprint".to_string(), "Sprint 24".to_string());

    match dev_config.validate_metadata(&test_metadata) {
        Ok(_) => println!("✓ Metadata validation passed"),
        Err(errors) => {
            println!("✗ Metadata validation failed:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Test state transitions
    println!("\nState Transition Test");
    println!("=====================\n");

    let available = dev_config.get_available_transitions("in_progress");
    println!("Available transitions from 'in_progress':");
    for transition in available {
        println!("  - {} ({})", transition.to_state, transition.action_name);
    }

    println!("\n✅ Task Configuration System Demo Complete!");
}
