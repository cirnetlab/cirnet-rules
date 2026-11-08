CREATE TABLE catalog_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

INSERT INTO catalog_settings (key, value)
VALUES ('machine_id', 'machine-' || lower(hex(randomblob(16))));

CREATE TABLE repositories (
    id TEXT PRIMARY KEY,
    common_identity TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL CHECK (length(display_name) BETWEEN 1 AND 120),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

CREATE TABLE bindings (
    id TEXT PRIMARY KEY,
    repo_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE RESTRICT,
    machine_id TEXT NOT NULL,
    selected_path TEXT NOT NULL,
    root TEXT NOT NULL,
    git_dir TEXT NOT NULL,
    common_dir TEXT NOT NULL,
    identity TEXT NOT NULL,
    anchor TEXT NOT NULL,
    branch TEXT,
    file_hint TEXT,
    linked_worktree INTEGER NOT NULL CHECK (linked_worktree IN (0, 1)),
    status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'MISSING', 'INACCESSIBLE', 'REBIND_REQUIRED')),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
    verified_at INTEGER NOT NULL,
    selected_at INTEGER,
    UNIQUE (machine_id, identity),
    UNIQUE (machine_id, root)
) STRICT;

CREATE INDEX bindings_repository ON bindings(repo_id, id);
CREATE INDEX bindings_recent ON bindings(machine_id, selected_at DESC, id);
