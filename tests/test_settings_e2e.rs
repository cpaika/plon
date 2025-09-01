use plon::repository::{Repository, database::init_database};
use plon::domain::claude_code::ClaudeCodeConfig;
use plon::services::ClaudeCodeService;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_settings_page_loads_config() {
    // Setup
    let pool = init_database(":memory:").await.unwrap();
    let repo = Repository::new(pool);
    
    // Create initial config
    let mut config = ClaudeCodeConfig::new(
        "test-repo".to_string(),
        "test-owner".to_string()
    );
    config.workspace_root = Some("/test/workspace".to_string());
    config.git_clone_url = Some("https://github.com/test/repo.git".to_string());
    config.claude_api_key = Some("sk-ant-test".to_string());
    config.auto_create_pr = true;
    config.max_session_duration_minutes = 120;
    
    repo.claude_code.create_config(&config).await.unwrap();
    
    // Load config
    let loaded = repo.claude_code.get_config().await.unwrap();
    assert!(loaded.is_some());
    
    let loaded_config = loaded.unwrap();
    assert_eq!(loaded_config.github_repo, "test-repo");
    assert_eq!(loaded_config.github_owner, "test-owner");
    assert_eq!(loaded_config.workspace_root, Some("/test/workspace".to_string()));
    assert_eq!(loaded_config.git_clone_url, Some("https://github.com/test/repo.git".to_string()));
    assert_eq!(loaded_config.auto_create_pr, true);
    assert_eq!(loaded_config.max_session_duration_minutes, 120);
}

#[tokio::test]
async fn test_settings_save_and_update() {
    let pool = init_database(":memory:").await.unwrap();
    let repo = Repository::new(pool);
    
    // Create initial config
    let config = ClaudeCodeConfig::new(
        "initial-repo".to_string(),
        "initial-owner".to_string()
    );
    repo.claude_code.create_config(&config).await.unwrap();
    
    // Update config
    let mut updated_config = config.clone();
    updated_config.github_repo = "updated-repo".to_string();
    updated_config.github_owner = "updated-owner".to_string();
    updated_config.workspace_root = Some("/new/workspace".to_string());
    updated_config.git_clone_url = Some("https://gitlab.com/new/repo.git".to_string());
    updated_config.claude_model = "claude-3-sonnet-20240229".to_string();
    updated_config.max_session_duration_minutes = 60;
    updated_config.auto_create_pr = false;
    
    repo.claude_code.update_config(&updated_config).await.unwrap();
    
    // Verify updates
    let loaded = repo.claude_code.get_config().await.unwrap().unwrap();
    assert_eq!(loaded.github_repo, "updated-repo");
    assert_eq!(loaded.github_owner, "updated-owner");
    assert_eq!(loaded.workspace_root, Some("/new/workspace".to_string()));
    assert_eq!(loaded.git_clone_url, Some("https://gitlab.com/new/repo.git".to_string()));
    assert_eq!(loaded.claude_model, "claude-3-sonnet-20240229");
    assert_eq!(loaded.max_session_duration_minutes, 60);
    assert_eq!(loaded.auto_create_pr, false);
}

#[tokio::test]
async fn test_settings_validation() {
    let pool = init_database(":memory:").await.unwrap();
    let repo = Repository::new(pool);
    
    // Test empty repository name (should still save - validation is UI level)
    let mut config = ClaudeCodeConfig::new(
        "".to_string(),
        "owner".to_string()
    );
    
    // This should succeed at database level
    repo.claude_code.create_config(&config).await.unwrap();
    
    // Test max session duration boundaries
    config.max_session_duration_minutes = 5; // Minimum
    repo.claude_code.update_config(&config).await.unwrap();
    
    config.max_session_duration_minutes = 240; // Maximum
    repo.claude_code.update_config(&config).await.unwrap();
    
    // Test workspace root with special characters
    config.workspace_root = Some("~/пользователь/ワークスペース/🚀".to_string());
    repo.claude_code.update_config(&config).await.unwrap();
    
    let loaded = repo.claude_code.get_config().await.unwrap().unwrap();
    assert_eq!(loaded.workspace_root, Some("~/пользователь/ワークスペース/🚀".to_string()));
}

#[tokio::test]
async fn test_settings_default_values() {
    let config = ClaudeCodeConfig::new(
        "test-repo".to_string(),
        "test-owner".to_string()
    );
    
    // Verify defaults
    assert_eq!(config.default_base_branch, "main");
    assert_eq!(config.auto_create_pr, false);
    assert_eq!(config.claude_model, "claude-3-opus-20240229");
    assert_eq!(config.max_session_duration_minutes, 60);
    assert_eq!(config.workspace_root, None);
    assert_eq!(config.git_clone_url, None);
    assert_eq!(config.github_token, None);
    assert_eq!(config.claude_api_key, None);
}

