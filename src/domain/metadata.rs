use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetadataField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub options: Vec<String>, // For Select and MultiSelect types
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    Text,
    Number,
    Date,
    Select,
    MultiSelect,
    Boolean,
    Url,
    Email,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSchema {
    fields: HashMap<String, MetadataField>,
}

impl MetadataSchema {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, field: MetadataField) {
        self.fields.insert(field.name.clone(), field);
    }

    pub fn remove_field(&mut self, name: &str) -> Option<MetadataField> {
        self.fields.remove(name)
    }

    pub fn validate(&self, metadata: &HashMap<String, String>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check required fields
        for (name, field) in &self.fields {
            if field.required && !metadata.contains_key(name) {
                errors.push(format!("Required field '{}' is missing", name));
            }
        }

        // Validate field values
        for (key, value) in metadata {
            if let Some(field) = self.fields.get(key) {
                if let Err(e) = self.validate_field_value(field, value) {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_field_value(&self, field: &MetadataField, value: &str) -> Result<(), String> {
        match field.field_type {
            FieldType::Number => {
                value.parse::<f64>()
                    .map_err(|_| format!("Field '{}' must be a valid number", field.name))?;
            }
            FieldType::Boolean => {
                value.parse::<bool>()
                    .map_err(|_| format!("Field '{}' must be true or false", field.name))?;
            }
            FieldType::Select => {
                if !field.options.contains(&value.to_string()) {
                    return Err(format!(
                        "Field '{}' must be one of: {:?}",
                        field.name, field.options
                    ));
                }
            }
            FieldType::MultiSelect => {
                let values: HashSet<String> = value.split(',').map(|s| s.trim().to_string()).collect();
                let options: HashSet<String> = field.options.iter().cloned().collect();
                
                if !values.is_subset(&options) {
                    return Err(format!(
                        "Field '{}' values must be from: {:?}",
                        field.name, field.options
                    ));
                }
            }
            FieldType::Email => {
                if !value.contains('@') || !value.contains('.') {
                    return Err(format!("Field '{}' must be a valid email", field.name));
                }
            }
            FieldType::Url => {
                if !value.starts_with("http://") && !value.starts_with("https://") {
                    return Err(format!("Field '{}' must be a valid URL", field.name));
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn get_field(&self, name: &str) -> Option<&MetadataField> {
        self.fields.get(name)
    }

    pub fn all_fields(&self) -> Vec<&MetadataField> {
        self.fields.values().collect()
    }
}

// Common metadata presets
impl MetadataSchema {
    pub fn software_development_preset() -> Self {
        let mut schema = Self::new();
        
        schema.add_field(MetadataField {
            name: "category".to_string(),
            field_type: FieldType::Select,
            required: false,
            options: vec![
                "frontend".to_string(),
                "backend".to_string(),
                "infrastructure".to_string(),
                "database".to_string(),
                "testing".to_string(),
                "documentation".to_string(),
            ],
            default_value: None,
        });

        schema.add_field(MetadataField {
            name: "team".to_string(),
            field_type: FieldType::Select,
            required: false,
            options: vec![
                "engineering".to_string(),
                "design".to_string(),
                "product".to_string(),
                "qa".to_string(),
            ],
            default_value: None,
        });

        schema.add_field(MetadataField {
            name: "sprint".to_string(),
            field_type: FieldType::Text,
            required: false,
            options: vec![],
            default_value: None,
        });

        schema.add_field(MetadataField {
            name: "story_points".to_string(),
            field_type: FieldType::Number,
            required: false,
            options: vec![],
            default_value: None,
        });

        schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_schema() {
        let mut schema = MetadataSchema::new();
        
        schema.add_field(MetadataField {
            name: "priority".to_string(),
            field_type: FieldType::Select,
            required: true,
            options: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
            default_value: Some("medium".to_string()),
        });

        schema.add_field(MetadataField {
            name: "estimate".to_string(),
            field_type: FieldType::Number,
            required: false,
            options: vec![],
            default_value: None,
        });

        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), "high".to_string());
        metadata.insert("estimate".to_string(), "5".to_string());

        assert!(schema.validate(&metadata).is_ok());
    }

    #[test]
    fn test_validation_required_field() {
        let mut schema = MetadataSchema::new();
        
        schema.add_field(MetadataField {
            name: "required_field".to_string(),
            field_type: FieldType::Text,
            required: true,
            options: vec![],
            default_value: None,
        });

        let metadata = HashMap::new();
        let result = schema.validate(&metadata);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("required_field"));
    }

    #[test]
    fn test_validation_select_field() {
        let mut schema = MetadataSchema::new();
        
        schema.add_field(MetadataField {
            name: "status".to_string(),
            field_type: FieldType::Select,
            required: false,
            options: vec!["open".to_string(), "closed".to_string()],
            default_value: None,
        });

        let mut metadata = HashMap::new();
        metadata.insert("status".to_string(), "invalid".to_string());
        
        let result = schema.validate(&metadata);
        assert!(result.is_err());

        metadata.insert("status".to_string(), "open".to_string());
        assert!(schema.validate(&metadata).is_ok());
    }

    #[test]
    fn test_validation_number_field() {
        let mut schema = MetadataSchema::new();
        
        schema.add_field(MetadataField {
            name: "count".to_string(),
            field_type: FieldType::Number,
            required: false,
            options: vec![],
            default_value: None,
        });

        let mut metadata = HashMap::new();
        metadata.insert("count".to_string(), "not_a_number".to_string());
        assert!(schema.validate(&metadata).is_err());

        metadata.insert("count".to_string(), "42".to_string());
        assert!(schema.validate(&metadata).is_ok());

        metadata.insert("count".to_string(), "3.14".to_string());
        assert!(schema.validate(&metadata).is_ok());
    }

    #[test]
    fn test_validation_multi_select() {
        let mut schema = MetadataSchema::new();
        
        schema.add_field(MetadataField {
            name: "tags".to_string(),
            field_type: FieldType::MultiSelect,
            required: false,
            options: vec!["bug".to_string(), "feature".to_string(), "enhancement".to_string()],
            default_value: None,
        });

        let mut metadata = HashMap::new();
        metadata.insert("tags".to_string(), "bug,feature".to_string());
        assert!(schema.validate(&metadata).is_ok());

        metadata.insert("tags".to_string(), "bug,invalid_tag".to_string());
        assert!(schema.validate(&metadata).is_err());
    }

    #[test]
    fn test_validation_email() {
        let mut schema = MetadataSchema::new();
        
        schema.add_field(MetadataField {
            name: "email".to_string(),
            field_type: FieldType::Email,
            required: false,
            options: vec![],
            default_value: None,
        });

        let mut metadata = HashMap::new();
        metadata.insert("email".to_string(), "invalid".to_string());
        assert!(schema.validate(&metadata).is_err());

        metadata.insert("email".to_string(), "user@example.com".to_string());
        assert!(schema.validate(&metadata).is_ok());
    }

    #[test]
    fn test_validation_url() {
        let mut schema = MetadataSchema::new();
        
        schema.add_field(MetadataField {
            name: "website".to_string(),
            field_type: FieldType::Url,
            required: false,
            options: vec![],
            default_value: None,
        });

        let mut metadata = HashMap::new();
        metadata.insert("website".to_string(), "not_a_url".to_string());
        assert!(schema.validate(&metadata).is_err());

        metadata.insert("website".to_string(), "https://example.com".to_string());
        assert!(schema.validate(&metadata).is_ok());
    }

    #[test]
    fn test_preset_schema() {
        let schema = MetadataSchema::software_development_preset();
        
        assert!(schema.get_field("category").is_some());
        assert!(schema.get_field("team").is_some());
        assert!(schema.get_field("sprint").is_some());
        assert!(schema.get_field("story_points").is_some());
        
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "frontend".to_string());
        metadata.insert("team".to_string(), "engineering".to_string());
        metadata.insert("story_points".to_string(), "5".to_string());
        
        assert!(schema.validate(&metadata).is_ok());
    }
}