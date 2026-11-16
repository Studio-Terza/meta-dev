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

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;

type Checked = fn(&str) -> Option<String>;

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

const COLUMNS: [(&str, &str, Checked); 26] = [
    ("nodes", "facet", <Facet as Column>::check),
    ("nodes", "stage", <Stage as Column>::check),
    ("nodes", "role", <Role as Column>::check),
    ("node_edges", "relation", <NodeRelation as Column>::check),
    ("span_edges", "relation", <SpanRelation as Column>::check),
    ("span_edges", "ordinal", <Ordinal as Column>::check),
    ("segment_links", "relation", <SegmentRelation as Column>::check),
    ("segment_links", "source_ordinal", <Ordinal as Column>::check),
    ("segment_links", "target_ordinal", <Ordinal as Column>::check),
    ("senses", "kind", <SenseKind as Column>::check),
    ("senses", "specificity", <Specificity as Column>::check),
    ("senses", "confidence", <Confidence as Column>::check),
    ("senses", "explicitness", <Explicitness as Column>::check),
    ("senses", "ordinal", <Ordinal as Column>::check),
    ("segments", "ordinal", <Ordinal as Column>::check),
    ("conversational", "ordinal", <Ordinal as Column>::check),
    ("flailed", "ordinal", <Ordinal as Column>::check),
    ("jobs", "state", <JobState as Column>::check),
    ("runs", "state", <RunState as Column>::check),
    ("runs", "pid", <Pid as Column>::check),
    ("runs", "input_tokens", <Option<Tokens> as Column>::check),
    ("runs", "output_tokens", <Option<Tokens> as Column>::check),
    ("runs", "cache_written_tokens", <Option<Tokens> as Column>::check),
    ("runs", "cache_read_tokens", <Option<Tokens> as Column>::check),
    ("failures", "halted", <Halted as Column>::check),
    ("failures", "delivery", <Delivery as Column>::check),
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
