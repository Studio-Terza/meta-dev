DROP TABLE events;

DROP TABLE stops;

CREATE TABLE stops (
    id INTEGER PRIMARY KEY,
    session TEXT NOT NULL CHECK (session <> ''),
    at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) CHECK (at <> ''),
    shells INTEGER NOT NULL CHECK (shells >= 0),
    agents INTEGER NOT NULL CHECK (agents >= 0),
    halt TEXT NOT NULL CHECK (halt IN ('live', 'stalled', 'overrun')),
    read INTEGER NOT NULL CHECK (read >= 0),
    quoted INTEGER NOT NULL CHECK (quoted >= 0),
    kept INTEGER NOT NULL CHECK (kept >= 0),
    fate TEXT NOT NULL CHECK (fate IN ('blocked', 'passed')),
    tokens INTEGER NOT NULL CHECK (tokens >= 0),
    millis INTEGER NOT NULL CHECK (millis >= 0)
) STRICT;

CREATE TABLE overruns (
    task TEXT PRIMARY KEY NOT NULL CHECK (task <> ''),
    at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) CHECK (at <> '')
) STRICT;
