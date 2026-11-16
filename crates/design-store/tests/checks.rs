use std::error::Error;

use design_jobs::JobState;
use design_model::{
    Confidence, Explicitness, Facet, NodeRelation, Ordinal, Role, SegmentRelation, SenseKind, SpanRelation,
    Specificity, Stage,
};
use design_runs::{Delivery, Halted, RunState};
use design_store::StreamSchema;
use dispatch_ports::{Pid, Tokens};
use port_core::Op;
use port_core::Run;
use sqlite_ports::SqliteOpener;
use store_ports::{Argument, Column, FromValue, Query, Statements, Store, StoreFailure};
use stream::{Relation, Shape};

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;

type Checked = fn(&str) -> Option<String>;

struct Spelled;

impl Spelled {
    fn clause<Checking: Column>(column: &str) -> Option<String> {
        Checking::check(column).map(|clause| clause.to_string())
    }
}

#[derive(Clone, Copy, Debug, Op)]
struct Declared(&'static str);

impl Query for Declared {
    type Rows = String;

    fn rows(self, statements: &mut Statements<'_>) -> Result<String, StoreFailure> {
        let sql = "SELECT sql FROM sqlite_schema WHERE tbl_name = ?1 AND sql IS NOT NULL";
        let rows = statements.fetched(sql, vec![Argument::Text(self.0.to_owned())])?;
        Ok(rows
            .into_iter()
            .flatten()
            .filter_map(|value| String::from_value(value).ok())
            .collect::<Vec<String>>()
            .join("\n"))
    }
}

const COLUMNS: [(&str, &str, Checked); 28] = [
    ("segments", "shape", Spelled::clause::<Shape>),
    ("mechanical_links", "relation", Spelled::clause::<Relation>),
    ("nodes", "facet", Spelled::clause::<Facet>),
    ("nodes", "stage", Spelled::clause::<Stage>),
    ("nodes", "role", Spelled::clause::<Role>),
    ("node_edges", "relation", Spelled::clause::<NodeRelation>),
    ("span_edges", "relation", Spelled::clause::<SpanRelation>),
    ("span_edges", "ordinal", Spelled::clause::<Ordinal>),
    ("segment_links", "relation", Spelled::clause::<SegmentRelation>),
    ("segment_links", "source_ordinal", Spelled::clause::<Ordinal>),
    ("segment_links", "target_ordinal", Spelled::clause::<Ordinal>),
    ("senses", "kind", Spelled::clause::<SenseKind>),
    ("senses", "specificity", Spelled::clause::<Specificity>),
    ("senses", "confidence", Spelled::clause::<Confidence>),
    ("senses", "explicitness", Spelled::clause::<Explicitness>),
    ("senses", "ordinal", Spelled::clause::<Ordinal>),
    ("segments", "ordinal", Spelled::clause::<Ordinal>),
    ("conversational", "ordinal", Spelled::clause::<Ordinal>),
    ("flailed", "ordinal", Spelled::clause::<Ordinal>),
    ("jobs", "state", Spelled::clause::<JobState>),
    ("runs", "state", Spelled::clause::<RunState>),
    ("runs", "pid", Spelled::clause::<Pid>),
    ("runs", "input_tokens", Spelled::clause::<Option<Tokens>>),
    ("runs", "output_tokens", Spelled::clause::<Option<Tokens>>),
    ("runs", "cache_written_tokens", Spelled::clause::<Option<Tokens>>),
    ("runs", "cache_read_tokens", Spelled::clause::<Option<Tokens>>),
    ("failures", "halted", Spelled::clause::<Halted>),
    ("failures", "delivery", Spelled::clause::<Delivery>),
];

#[test]
fn every_listed_and_counted_column_refuses_what_its_type_refuses() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    COLUMNS.iter().try_for_each(|(table, column, checked)| -> Result<(), Box<dyn Error>> {
        let clause = checked(column).ok_or(format!("{table}.{column} has no generated check"))?;
        let declared = store.read(Declared(table))?;
        assert!(
            declared.contains(&clause),
            "{table}.{column} enforces what its type refuses: add `{clause}` to its checks"
        );
        Ok(())
    })
}

#[test]
fn a_column_whose_type_has_no_generated_check_yields_no_clause() {
    assert_eq!(Spelled::clause::<transcript_ports::SessionId>("session"), None);
    assert_eq!(Spelled::clause::<transcript_ports::Authorship>("author"), None);
    assert_eq!(Spelled::clause::<design_model::Provenance>("provenance"), None);
    assert_eq!(Spelled::clause::<Option<transcript_ports::SessionId>>("session"), None);
    assert_eq!(Spelled::clause::<file_ports::Offset>("read_to"), None);
    assert_eq!(Spelled::clause::<design_model::RunId>("run"), None);
}
