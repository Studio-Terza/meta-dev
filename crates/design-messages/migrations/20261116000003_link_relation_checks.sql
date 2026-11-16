CREATE TEMP TABLE compliance (
    mechanical_links INTEGER NOT NULL CHECK (mechanical_links = 0)
);

INSERT INTO compliance (mechanical_links)
SELECT
  (SELECT count(*) FROM mechanical_links WHERE NOT ((relation IN ('replies-to', 'answers', 'cites', 'resends'))));

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
