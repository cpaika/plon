-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    metadata TEXT NOT NULL, -- JSON
    tags TEXT NOT NULL, -- JSON array
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    due_date TEXT,
    scheduled_date TEXT,
    completed_at TEXT,
    estimated_hours REAL,
    actual_hours REAL,
    assigned_resource_id TEXT,
    goal_id TEXT,
    parent_task_id TEXT,
    position_x REAL NOT NULL,
    position_y REAL NOT NULL,
    FOREIGN KEY (assigned_resource_id) REFERENCES resources(id),
    FOREIGN KEY (goal_id) REFERENCES goals(id),
    FOREIGN KEY (parent_task_id) REFERENCES tasks(id)
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_assigned_resource ON tasks(assigned_resource_id);
CREATE INDEX idx_tasks_goal ON tasks(goal_id);
CREATE INDEX idx_tasks_parent ON tasks(parent_task_id);
CREATE INDEX idx_tasks_scheduled_date ON tasks(scheduled_date);
CREATE INDEX idx_tasks_due_date ON tasks(due_date);

-- Subtasks table
CREATE TABLE IF NOT EXISTS subtasks (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    description TEXT NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_subtasks_task ON subtasks(task_id);

-- Goals table
CREATE TABLE IF NOT EXISTS goals (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    target_date TEXT,
    completed_at TEXT,
    parent_goal_id TEXT,
    estimated_hours REAL,
    position_x REAL NOT NULL,
    position_y REAL NOT NULL,
    position_width REAL NOT NULL,
    position_height REAL NOT NULL,
    color TEXT NOT NULL,
    FOREIGN KEY (parent_goal_id) REFERENCES goals(id)
);

CREATE INDEX idx_goals_status ON goals(status);
CREATE INDEX idx_goals_parent ON goals(parent_goal_id);

-- Goal-Task relationship table
CREATE TABLE IF NOT EXISTS goal_tasks (
    goal_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    PRIMARY KEY (goal_id, task_id),
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_goal_tasks_goal ON goal_tasks(goal_id);
CREATE INDEX idx_goal_tasks_task ON goal_tasks(task_id);

-- Resources table
CREATE TABLE IF NOT EXISTS resources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    role TEXT NOT NULL,
    skills TEXT NOT NULL, -- JSON array
    metadata_filters TEXT NOT NULL, -- JSON
    weekly_hours REAL NOT NULL,
    current_load REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_resources_role ON resources(role);

-- Resource availability table
CREATE TABLE IF NOT EXISTS resource_availability (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id TEXT NOT NULL,
    date TEXT NOT NULL,
    hours_available REAL NOT NULL,
    UNIQUE(resource_id, date),
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE
);

CREATE INDEX idx_resource_availability_resource ON resource_availability(resource_id);
CREATE INDEX idx_resource_availability_date ON resource_availability(date);

-- Resource allocations table
CREATE TABLE IF NOT EXISTS resource_allocations (
    id TEXT PRIMARY KEY,
    resource_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    hours_allocated REAL NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_allocations_resource ON resource_allocations(resource_id);
CREATE INDEX idx_allocations_task ON resource_allocations(task_id);
CREATE INDEX idx_allocations_dates ON resource_allocations(start_date, end_date);

-- Comments table
CREATE TABLE IF NOT EXISTS comments (
    id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    author_id TEXT,
    author_name TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    edited INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_comments_entity ON comments(entity_id, entity_type);
CREATE INDEX idx_comments_created ON comments(created_at);

-- Attachments table
CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    comment_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    url TEXT NOT NULL,
    FOREIGN KEY (comment_id) REFERENCES comments(id) ON DELETE CASCADE
);

CREATE INDEX idx_attachments_comment ON attachments(comment_id);

-- Dependencies table
CREATE TABLE IF NOT EXISTS dependencies (
    id TEXT PRIMARY KEY,
    from_task_id TEXT NOT NULL,
    to_task_id TEXT NOT NULL,
    dependency_type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(from_task_id, to_task_id),
    FOREIGN KEY (from_task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (to_task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_dependencies_from ON dependencies(from_task_id);
CREATE INDEX idx_dependencies_to ON dependencies(to_task_id);

-- Recurring task templates table
CREATE TABLE IF NOT EXISTS recurring_templates (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    priority TEXT NOT NULL,
    metadata TEXT NOT NULL, -- JSON
    assigned_resource_id TEXT,
    estimated_hours REAL,
    recurrence_pattern TEXT NOT NULL,
    recurrence_interval INTEGER NOT NULL,
    days_of_week TEXT, -- JSON array
    day_of_month INTEGER,
    month_of_year INTEGER,
    time_of_day TEXT NOT NULL,
    end_date TEXT,
    max_occurrences INTEGER,
    occurrences_count INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_generated TEXT,
    next_occurrence TEXT,
    FOREIGN KEY (assigned_resource_id) REFERENCES resources(id)
);

CREATE INDEX idx_recurring_active ON recurring_templates(active);
CREATE INDEX idx_recurring_next ON recurring_templates(next_occurrence);

-- Metadata schema table
CREATE TABLE IF NOT EXISTS metadata_schema (
    name TEXT PRIMARY KEY,
    field_type TEXT NOT NULL,
    required INTEGER NOT NULL DEFAULT 0,
    options TEXT, -- JSON array
    default_value TEXT
);

-- Spatial index for map view (using R-tree)
-- Note: R-tree virtual tables commented out for now as they may not be available in all SQLite builds
-- CREATE VIRTUAL TABLE IF NOT EXISTS tasks_spatial USING rtree(
--     id,
--     min_x, max_x,
--     min_y, max_y
-- );
-- 
-- CREATE VIRTUAL TABLE IF NOT EXISTS goals_spatial USING rtree(
--     id,
--     min_x, max_x,
--     min_y, max_y
-- );

-- Create regular tables for spatial indexing instead
CREATE TABLE IF NOT EXISTS tasks_spatial (
    id INTEGER PRIMARY KEY,
    min_x REAL,
    max_x REAL,
    min_y REAL,
    max_y REAL
);

CREATE TABLE IF NOT EXISTS goals_spatial (
    id INTEGER PRIMARY KEY,
    min_x REAL,
    max_x REAL,
    min_y REAL,
    max_y REAL
);