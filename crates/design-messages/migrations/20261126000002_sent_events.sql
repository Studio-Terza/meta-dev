CREATE TABLE events_widened (
    uuid TEXT PRIMARY KEY NOT NULL CHECK (uuid <> ''),
    session TEXT NOT NULL CHECK (session <> ''),
    at TEXT NOT NULL CHECK (at <> ''),
    happening TEXT NOT NULL CHECK (happening IN ('notice', 'relayed', 'launch', 'sent')),
    body TEXT NOT NULL CHECK (body <> '')
) STRICT;

INSERT INTO events_widened (uuid, session, at, happening, body)
SELECT uuid, session, at, happening, body FROM events;

DROP TABLE events;

ALTER TABLE events_widened RENAME TO events;

CREATE INDEX events_by_session ON events (session, at);
