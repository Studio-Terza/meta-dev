CREATE TABLE applied_files (
    id INTEGER PRIMARY KEY NOT NULL,
    content BLOB NOT NULL UNIQUE,
    applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    provenance TEXT NOT NULL
) STRICT;
