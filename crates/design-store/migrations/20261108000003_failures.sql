CREATE TABLE failures (
    job INTEGER PRIMARY KEY NOT NULL REFERENCES jobs (id),
    run INTEGER REFERENCES runs (id),
    halted TEXT NOT NULL,
    account TEXT NOT NULL,
    delivery TEXT NOT NULL
) STRICT;

CREATE INDEX failures_by_delivery ON failures (delivery);
