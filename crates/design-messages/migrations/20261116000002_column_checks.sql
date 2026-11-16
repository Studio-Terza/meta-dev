CREATE TEMP TABLE compliance (
    nodes INTEGER NOT NULL CHECK (nodes = 0),
    node_edges INTEGER NOT NULL CHECK (node_edges = 0),
    span_edges INTEGER NOT NULL CHECK (span_edges = 0),
    segment_links INTEGER NOT NULL CHECK (segment_links = 0),
    senses INTEGER NOT NULL CHECK (senses = 0),
    segments INTEGER NOT NULL CHECK (segments = 0),
    conversational INTEGER NOT NULL CHECK (conversational = 0),
    flailed INTEGER NOT NULL CHECK (flailed = 0),
    jobs INTEGER NOT NULL CHECK (jobs = 0),
    runs INTEGER NOT NULL CHECK (runs = 0),
    failures INTEGER NOT NULL CHECK (failures = 0)
);

INSERT INTO compliance (nodes, node_edges, span_edges, segment_links, senses, segments, conversational, flailed, jobs, runs, failures)
SELECT
  (SELECT count(*) FROM nodes WHERE NOT ((facet IN ('project', 'feature', 'smell')) AND (stage IN ('vague', 'refining', 'terminal', 'dropped')) AND (role IN ('core', 'secondary')))),
  (SELECT count(*) FROM node_edges WHERE NOT ((relation IN ('refines', 'supersedes', 'consumes')))),
  (SELECT count(*) FROM span_edges WHERE NOT ((relation IN ('defines', 'decides', 'questions', 'revises', 'mentions')) AND (ordinal BETWEEN 0 AND 4294967295))),
  (SELECT count(*) FROM segment_links WHERE NOT ((relation IN ('answers', 'refers-to', 'supersedes', 'corrects')) AND (source_ordinal BETWEEN 0 AND 4294967295) AND (target_ordinal BETWEEN 0 AND 4294967295))),
  (SELECT count(*) FROM senses WHERE NOT ((kind IN ('decision', 'opinion', 'question', 'ask', 'instruction', 'control', 'remark', 'smell')) AND (specificity IN ('general', 'specific', 'exact')) AND (confidence IN ('tentative', 'leaning', 'firm')) AND (explicitness IN ('explicit', 'implicit')) AND (ordinal BETWEEN 0 AND 4294967295))),
  (SELECT count(*) FROM segments WHERE NOT ((ordinal BETWEEN 0 AND 4294967295))),
  (SELECT count(*) FROM conversational WHERE NOT ((ordinal BETWEEN 0 AND 4294967295))),
  (SELECT count(*) FROM flailed WHERE NOT ((ordinal BETWEEN 0 AND 4294967295))),
  (SELECT count(*) FROM jobs WHERE NOT ((state IN ('open', 'applied', 'repeated', 'flailed', 'stalled', 'interrupted')))),
  (SELECT count(*) FROM runs WHERE NOT ((state IN ('running', 'replied', 'unreadable', 'errored', 'overdue', 'killed', 'gone', 'failed')) AND (pid BETWEEN 0 AND 4294967295) AND (input_tokens IS NULL OR (input_tokens BETWEEN 0 AND 4294967295)) AND (output_tokens IS NULL OR (output_tokens BETWEEN 0 AND 4294967295)) AND (cache_written_tokens IS NULL OR (cache_written_tokens BETWEEN 0 AND 4294967295)) AND (cache_read_tokens IS NULL OR (cache_read_tokens BETWEEN 0 AND 4294967295)))),
  (SELECT count(*) FROM failures WHERE NOT ((halted IN ('flailed', 'stalled')) AND (delivery IN ('unreported', 'reported', 'withdrawn'))));

DROP TABLE temp.compliance;

CREATE TRIGGER nodes_checked_on_insert BEFORE INSERT ON nodes
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.facet AS facet, NEW.stage AS stage, NEW.role AS role) WHERE (facet IN ('project', 'feature', 'smell')) AND (stage IN ('vague', 'refining', 'terminal', 'dropped')) AND (role IN ('core', 'secondary'))
)
BEGIN
SELECT RAISE(ABORT, 'nodes refuses a value its column type refuses');
END;

