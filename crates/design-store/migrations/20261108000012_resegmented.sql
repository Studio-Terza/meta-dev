CREATE TRIGGER segments_reindexed AFTER UPDATE OF body ON segments BEGIN
    INSERT INTO segments_fts (segments_fts, rowid, body) VALUES ('delete', old.rowid, old.body);
    INSERT INTO segments_fts (rowid, body) VALUES (new.rowid, new.body);
END;

CREATE TRIGGER segments_unindexed AFTER DELETE ON segments BEGIN
    INSERT INTO segments_fts (segments_fts, rowid, body) VALUES ('delete', old.rowid, old.body);
END;

INSERT INTO segments_fts (segments_fts) VALUES ('rebuild');
