use std::path::PathBuf;

use design_messages::StreamSchema;
use file_ports::Disk;
use fixture::{Cases, Debugged, Failed, Fixture, Parsed, Spelled};
use port_core::Run;
use query_cache::{Cache, Entry, Origin, Table};
use serde::Deserialize;
use sqlite_ports::SqliteOpener;
use store_ports::{Opens, Parses};

#[derive(Fixture, Deserialize, Debug)]
struct Cached {
    entry: Entry<Spelled>,
    #[serde(default)]
    tables: Vec<Named>,
}

#[derive(Deserialize, Debug)]
struct Named {
    name: Spelled,
    columns: Vec<Declared>,
    overwritten: Spelled,
}

#[derive(Deserialize, Debug)]
struct Declared {
    name: Spelled,
    origin: Parsed<Origin>,
}

#[derive(Fixture, Deserialize, Debug)]
struct Workspace {
    root: PathBuf,
}

#[derive(Fixture, Deserialize, Debug)]
struct Found {
    sql: Spelled,
}

#[test]
fn a_query_reads_its_cache_file_from_a_sqlx_directory_above_its_crate() -> Result<(), Failed> {
    let drifted = Cases::<Found>::drifted(|case| {
        let run = Run::root();
        let found = Entry::<Spelled>::found(&run.port(Disk), case.row().sql.as_str());
        Ok(Debugged(found).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn every_cached_query_binds_what_it_takes_and_every_record_table_spells_its_statements() -> Result<(), Failed> {
    let drifted = Cases::<Cached>::drifted(|case| {
        let cached = case.into_row();
        let bound: Vec<u8> =
            (u8::MIN..=u8::MAX).filter(|given| cached.entry.bound(usize::from(*given)).is_ok()).collect();
        let refused = cached.entry.bound(usize::from(u8::MAX)).map_err(|refusal| refusal.to_string());
        let tables: Vec<_> = cached
            .tables
            .iter()
            .map(|named| {
                let table = named.columns.iter().fold(Table::named(named.name.as_str()), |table, column| {
                    table.column(column.name.as_str(), *column.origin.value())
                });
                (table.statements(), table.overwrite(named.overwritten.as_str()))
            })
            .collect();
        let written = cached.entry.written()?;
        let columns: Vec<_> = cached
            .entry
            .columns()
            .into_iter()
            .map(|column| (column.storage().spelled(), column.nullable().possible(), column))
            .collect();
        Ok(Debugged((bound, refused, tables, columns, written)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn the_workspace_cache_holds_what_the_schema_describes_of_every_query_in_its_sources() -> Result<(), Failed> {
    let drifted = Cases::<Workspace>::drifted(|case| {
        let run = Run::root();
        let store = run.port(SqliteOpener.in_memory(&SqliteOpener.parsed(StreamSchema::SQL)?));
        Ok(Cache::rooted(&case.row().root).drifted(&run.port(Disk), &store)?.to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
