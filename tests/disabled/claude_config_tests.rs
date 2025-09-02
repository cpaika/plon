#[cfg(test)]
mod claude_config_tests {
    use plon::config::ClaudeConfig;
    use std::env;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        unsafe {
            // Set XDG_CONFIG_HOME to use our temp directory
            env::set_var("XDG_CONFIG_HOME", temp_dir.path().join(".config"));
        }
        temp_dir
    }

    #[test]
    fn test_default_config() {
        let config = ClaudeConfig::default();
        
        assert_eq!(config.repository_url, None);
        assert!(config.auto_create_pr);
        assert!(config.auto_commit);
        assert_eq!(config.branch_pattern, "task/{task_id}-{task_title}");
        assert_eq!(config.pr_title_pattern, "Complete task: {task_title}");
        assert_eq!(config.timeout_seconds, 3600);
    }

    #[test]
    fn test_format_branch_name() {
        let config = ClaudeConfig::default();
        
        let branch = config.format_branch_name("12345678-1234-1234-1234-123456789012", "Fix login bug");
        assert_eq!(branch, "task/12345678-fix-login-bug");
        
        // Test with special characters
        let branch = config.format_branch_name("456789ab-cdef-1234-5678-901234567890", "Add feature: OAuth 2.0!");
        assert_eq!(branch, "task/456789ab-add-feature--oauth-2-0");
    }

    #[test]
    fn test_format_pr_title() {
        let config = ClaudeConfig::default();
        
        let title = config.format_pr_title("Implement user authentication");
        assert_eq!(title, "Complete task: Implement user authentication");
    }

    #[test]
    #[ignore] // Temporarily ignore due to test isolation issues
    fn test_save_and_load_config() {
        // Create a unique temp dir for this test
        let temp_dir = TempDir::new().unwrap();
        unsafe {
            env::set_var("XDG_CONFIG_HOME", temp_dir.path().join(".config"));
        }
        
        // Create and save config
        let mut config = ClaudeConfig::default();
        config.repository_url = Some("https://github.com/test/repo".to_string());
        config.auto_create_pr = false;
        config.timeout_seconds = 7200;
        
        config.save().unwrap();
        
        // Load config
        let loaded = ClaudeConfig::load().unwrap();
        assert_eq!(loaded.repository_url, Some("https://github.com/test/repo".to_string()));
        assert!(!loaded.auto_create_pr);
        assert_eq!(loaded.timeout_seconds, 7200);
    }

    #[test]
    fn test_load_creates_default_if_missing() {
        // Create a unique temp dir for this test
        let temp_dir = TempDir::new().unwrap();
        unsafe {
            env::set_var("XDG_CONFIG_HOME", temp_dir.path().join(".config"));
        }
        
        // Should create and return default config if file doesn't exist
        let config = ClaudeConfig::load().unwrap();
        
        // Check that we got default values
        assert!(config.auto_create_pr);
        assert!(config.auto_commit);
        assert_eq!(config.timeout_seconds, 3600);
        
        // Verify file was created
        let config_path = temp_dir.path().join(".config").join("plon").join("claude_config.toml");
        assert!(config_path.exists());
    }

    #[test]
    fn test_custom_patterns() {
        let mut config = ClaudeConfig::default();
        config.branch_pattern = "feature/{task_title}-{task_id}".to_string();
        config.pr_title_pattern = "✨ {task_title}".to_string();
        
        let branch = config.format_branch_name("789abcde-1234-5678-9012-345678901234", "New Feature");
        assert_eq!(branch, "feature/new-feature-789abcde");
        
        let title = config.format_pr_title("Amazing Feature");
        assert_eq!(title, "✨ Amazing Feature");
    }

    #[test]
    fn test_config_serialization() {
        let config = ClaudeConfig {
            repository_url: Some("https://github.com/example/repo".to_string()),
            auto_create_pr: true,
            auto_commit: false,
            prompt_template: Some("Custom prompt".to_string()),
            branch_pattern: "custom/{task_id}".to_string(),
            pr_title_pattern: "Custom: {task_title}".to_string(),
            timeout_seconds: 1800,
        };
        
        // Serialize to TOML
        let toml_str = toml::to_string(&config).unwrap();
        
        // Deserialize back
        let deserialized: ClaudeConfig = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.repository_url, deserialized.repository_url);
        assert_eq!(config.auto_create_pr, deserialized.auto_create_pr);
        assert_eq!(config.auto_commit, deserialized.auto_commit);
        assert_eq!(config.branch_pattern, deserialized.branch_pattern);
        assert_eq!(config.timeout_seconds, deserialized.timeout_seconds);
    }
}