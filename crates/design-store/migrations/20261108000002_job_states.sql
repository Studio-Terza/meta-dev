ALTER TABLE jobs ADD COLUMN state TEXT NOT NULL DEFAULT 'open';

UPDATE jobs
SET
  state = CASE
    WHEN EXISTS (
      SELECT
        1
      FROM
        runs
      WHERE
        runs.job = jobs.id
        AND runs.state = 'replied'
    ) THEN 'applied'
    ELSE 'interrupted'
  END;

CREATE INDEX jobs_by_state ON jobs (state);
