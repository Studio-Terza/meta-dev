CREATE TABLE node_cites (
  node INTEGER NOT NULL REFERENCES nodes (id),
  message TEXT NOT NULL REFERENCES messages (uuid),
  provenance TEXT NOT NULL,
  PRIMARY KEY (node, message)
) STRICT;

CREATE INDEX node_cites_by_message ON node_cites (message);
