use crate::domain::task_config::{
    MetadataFieldConfig, StateDefinition, StateTransition, TaskConfiguration, TransitionContext,
};
use crate::repository::Repository;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

pub struct TaskConfigService {
    repository: Arc<Repository>,
}

impl TaskConfigService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn create_configuration(
        &self,
        config: TaskConfiguration,
    ) -> Result<TaskConfiguration> {
        self.repository.task_configs.create(&config).await?;
        Ok(config)
    }

    pub async fn update_configuration(
        &self,
        config: TaskConfiguration,
    ) -> Result<TaskConfiguration> {
        self.repository.task_configs.update(&config).await?;
        Ok(config)
    }

    pub async fn get_configuration(&self, id: Uuid) -> Result<Option<TaskConfiguration>> {
        self.repository.task_configs.get(id).await
    }

    pub async fn get_configuration_by_name(&self, name: &str) -> Result<Option<TaskConfiguration>> {
        self.repository.task_configs.get_by_name(name).await
    }

    pub async fn list_configurations(&self) -> Result<Vec<TaskConfiguration>> {
        self.repository.task_configs.list_all().await
    }

    pub async fn delete_configuration(&self, id: Uuid) -> Result<()> {
        self.repository.task_configs.delete(id).await
    }

    pub async fn add_metadata_field(
        &self,
        config_id: Uuid,
        field: MetadataFieldConfig,
    ) -> Result<()> {
        if let Some(mut config) = self.get_configuration(config_id).await? {
            config.add_metadata_field(field);
            self.update_configuration(config).await?;
        }
        Ok(())
    }

    pub async fn remove_metadata_field(&self, config_id: Uuid, field_name: &str) -> Result<()> {
        if let Some(mut config) = self.get_configuration(config_id).await? {
            config.metadata_schema.fields.remove(field_name);
            config.updated_at = chrono::Utc::now();
            self.update_configuration(config).await?;
        }
        Ok(())
    }

    pub async fn add_state(&self, config_id: Uuid, state: StateDefinition) -> Result<()> {
        if let Some(mut config) = self.get_configuration(config_id).await? {
            config.add_state(state);
            self.update_configuration(config).await?;
        }
        Ok(())
    }

    pub async fn remove_state(&self, config_id: Uuid, state_name: &str) -> Result<()> {
        if let Some(mut config) = self.get_configuration(config_id).await? {
            config.state_machine.states.remove(state_name);
            config
                .state_machine
                .transitions
                .retain(|t| t.from_state != state_name && t.to_state != state_name);
            config.updated_at = chrono::Utc::now();
            self.update_configuration(config).await?;
        }
        Ok(())
    }

    pub async fn add_transition(&self, config_id: Uuid, transition: StateTransition) -> Result<()> {
        if let Some(mut config) = self.get_configuration(config_id).await? {
            config.add_transition(transition);
            self.update_configuration(config).await?;
        }
        Ok(())
    }

    pub async fn remove_transition(
        &self,
        config_id: Uuid,
        from_state: &str,
        to_state: &str,
    ) -> Result<()> {
        if let Some(mut config) = self.get_configuration(config_id).await? {
            config
                .state_machine
                .transitions
                .retain(|t| !(t.from_state == from_state && t.to_state == to_state));
            config.updated_at = chrono::Utc::now();
            self.update_configuration(config).await?;
        }
        Ok(())
    }

    pub async fn validate_task_metadata(
        &self,
        config_id: Uuid,
        metadata: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<String>> {
        if let Some(config) = self.get_configuration(config_id).await? {
            match config.validate_metadata(metadata) {
                Ok(_) => Ok(vec![]),
                Err(errors) => Ok(errors),
            }
        } else {
            Ok(vec!["Configuration not found".to_string()])
        }
    }

    pub async fn can_transition(
        &self,
        config_id: Uuid,
        from_state: &str,
        to_state: &str,
        context: &TransitionContext,
    ) -> Result<bool> {
        if let Some(config) = self.get_configuration(config_id).await? {
            Ok(config.can_transition(from_state, to_state, context).is_ok())
        } else {
            Ok(false)
        }
    }

    pub async fn get_available_transitions(
        &self,
        config_id: Uuid,
        current_state: &str,
    ) -> Result<Vec<StateTransition>> {
        if let Some(config) = self.get_configuration(config_id).await? {
            Ok(config
                .get_available_transitions(current_state)
                .into_iter()
                .cloned()
                .collect())
        } else {
            Ok(vec![])
        }
    }

    pub async fn create_default_configurations(&self) -> Result<()> {
        use crate::domain::task_config::create_software_development_config;

        let dev_config = create_software_development_config();
        if self
            .get_configuration_by_name(&dev_config.name)
            .await?
            .is_none()
        {
            self.create_configuration(dev_config).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task_config::FieldType;
    use crate::repository::Repository;
    use sqlx::SqlitePool;

    async fn setup_test_service() -> TaskConfigService {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        repository.task_configs.create_tables().await.unwrap();
        TaskConfigService::new(repository)
    }

    #[tokio::test]
    async fn test_create_and_get_configuration() {
        let service = setup_test_service().await;

        let config = TaskConfiguration::new("Test Config".to_string());
        let created = service.create_configuration(config.clone()).await.unwrap();

        let retrieved = service.get_configuration(created.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Config");
    }

    #[tokio::test]
    async fn test_add_metadata_field() {
        let service = setup_test_service().await;

        let config = TaskConfiguration::new("Test Config".to_string());
        let created = service.create_configuration(config).await.unwrap();

        let field = MetadataFieldConfig {
            name: "test_field".to_string(),
            display_name: "Test Field".to_string(),
            field_type: FieldType::Text,
            required: false,
            options: vec![],
            default_value: None,
            validation_rules: vec![],
            help_text: String::new(),
            show_in_list: true,
            show_in_card: true,
            sortable: false,
            searchable: false,
        };

        service.add_metadata_field(created.id, field).await.unwrap();

        let updated = service
            .get_configuration(created.id)
            .await
            .unwrap()
            .unwrap();
        assert!(updated.metadata_schema.fields.contains_key("test_field"));
    }

    #[tokio::test]
    async fn test_add_state() {
        let service = setup_test_service().await;

        let config = TaskConfiguration::new("Test Config".to_string());
        let created = service.create_configuration(config).await.unwrap();

        let state = StateDefinition {
            name: "custom_state".to_string(),
            display_name: "Custom State".to_string(),
            color: "#ff0000".to_string(),
            description: "A custom state".to_string(),
            is_final: false,
            auto_actions: vec![],
        };

        service.add_state(created.id, state).await.unwrap();

        let updated = service
            .get_configuration(created.id)
            .await
            .unwrap()
            .unwrap();
        assert!(updated.state_machine.states.contains_key("custom_state"));
    }

    #[tokio::test]
    async fn test_create_default_configurations() {
        let service = setup_test_service().await;

        service.create_default_configurations().await.unwrap();

        let configs = service.list_configurations().await.unwrap();
        assert!(!configs.is_empty());

        let dev_config = service
            .get_configuration_by_name("Software Development")
            .await
            .unwrap();
        assert!(dev_config.is_some());
    }
}
