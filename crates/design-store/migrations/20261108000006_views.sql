CREATE VIEW ended_nodes (id, ending) AS
SELECT
  nodes.id,
  'dropped'
FROM
  nodes
WHERE
  nodes.stage = 'dropped'
UNION ALL
SELECT
  node_edges.target,
  'superseded'
FROM
  node_edges
WHERE
  node_edges.relation = 'supersedes';

CREATE VIEW ended_subtrees (id, ending) AS
SELECT
  ended_nodes.id,
  ended_nodes.ending
FROM
  ended_nodes
UNION ALL
SELECT
  closure.descendant,
  ended_nodes.ending
FROM
  closure
  JOIN ended_nodes ON ended_nodes.id = closure.ancestor
WHERE
  closure.relation = 'refines';

CREATE VIEW live_nodes (id) AS
SELECT
  nodes.id
FROM
  nodes
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      ended_subtrees
    WHERE
      ended_subtrees.id = nodes.id
      AND ended_subtrees.ending = 'dropped'
  );

CREATE VIEW current_nodes (id) AS
SELECT
  nodes.id
FROM
  nodes
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      ended_subtrees
    WHERE
      ended_subtrees.id = nodes.id
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
