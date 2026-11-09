use std::error::Error;
use std::path::PathBuf;

use design_model::{
    Artifact, ArtifactKind, Condition, Confidence, Entered, Explicitness, Facet, Kin, NodeId, NodeRelation, Ordinal,
    Prefix, Project, Provenance, Reason, Role, RunId, SegmentRelation, SenseKind, SpanRelation, Specificity, Stage,
    Standing, Title,
};
use design_store::{RunState, SessionCursors, ShownMessage, StreamSchema};
use dispatch_ports::{Pid, Started, Tokens, Wall};
use file_ports::{Bytes, Offset};
use grammar::search::Term;
use laws::proptest::option;
use laws::proptest::prelude::{any, prop_assert, prop_oneof};
use laws::proptest::sample::select;
use laws::proptest::strategy::Strategy;
use newtype::{Arbitrary, Newtype};
use port_core::{Op, Run};
use store_ports::{
    Batch, Busy, Column, ColumnFailure, Columns, Query, Schema, Sqlite, Statements, Store, StoreFailure, Version,
    Written,
};
use strum::VariantArray;
use tempfile::TempDir;
use transcript_ports::{Authorship, Body, Cursor, Instant, SessionId, Uuid};

#[derive(Op)]
struct Resumed;

#[derive(Op)]
struct HalfWritten;

#[derive(Op)]
struct Applied;

impl Batch for Resumed {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  cursors (session, read_to)
VALUES
  ('s', 7)
"#
        ))
    }
}

impl Batch for HalfWritten {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  cursors (session, read_to)
VALUES
  ('s', 1)
"#
        ))?;
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  cursors (session, read_to)
VALUES
  ('s', 2)
"#
        ))
    }
}

#[derive(Op)]
struct Stale;

impl Batch for Stale {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  messages (uuid, session, parent, at, author, body)
VALUES
  (
    'o1',
    's',
    NULL,
    '2026-01-01T00:00:00Z',
    'owner',
    'kept forever'
  )
"#
        ))?;
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  segments (message, ordinal, shape, body)
VALUES
  ('o1', 0, 'sentence', 'kept')
"#
        ))
    }
}

#[derive(Columns)]
struct Migrated {
    version: Version,
}

impl Query for Applied {
    type Rows = Vec<Version>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Version>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            Migrated,
            r#"
SELECT
  version AS "version!"
FROM
  _sqlx_migrations
"#
        ))?;
        Ok(rows.into_iter().map(|row| row.version).collect())
    }
}

struct Place {
    directory: TempDir,
}

impl Place {
    fn new() -> Result<Place, std::io::Error> {
        TempDir::new().map(|directory| Place { directory })
    }

    fn store(&self) -> PathBuf {
        self.directory.path().join(".meta").join("design.sqlite")
    }

    fn opened(&self) -> Sqlite {
        Sqlite::file::<StreamSchema>(self.store())
    }
}

#[test]
fn a_store_opens_lazily_and_creates_its_directory_on_the_first_op() -> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    let store = run.port(place.opened());
    assert!(place.store().metadata().is_err());
    store.write(Resumed)?;
    assert!(place.store().metadata().is_ok());
    Ok(())
}

#[test]
fn reopening_a_store_keeps_its_rows_and_runs_no_migration_or_rewrite_again() -> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    let first = run.port(place.opened());
    first.write(Resumed)?;
    first.write(Stale)?;
    let reopened = run.port(place.opened());
    let cursors = reopened.read(SessionCursors)?;
    assert_eq!(cursors.resuming(&SessionId::new("s".to_owned())), Cursor::new(Offset::new(7)));
    assert_eq!(reopened.read(Applied)?.len(), StreamSchema::migrator().iter().count());
    let shown = reopened.read(ShownMessage::named(Uuid::new("o1".to_owned())))?.ok_or("o1 was not stored")?;
    let bodies: Vec<String> = shown.segments().iter().map(|segment| Body::from(segment).to_string()).collect();
    assert_eq!(bodies, ["kept"]);
    Ok(())
}

