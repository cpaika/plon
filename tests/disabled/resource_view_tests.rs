use plon::domain::resource::*;
use plon::ui::views::resource_view::*;
use uuid::Uuid;

fn create_test_resource(name: &str, role: &str, hours: f32) -> Resource {
    Resource::new(name.to_string(), role.to_string(), hours)
}

#[cfg(test)]
mod resource_view_tests {
    use super::*;

    #[test]
    fn test_resource_view_initialization() {
        let view = ResourceView::new();
        assert!(view.selected_resource.is_none());
        assert_eq!(view.new_resource_name, "");
        assert_eq!(view.new_resource_role, "");
        assert_eq!(view.new_resource_hours, 40.0);
        assert!(!view.show_create_dialog);
        assert!(!view.show_edit_dialog);
        assert_eq!(view.edit_resource_name, "");
        assert_eq!(view.edit_resource_role, "");
        assert_eq!(view.edit_resource_hours, 40.0);
        assert_eq!(view.edit_resource_email, "");
        assert!(view.edit_resource_skills.is_empty());
    }

    #[test]
    fn test_resource_creation_in_view() {
        let mut view = ResourceView::new();
        let mut resources = Vec::new();

        // Simulate creating a new resource
        view.new_resource_name = "Alice".to_string();
        view.new_resource_role = "Developer".to_string();
        view.new_resource_hours = 35.0;

        // Would normally happen through UI interaction
        if !view.new_resource_name.trim().is_empty() {
            let resource = Resource::new(
                view.new_resource_name.clone(),
                view.new_resource_role.clone(),
                view.new_resource_hours,
            );
            resources.push(resource);
        }

        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].name, "Alice");
        assert_eq!(resources[0].role, "Developer");
        assert_eq!(resources[0].weekly_hours, 35.0);
    }

    #[test]
    fn test_resource_filtering() {
        let mut view = ResourceView::new();
        let mut resources = vec![
            create_test_resource("Alice", "Developer", 40.0),
            create_test_resource("Bob", "Designer", 35.0),
            create_test_resource("Charlie", "Developer", 40.0),
            create_test_resource("David", "Manager", 45.0),
        ];

        // Test name filtering
        view.filter_text = "alice".to_string();
        let filtered: Vec<&Resource> = resources
            .iter()
            .filter(|r| {
                if view.filter_text.is_empty() {
                    true
                } else {
                    r.name
                        .to_lowercase()
                        .contains(&view.filter_text.to_lowercase())
                        || r.role
                            .to_lowercase()
                            .contains(&view.filter_text.to_lowercase())
                }
            })
            .collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Alice");

        // Test role filtering
        view.filter_text = "developer".to_string();
        let filtered: Vec<&Resource> = resources
            .iter()
            .filter(|r| {
                if view.filter_text.is_empty() {
                    true
                } else {
                    r.name
                        .to_lowercase()
                        .contains(&view.filter_text.to_lowercase())
                        || r.role
                            .to_lowercase()
                            .contains(&view.filter_text.to_lowercase())
                }
            })
            .collect();
        assert_eq!(filtered.len(), 2);

        // Test empty filter shows all
        view.filter_text = "".to_string();
        let filtered: Vec<&Resource> = resources
            .iter()
            .filter(|r| {
                if view.filter_text.is_empty() {
                    true
                } else {
                    r.name
                        .to_lowercase()
                        .contains(&view.filter_text.to_lowercase())
                        || r.role
                            .to_lowercase()
                            .contains(&view.filter_text.to_lowercase())
                }
            })
            .collect();
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn test_resource_selection() {
        let mut view = ResourceView::new();
        let resource_id = Uuid::new_v4();

        assert!(view.selected_resource.is_none());

        view.selected_resource = Some(resource_id);
        assert_eq!(view.selected_resource, Some(resource_id));
    }

    #[test]
    fn test_create_dialog_toggle() {
        let mut view = ResourceView::new();

        assert!(!view.show_create_dialog);

        // Simulate clicking Add Resource button
        view.show_create_dialog = true;
        view.new_resource_name.clear();
        view.new_resource_role.clear();
        view.new_resource_hours = 40.0;

        assert!(view.show_create_dialog);
        assert_eq!(view.new_resource_name, "");
        assert_eq!(view.new_resource_role, "");
        assert_eq!(view.new_resource_hours, 40.0);

        // Simulate cancelling
        view.show_create_dialog = false;
        assert!(!view.show_create_dialog);
    }

    #[test]
    fn test_resource_skill_management() {
        let mut view = ResourceView::new();
        view.new_skill = "Python".to_string();

        assert_eq!(view.new_skill, "Python");

        view.new_skill = "Rust".to_string();
        assert_eq!(view.new_skill, "Rust");
    }

    #[test]
    fn test_edit_dialog_toggle() {
        let mut view = ResourceView::new();
        let resource_id = Uuid::new_v4();

        assert!(!view.show_edit_dialog);
        assert!(view.selected_resource.is_none());

        // Simulate clicking Edit button
        view.selected_resource = Some(resource_id);
        view.show_edit_dialog = true;

        assert!(view.show_edit_dialog);
        assert_eq!(view.selected_resource, Some(resource_id));

        // Simulate cancelling
        view.show_edit_dialog = false;
        view.selected_resource = None;

        assert!(!view.show_edit_dialog);
        assert!(view.selected_resource.is_none());
    }

    #[test]
    fn test_resource_delete_functionality() {
        let mut view = ResourceView::new();
        let mut resources = vec![
            create_test_resource("Alice", "Developer", 40.0),
            create_test_resource("Bob", "Designer", 35.0),
            create_test_resource("Charlie", "Manager", 45.0),
        ];

        let alice_id = resources[0].id;
        let initial_count = resources.len();

        // Simulate selecting and deleting Alice
        view.selected_resource = Some(alice_id);
        view.show_edit_dialog = true;

        // Simulate clicking delete button
        resources.retain(|r| r.id != alice_id);
        view.show_edit_dialog = false;
        view.selected_resource = None;

        // Verify deletion
        assert_eq!(resources.len(), initial_count - 1);
        assert!(!resources.iter().any(|r| r.id == alice_id));
        assert!(!resources.iter().any(|r| r.name == "Alice"));

        // Verify the remaining resources are intact
        assert_eq!(resources[0].name, "Bob");
        assert_eq!(resources[1].name, "Charlie");
    }

    #[test]
    fn test_edit_skills_management() {
        let mut view = ResourceView::new();
        let mut resources = vec![create_test_resource("Developer", "Senior", 40.0)];

        // Add initial skills
        resources[0].skills.insert("Python".to_string());
        resources[0].skills.insert("Rust".to_string());

        // Simulate editing skills
        view.selected_resource = Some(resources[0].id);
        view.show_edit_dialog = true;
        view.edit_resource_skills = resources[0].skills.iter().cloned().collect();

        // Add a new skill
        view.new_skill = "JavaScript".to_string();
        if !view.new_skill.trim().is_empty() {
            view.edit_resource_skills.push(view.new_skill.clone());
            view.new_skill.clear();
        }

        // Remove a skill (simulate removing "Python")
        view.edit_resource_skills.retain(|s| s != "Python");

        // Apply changes
        resources[0].skills.clear();
        for skill in &view.edit_resource_skills {
            resources[0].skills.insert(skill.clone());
        }

        // Verify skill changes
        assert!(!resources[0].skills.contains("Python"));
        assert!(resources[0].skills.contains("Rust"));
        assert!(resources[0].skills.contains("JavaScript"));
        assert_eq!(resources[0].skills.len(), 2);
    }

    #[test]
    fn test_edit_email_field() {
        let mut view = ResourceView::new();
        let mut resources = vec![create_test_resource("Alice", "Developer", 40.0)];

        // Initially no email
        assert!(resources[0].email.is_none());

        // Add email through edit
        view.selected_resource = Some(resources[0].id);
        view.edit_resource_email = "alice@example.com".to_string();
        resources[0].email = Some(view.edit_resource_email.clone());

        assert_eq!(resources[0].email, Some("alice@example.com".to_string()));

        // Clear email (empty string should result in None)
        view.edit_resource_email = "".to_string();
        resources[0].email = if view.edit_resource_email.trim().is_empty() {
            None
        } else {
            Some(view.edit_resource_email.clone())
        };

        assert!(resources[0].email.is_none());
    }

    #[test]
    fn test_edit_hours_validation() {
        let mut view = ResourceView::new();
        let mut resources = vec![create_test_resource("Worker", "Full-time", 40.0)];

        // Test various hour values
        view.selected_resource = Some(resources[0].id);

        // Normal update
        view.edit_resource_hours = 35.0;
        resources[0].weekly_hours = view.edit_resource_hours;
        assert_eq!(resources[0].weekly_hours, 35.0);

        // Minimum hours (1.0)
        view.edit_resource_hours = 1.0;
        resources[0].weekly_hours = view.edit_resource_hours;
        assert_eq!(resources[0].weekly_hours, 1.0);

        // Maximum hours (60.0)
        view.edit_resource_hours = 60.0;
        resources[0].weekly_hours = view.edit_resource_hours;
        assert_eq!(resources[0].weekly_hours, 60.0);
    }

    #[test]
    fn test_edit_cancel_reverts_changes() {
        let mut view = ResourceView::new();
        let mut resources = vec![create_test_resource("Original", "Developer", 40.0)];
        let original_resource = resources[0].clone();

        // Start editing
        view.selected_resource = Some(resources[0].id);
        view.show_edit_dialog = true;
        view.edit_resource_name = "Modified".to_string();
        view.edit_resource_role = "Manager".to_string();
        view.edit_resource_hours = 50.0;

        // Cancel without saving
        view.show_edit_dialog = false;
        view.selected_resource = None;

        // Resource should remain unchanged
        assert_eq!(resources[0].name, original_resource.name);
        assert_eq!(resources[0].role, original_resource.role);
        assert_eq!(resources[0].weekly_hours, original_resource.weekly_hours);
    }

    #[test]
    fn test_edit_resource_fields() {
        let mut view = ResourceView::new();
        let mut resources = vec![create_test_resource("Alice", "Developer", 40.0)];

        // Add some initial data to the resource
        resources[0].email = Some("alice@example.com".to_string());
        resources[0].skills.insert("Rust".to_string());
        resources[0].skills.insert("Python".to_string());

        // Simulate populating edit fields
        view.selected_resource = Some(resources[0].id);
        view.show_edit_dialog = true;
        view.edit_resource_name = resources[0].name.clone();
        view.edit_resource_role = resources[0].role.clone();
        view.edit_resource_hours = resources[0].weekly_hours;
        view.edit_resource_email = resources[0].email.clone().unwrap_or_default();
        view.edit_resource_skills = resources[0].skills.iter().cloned().collect();

        assert_eq!(view.edit_resource_name, "Alice");
        assert_eq!(view.edit_resource_role, "Developer");
        assert_eq!(view.edit_resource_hours, 40.0);
        assert_eq!(view.edit_resource_email, "alice@example.com");
        assert_eq!(view.edit_resource_skills.len(), 2);

        // Simulate editing the fields
        view.edit_resource_name = "Alice Smith".to_string();
        view.edit_resource_role = "Senior Developer".to_string();
        view.edit_resource_hours = 45.0;
        view.edit_resource_email = "alice.smith@example.com".to_string();
        view.edit_resource_skills.push("JavaScript".to_string());

        // Apply changes (would normally happen through UI interaction)
        if let Some(resource) = resources
            .iter_mut()
            .find(|r| r.id == view.selected_resource.unwrap())
        {
            resource.name = view.edit_resource_name.clone();
            resource.role = view.edit_resource_role.clone();
            resource.weekly_hours = view.edit_resource_hours;
            resource.email = Some(view.edit_resource_email.clone());
            resource.skills.clear();
            for skill in &view.edit_resource_skills {
                resource.skills.insert(skill.clone());
            }
        }

        // Verify the changes were applied
        assert_eq!(resources[0].name, "Alice Smith");
        assert_eq!(resources[0].role, "Senior Developer");
        assert_eq!(resources[0].weekly_hours, 45.0);
        assert_eq!(
            resources[0].email,
            Some("alice.smith@example.com".to_string())
        );
        assert_eq!(resources[0].skills.len(), 3);
        assert!(resources[0].skills.contains("JavaScript"));
    }
}

