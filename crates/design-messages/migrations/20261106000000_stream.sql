CREATE TABLE messages (
    uuid TEXT PRIMARY KEY NOT NULL,
    session TEXT NOT NULL,
    parent TEXT,
    at TEXT NOT NULL,
    author TEXT NOT NULL,
    body TEXT NOT NULL
) STRICT;

CREATE TABLE segments (
    message TEXT NOT NULL REFERENCES messages (uuid),
    ordinal INTEGER NOT NULL,
    shape TEXT NOT NULL,
    body TEXT NOT NULL,
    PRIMARY KEY (message, ordinal)
) STRICT;

CREATE VIRTUAL TABLE segments_fts USING fts5 (body, content = 'segments', content_rowid = 'rowid');

CREATE TRIGGER segments_indexed AFTER INSERT ON segments BEGIN
    INSERT INTO segments_fts (rowid, body) VALUES (new.rowid, new.body);
END;

CREATE TABLE mechanical_links (
    source TEXT NOT NULL,
    relation TEXT NOT NULL,
    target TEXT NOT NULL,
    source_message TEXT NOT NULL,
    target_message TEXT NOT NULL,
    PRIMARY KEY (source, relation, target)
) STRICT;

CREATE INDEX mechanical_links_by_source ON mechanical_links (source_message);

CREATE INDEX mechanical_links_by_target ON mechanical_links (target_message);

CREATE TABLE cursors (
    session TEXT PRIMARY KEY NOT NULL,
    read_to INTEGER NOT NULL
) STRICT;
