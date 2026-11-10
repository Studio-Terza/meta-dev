ALTER TABLE nodes ADD COLUMN stage_provenance TEXT NOT NULL DEFAULT 'init';

UPDATE nodes
SET
  stage_provenance = 'stage'
WHERE
  stage <> 'vague';

CREATE INDEX messages_by_session ON messages (session, author, at);

DROP VIEW current_spans;

DROP VIEW open_questions;

CREATE VIEW open_questions (node, relation, message, ordinal, provenance) AS
SELECT
  span_edges.node,
  span_edges.relation,
  span_edges.message,
  span_edges.ordinal,
  span_edges.provenance
FROM
  span_edges
  JOIN current_nodes ON current_nodes.id = span_edges.node
  JOIN messages AS asked ON asked.uuid = span_edges.message
  JOIN senses ON senses.message = span_edges.message
  AND senses.ordinal = span_edges.ordinal
WHERE
  span_edges.relation = 'questions'
  AND senses.kind = 'question'
  AND NOT EXISTS (
    SELECT
      1
    FROM
      senses AS later
    WHERE
      later.supersedes = senses.id
  )
  AND NOT EXISTS (
    SELECT
      1
    FROM
      segment_links
      JOIN messages AS answering ON answering.uuid = segment_links.source_message
    WHERE
      segment_links.relation IN ('answers', 'supersedes', 'corrects')
      AND answering.author = 'owner'
      AND segment_links.target_message = span_edges.message
      AND segment_links.target_ordinal = span_edges.ordinal
  )
  AND NOT EXISTS (
    SELECT
      1
    FROM
      current_decision
    WHERE
      current_decision.node = span_edges.node
      AND current_decision.message = (
        SELECT
          next.uuid
        FROM
          messages AS next
        WHERE
          next.session = asked.session
          AND next.author = 'owner'
          AND next.at > asked.at
        ORDER BY
          next.at
        LIMIT
          1
      )
  );

CREATE VIEW current_spans (node, relation, message, ordinal, provenance) AS
SELECT
  node,
  relation,
  message,
  ordinal,
  provenance
FROM
  current_decision
UNION ALL
SELECT
  node,
  relation,
  message,
  ordinal,
  provenance
FROM
  open_questions;

CREATE VIEW staged_nodes (id, basis) AS
SELECT
  nodes.id,
  CASE
    WHEN nodes.stage_provenance <> 'init' THEN 'set'
    WHEN EXISTS (
      SELECT
        1
      FROM
        current_decision
      WHERE
        current_decision.node = nodes.id
    )
    OR EXISTS (
      SELECT
        1
      FROM
        closure
        JOIN current_decision ON current_decision.node = closure.descendant
      WHERE
        closure.relation = 'refines'
        AND closure.ancestor = nodes.id
    ) THEN 'decided'
    ELSE 'vague'
  END
FROM
  nodes;