#[cfg(test)]
mod resource_view_integration_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_complete_edit_workflow() {
        let mut view = ResourceView::new();
        let mut resources = vec![create_test_resource("John Doe", "Junior Developer", 40.0)];

        // Set initial state
        resources[0].email = Some("john@old.com".to_string());
        resources[0].skills.insert("Java".to_string());

        let resource_id = resources[0].id;

        // Step 1: Click edit button (simulated)
        view.selected_resource = Some(resource_id);
        view.show_edit_dialog = true;

        // Step 2: Populate edit fields with current data
        view.edit_resource_name = resources[0].name.clone();
        view.edit_resource_role = resources[0].role.clone();
        view.edit_resource_hours = resources[0].weekly_hours;
        view.edit_resource_email = resources[0].email.clone().unwrap_or_default();
        view.edit_resource_skills = resources[0].skills.iter().cloned().collect();

        // Step 3: Make comprehensive changes
        view.edit_resource_name = "John Smith".to_string();
        view.edit_resource_role = "Senior Developer".to_string();
        view.edit_resource_hours = 45.0;
        view.edit_resource_email = "john.smith@new.com".to_string();

        // Add new skills
        view.edit_resource_skills.push("Rust".to_string());
        view.edit_resource_skills.push("Python".to_string());

        // Remove old skill
        view.edit_resource_skills.retain(|s| s != "Java");

