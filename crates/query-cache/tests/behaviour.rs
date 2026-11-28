use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use query_cache::{Entry, Origin, Table};
use rusqlite::{Connection, Statement};
use serde::Serialize;
use sha2::{Digest, Sha256};
use syn::parse::{ParseStream, Parser};
use syn::visit::Visit;
use syn::{Field, Ident, ItemStruct, LitStr, Macro, Token, Type};
use test_failure::Failed;

#[derive(Debug)]
struct Cache {
    root: PathBuf,
}

#[derive(Debug, Default)]
struct Queries(Vec<syn::Result<Vec<String>>>);

#[derive(Debug, Serialize)]
struct Written<'query> {
    query: &'query str,
    describe: Description,
}

#[derive(Debug, Serialize)]
struct Description {
    columns: Vec<Column>,
    parameters: Parameters,
    nullable: Vec<Option<bool>>,
}

#[derive(Debug, Serialize)]
struct Column {
    name: String,
    type_info: &'static str,
}

#[derive(Debug, Serialize)]
enum Parameters {
    Right(usize),
}

impl Cache {
    const ROOT: &'static str = "../..";
    const DIRECTORY: &'static str = ".sqlx";
    const CRATES: &'static str = "crates";
    const SCHEMA: &'static str = "crates/design-messages/src/schema.sql";
    const FIXTURES: &'static str = "fixtures";
    const RUST: &'static str = "rs";
    const RECORD: &'static str = "FIXTURE_RECORD";

    fn found() -> Cache {
        Cache { root: Path::new(env!("CARGO_MANIFEST_DIR")).join(Cache::ROOT) }
    }

    fn sources(directory: &Path) -> Result<Vec<PathBuf>, Failed> {
        let nested = fs::read_dir(directory)?
            .map(|entry| {
                let path = entry?.path();
                let fixture = path.file_name().filter(|name| *name == Cache::FIXTURES);
                let rust =
                    path.extension().filter(|extension| *extension == Cache::RUST).map(|_rust| vec![path.clone()]);
                match (fixture, path.is_dir()) {
                    (Some(_fixture), _) => Ok(Vec::new()),
                    (None, true) => Cache::sources(&path),
                    (None, false) => Ok(rust.unwrap_or_default()),
                }
            })
            .collect::<Result<Vec<Vec<PathBuf>>, Failed>>()?;
        Ok(nested.into_iter().flatten().collect())
    }

    fn queries(&self) -> Result<Vec<String>, Failed> {
        let queries = Cache::sources(&self.root.join(Cache::CRATES))?.iter().try_fold(
            Queries::default(),
            |mut found, path| {
                found.visit_file(&syn::parse_file(&fs::read_to_string(path)?)?);
                Ok::<Queries, Failed>(found)
            },
        )?;
        let found = queries.0.into_iter().collect::<syn::Result<Vec<Vec<String>>>>()?;
        Ok(found.into_iter().flatten().collect())
    }

    fn regenerated(&self, queries: &[String]) -> Result<BTreeMap<String, String>, Failed> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch(&fs::read_to_string(self.root.join(Cache::SCHEMA))?)?;
        queries
            .iter()
            .map(|query| {
                let describe = Description::of(&connection.prepare(query)?)?;
                let text = serde_json::to_string_pretty(&Written { query, describe })?;
                Ok((format!("query-{}.json", hex::encode(Sha256::digest(query.as_bytes()))), format!("{text}\n")))
            })
            .collect()
    }

    fn held(&self) -> Result<BTreeMap<String, String>, Failed> {
        fs::read_dir(self.root.join(Cache::DIRECTORY))?
            .map(|entry| {
                let held = entry?;
                Ok((held.file_name().to_string_lossy().into_owned(), fs::read_to_string(held.path())?))
            })
            .collect()
    }

    fn rewritten(&self, wanted: &BTreeMap<String, String>, drifted: &BTreeSet<&String>) -> Result<(), Failed> {
        drifted.iter().try_for_each(|name| {
            let path = self.root.join(Cache::DIRECTORY).join(name);
            wanted.get(*name).map_or_else(|| fs::remove_file(&path), |text| fs::write(&path, text))
        })?;
        Ok(())
    }
}

