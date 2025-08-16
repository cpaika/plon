use crate::domain::resource::Resource;
use crate::repository::Repository;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ResourceService {
    repository: Arc<Repository>,
    // Temporary in-memory storage for tests
    resources: Arc<Mutex<Vec<Resource>>>,
}

impl ResourceService {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { 
            repository,
            resources: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn create(&self, resource: Resource) -> Result<Resource> {
        // Store in memory for tests
        self.resources.lock().await.push(resource.clone());
        Ok(resource)
    }

    pub async fn list_all(&self) -> Result<Vec<Resource>> {
        // Return from memory for tests
        Ok(self.resources.lock().await.clone())
    }
}