        // Step 4: Save changes (simulated)
        if let Some(resource) = resources.iter_mut().find(|r| r.id == resource_id) {
            resource.name = view.edit_resource_name.clone();
            resource.role = view.edit_resource_role.clone();
            resource.weekly_hours = view.edit_resource_hours;
            resource.email = if view.edit_resource_email.trim().is_empty() {
                None
            } else {
                Some(view.edit_resource_email.clone())
            };
            resource.skills.clear();
            for skill in &view.edit_resource_skills {
                resource.skills.insert(skill.clone());
            }
        }

        view.show_edit_dialog = false;
        view.selected_resource = None;

        // Step 5: Verify all changes were applied
        assert_eq!(resources[0].name, "John Smith");
        assert_eq!(resources[0].role, "Senior Developer");
        assert_eq!(resources[0].weekly_hours, 45.0);
        assert_eq!(resources[0].email, Some("john.smith@new.com".to_string()));
        assert!(!resources[0].skills.contains("Java"));
        assert!(resources[0].skills.contains("Rust"));
        assert!(resources[0].skills.contains("Python"));
        assert_eq!(resources[0].skills.len(), 2);

        // Step 6: Verify view state is reset
        assert!(view.selected_resource.is_none());
        assert!(!view.show_edit_dialog);
    }

    #[test]
    fn test_resource_view_with_multiple_resources() {
        let mut view = ResourceView::new();
        let mut resources = vec![
            create_test_resource("Alice", "Senior Developer", 40.0),
            create_test_resource("Bob", "Junior Developer", 40.0),
            create_test_resource("Charlie", "Designer", 35.0),
            create_test_resource("David", "Project Manager", 45.0),
            create_test_resource("Eve", "QA Engineer", 40.0),
        ];

        // Verify all resources are present
        assert_eq!(resources.len(), 5);

        // Test filtering by role
        view.filter_text = "Developer".to_string();
        let developers: Vec<&Resource> = resources
            .iter()
            .filter(|r| {
                r.role
                    .to_lowercase()
                    .contains(&view.filter_text.to_lowercase())
            })
            .collect();
        assert_eq!(developers.len(), 2);

        // Test selection
        view.selected_resource = Some(resources[0].id);
        assert!(view.selected_resource.is_some());
    }

    #[test]
    fn test_resource_capacity_tracking() {
        let resources = vec![
            create_test_resource("Full Time", "Developer", 40.0),
            create_test_resource("Part Time", "Developer", 20.0),
            create_test_resource("Contractor", "Developer", 50.0),
        ];

        // Verify different weekly hour capacities
        assert_eq!(resources[0].weekly_hours, 40.0);
        assert_eq!(resources[1].weekly_hours, 20.0);
        assert_eq!(resources[2].weekly_hours, 50.0);

        // Calculate total capacity
        let total_capacity: f32 = resources.iter().map(|r| r.weekly_hours).sum();
        assert_eq!(total_capacity, 110.0);
    }

    #[test]
    fn test_resource_skill_filtering() {
        let mut resources = vec![
            create_test_resource("Alice", "Rust Developer", 40.0),
            create_test_resource("Bob", "Python Developer", 40.0),
            create_test_resource("Charlie", "Full Stack Developer", 40.0),
        ];

        // Add skills to resources
        resources[0].skills.insert("Rust".to_string());
        resources[0]
            .skills
            .insert("Systems Programming".to_string());

        resources[1].skills.insert("Python".to_string());
        resources[1].skills.insert("Machine Learning".to_string());

        resources[2].skills.insert("Rust".to_string());
        resources[2].skills.insert("Python".to_string());
        resources[2].skills.insert("JavaScript".to_string());

        // Find resources with Rust skills
        let rust_devs: Vec<&Resource> = resources
            .iter()
            .filter(|r| r.skills.contains("Rust"))
            .collect();
        assert_eq!(rust_devs.len(), 2);

        // Find resources with Python skills
        let python_devs: Vec<&Resource> = resources
            .iter()
            .filter(|r| r.skills.contains("Python"))
            .collect();
        assert_eq!(python_devs.len(), 2);
    }

    #[test]
    fn test_resource_availability_calculation() {
        let mut resource = create_test_resource("Developer", "Senior", 40.0);

        // Test full availability
        assert_eq!(resource.weekly_hours, 40.0);

        // Simulate allocating hours to tasks
        let allocated_hours = 25.0;
        let available_hours = resource.weekly_hours - allocated_hours;
        assert_eq!(available_hours, 15.0);

        // Test overallocation scenario
        let overallocated_hours = 50.0;
        let is_overallocated = overallocated_hours > resource.weekly_hours;
        assert!(is_overallocated);
    }
}