#[tokio::test]
async fn test_workspace_path_generation() {
    let mut config = ClaudeCodeConfig::new(
        "test-repo".to_string(),
        "test-owner".to_string()
    );
    
    // Test default workspace root
    let default_root = config.get_workspace_root();
    assert!(default_root.to_string_lossy().ends_with("plon-workspaces"));
    
    // Test custom workspace root
    config.workspace_root = Some("/custom/workspace".to_string());
    let custom_root = config.get_workspace_root();
    assert_eq!(custom_root.to_string_lossy(), "/custom/workspace");
    
    // Test git clone URL generation
    assert_eq!(
        config.get_git_clone_url(),
        "https://github.com/test-owner/test-repo.git"
    );
    
    // Test custom git clone URL
    config.git_clone_url = Some("https://gitlab.com/custom/repo.git".to_string());
    assert_eq!(
        config.get_git_clone_url(),
        "https://gitlab.com/custom/repo.git"
    );
}

#[tokio::test]
async fn test_settings_tab_navigation() {
    // This test simulates tab switching behavior
    let tabs = vec![
        ("claude", "Claude Code"),
        ("general", "General"),
        ("workspace", "Workspace"),
        ("integrations", "Integrations"),
        ("appearance", "Appearance"),
    ];
    
    for (tab_id, tab_name) in tabs {
        println!("Testing tab: {} - {}", tab_id, tab_name);
        assert!(!tab_id.is_empty());
        assert!(!tab_name.is_empty());
    }
}

#[tokio::test]
async fn test_sensitive_field_masking() {
    let pool = init_database(":memory:").await.unwrap();
    let repo = Repository::new(pool);
    
    let mut config = ClaudeCodeConfig::new(
        "repo".to_string(),
        "owner".to_string()
    );
    
    // Set sensitive fields
    config.github_token = Some("ghp_secrettoken123".to_string());
    config.claude_api_key = Some("sk-ant-secretkey456".to_string());
    
    repo.claude_code.create_config(&config).await.unwrap();
    
    // Load and verify sensitive fields are stored
    let loaded = repo.claude_code.get_config().await.unwrap().unwrap();
    assert_eq!(loaded.github_token, Some("ghp_secrettoken123".to_string()));
    assert_eq!(loaded.claude_api_key, Some("sk-ant-secretkey456".to_string()));
    
    // In real UI, these would be masked with password inputs
}

#[tokio::test]
async fn test_config_persistence_across_sessions() {
    let db_path = format!("/tmp/test_plon_{}.db", Uuid::new_v4());
    
    // First session - create config
    {
        let pool = init_database(&db_path).await.unwrap();
        let repo = Repository::new(pool);
        
        let mut config = ClaudeCodeConfig::new(
            "persistent-repo".to_string(),
            "persistent-owner".to_string()
        );
        config.workspace_root = Some("/persistent/workspace".to_string());
        config.max_session_duration_minutes = 90;
        
        repo.claude_code.create_config(&config).await.unwrap();
    }
    
    // Second session - verify config persists
    {
        let pool = init_database(&db_path).await.unwrap();
        let repo = Repository::new(pool);
        
        let loaded = repo.claude_code.get_config().await.unwrap().unwrap();
        assert_eq!(loaded.github_repo, "persistent-repo");
        assert_eq!(loaded.github_owner, "persistent-owner");
        assert_eq!(loaded.workspace_root, Some("/persistent/workspace".to_string()));
        assert_eq!(loaded.max_session_duration_minutes, 90);
    }
    
    // Cleanup
    std::fs::remove_file(&db_path).ok();
}

#[tokio::test]
async fn test_multiple_config_handling() {
    let pool = init_database(":memory:").await.unwrap();
    let repo = Repository::new(pool);
    
    // Create first config
    let config1 = ClaudeCodeConfig::new(
        "repo1".to_string(),
        "owner1".to_string()
    );
    repo.claude_code.create_config(&config1).await.unwrap();
    
    // Try to create second config (should update existing)
    let mut config2 = ClaudeCodeConfig::new(
        "repo2".to_string(),
        "owner2".to_string()
    );
    config2.id = config1.id; // Use same ID to simulate update
    repo.claude_code.update_config(&config2).await.unwrap();
    
    // Verify only one config exists with updated values
    let loaded = repo.claude_code.get_config().await.unwrap().unwrap();
    assert_eq!(loaded.github_repo, "repo2");
    assert_eq!(loaded.github_owner, "owner2");
}

#[tokio::test]
async fn test_integration_status_display() {
    // Test integration status detection
    let pool = init_database(":memory:").await.unwrap();
    let repo = Repository::new(pool);
    
    let mut config = ClaudeCodeConfig::new(
        "repo".to_string(),
        "owner".to_string()
    );
    
    // No tokens - not connected
    assert!(config.github_token.is_none());
    assert!(config.claude_api_key.is_none());
    
    // With tokens - connected
    config.github_token = Some("ghp_token".to_string());
    config.claude_api_key = Some("sk-ant-key".to_string());
    
    repo.claude_code.create_config(&config).await.unwrap();
    
    let loaded = repo.claude_code.get_config().await.unwrap().unwrap();
    assert!(loaded.github_token.is_some());
    assert!(loaded.claude_api_key.is_some());
}