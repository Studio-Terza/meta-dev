use std::fs;
use std::path::Path;

use query_cache::{Entry, MigrationFile, Origin, Table};
use test_failure::Failed;

const COLUMNS: &str = r#"{"describe": {"columns": [{"name": "a", "type_info": "Text"}, {"name": "b", "type_info": "Integer"}, {"name": "c", "type_info": "Null"}], "parameters": {"Right": 0}, "nullable": [true, false]}}"#;
const TWO_PARAMETERS: &str = r#"{"describe": {"columns": [], "parameters": {"Right": 2}, "nullable": []}}"#;

#[test]
fn a_record_table_spells_its_statements_and_a_cached_query_binds_only_what_it_takes() -> Result<(), serde_json::Error> {
    let entry: Entry<String> = serde_json::from_str(TWO_PARAMETERS)?;
    let refused = [1, 2, 3].map(|given| entry.bound(given).map_err(|refusal| refusal.to_string()));
    let unbound = |given: usize| Err(format!("the query takes 2 parameters but {given} are bound"));
    assert_eq!(refused, [unbound(1), Ok(()), unbound(3)]);
    let described: Entry<String> = serde_json::from_str(COLUMNS)?;
    let columns: Vec<String> = described
        .columns()
        .iter()
        .map(|column| {
            format!("{} {:?} {:?}", column.as_ref(), column.storage().spelled(), column.nullable().possible())
        })
        .collect();
    assert_eq!(columns, ["a Some(\"::std::string::String\") Some(())", "b Some(\"i64\") None", "c None Some(())"]);
    let full = Table::named("r")
        .column("id", Origin::GeneratedKey)
        .column("body", Origin::Given)
        .column("at", Origin::Generated);
    let bare = Table::named("s").column("id", Origin::GeneratedKey);
    let spelled = [full.statements().join("\n"), bare.statements().join("\n"), full.overwrite("body")];
    assert_eq!(
        spelled,
        [
            "INSERT INTO r (body) VALUES (?1) RETURNING id AS \"id!\", body AS \"body!\", at AS \"at!\"\nSELECT id AS \"id!\", body AS \"body!\", at AS \"at!\" FROM r WHERE id = ?1\nUPDATE r SET body = ?1, at = ?2 WHERE id = ?3\nDELETE FROM r WHERE id = ?1\nSELECT id AS \"id!\", body AS \"body!\", at AS \"at!\" FROM r",
            "INSERT INTO s DEFAULT VALUES RETURNING id AS \"id!\"\nSELECT id AS \"id!\" FROM s WHERE id = ?1\nUPDATE s SET id = id WHERE id = ?1\nDELETE FROM s WHERE id = ?1\nSELECT id AS \"id!\" FROM s",
            "UPDATE r SET body = ?1",
        ]
    );
    Ok(())
}

#[test]
fn every_migration_file_name_reads_as_its_golden_says() -> Result<(), Failed> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let names: Vec<String> = serde_json::from_str(&fs::read_to_string(directory.join("migrations.json"))?)?;
    let shown: Vec<String> = names
        .iter()
        .map(|name| {
            let file = name.parse::<MigrationFile>();
            format!("migration {name:?}: {:?}", file.map(|read| (read.version(), read.description())))
        })
        .collect();
    let said = shown.join("\n");
    let golden = directory.join("migrations.txt");
    std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| fs::write(&golden, &said))?;
    assert_eq!(said, fs::read_to_string(golden)?);
    Ok(())
}
