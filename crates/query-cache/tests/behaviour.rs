use fixture::{Cases, Debugged, Failed, Fixture, Parsed, Spelled};
use query_cache::{Entry, Origin, Table};
use serde::Deserialize;

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
struct Found {
    sql: Spelled,
}

#[test]
fn a_query_reads_its_cache_file_from_a_sqlx_directory_above_its_crate() -> Result<(), Failed> {
    let drifted = Cases::<Found>::drifted(|case| {
        let found = Entry::<Spelled>::found(case.row().sql.as_str());
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
