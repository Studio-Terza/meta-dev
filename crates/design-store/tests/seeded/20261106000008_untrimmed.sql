WITH
RECURSIVE points (n) AS (
  SELECT
    0
  UNION ALL
  SELECT
    n + 1
  FROM
    points
  WHERE
    n < 1114111
)
INSERT INTO messages (uuid, session, parent, at, author, body)
SELECT
  printf('%08x-0000-4000-8000-000000000000', n),
  's',
  NULL,
  '2026-01-01T00:00:00Z',
  'owner',
  char(n) || 'w' || char(n)
FROM
  points
WHERE
  n NOT BETWEEN 55296 AND 57343;

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
