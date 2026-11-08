ALTER TABLE jobs ADD COLUMN state TEXT NOT NULL DEFAULT 'open';

CREATE INDEX jobs_by_state ON jobs (state);
