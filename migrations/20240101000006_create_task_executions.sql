-- Task execution history table
CREATE TABLE IF NOT EXISTS task_executions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    status TEXT NOT NULL,
    branch_name TEXT NOT NULL,
    pr_url TEXT,
    error_message TEXT,
    output_log TEXT NOT NULL DEFAULT '[]',
    
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Index for querying by task
CREATE INDEX IF NOT EXISTS idx_task_executions_task_id ON task_executions(task_id);

-- Index for querying recent executions
CREATE INDEX IF NOT EXISTS idx_task_executions_started_at ON task_executions(started_at DESC);

-- Index for finding active executions
CREATE INDEX IF NOT EXISTS idx_task_executions_status ON task_executions(status) WHERE status IN ('Running', 'PendingReview');