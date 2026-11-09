#![deny(clippy::wildcard_enum_match_arm)]

use grammar_migration::{MigrationFile, Transaction};

#[derive(Debug, PartialEq, Eq)]
struct Named {
    version: i64,
    description: String,
}

impl Named {
    fn read(name: &&str) -> Option<Named> {
        name.parse::<MigrationFile>()
            .ok()
            .map(|file| Named { version: file.version(), description: file.description() })
    }

    fn of(version: i64, description: &str) -> Named {
        Named { version, description: description.to_owned() }
    }
}

laws::table!(
    a_migration_file_name_reads_as_its_version_and_spaced_description,
    Named::read,
    [
        "20261106000000_stream.sql" => Some(Named::of(20_261_106_000_000, "stream")),
        "20261106000005_applied_files.sql" => Some(Named::of(20_261_106_000_005, "applied files")),
        "20261107000002_unicode_trimmed_segments.sql" => Some(Named::of(20_261_107_000_002, "unicode trimmed segments")),
        "1_a2b.sql" => Some(Named::of(1, "a2b")),
    ]
);

laws::table!(
    a_name_sqlx_would_not_read_as_a_simple_migration_is_refused,
    Named::read,
    [
        "20261106000000.sql" => None,
        "20261106000000_.sql" => None,
        "20261106000000_stream" => None,
        "20261106000000_stream.up.sql" => None,
        "stream_20261106000000.sql" => None,
        "20261106000000__stream.sql" => None,
        "99999999999999999999_stream.sql" => None,
        "" => None,
    ]
);

laws::table!(
    only_a_migration_opening_with_sqlx_s_header_runs_outside_a_transaction,
    |sql: &&str| Transaction::from(*sql),
    [
        "-- no-transaction\nCREATE INDEX a ON b (c);" => Transaction::Unwrapped,
        "-- no-transaction" => Transaction::Unwrapped,
        "CREATE TABLE a (b TEXT);" => Transaction::Wrapped,
        " -- no-transaction" => Transaction::Wrapped,
        "-- no transaction" => Transaction::Wrapped,
        "" => Transaction::Wrapped,
    ]
);
