CREATE TABLE closure (
    ancestor INTEGER NOT NULL REFERENCES nodes (id),
    descendant INTEGER NOT NULL REFERENCES nodes (id),
    depth INTEGER NOT NULL,
    PRIMARY KEY (ancestor, descendant)
) STRICT;

CREATE INDEX closure_by_descendant ON closure (descendant, depth);

INSERT INTO closure (ancestor, descendant, depth) SELECT id, id, 0 FROM nodes;

ALTER TABLE nodes ADD COLUMN title TEXT;

CREATE TABLE span_edges (
    message TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    relation TEXT NOT NULL,
    node INTEGER NOT NULL REFERENCES nodes (id),
    PRIMARY KEY (message, ordinal, relation, node),
    FOREIGN KEY (message, ordinal) REFERENCES segments (message, ordinal)
) STRICT;

CREATE INDEX span_edges_by_node ON span_edges (node);

CREATE TABLE segment_links (
    source_message TEXT NOT NULL,
    source_ordinal INTEGER NOT NULL,
    relation TEXT NOT NULL,
    target_message TEXT NOT NULL,
    target_ordinal INTEGER NOT NULL,
    PRIMARY KEY (source_message, source_ordinal, relation, target_message, target_ordinal),
    FOREIGN KEY (source_message, source_ordinal) REFERENCES segments (message, ordinal),
    FOREIGN KEY (target_message, target_ordinal) REFERENCES segments (message, ordinal)
) STRICT;

CREATE INDEX segment_links_by_target ON segment_links (target_message, target_ordinal);

CREATE TABLE senses (
    message TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    kind TEXT NOT NULL,
    specificity TEXT NOT NULL,
    confidence TEXT NOT NULL,
    explicitness TEXT NOT NULL,
    PRIMARY KEY (message, ordinal),
    FOREIGN KEY (message, ordinal) REFERENCES segments (message, ordinal)
) STRICT;

CREATE TABLE conversational (
    message TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    PRIMARY KEY (message, ordinal),
    FOREIGN KEY (message, ordinal) REFERENCES segments (message, ordinal)
) STRICT;
