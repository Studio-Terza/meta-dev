CREATE TEMP TABLE compliance (
    mechanical_links INTEGER NOT NULL CHECK (mechanical_links = 0),
    segments INTEGER NOT NULL CHECK (segments = 0)
);

INSERT INTO compliance (mechanical_links, segments)
SELECT
  (SELECT count(*) FROM mechanical_links WHERE NOT ((relation IN ('replies-to', 'answers', 'cites', 'resends')))),
  (SELECT count(*) FROM segments WHERE NOT ((shape IN ('sentence', 'item', 'bullet', 'quote'))));

DROP TABLE temp.compliance;

CREATE TRIGGER mechanical_links_checked_on_insert BEFORE INSERT ON mechanical_links
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.relation AS relation) WHERE (relation IN ('replies-to', 'answers', 'cites', 'resends'))
)
BEGIN
SELECT RAISE(ABORT, 'mechanical_links refuses a value its column type refuses');
END;

CREATE TRIGGER mechanical_links_checked_on_update BEFORE UPDATE ON mechanical_links
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.relation AS relation) WHERE (relation IN ('replies-to', 'answers', 'cites', 'resends'))
)
BEGIN
SELECT RAISE(ABORT, 'mechanical_links refuses a value its column type refuses');
END;

CREATE TRIGGER segments_shape_checked_on_insert BEFORE INSERT ON segments
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.shape AS shape) WHERE (shape IN ('sentence', 'item', 'bullet', 'quote'))
)
BEGIN
SELECT RAISE(ABORT, 'segments refuses a value its column type refuses');
END;

CREATE TRIGGER segments_shape_checked_on_update BEFORE UPDATE ON segments
WHEN NOT EXISTS (
  SELECT 1 FROM (SELECT NEW.shape AS shape) WHERE (shape IN ('sentence', 'item', 'bullet', 'quote'))
)
BEGIN
SELECT RAISE(ABORT, 'segments refuses a value its column type refuses');
END;
