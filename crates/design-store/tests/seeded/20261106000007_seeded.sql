INSERT INTO messages (uuid, session, parent, at, author, body)
VALUES ('o1', 's', NULL, '2026-01-01T00:00:01Z', 'owner', 'banning dyn seems desirable');

INSERT INTO segments (message, ordinal, shape, body)
VALUES ('o1', 0, 'sentence', 'banning dyn seems desirable');

INSERT INTO messages (uuid, session, parent, at, author, body)
VALUES ('o2', 's', NULL, '2026-01-01T00:00:02Z', 'owner', 'kept forever');

INSERT INTO segments (message, ordinal, shape, body)
VALUES ('o2', 0, 'sentence', 'kept');

INSERT INTO messages (uuid, session, parent, at, author, body)
VALUES ('o3', 's', NULL, '2026-01-01T00:00:03Z', 'owner', 'First point. Second point.');

INSERT INTO segments (message, ordinal, shape, body)
VALUES ('o3', 0, 'question', 'First point. Second point.');

INSERT INTO messages (uuid, session, parent, at, author, body)
VALUES ('o4', 's', NULL, '2026-01-01T00:00:04Z', 'owner', 'Only one.');

INSERT INTO segments (message, ordinal, shape, body)
VALUES ('o4', 0, 'sentence', 'Only one.'), ('o4', 1, 'sentence', 'gone astray');
