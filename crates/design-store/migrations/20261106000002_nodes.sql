CREATE TABLE nodes (
    id INTEGER PRIMARY KEY NOT NULL,
    project TEXT NOT NULL,
    facet TEXT NOT NULL,
    stage TEXT NOT NULL,
    provenance TEXT NOT NULL
) STRICT;

CREATE INDEX nodes_by_project ON nodes (project, facet);

CREATE TABLE node_edges (
    source INTEGER NOT NULL REFERENCES nodes (id),
    relation TEXT NOT NULL,
    target INTEGER NOT NULL REFERENCES nodes (id),
    PRIMARY KEY (source, relation, target)
) STRICT;
