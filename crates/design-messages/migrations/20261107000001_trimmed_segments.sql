UPDATE segments
SET
  body = trim(body, ' ' || char(9) || char(10) || char(13));

INSERT INTO segments_fts (segments_fts) VALUES ('rebuild');
