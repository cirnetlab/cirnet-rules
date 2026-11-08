ALTER TABLE automation_definitions ADD COLUMN deleted_at INTEGER
    CHECK (deleted_at IS NULL OR (enabled = 0 AND next_at IS NULL));
