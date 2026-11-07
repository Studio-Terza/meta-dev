WITH
RECURSIVE points (n) AS (
  SELECT
    32
  UNION ALL
  SELECT
    n + 1
  FROM
    points
  WHERE
    n < 12288
)
INSERT INTO messages (uuid, session, parent, at, author, body)
SELECT
  printf('%08x-0000-4000-8000-000000000000', n),
  's',
  NULL,
  '2026-01-01T00:00:00Z',
  'owner',
  'w' || char(n) || char(160)
FROM
  points;

INSERT INTO segments (message, ordinal, shape, body)
SELECT
  uuid,
  0,
  'sentence',
  body
FROM
  messages
WHERE
  uuid LIKE '%-0000-4000-8000-000000000000';
