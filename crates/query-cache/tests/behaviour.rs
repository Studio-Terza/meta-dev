use std::io;
use std::path::{Path, PathBuf};

use query_cache::{Entry, Table};
use serde::Deserialize;
use test_failure::Failed;

#[test]
fn a_record_table_spells_its_statements_and_a_query_binds_only_what_it_takes() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let bytes = std::fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    let golden = PathBuf::deserialize(&mut rows)?;
    let described = Entry::<String>::deserialize(&mut rows)?;
    let bound = Vec::<usize>::deserialize(&mut rows)?;
    let tables = Vec::<(Table, String)>::deserialize(&mut rows)?;
    rows.end()?;
    let spelled: Vec<String> = tables
        .iter()
        .flat_map(|(built, overwritten)| built.statements().into_iter().chain([built.overwrite(overwritten)]))
        .collect();
    let refused: Vec<String> = bound
        .into_iter()
        .filter_map(|count| match described.bound(count) {
            Err(refusal) => Some(refusal.to_string()),
            Ok(()) => None,
        })
        .collect();
    assert_eq!((spelled, refused), serde_json::from_slice(&std::fs::read(golden)?)?);
    Ok(())
}
