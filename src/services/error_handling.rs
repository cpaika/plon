use anyhow::{Context, Result};
use std::fmt;
use thiserror::Error;
use tracing::{error, warn, info, debug, instrument};
use uuid::Uuid;

/// Custom error types with descriptive messages
#[derive(Error, Debug)]
pub enum PlonError {
    #[error("Task not found: {id}")]
    TaskNotFound { id: Uuid },
    
    #[error("Invalid task state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { 
        from: crate::domain::task::TaskStatus, 
        to: crate::domain::task::TaskStatus 
    },
    
    #[error("Circular dependency detected: {task1} <-> {task2}")]
    CircularDependency { task1: Uuid, task2: Uuid },
    
    #[error("Resource limit exceeded: {resource} (limit: {limit}, requested: {requested})")]
    ResourceLimitExceeded { 
        resource: String, 
        limit: usize, 
        requested: usize 
    },
    
    #[error("Validation failed for {field}: {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Database operation failed: {operation}")]
    DatabaseError { operation: String, source: anyhow::Error },
    
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    #[error("Operation timed out after {duration_secs} seconds")]
    TimeoutError { duration_secs: u64 },
    
    #[error("Concurrent modification detected for {entity_type} {id}")]
    ConcurrentModification { entity_type: String, id: Uuid },
    
    #[error("Insufficient permissions: {action} on {resource}")]
    PermissionDenied { action: String, resource: String },
}

/// Error context wrapper for better debugging
pub struct ErrorContext {
    operation: String,
    details: Vec<(String, String)>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            details: Vec::new(),
        }
    }
    
    pub fn with_detail(mut self, key: impl Into<String>, value: impl fmt::Display) -> Self {
        self.details.push((key.into(), value.to_string()));
        self
    }
    
    pub fn wrap<T>(self, result: Result<T>) -> Result<T> {
        result.with_context(|| {
            let mut msg = format!("Operation '{}' failed", self.operation);
            if !self.details.is_empty() {
                msg.push_str(" with context:");
                for (key, value) in self.details {
                    msg.push_str(&format!("\n  {}: {}", key, value));
                }
            }
            msg
        })
    }
}

/// Structured logging helpers
pub struct LogHelper;

impl LogHelper {
    #[instrument(skip(task_id))]
    pub fn log_task_operation(operation: &str, task_id: Uuid, success: bool) {
        if success {
            info!(
                task_id = %task_id,
                operation = %operation,
                "Task operation completed successfully"
            );
        } else {
            error!(
                task_id = %task_id,
                operation = %operation,
                "Task operation failed"
            );
        }
    }
    
    #[instrument(skip(error))]
    pub fn log_error_with_context(context: &str, error: &anyhow::Error) {
        error!(
            context = %context,
            error = %error,
            error_chain = ?error.chain().collect::<Vec<_>>(),
            "Error occurred"
        );
    }
    
    pub fn log_validation_failure(field: &str, value: &str, reason: &str) {
        warn!(
            field = %field,
            value = %value,
            reason = %reason,
            "Validation failed"
        );
    }
    
    pub fn log_performance_warning(operation: &str, duration_ms: u64, threshold_ms: u64) {
        if duration_ms > threshold_ms {
            warn!(
                operation = %operation,
                duration_ms = duration_ms,
                threshold_ms = threshold_ms,
                "Operation exceeded performance threshold"
            );
        }
    }
    
    pub fn log_retry_attempt(operation: &str, attempt: u32, max_attempts: u32, error: &str) {
        info!(
            operation = %operation,
            attempt = attempt,
            max_attempts = max_attempts,
            error = %error,
            "Retrying operation"
        );
    }
}

/// User-friendly error messages
pub struct UserErrorFormatter;

impl UserErrorFormatter {
    pub fn format_for_ui(error: &anyhow::Error) -> String {
        // Check if it's one of our custom errors
        if let Some(plon_error) = error.downcast_ref::<PlonError>() {
            return Self::format_plon_error(plon_error);
        }
        
        // Check for common error patterns
        let error_str = error.to_string();
        
        if error_str.contains("database") || error_str.contains("sqlite") {
            return "A database error occurred. Please try again or contact support if the issue persists.".to_string();
        }
        
        if error_str.contains("network") || error_str.contains("connection") {
            return "Network connection error. Please check your internet connection and try again.".to_string();
        }
        
        if error_str.contains("permission") || error_str.contains("unauthorized") {
            return "You don't have permission to perform this action.".to_string();
        }
        
        if error_str.contains("timeout") {
            return "The operation timed out. Please try again.".to_string();
        }
        
        // Generic fallback
        "An unexpected error occurred. Please try again.".to_string()
    }
    
