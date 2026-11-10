CREATE VIEW structured (message, ordinal) AS
SELECT message, ordinal FROM span_edges
UNION ALL
SELECT message, ordinal FROM senses
UNION ALL
SELECT message, ordinal FROM conversational
UNION ALL
SELECT source_message, source_ordinal FROM segment_links
UNION ALL
SELECT target_message, target_ordinal FROM segment_links;
