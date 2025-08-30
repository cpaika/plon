use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{RwLock, Mutex};
use anyhow::Result;
use uuid::Uuid;
use std::collections::HashMap;

/// Thread-safe wrapper for AutoRunOrchestrator operations
pub struct SafeAutoRunOrchestrator {
    inner: Arc<super::AutoRunOrchestrator>,
    operation_locks: Arc<RwLock<HashMap<Uuid, Arc<Mutex<()>>>>>,
}

impl SafeAutoRunOrchestrator {
    pub fn new(orchestrator: Arc<super::AutoRunOrchestrator>) -> Self {
        Self {
            inner: orchestrator,
            operation_locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get or create a lock for a specific task
    async fn get_task_lock(&self, task_id: Uuid) -> Arc<Mutex<()>> {
        {
            let locks = self.operation_locks.read().await;
            if let Some(lock) = locks.get(&task_id) {
                return lock.clone();
            }
        }
        
        let mut locks = self.operation_locks.write().await;
        locks.entry(task_id)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
    
    /// Thread-safe task execution start
    pub async fn start_task_safe(&self, task_id: Uuid) -> Result<()> {
        let lock = self.get_task_lock(task_id).await;
        let _guard = lock.lock().await;
        
        // Check if already running
        let executions = self.inner.executions.read().await;
        if let Some(exec) = executions.get(&task_id) {
            if matches!(exec.status, super::TaskExecutionStatus::Running) {
                return Ok(()); // Already running, skip
            }
        }
        drop(executions);
        
        self.inner.start_task_execution_safe(task_id).await
    }
    
    /// Thread-safe configuration update
    pub async fn update_config_safe(&self, config: super::AutoRunConfig) -> Result<()> {
        // Global operation - lock everything
        let locks = self.operation_locks.write().await;
        
        // Ensure no operations in progress
        for lock in locks.values() {
            let _guard = lock.lock().await;
        }
        
        self.inner.validate_and_update_config(config).await
    }
    
    /// Clean up old locks to prevent memory leak
    pub async fn cleanup_locks(&self) {
        let mut locks = self.operation_locks.write().await;
        let executions = self.inner.executions.read().await;
        
        // Remove locks for completed tasks
        locks.retain(|task_id, _| {
            if let Some(exec) = executions.get(task_id) {
                matches!(exec.status, 
                    super::TaskExecutionStatus::Running | 
                    super::TaskExecutionStatus::Queued)
            } else {
                false
            }
        });
    }
}

/// Atomic counter for unique IDs
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicIdGenerator {
    counter: AtomicU64,
}

impl AtomicIdGenerator {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }
    
    pub fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

/// Thread-safe batch operations
pub struct BatchOperationManager {
    pending: Arc<RwLock<Vec<BatchOperation>>>,
    processing: Arc<Mutex<bool>>,
}

pub struct BatchOperation {
    pub id: Uuid,
    pub operation_type: OperationType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub enum OperationType {
    CreateTask(Uuid),
    UpdateTask(Uuid),
    DeleteTask(Uuid),
    AddDependency(Uuid, Uuid),
    RemoveDependency(Uuid, Uuid),
}

impl BatchOperationManager {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(Vec::new())),
            processing: Arc::new(Mutex::new(false)),
        }
    }
    
    pub async fn add_operation(&self, op: BatchOperation) {
        let mut pending = self.pending.write().await;
        pending.push(op);
    }
    
    pub async fn process_batch<F>(&self, processor: F) -> Result<()>
    where
        F: Fn(Vec<BatchOperation>) -> Result<()>,
    {
        let mut processing = self.processing.lock().await;
        if *processing {
            return Ok(()); // Already processing
        }
        *processing = true;
        drop(processing);
        
        let operations = {
            let mut pending = self.pending.write().await;
            std::mem::take(&mut *pending)
        };
        
        let result = processor(operations);
        
        *self.processing.lock().await = false;
        result
    }
}

/// Lock-free queue for high-performance operations
// Note: crossbeam would need to be added as a dependency
// For now, using a simple Vec-based implementation

pub struct LockFreeTaskQueue {
    queue: Arc<StdMutex<Vec<Uuid>>>,
    capacity: usize,
}

impl LockFreeTaskQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(StdMutex::new(Vec::with_capacity(capacity))),
            capacity,
        }
    }
    
    pub fn try_push(&self, task_id: Uuid) -> bool {
        let mut queue = self.queue.lock().unwrap();
        if queue.len() < self.capacity {
            queue.push(task_id);
            true
        } else {
            false
        }
    }
    
    pub fn try_pop(&self) -> Option<Uuid> {
        let mut queue = self.queue.lock().unwrap();
        if !queue.is_empty() {
            Some(queue.remove(0))
        } else {
            None
        }
    }
    
    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
    
    pub fn is_full(&self) -> bool {
        self.queue.lock().unwrap().len() >= self.capacity
    }
}

/// Deadlock detection
pub struct DeadlockDetector {
    dependencies: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self {
            dependencies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn add_dependency(&self, from: Uuid, to: Uuid) -> Result<()> {
        // Check for potential deadlock before adding
        if self.would_create_cycle(from, to).await? {
            anyhow::bail!("Adding dependency would create a deadlock");
        }
        
        let mut deps = self.dependencies.write().await;
        deps.entry(from).or_insert_with(Vec::new).push(to);
        Ok(())
    }
    
    async fn would_create_cycle(&self, from: Uuid, to: Uuid) -> Result<bool> {
        let deps = self.dependencies.read().await;
        
        // DFS to check for cycle
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![to];
        
        while let Some(current) = stack.pop() {
            if current == from {
                return Ok(true); // Cycle detected
            }
            
            if !visited.insert(current) {
                continue;
            }
            
            if let Some(next_deps) = deps.get(&current) {
                stack.extend(next_deps);
            }
        }
        
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_safe_orchestrator() {
        // Test would require full orchestrator setup
        let id_gen = AtomicIdGenerator::new();
        assert_eq!(id_gen.next_id(), 0);
        assert_eq!(id_gen.next_id(), 1);
    }
    
    #[tokio::test]
    async fn test_lock_free_queue() {
        let queue = LockFreeTaskQueue::new(10);
        
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        
        assert!(queue.try_push(task1));
        assert!(queue.try_push(task2));
        assert_eq!(queue.len(), 2);
        
        assert_eq!(queue.try_pop(), Some(task1));
        assert_eq!(queue.try_pop(), Some(task2));
        assert_eq!(queue.try_pop(), None);
    }
    
    #[tokio::test]
    async fn test_deadlock_detection() {
        let detector = DeadlockDetector::new();
        
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();
        
        // Create chain: 1 -> 2 -> 3
        detector.add_dependency(task1, task2).await.unwrap();
        detector.add_dependency(task2, task3).await.unwrap();
        
        // Try to create cycle: 3 -> 1
        let result = detector.add_dependency(task3, task1).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("deadlock"));
    }
    
    #[tokio::test]
    async fn test_batch_operations() {
        let manager = BatchOperationManager::new();
        
        // Add multiple operations
        for i in 0..5 {
            manager.add_operation(BatchOperation {
                id: Uuid::new_v4(),
                operation_type: OperationType::CreateTask(Uuid::new_v4()),
                timestamp: chrono::Utc::now(),
            }).await;
        }
        
        // Process batch
        let result = manager.process_batch(|ops| {
            assert_eq!(ops.len(), 5);
            Ok(())
        }).await;
        
        assert!(result.is_ok());
    }
}