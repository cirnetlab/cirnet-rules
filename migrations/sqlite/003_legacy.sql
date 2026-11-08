CREATE TABLE legacy_imports (
    id TEXT PRIMARY KEY,
    source_path TEXT NOT NULL UNIQUE,
    manifest_sha256 TEXT NOT NULL,
    workspace_root TEXT,
    records INTEGER NOT NULL CHECK (records >= 0),
    imported_at INTEGER NOT NULL
) STRICT;

CREATE TABLE legacy_records (
    import_id TEXT NOT NULL REFERENCES legacy_imports(id) ON DELETE RESTRICT,
    relative_path TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('TASK', 'BINDING', 'LEASE')),
    legacy_id TEXT NOT NULL,
    legacy_revision INTEGER,
    state TEXT NOT NULL,
    source_sha256 TEXT NOT NULL,
    body TEXT NOT NULL,
    PRIMARY KEY (import_id, relative_path)
) STRICT;

CREATE TABLE legacy_binding_links (
    import_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    binding_id TEXT NOT NULL REFERENCES bindings(id) ON DELETE RESTRICT,
    binding_revision INTEGER NOT NULL,
    repo_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE RESTRICT,
    linked_at INTEGER NOT NULL,
    PRIMARY KEY (import_id, relative_path),
    FOREIGN KEY (import_id, relative_path) REFERENCES legacy_records(import_id, relative_path) ON DELETE RESTRICT
) STRICT;
