CREATE TABLE glosses (
    message TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    PRIMARY KEY (message, ordinal),
    FOREIGN KEY (message, ordinal) REFERENCES segments (message, ordinal)
) STRICT;

INSERT INTO glosses (message, ordinal) VALUES ('o1', 0);
