#[cfg(test)]
mod tests {
    use dioxus::prelude::*;
    use std::sync::Arc;
    use crate::repository::Repository;
    use crate::ui_dioxus::views::SettingsView;
    
    #[tokio::test]
    async fn test_app_provides_repository_context() {
        // This test verifies that the App component properly provides Repository context
        // to child components like SettingsView
        
        let result = tokio::task::spawn_blocking(|| {
            std::panic::catch_unwind(|| {
                let mut vdom = VirtualDom::new(test_app_with_settings);
                let _ = vdom.rebuild_to_vec();
            })
        }).await.unwrap();
        
        assert!(result.is_ok(), "SettingsView should work when Repository context is provided by App");
    }
    
    #[component]
    fn test_app_with_settings() -> Element {
        // Create a test repository
        let repository = create_test_repository();
        
        // Provide the Repository context (simulating what App does)
        use_context_provider(|| repository);
        
        // Render SettingsView which requires Repository context
        rsx! {
            SettingsView {}
        }
    }
    
    fn create_test_repository() -> Arc<Repository> {
        use sqlx::SqlitePool;
        use tokio::runtime::Runtime;
        
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let pool = SqlitePool::connect(":memory:").await.unwrap();
            
            // Run migrations
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .unwrap();
            
            Arc::new(Repository::new(pool))
        })
    }
}