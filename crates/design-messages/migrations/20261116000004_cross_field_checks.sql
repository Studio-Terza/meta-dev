CREATE TEMP TABLE compliance (
    runs INTEGER NOT NULL CHECK (runs = 0),
    senses INTEGER NOT NULL CHECK (senses = 0)
);

INSERT INTO compliance (runs, senses)
SELECT
  (
    SELECT count(*) FROM runs
    WHERE NOT (
      (input_tokens IS NULL AND output_tokens IS NULL AND cache_written_tokens IS NULL AND cache_read_tokens IS NULL)
      OR (input_tokens IS NOT NULL AND output_tokens IS NOT NULL AND cache_written_tokens IS NOT NULL AND cache_read_tokens IS NOT NULL)
    )
  ),
  (
    SELECT count(*) FROM senses AS later
    WHERE later.supersedes IS NOT NULL
      AND NOT EXISTS (
        SELECT 1 FROM senses AS earlier
        WHERE earlier.id = later.supersedes AND earlier.message = later.message AND earlier.ordinal = later.ordinal
      )
  );

DROP TABLE temp.compliance;

CREATE TRIGGER runs_usage_whole_on_insert BEFORE INSERT ON runs
WHEN NOT (
  (NEW.input_tokens IS NULL AND NEW.output_tokens IS NULL AND NEW.cache_written_tokens IS NULL AND NEW.cache_read_tokens IS NULL)
  OR (NEW.input_tokens IS NOT NULL AND NEW.output_tokens IS NOT NULL AND NEW.cache_written_tokens IS NOT NULL AND NEW.cache_read_tokens IS NOT NULL)
)
BEGIN
SELECT RAISE(ABORT, 'runs records a usage whole or not at all');
END;

CREATE TRIGGER runs_usage_whole_on_update BEFORE UPDATE ON runs
WHEN NOT (
  (NEW.input_tokens IS NULL AND NEW.output_tokens IS NULL AND NEW.cache_written_tokens IS NULL AND NEW.cache_read_tokens IS NULL)
  OR (NEW.input_tokens IS NOT NULL AND NEW.output_tokens IS NOT NULL AND NEW.cache_written_tokens IS NOT NULL AND NEW.cache_read_tokens IS NOT NULL)
)
BEGIN
SELECT RAISE(ABORT, 'runs records a usage whole or not at all');
END;

CREATE TRIGGER senses_supersede_their_segment_on_insert BEFORE INSERT ON senses
WHEN NEW.supersedes IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM senses AS earlier
    WHERE earlier.id = NEW.supersedes AND earlier.message = NEW.message AND earlier.ordinal = NEW.ordinal
  )
BEGIN
SELECT RAISE(ABORT, 'a sense supersedes only a sense of its own segment');
END;

CREATE TRIGGER senses_supersede_their_segment_on_update BEFORE UPDATE ON senses
WHEN NEW.supersedes IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM senses AS earlier
    WHERE earlier.id = NEW.supersedes AND earlier.message = NEW.message AND earlier.ordinal = NEW.ordinal
  )
BEGIN
SELECT RAISE(ABORT, 'a sense supersedes only a sense of its own segment');
END;
