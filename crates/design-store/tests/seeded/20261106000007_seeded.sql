INSERT INTO messages (uuid, session, parent, at, author, body)
VALUES ('o1', 's', NULL, '2026-01-01T00:00:01Z', 'owner', 'banning dyn seems desirable');

INSERT INTO segments (message, ordinal, shape, body)
VALUES ('o1', 0, 'sentence', 'banning dyn seems desirable');

INSERT INTO messages (uuid, session, parent, at, author, body)
VALUES ('o2', 's', NULL, '2026-01-01T00:00:02Z', 'owner', 'kept forever');

INSERT INTO segments (message, ordinal, shape, body)
VALUES ('o2', 0, 'sentence', 'kept');
