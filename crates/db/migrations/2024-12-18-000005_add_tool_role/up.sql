-- Add 'tool' role to match rig-core message types
-- Roles: user, assistant, system, tool

-- Drop existing constraint
ALTER TABLE messages DROP CONSTRAINT IF EXISTS messages_role_check;

-- Add new constraint with tool role
ALTER TABLE messages ADD CONSTRAINT messages_role_check
    CHECK (role IN ('user', 'assistant', 'system', 'tool'));