    fn format_plon_error(error: &PlonError) -> String {
        match error {
            PlonError::TaskNotFound { .. } => {
                "The requested task could not be found.".to_string()
            }
            PlonError::InvalidStateTransition { from, to } => {
                format!("Cannot change task status from {:?} to {:?}", from, to)
            }
            PlonError::CircularDependency { .. } => {
                "Cannot create this dependency as it would create a circular reference.".to_string()
            }
            PlonError::ResourceLimitExceeded { resource, limit, .. } => {
                format!("Too many {} (maximum: {})", resource, limit)
            }
            PlonError::ValidationError { field, reason } => {
                format!("Invalid {}: {}", field, reason)
            }
            PlonError::ConfigurationError { message } => {
                format!("Configuration error: {}", message)
            }
            PlonError::TimeoutError { duration_secs } => {
                format!("Operation timed out after {} seconds", duration_secs)
            }
            _ => error.to_string(),
        }
    }
}

/// Error recovery strategies
pub enum RecoveryStrategy {
    Retry { max_attempts: u32, delay_ms: u64 },
    Fallback { alternative: String },
    Skip,
    Abort,
}

pub struct ErrorRecovery;

impl ErrorRecovery {
    pub fn suggest_recovery(error: &anyhow::Error) -> RecoveryStrategy {
        let error_str = error.to_string().to_lowercase();
        
        if error_str.contains("network") || error_str.contains("timeout") {
            RecoveryStrategy::Retry { 
                max_attempts: 3, 
                delay_ms: 1000 
            }
        } else if error_str.contains("lock") || error_str.contains("concurrent") {
            RecoveryStrategy::Retry { 
                max_attempts: 5, 
                delay_ms: 100 
            }
        } else if error_str.contains("not found") {
            RecoveryStrategy::Skip
        } else {
            RecoveryStrategy::Abort
        }
    }
    
    pub async fn with_retry<F, T>(
        operation: F,
        max_attempts: u32,
        delay_ms: u64,
    ) -> Result<T>
    where
        F: Fn() -> Result<T>,
    {
        let mut last_error = None;
        
        for attempt in 1..=max_attempts {
            match operation() {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Operation succeeded after {} attempts", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_attempts {
                        LogHelper::log_retry_attempt(
                            "operation",
                            attempt,
                            max_attempts,
                            &last_error.as_ref().unwrap().to_string(),
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
}

/// Performance monitoring
use std::time::Instant;

pub struct PerformanceMonitor {
    operation: String,
    start: Instant,
    threshold_ms: u64,
}

impl PerformanceMonitor {
    pub fn new(operation: impl Into<String>, threshold_ms: u64) -> Self {
        Self {
            operation: operation.into(),
            start: Instant::now(),
            threshold_ms,
        }
    }
}

impl Drop for PerformanceMonitor {
    fn drop(&mut self) {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        LogHelper::log_performance_warning(
            &self.operation,
            duration_ms,
            self.threshold_ms,
        );
        
        debug!(
            operation = %self.operation,
            duration_ms = duration_ms,
            "Operation completed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_formatting() {
        let error = PlonError::TaskNotFound { 
            id: Uuid::new_v4() 
        };
        
        let formatted = UserErrorFormatter::format_plon_error(&error);
        assert!(formatted.contains("could not be found"));
    }
    
    #[test]
    fn test_error_context() {
        let result: Result<()> = Err(anyhow::anyhow!("Database connection failed"));
        
        let wrapped = ErrorContext::new("save_task")
            .with_detail("task_id", Uuid::new_v4())
            .with_detail("attempt", 1)
            .wrap(result);
        
        assert!(wrapped.is_err());
        let error_msg = wrapped.unwrap_err().to_string();
        assert!(error_msg.contains("save_task"));
        assert!(error_msg.contains("task_id"));
    }
    
    #[test]
    fn test_recovery_strategy() {
        let network_error = anyhow::anyhow!("Network timeout occurred");
        let strategy = ErrorRecovery::suggest_recovery(&network_error);
        
        match strategy {
            RecoveryStrategy::Retry { max_attempts, .. } => {
                assert!(max_attempts > 0);
            }
            _ => panic!("Expected retry strategy for network error"),
        }
    }
    
    #[tokio::test]
    async fn test_retry_logic() {
        let mut counter = 0;
        
        let result = ErrorRecovery::with_retry(
            || {
                counter += 1;
                if counter < 3 {
                    Err(anyhow::anyhow!("Temporary failure"))
                } else {
                    Ok("Success")
                }
            },
            5,
            10,
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(counter, 3);
    }
}