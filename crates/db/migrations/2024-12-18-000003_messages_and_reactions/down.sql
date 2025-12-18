-- Restore messages JSONB column to conversations
ALTER TABLE conversations ADD COLUMN messages JSONB NOT NULL DEFAULT '[]';

-- Drop reaction and messages tables
DROP TABLE IF EXISTS message_reactions;
DROP TABLE IF EXISTS messages;
