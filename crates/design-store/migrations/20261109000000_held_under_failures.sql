DROP TABLE retries;

INSERT INTO
  failures (job, halted, account, delivery)
SELECT DISTINCT
  runs.job,
  'flailed',
  'FLAILED: job ' || runs.job,
  'reported'
FROM
  flailed
  JOIN runs ON runs.id = flailed.run
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      failures
    WHERE
      failures.job = runs.job
  );

CREATE TABLE held (
    message TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    job INTEGER NOT NULL REFERENCES failures (job),
    run INTEGER NOT NULL REFERENCES runs (id),
    PRIMARY KEY (message, ordinal),
    FOREIGN KEY (message, ordinal) REFERENCES segments (message, ordinal)
) STRICT;

INSERT INTO
  held (message, ordinal, job, run)
SELECT
  flailed.message,
  flailed.ordinal,
  runs.job,
  flailed.run
FROM
  flailed
  JOIN runs ON runs.id = flailed.run;

DROP TABLE flailed;

ALTER TABLE held RENAME TO flailed;

CREATE INDEX flailed_by_job ON flailed (job);