CREATE TRIGGER nodes_checked_on_update BEFORE UPDATE ON nodes
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.facet AS facet, NEW.stage AS stage, NEW.role AS role) WHERE (facet IN ('project', 'feature', 'smell')) AND (stage IN ('vague', 'refining', 'terminal', 'dropped')) AND (role IN ('core', 'secondary'))
)
BEGIN
SELECT RAISE(ABORT, 'nodes refuses a value its column type refuses');
END;

CREATE TRIGGER node_edges_checked_on_insert BEFORE INSERT ON node_edges
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.relation AS relation) WHERE (relation IN ('refines', 'supersedes', 'consumes'))
)
BEGIN
SELECT RAISE(ABORT, 'node_edges refuses a value its column type refuses');
END;

CREATE TRIGGER node_edges_checked_on_update BEFORE UPDATE ON node_edges
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.relation AS relation) WHERE (relation IN ('refines', 'supersedes', 'consumes'))
)
BEGIN
SELECT RAISE(ABORT, 'node_edges refuses a value its column type refuses');
END;

CREATE TRIGGER span_edges_checked_on_insert BEFORE INSERT ON span_edges
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.relation AS relation, NEW.ordinal AS ordinal) WHERE (relation IN ('defines', 'decides', 'questions', 'revises', 'mentions')) AND (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'span_edges refuses a value its column type refuses');
END;

CREATE TRIGGER span_edges_checked_on_update BEFORE UPDATE ON span_edges
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.relation AS relation, NEW.ordinal AS ordinal) WHERE (relation IN ('defines', 'decides', 'questions', 'revises', 'mentions')) AND (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'span_edges refuses a value its column type refuses');
END;

CREATE TRIGGER segment_links_checked_on_insert BEFORE INSERT ON segment_links
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.relation AS relation, NEW.source_ordinal AS source_ordinal, NEW.target_ordinal AS target_ordinal) WHERE (relation IN ('answers', 'refers-to', 'supersedes', 'corrects')) AND (source_ordinal BETWEEN 0 AND 4294967295) AND (target_ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'segment_links refuses a value its column type refuses');
END;

CREATE TRIGGER segment_links_checked_on_update BEFORE UPDATE ON segment_links
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.relation AS relation, NEW.source_ordinal AS source_ordinal, NEW.target_ordinal AS target_ordinal) WHERE (relation IN ('answers', 'refers-to', 'supersedes', 'corrects')) AND (source_ordinal BETWEEN 0 AND 4294967295) AND (target_ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'segment_links refuses a value its column type refuses');
END;

CREATE TRIGGER senses_checked_on_insert BEFORE INSERT ON senses
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.kind AS kind, NEW.specificity AS specificity, NEW.confidence AS confidence, NEW.explicitness AS explicitness, NEW.ordinal AS ordinal) WHERE (kind IN ('decision', 'opinion', 'question', 'ask', 'instruction', 'control', 'remark', 'smell')) AND (specificity IN ('general', 'specific', 'exact')) AND (confidence IN ('tentative', 'leaning', 'firm')) AND (explicitness IN ('explicit', 'implicit')) AND (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'senses refuses a value its column type refuses');
END;

CREATE TRIGGER senses_checked_on_update BEFORE UPDATE ON senses
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.kind AS kind, NEW.specificity AS specificity, NEW.confidence AS confidence, NEW.explicitness AS explicitness, NEW.ordinal AS ordinal) WHERE (kind IN ('decision', 'opinion', 'question', 'ask', 'instruction', 'control', 'remark', 'smell')) AND (specificity IN ('general', 'specific', 'exact')) AND (confidence IN ('tentative', 'leaning', 'firm')) AND (explicitness IN ('explicit', 'implicit')) AND (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'senses refuses a value its column type refuses');
END;

CREATE TRIGGER segments_checked_on_insert BEFORE INSERT ON segments
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.ordinal AS ordinal) WHERE (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'segments refuses a value its column type refuses');
END;

CREATE TRIGGER segments_checked_on_update BEFORE UPDATE ON segments
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.ordinal AS ordinal) WHERE (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'segments refuses a value its column type refuses');
END;

CREATE TRIGGER conversational_checked_on_insert BEFORE INSERT ON conversational
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.ordinal AS ordinal) WHERE (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'conversational refuses a value its column type refuses');
END;

