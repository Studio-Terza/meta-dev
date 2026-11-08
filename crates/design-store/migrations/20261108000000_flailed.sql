CREATE TABLE flailed (
    message TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    run INTEGER NOT NULL REFERENCES runs (id),
    PRIMARY KEY (message, ordinal),
    FOREIGN KEY (message, ordinal) REFERENCES segments (message, ordinal)
) STRICT;
