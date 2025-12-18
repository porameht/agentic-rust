-- Recreate message_reactions table
CREATE TABLE message_reactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id VARCHAR(255),
    reaction_type VARCHAR(20) NOT NULL CHECK (reaction_type IN ('like', 'dislike')),
    feedback TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(message_id, user_id)
);

CREATE INDEX idx_message_reactions_message_id ON message_reactions(message_id);
CREATE INDEX idx_message_reactions_type ON message_reactions(reaction_type);

-- Migrate reactions back from messages
INSERT INTO message_reactions (message_id, user_id, reaction_type, feedback, created_at)
SELECT id, reaction_user_id, reaction_type, reaction_feedback, COALESCE(reacted_at, NOW())
FROM messages
WHERE reaction_type IS NOT NULL;

-- Remove reaction columns from messages
ALTER TABLE messages
    DROP COLUMN IF EXISTS reaction_type,
    DROP COLUMN IF EXISTS reaction_user_id,
    DROP COLUMN IF EXISTS reaction_feedback,
    DROP COLUMN IF EXISTS reacted_at;
