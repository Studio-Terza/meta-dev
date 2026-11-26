CREATE TABLE stops (
    id INTEGER PRIMARY KEY,
    session TEXT NOT NULL CHECK (session <> ''),
    at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) CHECK (at <> ''),
    gist TEXT NOT NULL CHECK (gist IN ('question', 'statement')),
    read INTEGER NOT NULL CHECK (read >= 0),
    quoted INTEGER NOT NULL CHECK (quoted >= 0),
    kept INTEGER NOT NULL CHECK (kept >= 0),
    awaited TEXT NOT NULL CHECK (awaited IN ('owner', 'shell', 'agent', 'nothing', 'unclassified', 'unchecked')),
    halt TEXT NOT NULL CHECK (halt IN ('live', 'deadlock', 'overrun')),
    fate TEXT NOT NULL CHECK (fate IN ('blocked', 'passed'))
) STRICT;
