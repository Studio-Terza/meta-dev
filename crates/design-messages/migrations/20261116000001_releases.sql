CREATE TABLE releases (
    job INTEGER PRIMARY KEY NOT NULL REFERENCES failures (job),
    released_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) CHECK (released_at <> '')
) STRICT;

CREATE TABLE successions (
    released INTEGER PRIMARY KEY NOT NULL REFERENCES releases (job),
    successor INTEGER NOT NULL REFERENCES jobs (id)
) STRICT;

CREATE TABLE flailed_keyed (
    message TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    job INTEGER NOT NULL REFERENCES failures (job),
    run INTEGER NOT NULL REFERENCES runs (id),
    PRIMARY KEY (message, ordinal, job),
    FOREIGN KEY (message, ordinal) REFERENCES segments (message, ordinal)
) STRICT;

INSERT INTO flailed_keyed (message, ordinal, job, run) SELECT message, ordinal, job, run FROM flailed;

DROP TABLE flailed;

ALTER TABLE flailed_keyed RENAME TO flailed;

CREATE INDEX flailed_by_job ON flailed (job);

CREATE VIEW held (message, ordinal, job, run) AS
SELECT
  flailed.message,
  flailed.ordinal,
  flailed.job,
  flailed.run
FROM
  flailed
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      releases
    WHERE
      releases.job = flailed.job
  );
