CREATE TABLE failed_checks (
    pull INTEGER NOT NULL CHECK (pull > 0),
    name TEXT NOT NULL CHECK (name <> ''),
    PRIMARY KEY (pull, name)
) STRICT;
