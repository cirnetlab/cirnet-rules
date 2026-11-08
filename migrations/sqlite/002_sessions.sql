CREATE TABLE conversations (
    id INTEGER PRIMARY KEY,
    create_key TEXT NOT NULL UNIQUE,
    create_digest TEXT NOT NULL,
    title TEXT NOT NULL,
    profile TEXT NOT NULL CHECK (profile = 'handoff'),
    state TEXT NOT NULL CHECK (state IN ('ACTIVE', 'ARCHIVED')),
    source TEXT NOT NULL CHECK (source IN ('DASHBOARD', 'AUTOMATION')),
    provider_thread TEXT,
    revision INTEGER NOT NULL CHECK (revision > 0),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

CREATE TABLE conversation_bindings (
    conversation_id INTEGER NOT NULL REFERENCES conversations(id),
    binding_id TEXT NOT NULL REFERENCES bindings(id),
    binding_revision INTEGER NOT NULL,
    ordinal INTEGER NOT NULL,
    PRIMARY KEY (conversation_id, binding_id),
    UNIQUE (conversation_id, ordinal)
) STRICT;

CREATE TABLE messages (
    id INTEGER PRIMARY KEY,
    conversation_id INTEGER NOT NULL REFERENCES conversations(id),
    request_key TEXT NOT NULL,
    request_digest TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('USER')),
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE (conversation_id, request_key),
    UNIQUE (conversation_id, ordinal)
) STRICT;

CREATE TABLE task_proposals (
    id INTEGER PRIMARY KEY,
    conversation_id INTEGER NOT NULL REFERENCES conversations(id),
    message_id INTEGER NOT NULL UNIQUE REFERENCES messages(id),
    state TEXT NOT NULL CHECK (state IN ('WAITING_APPROVAL','CANCELLED')),
    mode TEXT NOT NULL CHECK (mode IN ('WORK','RELEASE')),
    risk TEXT NOT NULL CHECK (risk IN ('SCOPED','REVIEW_REQUIRED')),
    route_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE TABLE automation_definitions (
    id INTEGER PRIMARY KEY,
    create_key TEXT NOT NULL UNIQUE,
    create_digest TEXT NOT NULL,
    name TEXT NOT NULL,
    binding_id TEXT NOT NULL REFERENCES bindings(id),
    binding_revision INTEGER NOT NULL,
    profile TEXT NOT NULL CHECK (profile = 'handoff'),
    prompt TEXT NOT NULL,
    paths_json TEXT NOT NULL,
    template_id TEXT,
    template_revision INTEGER,
    mode TEXT NOT NULL CHECK (mode IN ('WORK','RELEASE')),
    risk TEXT NOT NULL CHECK (risk IN ('SCOPED','REVIEW_REQUIRED')),
    revision INTEGER NOT NULL CHECK (revision > 0),
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    trigger_json TEXT NOT NULL,
    next_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

CREATE INDEX automation_due ON automation_definitions(enabled, next_at, id);

CREATE TABLE automation_runs (
    id INTEGER PRIMARY KEY,
    automation_id INTEGER NOT NULL REFERENCES automation_definitions(id),
    definition_revision INTEGER NOT NULL,
    binding_id TEXT NOT NULL REFERENCES bindings(id),
    binding_revision INTEGER NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('MANUAL','SCHEDULE')),
    request_key TEXT,
    occurrence_at INTEGER,
    status TEXT NOT NULL CHECK (status IN ('QUEUED','CLAIMED','WAITING_APPROVAL','BLOCKED','CANCELLED','RECOVERY_REQUIRED')),
    snapshot_json TEXT NOT NULL,
    lease_token TEXT,
    lease_expires_at INTEGER,
    attempt INTEGER NOT NULL DEFAULT 0,
    conversation_id INTEGER REFERENCES conversations(id),
    task_id INTEGER UNIQUE REFERENCES task_proposals(id),
    outcome TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    CHECK ((kind = 'MANUAL' AND request_key IS NOT NULL AND occurrence_at IS NULL)
        OR (kind = 'SCHEDULE' AND request_key IS NULL AND occurrence_at IS NOT NULL)),
    UNIQUE (automation_id, request_key),
    UNIQUE (automation_id, occurrence_at)
) STRICT;

CREATE INDEX automation_queue ON automation_runs(status, id);
CREATE INDEX automation_lease ON automation_runs(status, lease_expires_at);
CREATE INDEX automation_history ON automation_runs(automation_id, id DESC);

CREATE TABLE session_events (
    sequence INTEGER PRIMARY KEY,
    conversation_id INTEGER REFERENCES conversations(id),
    task_id INTEGER REFERENCES task_proposals(id),
    run_id INTEGER REFERENCES automation_runs(id),
    attempt INTEGER,
    correlation TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    CHECK (conversation_id IS NOT NULL OR run_id IS NOT NULL)
) STRICT;

CREATE INDEX conversation_events ON session_events(conversation_id, sequence);
CREATE INDEX run_events ON session_events(run_id, sequence);
