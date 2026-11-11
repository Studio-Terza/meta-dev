CREATE TABLE edges (
    session TEXT NOT NULL,
    uuid TEXT NOT NULL,
    parent TEXT,
    PRIMARY KEY (session, uuid)
) STRICT;
