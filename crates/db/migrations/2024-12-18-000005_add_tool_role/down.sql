-- Revert to original role constraint (without tool)
ALTER TABLE messages DROP CONSTRAINT IF EXISTS messages_role_check;

ALTER TABLE messages ADD CONSTRAINT messages_role_check
    CHECK (role IN ('user', 'assistant', 'system'));
