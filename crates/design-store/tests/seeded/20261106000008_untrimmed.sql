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
