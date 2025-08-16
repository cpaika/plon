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
        Self { 
            repository,
        }
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