CREATE TRIGGER conversational_checked_on_update BEFORE UPDATE ON conversational
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.ordinal AS ordinal) WHERE (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'conversational refuses a value its column type refuses');
END;

CREATE TRIGGER flailed_checked_on_insert BEFORE INSERT ON flailed
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.ordinal AS ordinal) WHERE (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'flailed refuses a value its column type refuses');
END;

CREATE TRIGGER flailed_checked_on_update BEFORE UPDATE ON flailed
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.ordinal AS ordinal) WHERE (ordinal BETWEEN 0 AND 4294967295)
)
BEGIN
SELECT RAISE(ABORT, 'flailed refuses a value its column type refuses');
END;

CREATE TRIGGER jobs_checked_on_insert BEFORE INSERT ON jobs
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.state AS state) WHERE (state IN ('open', 'applied', 'repeated', 'flailed', 'stalled', 'interrupted'))
)
BEGIN
SELECT RAISE(ABORT, 'jobs refuses a value its column type refuses');
END;

CREATE TRIGGER jobs_checked_on_update BEFORE UPDATE ON jobs
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.state AS state) WHERE (state IN ('open', 'applied', 'repeated', 'flailed', 'stalled', 'interrupted'))
)
BEGIN
SELECT RAISE(ABORT, 'jobs refuses a value its column type refuses');
END;

CREATE TRIGGER runs_checked_on_insert BEFORE INSERT ON runs
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.state AS state, NEW.pid AS pid, NEW.input_tokens AS input_tokens, NEW.output_tokens AS output_tokens, NEW.cache_written_tokens AS cache_written_tokens, NEW.cache_read_tokens AS cache_read_tokens) WHERE (state IN ('running', 'replied', 'unreadable', 'errored', 'overdue', 'killed', 'gone', 'failed')) AND (pid BETWEEN 0 AND 4294967295) AND (input_tokens IS NULL OR (input_tokens BETWEEN 0 AND 4294967295)) AND (output_tokens IS NULL OR (output_tokens BETWEEN 0 AND 4294967295)) AND (cache_written_tokens IS NULL OR (cache_written_tokens BETWEEN 0 AND 4294967295)) AND (cache_read_tokens IS NULL OR (cache_read_tokens BETWEEN 0 AND 4294967295))
)
BEGIN
SELECT RAISE(ABORT, 'runs refuses a value its column type refuses');
END;

CREATE TRIGGER runs_checked_on_update BEFORE UPDATE ON runs
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.state AS state, NEW.pid AS pid, NEW.input_tokens AS input_tokens, NEW.output_tokens AS output_tokens, NEW.cache_written_tokens AS cache_written_tokens, NEW.cache_read_tokens AS cache_read_tokens) WHERE (state IN ('running', 'replied', 'unreadable', 'errored', 'overdue', 'killed', 'gone', 'failed')) AND (pid BETWEEN 0 AND 4294967295) AND (input_tokens IS NULL OR (input_tokens BETWEEN 0 AND 4294967295)) AND (output_tokens IS NULL OR (output_tokens BETWEEN 0 AND 4294967295)) AND (cache_written_tokens IS NULL OR (cache_written_tokens BETWEEN 0 AND 4294967295)) AND (cache_read_tokens IS NULL OR (cache_read_tokens BETWEEN 0 AND 4294967295))
)
BEGIN
SELECT RAISE(ABORT, 'runs refuses a value its column type refuses');
END;

CREATE TRIGGER failures_checked_on_insert BEFORE INSERT ON failures
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.halted AS halted, NEW.delivery AS delivery) WHERE (halted IN ('flailed', 'stalled')) AND (delivery IN ('unreported', 'reported', 'withdrawn'))
)
BEGIN
SELECT RAISE(ABORT, 'failures refuses a value its column type refuses');
END;

CREATE TRIGGER failures_checked_on_update BEFORE UPDATE ON failures
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.halted AS halted, NEW.delivery AS delivery) WHERE (halted IN ('flailed', 'stalled')) AND (delivery IN ('unreported', 'reported', 'withdrawn'))
)
BEGIN
SELECT RAISE(ABORT, 'failures refuses a value its column type refuses');
END;
