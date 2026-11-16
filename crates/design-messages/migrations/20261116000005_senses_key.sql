CREATE UNIQUE INDEX senses_natural ON senses (message, ordinal, kind, provenance, IFNULL(supersedes, 0));
