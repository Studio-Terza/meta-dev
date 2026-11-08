CREATE VIEW live_nodes (id) AS
SELECT
  nodes.id
FROM
  nodes
WHERE
  nodes.stage != 'dropped'
  AND NOT EXISTS (
    SELECT
      1
    FROM
      closure
      JOIN nodes AS above ON above.id = closure.ancestor
    WHERE
      closure.relation = 'refines'
      AND closure.descendant = nodes.id
      AND above.stage = 'dropped'
  );

CREATE VIEW current_nodes (id) AS
SELECT
  live_nodes.id
FROM
  live_nodes
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      node_edges
    WHERE
      node_edges.relation = 'supersedes'
      AND node_edges.target = live_nodes.id
  );

CREATE VIEW current_decision (node, relation, message, ordinal, provenance) AS
SELECT
  span_edges.node,
  span_edges.relation,
  span_edges.message,
  span_edges.ordinal,
  span_edges.provenance
FROM
  span_edges
  JOIN current_nodes ON current_nodes.id = span_edges.node
  JOIN messages ON messages.uuid = span_edges.message
  JOIN senses ON senses.message = span_edges.message
  AND senses.ordinal = span_edges.ordinal
WHERE
  span_edges.relation = 'decides'
  AND messages.author = 'owner'
  AND senses.explicitness = 'explicit'
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
    WHERE
      segment_links.relation IN ('supersedes', 'corrects')
      AND segment_links.target_message = span_edges.message
      AND segment_links.target_ordinal = span_edges.ordinal
  );

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
WHERE
  span_edges.relation = 'questions'
  AND NOT EXISTS (
    SELECT
      1
    FROM
      segment_links
    WHERE
      segment_links.relation IN ('answers', 'supersedes', 'corrects')
      AND segment_links.target_message = span_edges.message
      AND segment_links.target_ordinal = span_edges.ordinal
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

CREATE VIEW unplaced (message, ordinal) AS
SELECT
  segments.message,
  segments.ordinal
FROM
  segments
  JOIN messages ON messages.uuid = segments.message
WHERE
  messages.author = 'owner'
  AND NOT EXISTS (
    SELECT
      1
    FROM
      structured
    WHERE
      structured.message = segments.message
      AND structured.ordinal = segments.ordinal
  );
