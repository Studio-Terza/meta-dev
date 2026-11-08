CREATE TABLE node_endings (
  node INTEGER PRIMARY KEY NOT NULL REFERENCES nodes (id),
  ending TEXT NOT NULL
) STRICT;

INSERT INTO
  node_endings (node, ending)
SELECT
  MAX(nodes.id),
  notes_endings.ending
FROM
  notes_endings
  JOIN nodes ON nodes.project = notes_endings.project
WHERE
  nodes.provenance LIKE 'import-%'
GROUP BY
  notes_endings.project;

DROP TABLE notes_endings;

ALTER TABLE node_endings RENAME TO notes_endings;
