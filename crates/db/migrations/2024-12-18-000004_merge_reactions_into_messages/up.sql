-- Merge message_reactions into messages table

ALTER TABLE messages
    ADD COLUMN reaction_type VARCHAR(20) CHECK (reaction_type IS NULL OR reaction_type IN ('like', 'dislike')),
    ADD COLUMN reaction_user_id VARCHAR(255),
    ADD COLUMN reaction_feedback TEXT,
    ADD COLUMN reacted_at TIMESTAMPTZ;

-- Migrate existing reactions (if any)
UPDATE messages m
SET
    reaction_type = mr.reaction_type,
    reaction_user_id = mr.user_id,
    reaction_feedback = mr.feedback,
    reacted_at = mr.created_at
FROM message_reactions mr
WHERE m.id = mr.message_id;

-- Drop message_reactions table
DROP TABLE IF EXISTS message_reactions;