#[test]
fn a_failing_statement_rolls_back_the_whole_batch() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    assert!(store.write(HalfWritten).is_err());
    let cursors = store.read(SessionCursors)?;
    assert_eq!(cursors.resuming(&SessionId::new("s".to_owned())), Cursor::default());
    Ok(())
}

#[test]
fn a_held_file_lock_refuses_every_other_handle_until_it_ends() -> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    let (first, second) = (run.port(place.opened()), run.port(place.opened()));
    let held = first.lock()?;
    assert!(matches!(second.lock(), Err(Busy::Taken(_))));
    held.end()?;
    assert!(second.lock().is_ok());
    Ok(())
}

#[test]
fn a_held_memory_lock_refuses_a_second_lock_until_it_is_dropped() -> Result<(), Busy> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    let held = store.lock()?;
    assert!(matches!(store.lock(), Err(Busy::Claimed)));
    drop(held);
    assert!(store.lock().is_ok());
    Ok(())
}

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq)]
#[newtype(strategy = -1.0e300..1.0e300)]
struct Score(f64);

struct Stored;

impl Stored {
    fn back<Value: Column<Raw: Clone>>(raw: &Value::Raw) -> Option<Value> {
        Value::from_raw(raw.clone()).ok()
    }

    fn unread<Named: VariantArray + Column<Raw: Clone> + PartialEq + Copy>() -> Vec<Named> {
        Named::VARIANTS.iter().copied().filter(|named| Stored::back::<Named>(&named.raw()) != Some(*named)).collect()
    }
}

laws::roundtrip!(a_node_id_column_reads_back, any::<NodeId>(), Column::raw, Stored::back::<NodeId>, Some);

laws::roundtrip!(a_title_column_reads_back, any::<Title>(), Column::raw, Stored::back::<Title>, Some);

laws::roundtrip!(a_prefix_column_reads_back, any::<Prefix>(), Column::raw, Stored::back::<Prefix>, Some);

laws::roundtrip!(
    an_ordinal_column_reads_back,
    (0..=i64::MAX.cast_unsigned()).prop_filter_map("a count", |count| usize::try_from(count).ok().map(Ordinal::new)),
    Column::raw,
    Stored::back::<Ordinal>,
    Some
);

laws::roundtrip!(a_uuid_column_reads_back, any::<Uuid>(), Column::raw, Stored::back::<Uuid>, Some);

laws::roundtrip!(a_session_column_reads_back, any::<SessionId>(), Column::raw, Stored::back::<SessionId>, Some);

laws::roundtrip!(an_instant_column_reads_back, any::<Instant>(), Column::raw, Stored::back::<Instant>, Some);

laws::roundtrip!(a_body_column_reads_back, any::<Body>(), Column::raw, Stored::back::<Body>, Some);

laws::roundtrip!(a_bytes_column_reads_back, any::<Bytes>(), Column::raw, Stored::back::<Bytes>, Some);

laws::roundtrip!(
    an_offset_column_reads_back,
    (0..=i64::MAX.cast_unsigned()).prop_map(Offset::new),
    Column::raw,
    Stored::back::<Offset>,
    Some
);

laws::roundtrip!(a_term_column_reads_back, any::<Term>(), Column::raw, Stored::back::<Term>, Some);

laws::roundtrip!(a_float_column_reads_back, any::<Score>(), Column::raw, Stored::back::<Score>, Some);

laws::roundtrip!(
    an_optional_column_reads_back,
    option::of(any::<Title>()),
    Column::raw,
    Stored::back::<Option<Title>>,
    Some
);

laws::roundtrip!(
    a_project_column_reads_back,
    "[A-Za-z0-9][A-Za-z0-9._-]{0,15}".prop_filter_map("a project name", |name| name.parse::<Project>().ok()),
    Column::raw,
    Stored::back::<Project>,
    Some
);

laws::roundtrip!(
    a_condition_column_reads_back,
    "[ -~]{0,4}[!-~][ -~]{0,20}".prop_filter_map("one line", |text| text.parse::<Condition>().ok()),
    Column::raw,
    Stored::back::<Condition>,
    Some
);

