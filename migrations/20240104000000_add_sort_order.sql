-- Add sort_order column for maintaining card order within Kanban columns
ALTER TABLE tasks ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;

-- Create index for efficient sorting queries
CREATE INDEX IF NOT EXISTS idx_tasks_status_sort_order ON tasks(status, sort_order);

-- Update existing tasks to have sequential sort_order within each status
-- This ensures existing tasks maintain a consistent order
UPDATE tasks 
SET sort_order = (
    SELECT COUNT(*) 
    FROM tasks t2 
    WHERE t2.status = tasks.status 
    AND t2.created_at <= tasks.created_at
) * 100
WHERE sort_order = 0;