CREATE TABLE messages (
    uuid TEXT PRIMARY KEY NOT NULL,
    session TEXT NOT NULL,
    at TEXT NOT NULL,
    author TEXT NOT NULL,
    body TEXT NOT NULL
) STRICT;

CREATE INDEX messages_by_session ON messages (session, author, at);

CREATE TABLE cursors (
    session TEXT PRIMARY KEY NOT NULL CHECK (session <> ''),
    read_to INTEGER NOT NULL CHECK (read_to >= 0)
) STRICT;

CREATE TABLE ended (
    task TEXT PRIMARY KEY NOT NULL CHECK (task <> '')
) STRICT;

CREATE TABLE overruns (
    task TEXT PRIMARY KEY NOT NULL CHECK (task <> '')
) STRICT;

CREATE TABLE failed_checks (
    pull INTEGER NOT NULL CHECK (pull > 0),
    name TEXT NOT NULL CHECK (name <> ''),
    PRIMARY KEY (pull, name)
) STRICT;
