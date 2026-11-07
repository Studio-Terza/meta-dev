CREATE TABLE jobs (
    id INTEGER PRIMARY KEY NOT NULL,
    state TEXT NOT NULL
) STRICT;

CREATE TABLE job_segments (
    job INTEGER NOT NULL REFERENCES jobs (id),
    message TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    PRIMARY KEY (job, message, ordinal),
    FOREIGN KEY (message, ordinal) REFERENCES segments (message, ordinal)
) STRICT;

CREATE TABLE runs (
    id INTEGER PRIMARY KEY NOT NULL,
    job INTEGER NOT NULL REFERENCES jobs (id),
    pid INTEGER NOT NULL,
    started TEXT,
    deadline INTEGER NOT NULL,
    state TEXT NOT NULL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cache_written_tokens INTEGER,
    cache_read_tokens INTEGER
) STRICT;

CREATE INDEX runs_by_state ON runs (state);
