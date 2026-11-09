DROP TABLE retries;

CREATE TABLE retries (
    job INTEGER NOT NULL REFERENCES jobs (id),
    message TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    PRIMARY KEY (job, message, ordinal),
    FOREIGN KEY (message, ordinal) REFERENCES segments (message, ordinal)
) STRICT;
