CREATE TABLE events (
    uuid TEXT PRIMARY KEY NOT NULL CHECK (uuid <> ''),
    session TEXT NOT NULL CHECK (session <> ''),
    at TEXT NOT NULL CHECK (at <> ''),
    happening TEXT NOT NULL CHECK (happening IN ('notice', 'relayed', 'launch')),
    body TEXT NOT NULL CHECK (body <> '')
) STRICT;

CREATE INDEX events_by_session ON events (session, at);
