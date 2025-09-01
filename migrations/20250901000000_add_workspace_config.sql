-- Add workspace configuration fields to claude_code_config table
ALTER TABLE claude_code_config
ADD COLUMN workspace_root TEXT;

ALTER TABLE claude_code_config
ADD COLUMN git_clone_url TEXT;