use crate::domain::resource::Resource;
use crate::repository::Repository;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

pub struct ResourceService {
    repository: Arc<Repository>,
}

impl ResourceService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, resource: Resource) -> Result<Resource> {
        self.repository.resources.create(&resource).await?;
        Ok(resource)
    }

    pub async fn update(&self, resource: Resource) -> Result<Resource> {
        self.repository.resources.update(&resource).await?;
        Ok(resource)
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Resource>> {
        self.repository.resources.get(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        self.repository.resources.delete(id).await
    }

    pub async fn list_all(&self) -> Result<Vec<Resource>> {
        self.repository.resources.list_all().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::database::init_test_database;

    async fn setup() -> ResourceService {
        let pool = init_test_database().await.unwrap();
        let repository = Arc::new(Repository::new(pool));
        ResourceService::new(repository)
    }

    #[tokio::test]
    async fn test_create_resource() {
        let service = setup().await;
        let resource = Resource::new(
            "Developer 1".to_string(),
            "Software Engineer".to_string(),
            40.0,
        );

        let created = service.create(resource.clone()).await.unwrap();
        assert_eq!(created.name, resource.name);
        assert_eq!(created.role, "Software Engineer");
        assert_eq!(created.weekly_hours, 40.0);
    }

    #[tokio::test]
    async fn test_get_resource() {
        let service = setup().await;
        let resource = Resource::new(
            "Test Resource".to_string(),
            "DevOps Engineer".to_string(),
            40.0,
        );
        let created = service.create(resource).await.unwrap();

        let retrieved = service.get(created.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_update_resource() {
        let service = setup().await;
        let mut resource = Resource::new(
            "Original Name".to_string(),
            "Backend Developer".to_string(),
            40.0,
        );
        let created = service.create(resource.clone()).await.unwrap();

        resource = created;
        resource.name = "Updated Name".to_string();
        resource.role = "Senior Backend Developer".to_string();
        resource.current_load = 25.0;

        let updated = service.update(resource.clone()).await.unwrap();
        assert_eq!(updated.name, "Updated Name");

        let retrieved = service.get(updated.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Name");
        assert_eq!(retrieved.role, "Senior Backend Developer");
        assert_eq!(retrieved.current_load, 25.0);
    }

    #[tokio::test]
    async fn test_delete_resource() {
        let service = setup().await;
        let resource = Resource::new("To Delete".to_string(), "Intern".to_string(), 20.0);
        let created = service.create(resource).await.unwrap();

        let deleted = service.delete(created.id).await.unwrap();
        assert!(deleted);

        let retrieved = service.get(created.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_all_resources() {
        let service = setup().await;

        // Create multiple resources with different roles
        let resources = vec![
            Resource::new("Resource 1".to_string(), "Developer".to_string(), 40.0),
            Resource::new("Resource 2".to_string(), "Designer".to_string(), 35.0),
            Resource::new("Resource 3".to_string(), "Manager".to_string(), 45.0),
        ];

        for resource in &resources {
            service.create(resource.clone()).await.unwrap();
        }

        let all_resources = service.list_all().await.unwrap();
        assert_eq!(all_resources.len(), 3);

        // Verify all resource roles are present
        let roles: Vec<String> = all_resources.iter().map(|r| r.role.clone()).collect();
        assert!(roles.contains(&"Developer".to_string()));
        assert!(roles.contains(&"Designer".to_string()));
        assert!(roles.contains(&"Manager".to_string()));
    }

    #[tokio::test]
    async fn test_resource_with_skills() {
        let service = setup().await;
        let mut resource = Resource::new(
            "Skilled Developer".to_string(),
            "Full Stack Developer".to_string(),
            40.0,
        );

        // Add skills
        resource.skills.insert("Rust".to_string());
        resource.skills.insert("TypeScript".to_string());
        resource.skills.insert("Docker".to_string());

        let created = service.create(resource).await.unwrap();
        assert_eq!(created.skills.len(), 3);
        assert!(created.skills.contains(&"Rust".to_string()));
    }
}
