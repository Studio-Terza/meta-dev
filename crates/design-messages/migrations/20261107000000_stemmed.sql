DROP TRIGGER segments_indexed;

DROP TABLE segments_fts;

CREATE VIRTUAL TABLE segments_fts USING fts5 (
    body,
    content = 'segments',
    content_rowid = 'rowid',
    tokenize = 'porter unicode61'
);

CREATE TRIGGER segments_indexed AFTER INSERT ON segments BEGIN
    INSERT INTO segments_fts (rowid, body) VALUES (new.rowid, new.body);
END;

INSERT INTO segments_fts (segments_fts) VALUES ('rebuild');
