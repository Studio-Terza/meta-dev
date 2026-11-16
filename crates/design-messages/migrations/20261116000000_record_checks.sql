CREATE TABLE cursors_checked (
    session TEXT PRIMARY KEY NOT NULL CHECK (session <> ''),
    read_to INTEGER NOT NULL CHECK (read_to >= 0)
) STRICT;

INSERT INTO cursors_checked (session, read_to) SELECT session, read_to FROM cursors;

DROP TABLE cursors;

ALTER TABLE cursors_checked RENAME TO cursors;

CREATE TABLE applied_files_checked (
    id INTEGER PRIMARY KEY NOT NULL,
    content BLOB NOT NULL UNIQUE,
    applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) CHECK (applied_at <> ''),
    provenance TEXT NOT NULL CHECK (
        provenance IN ('init', 'apply', 'role', 'inheritance', 'stage', 'agent-0')
        OR (provenance GLOB 'agent-[1-9]*' AND substr(provenance, 7) NOT GLOB '*[^0-9]*')
        OR (provenance GLOB 'agent--[1-9]*' AND substr(provenance, 8) NOT GLOB '*[^0-9]*')
        OR (provenance GLOB 'import-[1-9]*' AND substr(provenance, 8) NOT GLOB '*[^0-9]*')
    )
) STRICT;

INSERT INTO applied_files_checked (id, content, applied_at, provenance)
SELECT id, content, applied_at, provenance FROM applied_files;

DROP TABLE applied_files;

ALTER TABLE applied_files_checked RENAME TO applied_files;
