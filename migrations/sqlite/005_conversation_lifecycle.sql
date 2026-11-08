CREATE TABLE session_id_sequences (
    name TEXT PRIMARY KEY,
    last_id INTEGER NOT NULL CHECK (last_id >= 0)
) STRICT;

INSERT INTO session_id_sequences
    SELECT 'conversations', COALESCE(MAX(id), 0) FROM conversations
    UNION ALL SELECT 'messages', COALESCE(MAX(id), 0) FROM messages
    UNION ALL SELECT 'task_proposals', COALESCE(MAX(id), 0) FROM task_proposals
    UNION ALL SELECT 'session_events', COALESCE(MAX(sequence), 0) FROM session_events;

CREATE TABLE session_message_ordinals (
    conversation_id INTEGER PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
    last_ordinal INTEGER NOT NULL CHECK (last_ordinal > 0)
) STRICT;

INSERT INTO session_message_ordinals
    SELECT conversation_id, MAX(ordinal) FROM messages GROUP BY conversation_id;

CREATE TABLE session_delete_previews (
    token TEXT PRIMARY KEY,
    conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    message_id INTEGER REFERENCES messages(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL,
    scope_digest TEXT NOT NULL,
    expires_at INTEGER NOT NULL
) STRICT;

CREATE INDEX session_delete_preview_expiry ON session_delete_previews(expires_at);

CREATE TABLE session_delete_receipts (
    key_digest TEXT PRIMARY KEY,
    conversation_id INTEGER NOT NULL,
    message_id INTEGER,
    revision INTEGER NOT NULL,
    token_digest TEXT NOT NULL,
    deleted_at INTEGER NOT NULL
) STRICT;

CREATE TABLE session_retired_conversation_keys (
    key_digest TEXT PRIMARY KEY
) STRICT;

CREATE TABLE session_retired_message_keys (
    conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    key_digest TEXT NOT NULL,
    PRIMARY KEY (conversation_id, key_digest)
) STRICT;

CREATE INDEX conversations_state_page ON conversations(state, id DESC);
CREATE INDEX task_proposals_conversation ON task_proposals(conversation_id, id);
CREATE INDEX automation_runs_conversation ON automation_runs(conversation_id, id);
CREATE INDEX session_events_task ON session_events(task_id, sequence);
