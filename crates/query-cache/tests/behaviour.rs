use std::path::Path;

use query_cache::{Entry, Origin, Table};
use test_failure::Failed;

const STATEMENTS: &str = "tests/goldens/statements.txt";
const REFUSALS: &str = "tests/goldens/refusals.txt";
const TWO_PARAMETERS: &str = r#"{"describe": {"columns": [], "parameters": {"Right": 2}, "nullable": []}}"#;
const BOUND: [usize; 3] = [1, 2, 3];
const NEWLINE: &str = "\n";
const TABLES: [&str; 2] = ["r", "s"];
const COLUMNS: [&[(&str, Origin)]; 2] = [
    &[("id", Origin::GeneratedKey), ("body", Origin::Given), ("at", Origin::Generated)],
    &[("id", Origin::GeneratedKey)],
];
const OVERWRITTEN: [&str; 2] = ["body", "id"];

#[test]
fn a_record_table_spells_its_statements() -> Result<(), Failed> {
    let spelled: Vec<String> = TABLES
        .iter()
        .zip(COLUMNS)
        .zip(OVERWRITTEN)
        .flat_map(|((name, columns), overwritten)| {
            let table = columns.iter().fold(Table::named(name), |table, (column, origin)| table.column(column, *origin));
            table.statements().into_iter().chain([Table::named(name).overwrite(overwritten)])
        })
        .collect();
    assert_eq!([spelled.join(NEWLINE), NEWLINE.to_owned()].concat(), std::fs::read_to_string(Path::new(STATEMENTS))?);
    Ok(())
}

#[test]
fn a_cached_query_binds_only_as_many_values_as_it_takes() -> Result<(), Failed> {
    let entry = serde_json::from_str::<Entry<String>>(TWO_PARAMETERS)?;
    let refused: Vec<String> = BOUND
        .iter()
        .filter_map(|count| match entry.bound(*count) {
            Err(refusal) => Some(refusal.to_string()),
            Ok(()) => None,
        })
        .collect();
    assert_eq!([refused.join(NEWLINE), NEWLINE.to_owned()].concat(), std::fs::read_to_string(Path::new(REFUSALS))?);
    Ok(())
}
