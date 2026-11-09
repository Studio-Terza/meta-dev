CREATE TABLE anchored_links (
    source TEXT NOT NULL,
    relation TEXT NOT NULL,
    target TEXT NOT NULL,
    source_message TEXT NOT NULL,
    target_message TEXT NOT NULL,
    source_ordinal INTEGER GENERATED ALWAYS AS (
        CASE WHEN instr(source, '#') > 0 THEN CAST(substr(source, instr(source, '#') + 1) AS INTEGER) END
    ) VIRTUAL,
    target_ordinal INTEGER GENERATED ALWAYS AS (
        CASE WHEN instr(target, '#') > 0 THEN CAST(substr(target, instr(target, '#') + 1) AS INTEGER) END
    ) VIRTUAL,
    PRIMARY KEY (source, relation, target),
    FOREIGN KEY (source_message, source_ordinal) REFERENCES segments (message, ordinal),
    FOREIGN KEY (target_message, target_ordinal) REFERENCES segments (message, ordinal)
) STRICT;

INSERT INTO anchored_links (source, relation, target, source_message, target_message)
SELECT source, relation, target, source_message, target_message FROM mechanical_links;

DROP TABLE mechanical_links;

ALTER TABLE anchored_links RENAME TO mechanical_links;

CREATE INDEX mechanical_links_by_source ON mechanical_links (source_message, source_ordinal);

CREATE INDEX mechanical_links_by_target ON mechanical_links (target_message, target_ordinal);
