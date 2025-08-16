-- Create task_configurations table for storing task metadata configurations and state machines
CREATE TABLE IF NOT EXISTS task_configurations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    metadata_schema TEXT NOT NULL,
    state_machine TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Index for faster lookups by name
CREATE INDEX IF NOT EXISTS idx_task_configurations_name 
ON task_configurations(name);

-- configuration_id already exists in tasks table from initial schema
-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_tasks_configuration_id 
ON tasks(configuration_id);