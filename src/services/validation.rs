use anyhow::{Result, bail, Context};
use regex::Regex;

/// Input validation for all user-provided data
pub struct InputValidator;

impl InputValidator {
    /// Validate and sanitize task title
    pub fn validate_title(title: &str) -> Result<String> {
        let trimmed = title.trim();
        
        if trimmed.is_empty() {
            bail!("Title cannot be empty");
        }
        
        if trimmed.len() > 200 {
            bail!("Title cannot exceed 200 characters");
        }
        
        if SQL_INJECTION_PATTERN.is_match(trimmed) {
            bail!("Title contains potentially dangerous SQL keywords");
        }
        
        if SCRIPT_INJECTION_PATTERN.is_match(trimmed) {
            bail!("Title contains potentially dangerous script content");
        }
        
        // Remove any control characters
        let sanitized = trimmed
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect::<String>();
        
        Ok(sanitized)
    }
    
    /// Validate and sanitize description
    pub fn validate_description(description: &str) -> Result<String> {
        let trimmed = description.trim();
        
        if trimmed.len() > 10000 {
            bail!("Description cannot exceed 10000 characters");
        }
        
        let script_pattern = Regex::new(r"(?i)(<script|javascript:|on\w+\s*=|<iframe|<embed|<object)").unwrap();
        if script_pattern.is_match(trimmed) {
            bail!("Description contains potentially dangerous script content");
        }
        
        // Allow more characters in description but still sanitize
        let sanitized = trimmed
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect::<String>();
        
        Ok(sanitized)
    }
    
    /// Validate position coordinates
    pub fn validate_position(x: f32, y: f32) -> Result<(f32, f32)> {
        if x.is_nan() || y.is_nan() {
            bail!("Position coordinates cannot be NaN");
        }
        
        if x.is_infinite() || y.is_infinite() {
            bail!("Position coordinates cannot be infinite");
        }
        
        // Clamp to reasonable bounds
        const MAX_COORD: f32 = 10000.0;
        const MIN_COORD: f32 = -10000.0;
        
        let clamped_x = x.max(MIN_COORD).min(MAX_COORD);
        let clamped_y = y.max(MIN_COORD).min(MAX_COORD);
        
        Ok((clamped_x, clamped_y))
    }
    
    /// Validate estimated hours
    pub fn validate_hours(hours: Option<f32>) -> Result<Option<f32>> {
        match hours {
            None => Ok(None),
            Some(h) => {
                if h < 0.0 {
                    bail!("Estimated hours cannot be negative");
                }
                if h > 1000.0 {
                    bail!("Estimated hours cannot exceed 1000");
                }
                if h.is_nan() || h.is_infinite() {
                    bail!("Estimated hours must be a valid number");
                }
                Ok(Some(h))
            }
        }
    }
    
    /// Validate URL
    pub fn validate_url(url: &str) -> Result<String> {
        let trimmed = url.trim();
        
        if trimmed.is_empty() {
            bail!("URL cannot be empty");
        }
        
        if !URL_PATTERN.is_match(trimmed) {
            bail!("Invalid URL format");
        }
        
        // Additional checks for GitHub URLs
        if trimmed.contains("github.com") {
            if !trimmed.starts_with("https://github.com/") {
                bail!("GitHub URLs must use HTTPS");
            }
        }
        
        Ok(trimmed.to_string())
    }
    
    /// Validate configuration values
    pub fn validate_config(
        max_parallel: usize,
        max_retries: u32,
    ) -> Result<()> {
        if max_parallel == 0 {
            bail!("Maximum parallel instances must be at least 1");
        }
        
        if max_parallel > 100 {
            bail!("Maximum parallel instances cannot exceed 100");
        }
        
        if max_retries > 10 {
            bail!("Maximum retries cannot exceed 10");
        }
        
        Ok(())
    }
    
    /// Sanitize file paths
    pub fn sanitize_path(path: &str) -> Result<String> {
        let trimmed = path.trim();
        
        if trimmed.is_empty() {
            bail!("Path cannot be empty");
        }
        
        // Prevent directory traversal
        if trimmed.contains("..") {
            bail!("Path cannot contain directory traversal");
        }
        
        // Remove null bytes
        if trimmed.contains('\0') {
            bail!("Path cannot contain null bytes");
        }
        
        Ok(trimmed.to_string())
    }
    
    /// Validate command for execution
    pub fn validate_command(command: &str) -> Result<String> {
        let trimmed = command.trim();
        
        if trimmed.is_empty() {
            bail!("Command cannot be empty");
        }
        
        // Check for dangerous commands
        let dangerous = ["rm -rf /", ":(){ :|:& };:", "dd if=/dev/random"];
        for danger in dangerous {
            if trimmed.contains(danger) {
                bail!("Command contains potentially dangerous operation");
            }
        }
        
        Ok(trimmed.to_string())
    }
}

