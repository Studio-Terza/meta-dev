CREATE TABLE acyclic_closure (
    relation TEXT NOT NULL,
    ancestor INTEGER NOT NULL,
    descendant INTEGER NOT NULL,
    depth INTEGER NOT NULL,
    PRIMARY KEY (relation, ancestor, descendant)
) STRICT;

INSERT INTO acyclic_closure (relation, ancestor, descendant, depth)
SELECT 'refines', ancestor, descendant, depth FROM closure WHERE depth > 0;

INSERT INTO acyclic_closure (relation, ancestor, descendant, depth)
WITH RECURSIVE walk (ancestor, descendant, depth) AS (
    SELECT target, source, 1 FROM node_edges WHERE relation = 'supersedes'
    UNION
    SELECT node_edges.target, walk.descendant, walk.depth + 1
    FROM node_edges JOIN walk ON node_edges.source = walk.ancestor
    WHERE node_edges.relation = 'supersedes'
)
SELECT 'supersedes', ancestor, descendant, MIN(depth) FROM walk GROUP BY ancestor, descendant;

INSERT INTO acyclic_closure (relation, ancestor, descendant, depth)
WITH RECURSIVE
    keyed (relation, source, target) AS (
        SELECT 'segment-' || segment_links.relation, sources.rowid, targets.rowid
        FROM segment_links
        JOIN segments AS sources
            ON sources.message = segment_links.source_message AND sources.ordinal = segment_links.source_ordinal
        JOIN segments AS targets
            ON targets.message = segment_links.target_message AND targets.ordinal = segment_links.target_ordinal
        WHERE segment_links.relation IN ('supersedes', 'corrects')
    ),
    walk (relation, ancestor, descendant, depth) AS (
        SELECT relation, target, source, 1 FROM keyed
        UNION
        SELECT keyed.relation, keyed.target, walk.descendant, walk.depth + 1
        FROM keyed JOIN walk ON keyed.source = walk.ancestor AND keyed.relation = walk.relation
    )
SELECT relation, ancestor, descendant, MIN(depth) FROM walk GROUP BY relation, ancestor, descendant;

DROP TABLE closure;

ALTER TABLE acyclic_closure RENAME TO closure;

CREATE INDEX closure_by_descendant ON closure (relation, descendant, depth);
