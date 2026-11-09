INSERT INTO messages (uuid, session, parent, at, author, body)
VALUES ('u1', 's', NULL, '2026-01-01T00:00:01Z', 'owner', 'Only one.');

INSERT INTO segments (message, ordinal, shape, body)
VALUES ('u1', 0, 'sentence', 'Only one.');

INSERT INTO mechanical_links (source, relation, target, source_message, target_message)
VALUES ('u1#0', 'answers', 'u1#1', 'u1', 'u1');
