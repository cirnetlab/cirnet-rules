CREATE TABLE governance_assets (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    scope TEXT NOT NULL DEFAULT 'library',
    origin TEXT NOT NULL DEFAULT 'local',
    source_name TEXT,
    source_revision INTEGER,
    source_digest TEXT,
    revision INTEGER NOT NULL CHECK (revision > 0),
    active_revision INTEGER,
    draft_json TEXT,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    deleted_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (id, active_revision) REFERENCES governance_revisions(asset_id, revision)
) STRICT;

CREATE UNIQUE INDEX governance_live_slugs ON governance_assets(kind, slug) WHERE deleted_at IS NULL;

CREATE TABLE governance_revisions (
    asset_id TEXT NOT NULL REFERENCES governance_assets(id),
    revision INTEGER NOT NULL,
    digest TEXT NOT NULL,
    document_json TEXT NOT NULL,
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    canonical_bytes INTEGER NOT NULL CHECK (canonical_bytes >= 0),
    event TEXT NOT NULL CHECK (event IN ('PUBLISHED', 'ROLLBACK')),
    prior_revision INTEGER,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (asset_id, revision)
) STRICT;

CREATE TABLE governance_dependencies (
    asset_id TEXT NOT NULL REFERENCES governance_assets(id),
    target_id TEXT NOT NULL REFERENCES governance_assets(id),
    layer TEXT NOT NULL CHECK (layer IN ('draft', 'active')),
    PRIMARY KEY (asset_id, layer, target_id),
    CHECK (asset_id <> target_id)
) STRICT;

CREATE INDEX governance_dependents ON governance_dependencies(target_id, asset_id);

CREATE TABLE governance_assignments (
    binding_id TEXT NOT NULL REFERENCES bindings(id),
    asset_id TEXT NOT NULL REFERENCES governance_assets(id),
    created_at INTEGER NOT NULL,
    PRIMARY KEY (binding_id, asset_id)
) STRICT;

CREATE INDEX governance_assigned_assets ON governance_assignments(asset_id, binding_id);

CREATE TABLE governance_reviews (
    asset_id TEXT PRIMARY KEY REFERENCES governance_assets(id),
    expected_revision INTEGER NOT NULL,
    digest TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at INTEGER NOT NULL
) STRICT;

CREATE TABLE governance_events (
    id INTEGER PRIMARY KEY,
    asset_id TEXT NOT NULL REFERENCES governance_assets(id),
    asset_revision INTEGER NOT NULL,
    event TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX governance_asset_events ON governance_events(asset_id, id DESC);

CREATE TABLE governance_mutations (
    key TEXT PRIMARY KEY,
    fingerprint TEXT NOT NULL,
    asset_id TEXT NOT NULL REFERENCES governance_assets(id),
    result_revision INTEGER NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE TABLE governance_defaults (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    asset_id TEXT NOT NULL REFERENCES governance_assets(id)
) STRICT;

CREATE TABLE governance_session_snapshots (
    conversation_id INTEGER PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
    manifest_json TEXT NOT NULL
) STRICT;

CREATE TABLE governance_run_snapshots (
    run_id INTEGER PRIMARY KEY REFERENCES automation_runs(id) ON DELETE CASCADE,
    manifest_json TEXT NOT NULL
) STRICT;
