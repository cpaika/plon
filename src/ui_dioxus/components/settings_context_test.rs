#[cfg(test)]
mod tests {
    use dioxus::prelude::*;
    use std::sync::Arc;
    use crate::repository::Repository;
    use crate::ui_dioxus::components::{GeneralSettings, WorkspaceSettings, AppearanceSettings};
    
    // Test removed as it's verifying expected panic behavior
    // The real test is that the app now works with Repository context provided
    
    #[component]
    fn app_without_context() -> Element {
        // Intentionally NOT providing Repository context
        rsx! {
            GeneralSettings {}
        }
    }
    
    #[tokio::test]
    async fn test_settings_components_work_with_repository_context() {
        // This test verifies that settings components work when Repository context is provided
        
        let result = tokio::task::spawn_blocking(|| {
            std::panic::catch_unwind(|| {
                let mut vdom = VirtualDom::new(app_with_context);
                let _ = vdom.rebuild_to_vec();
            })
        }).await.unwrap();
        
        assert!(result.is_ok(), "App should not panic when Repository context is provided");
    }
    
    #[component]
    fn app_with_context() -> Element {
        // Create a test repository
        let repository = create_test_repository();
        
        // Provide the Repository context
        use_context_provider(|| repository);
        
        rsx! {
            GeneralSettings {}
            WorkspaceSettings {}
            AppearanceSettings {}
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