impl Description {
    fn of(statement: &Statement<'_>) -> Result<Description, Failed> {
        let described = (0..statement.column_count())
            .map(|at| {
                let name = statement.column_name(at)?.to_owned();
                let origin = statement.column_metadata(at)?;
                let declared = origin.and_then(|(_, _, _, declared, _, _, _, _)| declared);
                let type_info = declared.map_or("Null", |spelled| Column::stored(spelled.to_bytes()));
                Ok((Column { name, type_info }, origin.map(|(_, _, _, _, _, not_null, _, _)| !not_null)))
            })
            .collect::<Result<Vec<(Column, Option<bool>)>, Failed>>()?;
        let (columns, nullable) = described.into_iter().unzip();
        Ok(Description { columns, parameters: Parameters::Right(statement.parameter_count()), nullable })
    }
}

impl Column {
    fn stored(declared: &[u8]) -> &'static str {
        match declared.to_ascii_uppercase().as_slice() {
            b"INTEGER" | b"INT" => "Integer",
            b"REAL" => "Real",
            b"TEXT" => "Text",
            b"BLOB" => "Blob",
            _ => "Null",
        }
    }
}

impl Queries {
    const PORT: &'static str = "store_ports";
    const QUERY: &'static str = "query";
    const RECORD: &'static str = "record";

    const UNNAMED: &'static str = "a record is a struct with named fields";

    fn literal(input: ParseStream<'_>) -> syn::Result<LitStr> {
        if !input.peek(LitStr) {
            input.parse::<Type>()?;
            input.parse::<Token![,]>()?;
        }
        let literal = input.parse::<LitStr>()?;
        input.parse::<TokenStream>()?;
        Ok(literal)
    }

    fn table(input: ParseStream<'_>) -> syn::Result<LitStr> {
        input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        input.parse::<LitStr>()
    }

    fn placed(table: Table, field: &Field) -> syn::Result<Table> {
        let name = field.ident.as_ref().ok_or_else(|| syn::Error::new_spanned(field, Queries::UNNAMED))?;
        let marked = field.attrs.iter().find(|attribute| attribute.path().is_ident(Queries::RECORD));
        let origin = marked
            .map(|attribute| {
                let word = attribute.parse_args::<Ident>()?;
                word.to_string().parse::<Origin>().map_err(|failed| syn::Error::new(word.span(), failed))
            })
            .transpose()?;
        Ok(table.column(&name.to_string(), origin.unwrap_or(Origin::Given)))
    }
}

impl<'ast> Visit<'ast> for Queries {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let named = invocation.path.segments.iter().map(|segment| segment.ident.to_string()).collect::<Vec<String>>();
        let query = named
            .first()
            .filter(|port| *port == Queries::PORT)
            .and(named.last())
            .filter(|last| *last == Queries::QUERY);
        self.0.extend(
            query.map(|_query| Queries::literal.parse2(invocation.tokens.clone()).map(|literal| vec![literal.value()])),
        );
        syn::visit::visit_macro(self, invocation);
    }

    fn visit_item_struct(&mut self, record: &'ast ItemStruct) {
        let marked = record.attrs.iter().find(|attribute| attribute.path().is_ident(Queries::RECORD));
        self.0.extend(marked.map(|attribute| {
            let named = attribute.parse_args_with(Queries::table)?;
            let described = record.fields.iter().try_fold(Table::named(&named.value()), Queries::placed)?;
            Ok(described.statements().to_vec())
        }));
        syn::visit::visit_item_struct(self, record);
    }
}

#[test]
fn every_cached_query_is_what_the_schema_describes_and_no_other_is_cached() -> Result<(), Failed> {
    let cache = Cache::found();
    let queries = cache.queries()?;
    let wanted = cache.regenerated(&queries)?;
    let held = cache.held()?;
    let drifted: BTreeSet<&String> =
        wanted.keys().chain(held.keys()).filter(|name| wanted.get(*name) != held.get(*name)).collect();
    let unrecorded = env::var_os(Cache::RECORD)
        .filter(|switch| *switch == "1")
        .map_or_else(|| Ok(drifted.clone()), |_record| cache.rewritten(&wanted, &drifted).map(|()| BTreeSet::new()))?;
    assert_eq!(
        unrecorded,
        BTreeSet::new(),
        "the .sqlx cache drifted from the schema; rewrite it with FIXTURE_RECORD=1 cargo test -p query-cache"
    );
    queries.iter().try_for_each(|query| Entry::<String>::found(query).map(|_entry| ()))?;
    Ok(())
}

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