laws::roundtrip!(
    a_reason_column_reads_back,
    "[ -~]{0,4}[!-~][ -~]{0,20}".prop_filter_map("one line", |text| text.parse::<Reason>().ok()),
    Column::raw,
    Stored::back::<Reason>,
    Some
);

laws::roundtrip!(
    an_artifact_column_reads_back,
    (select(ArtifactKind::VARIANTS), "[A-Za-z0-9_][A-Za-z0-9_.:/-]{0,12}")
        .prop_filter_map("an artifact", |(kind, name)| format!("{kind}:{name}").parse::<Artifact>().ok()),
    Column::raw,
    Stored::back::<Artifact>,
    Some
);

laws::roundtrip!(
    an_authorship_column_reads_back,
    select(vec![Authorship::Owner, Authorship::Lead]),
    Column::raw,
    Stored::back::<Authorship>,
    Some
);

#[test]
fn every_named_column_reads_back_as_its_variant() {
    assert_eq!(Stored::unread::<Stage>(), []);
    assert_eq!(Stored::unread::<Role>(), []);
    assert_eq!(Stored::unread::<Facet>(), []);
    assert_eq!(Stored::unread::<Entered>(), []);
    assert_eq!(Stored::unread::<Kin>(), []);
    assert_eq!(Stored::unread::<NodeRelation>(), []);
    assert_eq!(Stored::unread::<SegmentRelation>(), []);
    assert_eq!(Stored::unread::<SpanRelation>(), []);
    assert_eq!(Stored::unread::<SenseKind>(), []);
    assert_eq!(Stored::unread::<Specificity>(), []);
    assert_eq!(Stored::unread::<Confidence>(), []);
    assert_eq!(Stored::unread::<Explicitness>(), []);
    assert_eq!(Stored::unread::<Standing>(), []);
    assert_eq!(Stored::unread::<ArtifactKind>(), []);
    assert_eq!(Stored::unread::<RunState>(), []);
}

laws::roundtrip!(a_pid_column_reads_back, any::<Pid>(), Column::raw, Stored::back::<Pid>, Some);

laws::roundtrip!(a_token_count_column_reads_back, any::<Tokens>(), Column::raw, Stored::back::<Tokens>, Some);

laws::roundtrip!(a_wall_time_column_reads_back, any::<Wall>(), Column::raw, Stored::back::<Wall>, Some);

laws::roundtrip!(
    a_provenance_column_reads_back,
    prop_oneof![
        select(Entered::VARIANTS).prop_map(Provenance::from),
        any::<i64>().prop_map(|run| Provenance::Agent(RunId::new(run))),
    ],
    Column::raw,
    Stored::back::<Provenance>,
    Some
);

laws::roundtrip!(
    a_start_time_column_reads_back,
    (select(vec!["Mon", "Sun"]), select(vec!["Jan", "Dec"]), 1..32_u8, 0..24_u8, 0..60_u8, 1970..2200_u16)
        .prop_filter_map("a start time", |(weekday, month, day, hour, minute, year)| {
            format!("{weekday} {month} {day:>2} {hour:02}:{minute:02}:{minute:02} {year}").parse::<Started>().ok()
        }),
    Column::raw,
    Stored::back::<Started>,
    Some
);

laws::law!(a_count_beyond_a_pid_is_refused_as_unfit, i64::from(u32::MAX) + 1..i64::MAX, number, {
    prop_assert!(matches!(Pid::from_raw(number), Err(ColumnFailure::Unfit(_))));
});

laws::law!(an_unknown_spelling_is_refused_as_misspelled, "[a-z]{0,3}[A-Z ]{1,3}", text, {
    prop_assert!(matches!(Stage::from_raw(text.clone()), Err(ColumnFailure::Misspelled(_))));
    prop_assert!(matches!(Authorship::from_raw(text), Err(ColumnFailure::Misspelled(_))));
});

laws::law!(a_negative_count_is_refused_as_unfit, i64::MIN..0, number, {
    prop_assert!(matches!(Ordinal::from_raw(number), Err(ColumnFailure::Unfit(_))));
    prop_assert!(matches!(Offset::from_raw(number), Err(ColumnFailure::Unfit(_))));
});