/// Rate limiting for API calls
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct RateLimiter {
    limits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }
    
    pub async fn check_rate_limit(&self, key: &str) -> Result<()> {
        let mut limits = self.limits.lock().await;
        let now = Instant::now();
        
        let requests = limits.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside window
        requests.retain(|&time| now.duration_since(time) < self.window);
        
        if requests.len() >= self.max_requests {
            bail!("Rate limit exceeded. Please wait before making more requests.");
        }
        
        requests.push(now);
        Ok(())
    }
    
    pub async fn cleanup(&self) {
        let mut limits = self.limits.lock().await;
        let now = Instant::now();
        
        limits.retain(|_, requests| {
            requests.retain(|&time| now.duration_since(time) < self.window);
            !requests.is_empty()
        });
    }
}

/// Size limits for various operations
pub struct SizeLimits;

impl SizeLimits {
    pub const MAX_TITLE_LENGTH: usize = 200;
    pub const MAX_DESCRIPTION_LENGTH: usize = 10000;
    pub const MAX_BATCH_SIZE: usize = 100;
    pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
    pub const MAX_TASKS_PER_GOAL: usize = 1000;
    pub const MAX_DEPENDENCIES_PER_TASK: usize = 50;
    
    pub fn check_batch_size(size: usize) -> Result<()> {
        if size > Self::MAX_BATCH_SIZE {
            bail!("Batch size {} exceeds maximum of {}", size, Self::MAX_BATCH_SIZE);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_title_validation() {
        // Valid titles
        assert!(InputValidator::validate_title("Normal Task").is_ok());
        assert!(InputValidator::validate_title("Task-123").is_ok());
        assert!(InputValidator::validate_title("Important! Do this.").is_ok());
        
        // Invalid titles
        assert!(InputValidator::validate_title("").is_err());
        assert!(InputValidator::validate_title("   ").is_err());
        assert!(InputValidator::validate_title("DROP TABLE tasks").is_err());
        assert!(InputValidator::validate_title("<script>alert('xss')</script>").is_err());
        
        // Edge cases
        let long_title = "x".repeat(201);
        assert!(InputValidator::validate_title(&long_title).is_err());
    }
    
    #[test]
    fn test_position_validation() {
        // Valid positions
        assert!(InputValidator::validate_position(100.0, 200.0).is_ok());
        assert!(InputValidator::validate_position(-500.0, 500.0).is_ok());
        assert!(InputValidator::validate_position(0.0, 0.0).is_ok());
        
        // Invalid positions
        assert!(InputValidator::validate_position(f32::NAN, 100.0).is_err());
        assert!(InputValidator::validate_position(100.0, f32::INFINITY).is_err());
        assert!(InputValidator::validate_position(f32::NEG_INFINITY, f32::NEG_INFINITY).is_err());
        
        // Clamping
        let (x, y) = InputValidator::validate_position(20000.0, -20000.0).unwrap();
        assert_eq!(x, 10000.0);
        assert_eq!(y, -10000.0);
    }
    
    #[test]
    fn test_url_validation() {
        // Valid URLs
        assert!(InputValidator::validate_url("https://github.com/user/repo").is_ok());
        assert!(InputValidator::validate_url("http://example.com/path?query=1").is_ok());
        
        // Invalid URLs
        assert!(InputValidator::validate_url("").is_err());
        assert!(InputValidator::validate_url("not-a-url").is_err());
        assert!(InputValidator::validate_url("javascript:alert('xss')").is_err());
        assert!(InputValidator::validate_url("http://github.com/repo").is_err()); // GitHub must use HTTPS
    }
    
    #[test]
    fn test_path_sanitization() {
        // Valid paths
        assert!(InputValidator::sanitize_path("/home/user/file.txt").is_ok());
        assert!(InputValidator::sanitize_path("./relative/path").is_ok());
        
        // Invalid paths
        assert!(InputValidator::sanitize_path("../../etc/passwd").is_err());
        assert!(InputValidator::sanitize_path("path\0with\0null").is_err());
        assert!(InputValidator::sanitize_path("").is_err());
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = RateLimiter::new(3, Duration::from_secs(1));
        
        // First 3 requests should succeed
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        assert!(limiter.check_rate_limit("user1").await.is_ok());
        
        // 4th request should fail
        assert!(limiter.check_rate_limit("user1").await.is_err());
        
        // Different key should work
        assert!(limiter.check_rate_limit("user2").await.is_ok());
        
        // After waiting, should work again
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(limiter.check_rate_limit("user1").await.is_ok());
    }
    
    #[test]
    fn test_command_validation() {
        // Valid commands
        assert!(InputValidator::validate_command("npm test").is_ok());
        assert!(InputValidator::validate_command("cargo build --release").is_ok());
        
        // Dangerous commands
        assert!(InputValidator::validate_command("rm -rf /").is_err());
        assert!(InputValidator::validate_command(":(){ :|:& };:").is_err()); // Fork bomb
    }
}