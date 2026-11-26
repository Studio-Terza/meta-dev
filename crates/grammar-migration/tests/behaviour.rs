#![deny(clippy::wildcard_enum_match_arm)]

use grammar::{Ignoring, Unlexed};
use grammar_migration::MigrationFile;

#[derive(Debug, PartialEq, Eq)]
struct Named {
    version: i64,
    description: String,
}

impl Named {
    fn read(name: &str) -> Option<Named> {
        name.parse::<MigrationFile>()
            .ignoring(Unlexed::NotThisGrammar)
            .map(|file| Named { version: file.version(), description: file.description() })
    }

    fn of(version: i64, description: &str) -> Named {
        Named { version, description: description.to_owned() }
    }
}

#[test]
fn a_migration_file_name_reads_as_its_version_and_spaced_description() {
    for (name, want) in [
        ("20261106000000_stream.sql", Named::of(20_261_106_000_000, "stream")),
        ("20261106000005_applied_files.sql", Named::of(20_261_106_000_005, "applied files")),
        ("20261107000002_unicode_trimmed_segments.sql", Named::of(20_261_107_000_002, "unicode trimmed segments")),
        ("1_a2b.sql", Named::of(1, "a2b")),
    ] {
        assert_eq!(Named::read(name), Some(want), "{name}");
    }
}

#[test]
fn a_name_sqlx_would_not_read_as_a_simple_migration_is_refused() {
    for name in [
        "20261106000000.sql",
        "20261106000000_.sql",
        "20261106000000_stream",
        "20261106000000_stream.up.sql",
        "stream_20261106000000.sql",
        "20261106000000__stream.sql",
        "99999999999999999999_stream.sql",
        "",
    ] {
        assert_eq!(Named::read(name), None, "{name}");
    }
}
