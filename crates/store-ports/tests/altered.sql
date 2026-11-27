CREATE TABLE cursors (
    session TEXT PRIMARY KEY NOT NULL CHECK (session <> ''),
    read_to INTEGER NOT NULL CHECK (read_to >= 0)
) STRICT;

CREATE TABLE altered (
    kept INTEGER NOT NULL
) STRICT;
