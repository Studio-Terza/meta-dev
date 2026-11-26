CREATE TABLE ideals (
    id INTEGER PRIMARY KEY CHECK (id > 0),
    superseded_by INTEGER CHECK (superseded_by > 0)
) STRICT;

CREATE TABLE ideal_quotes (
    ideal INTEGER NOT NULL REFERENCES ideals (id),
    place INTEGER NOT NULL CHECK (place BETWEEN 0 AND 4294967295),
    message TEXT NOT NULL REFERENCES messages (uuid),
    excerpt TEXT NOT NULL CHECK (excerpt <> ''),
    PRIMARY KEY (ideal, place)
) STRICT;

CREATE TABLE features (
    name TEXT PRIMARY KEY NOT NULL CHECK (name <> ''),
    place INTEGER NOT NULL UNIQUE CHECK (place BETWEEN 0 AND 4294967295),
    consideration TEXT NOT NULL CHECK (consideration IN ('checks', 'process', 'shipping'))
) STRICT;

CREATE TABLE feature_quotes (
    feature TEXT NOT NULL REFERENCES features (name),
    place INTEGER NOT NULL CHECK (place BETWEEN 0 AND 4294967295),
    message TEXT NOT NULL REFERENCES messages (uuid),
    excerpt TEXT NOT NULL CHECK (excerpt <> ''),
    PRIMARY KEY (feature, place)
) STRICT;

CREATE TABLE feature_crates (
    feature TEXT NOT NULL REFERENCES features (name),
    member TEXT NOT NULL CHECK (member <> ''),
    PRIMARY KEY (feature, member)
) STRICT;

CREATE TABLE judgments (
    id INTEGER PRIMARY KEY,
    feature TEXT NOT NULL CHECK (feature <> ''),
    ideal INTEGER NOT NULL CHECK (ideal > 0),
    verdict TEXT NOT NULL CHECK (verdict IN ('holds', 'violated', 'unclear')),
    file TEXT CHECK (file <> ''),
    line INTEGER CHECK (line BETWEEN 1 AND 4294967295),
    revision TEXT NOT NULL CHECK (revision <> ''),
    at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) CHECK (at <> ''),
    CHECK ((file IS NULL) = (line IS NULL))
) STRICT;
