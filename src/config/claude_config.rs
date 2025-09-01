use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// The repository URL for cloning/PR creation
    pub repository_url: Option<String>,
    
    /// Whether to automatically create PRs
    pub auto_create_pr: bool,
    
    /// Whether to automatically commit changes
    pub auto_commit: bool,
    
    /// Custom prompt template for tasks
    pub prompt_template: Option<String>,
    
    /// Branch naming pattern (supports {task_id}, {task_title})
    pub branch_pattern: String,
    
    /// PR title pattern (supports {task_title})
    pub pr_title_pattern: String,
    
    /// Maximum time to wait for Claude Code to complete (in seconds)
    pub timeout_seconds: u64,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            repository_url: None,
            auto_create_pr: true,
            auto_commit: true,
            prompt_template: None,
            branch_pattern: "task/{task_id}-{task_title}".to_string(),
            pr_title_pattern: "Complete task: {task_title}".to_string(),
            timeout_seconds: 3600, // 1 hour default
        }
    }
}

impl ClaudeConfig {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config file
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        println!("✅ Configuration saved to: {:?}", config_path);
        Ok(())
    }
    
    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        
        Ok(config_dir.join("plon").join("claude_config.toml"))
    }
    
    /// Get repository URL, either from config or by detecting from git
    pub fn get_repository_url(&self) -> Result<String> {
        if let Some(url) = &self.repository_url {
            return Ok(url.clone());
        }
        
        // Try to detect from git remote
        let output = std::process::Command::new("git")
            .args(&["remote", "get-url", "origin"])
            .output()?;
        
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(url)
        } else {
            Err(anyhow::anyhow!(
                "No repository URL configured and could not detect from git remote.\n\
                Configure with: plon config set repository_url <url>"
            ))
        }
    }
    
    /// Format branch name with task information
    pub fn format_branch_name(&self, task_id: &str, task_title: &str) -> String {
        self.branch_pattern
            .replace("{task_id}", &task_id.chars().take(8).collect::<String>())
            .replace("{task_title}", &sanitize_for_branch(task_title))
    }
    
    /// Format PR title with task information
    pub fn format_pr_title(&self, task_title: &str) -> String {
        self.pr_title_pattern
            .replace("{task_title}", task_title)
    }
}

fn sanitize_for_branch(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(50) // Limit length
        .collect()
}