CREATE TABLE desktop_preferences (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    revision INTEGER NOT NULL CHECK (revision BETWEEN 1 AND 2147483647),
    settings TEXT NOT NULL CHECK (length(settings) <= 2048 AND json_valid(settings))
) STRICT;
