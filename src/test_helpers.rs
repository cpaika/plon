// Test helpers for integration testing

use crate::repository::Repository;
use std::sync::Arc;

pub struct TestContext {
    pub repository: Arc<Repository>,
}

impl TestContext {
    pub fn new_for_test() -> Self {
        let repository = Arc::new(Repository::new_memory());
        Self { repository }
    }
}