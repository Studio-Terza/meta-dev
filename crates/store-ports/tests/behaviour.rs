use std::collections::BTreeSet;
use std::error::Error;
use std::path::PathBuf;
use std::sync::LazyLock;

use design_model::{
    Artifact, ArtifactKind, Condition, Confidence, Entered, Explicitness, Facet, Kin, NodeId, NodeRelation, Ordinal,
    Prefix, Project, Provenance, Reason, Role, RunId, SegmentRelation, SenseKind, SpanRelation, Specificity, Stage,
    Standing, Title,
};
use design_store::{RelationName, RunState, SessionCursors, ShapeName, ShownMessage, StreamSchema};
use dispatch_ports::{Pid, Started, Tokens, Wall};
use file_ports::{Bytes, Offset};
use grammar::search::Term;
use laws::proptest::option;
use laws::proptest::prelude::{any, prop_assert, prop_oneof};
use laws::proptest::sample::select;
use laws::proptest::strategy::Strategy;
use newtype::{Arbitrary, Counter, Newtype};
use port_core::{Op, Run};
use store_ports::{
    Batch, Busy, Column, ColumnFailure, Columns, Migrator, Query, Schema, Sqlite, Statements, Store, StoreFailure,
    Version, Written,
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
        self.opened_as::<StreamSchema>()
    }

    fn opened_as<S: Schema>(&self) -> Sqlite {
        Sqlite::file::<S>(self.store())
    }

    fn bodies(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(self.opened_as::<Untouched>());
        let shown = store.read(ShownMessage::named(Uuid::new("o1".to_owned())))?.ok_or("o1 was not stored")?;
        Ok(shown.segments().iter().map(|segment| Body::from(segment).to_string()).collect())
    }

    const REFERENCED: &str = "the store's rewrite refuses to open it: re-deriving the stored segments would change these segments that stored rows refer to: o1#0";

    fn shown(&self, message: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(self.opened_as::<Untouched>());
        let shown =
            store.read(ShownMessage::named(Uuid::new(message.to_owned())))?.ok_or("the message is not stored")?;
        Ok(shown
            .segments()
            .iter()
            .map(|segment| {
                format!("{} {} {}", segment.ordinal(), AsRef::<ShapeName>::as_ref(segment), Body::from(segment))
            })
            .collect())
    }

    fn refusal(remark: Remark) -> Result<Refusal, Box<dyn Error>> {
        let place = Place::new()?;
        let run = Run::root();
        let older = run.port(place.opened_as::<Older>());
        older.write(Told::Stale)?;
        older.write(Remarked { remark, on: Uuid::new("o1".to_owned()) })?;
        let refused = run.port(place.opened()).read(Applied).err().ok_or("the open was not refused")?;
        let refused = matches!(&refused, StoreFailure::Refused(_)).then(|| refused.to_string());
        Ok(Refusal { remark, refused, applied: place.applied()?, stored: place.shown("o1")? })
    }

    fn refused(remark: Remark) -> Refusal {
        let stored = vec!["0 question First point. Second point.".to_owned()];
        Refusal { remark, refused: Some(Place::REFERENCED.to_owned()), applied: Older::versions(), stored }
    }

    fn linked(&self, message: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(self.opened_as::<Untouched>());
        let shown =
            store.read(ShownMessage::named(Uuid::new(message.to_owned())))?.ok_or("the message is not stored")?;
        Ok(shown
            .links()
            .iter()
            .map(|link| {
                let [source, target] = link.anchors();
                format!("{source} {} {target}", AsRef::<RelationName>::as_ref(link))
            })
            .collect())
    }

    fn applied(&self) -> Result<BTreeSet<Version>, Box<dyn Error>> {
        Ok(Run::root().port(self.opened_as::<Untouched>()).read(Applied)?.into_iter().collect())
    }
}

#[derive(Op)]
enum Rewriteless {}

impl Batch for Rewriteless {
    type Failure = StoreFailure;

    fn written(self, _statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        match self {}
    }
}

struct Older;

static OLDER: LazyLock<Migrator> = LazyLock::new(|| {
    Migrator::with_migrations(
        StreamSchema::migrator().iter().filter(|migration| migration.version < Older::RESEGMENTED).cloned().collect(),
    )
});

impl Older {
    const RESEGMENTED: i64 = 20_261_108_000_012;

    fn versions() -> BTreeSet<Version> {
        OLDER.iter().map(|migration| Version::new(migration.version)).collect()
    }
}

impl Schema for Older {
    type Rewrite = Rewriteless;

    fn migrator() -> &'static Migrator {
        &OLDER
    }

    fn rewrite(_pending: Version) -> Option<Rewriteless> {
        None
    }
}

struct Untouched;

static UNTOUCHED: LazyLock<Migrator> = LazyLock::new(|| {
    let mut migrator = Migrator::with_migrations(Vec::new());
    migrator.set_ignore_missing(true);
    migrator
});

impl Schema for Untouched {
    type Rewrite = Rewriteless;

    fn migrator() -> &'static Migrator {
        &UNTOUCHED
    }

    fn rewrite(_pending: Version) -> Option<Rewriteless> {
        None
    }
}

struct Faulted;

#[derive(Op)]
struct Crashing(<StreamSchema as Schema>::Rewrite);

impl Schema for Faulted {
    type Rewrite = Crashing;

    fn migrator() -> &'static Migrator {
        StreamSchema::migrator()
    }

    fn rewrite(pending: Version) -> Option<Crashing> {
        StreamSchema::rewrite(pending).map(Crashing)
    }
}

impl Batch for Crashing {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        self.0.written(statements)?;
        HalfWritten.written(statements)
    }
}

struct Broken;

static BREAKS: Migrator = store_ports::migrate!("tests/broken");

static BROKEN: LazyLock<Migrator> = LazyLock::new(|| {
    Migrator::with_migrations(StreamSchema::migrator().iter().chain(BREAKS.iter()).cloned().collect())
});

impl Schema for Broken {
    type Rewrite = <StreamSchema as Schema>::Rewrite;

    fn migrator() -> &'static Migrator {
        &BROKEN
    }

    fn rewrite(pending: Version) -> Option<Self::Rewrite> {
        StreamSchema::rewrite(pending)
    }
}

struct Early;

#[derive(Op)]
struct Staged;

impl Schema for Early {
    type Rewrite = Staged;

    fn migrator() -> &'static Migrator {
        StreamSchema::migrator()
    }

    fn rewrite(pending: Version) -> Option<Staged> {
        StreamSchema::migrator()
            .iter()
            .next()
            .is_some_and(|first| Version::new(first.version) == pending)
            .then_some(Staged)
    }
}

impl Batch for Staged {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  cursors (session, read_to)
SELECT
  'staged',
  7 + count(stage_provenance)
FROM
  nodes
"#
        ))
    }
}

#[derive(Op)]
struct Edited;

impl Batch for Edited {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(store_ports::query!(
            r#"
UPDATE
  _sqlx_migrations
SET
  checksum = X'00'
WHERE
  version = 20261107000002
"#
        ))
    }
}

#[derive(Op)]
struct Annotated;

impl Batch for Annotated {
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
  ('o1', 0, 'sentence', 'kept forever'),
  ('o1', 1, 'sentence', 'gone')
"#
        ))?;
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  conversational (message, ordinal, provenance)
VALUES
  ('o1', 1, 'init')
"#
        ))
    }
}

#[derive(Op, Clone, Copy, Debug)]
enum Told {
    Stale,
    Current,
}

impl Batch for Told {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let context = Told::context(statements)?;
        let told = self.segments(statements)?;
        Ok(context.plus(told))
    }
}

impl Told {
    fn context(statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let messages = statements.written(store_ports::query!(
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
    'First point. Second point.'
  ),
  (
    'o2',
    's',
    NULL,
    '2026-01-01T00:00:01Z',
    'owner',
    'Only one.'
  )
"#
        ))?;
        let segment = statements.written(store_ports::query!(
            r#"
INSERT INTO
  segments (message, ordinal, shape, body)
VALUES
  ('o2', 0, 'sentence', 'Only one.')
"#
        ))?;
        Ok(messages.plus(segment))
    }

    fn segments(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        match self {
            Told::Stale => statements.written(store_ports::query!(
                r#"
INSERT INTO
  segments (message, ordinal, shape, body)
VALUES
  (
    'o1',
    0,
    'question',
    'First point. Second point.'
  )
"#
            )),
            Told::Current => statements.written(store_ports::query!(
                r#"
INSERT INTO
  segments (message, ordinal, shape, body)
VALUES
  ('o1', 0, 'sentence', 'First point.'),
  ('o1', 1, 'sentence', 'Second point.')
"#
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, VariantArray)]
enum Remark {
    Conversational,
    Sense,
    SpanEdge,
    LinkFrom,
    LinkTo,
    Flailed,
    Cites,
    Answered,
}

#[derive(Debug, PartialEq)]
struct Refusal {
    remark: Remark,
    refused: Option<String>,
    applied: BTreeSet<Version>,
    stored: Vec<String>,
}

#[derive(Op, Clone, Debug)]
struct Remarked {
    remark: Remark,
    on: Uuid,
}

impl Batch for Remarked {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        self.remark.written(&self.on, statements)
    }
}

impl Remark {
    fn written(self, on: &Uuid, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        match self {
            Remark::Conversational => statements.written(store_ports::query!(
                r#"
INSERT INTO
  conversational (message, ordinal, provenance)
VALUES
  (?1, 0, 'init')
"#,
                on: Uuid
            )),
            Remark::Sense => statements.written(store_ports::query!(
                r#"
INSERT INTO
  senses (
    message,
    ordinal,
    kind,
    specificity,
    confidence,
    explicitness,
    provenance
  )
VALUES
  (
    ?1,
    0,
    'question',
    'general',
    'firm',
    'explicit',
    'init'
  )
"#,
                on: Uuid
            )),
            Remark::SpanEdge => Remarked::spanned(on, statements),
            Remark::LinkFrom => statements.written(store_ports::query!(
                r#"
INSERT INTO
  segment_links (
    source_message,
    source_ordinal,
    relation,
    target_message,
    target_ordinal,
    provenance
  )
VALUES
  (?1, 0, 'answers', 'o2', 0, 'init')
"#,
                on: Uuid
            )),
            Remark::LinkTo => statements.written(store_ports::query!(
                r#"
INSERT INTO
  segment_links (
    source_message,
    source_ordinal,
    relation,
    target_message,
    target_ordinal,
    provenance
  )
VALUES
  ('o2', 0, 'answers', ?1, 0, 'init')
"#,
                on: Uuid
            )),
            Remark::Flailed => Remarked::flailed(on, statements),
            Remark::Cites => statements.written(store_ports::query!(
                r#"
INSERT INTO
  mechanical_links (
    source,
    relation,
    target,
    source_message,
    target_message
  )
VALUES
  (?1 || '#0', 'cites', 'o2', ?1, 'o2')
"#,
                on: Uuid
            )),
            Remark::Answered => statements.written(store_ports::query!(
                r#"
INSERT INTO
  mechanical_links (
    source,
    relation,
    target,
    source_message,
    target_message
  )
VALUES
  ('o2#0', 'answers', ?1 || '#0', 'o2', ?1)
"#,
                on: Uuid
            )),
        }
    }
}

impl Remarked {
    fn spanned(on: &Uuid, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let node = statements.written(store_ports::query!(
            r#"
INSERT INTO
  nodes (id, project, facet, stage, provenance)
VALUES
  (1, 'p', 'problem', 'vague', 'init')
"#
        ))?;
        let edge = statements.written(store_ports::query!(
            r#"
INSERT INTO
  span_edges (message, ordinal, relation, node, provenance)
VALUES
  (?1, 0, 'decides', 1, 'init')
"#,
            on: Uuid
        ))?;
        Ok(node.plus(edge))
    }

    fn flailed(on: &Uuid, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let job = statements.written(store_ports::query!(
            r#"
INSERT INTO
  jobs (id)
VALUES
  (1)
"#
        ))?;
        let ran = statements.written(store_ports::query!(
            r#"
INSERT INTO
  runs (id, job, pid, deadline, state)
VALUES
  (1, 1, 1, 1, 'replied')
"#
        ))?;
        let flailed = statements.written(store_ports::query!(
            r#"
INSERT INTO
  flailed (message, ordinal, run)
VALUES
  (?1, 0, 1)
"#,
            on: Uuid
        ))?;
        Ok(job.plus(ran).plus(flailed))
    }
}

struct Glossed;

static GLOSSES: Migrator = store_ports::migrate!("tests/glossed");

static GLOSSED: LazyLock<Migrator> = LazyLock::new(|| {
    Migrator::with_migrations(StreamSchema::migrator().iter().chain(GLOSSES.iter()).cloned().collect())
});

impl Schema for Glossed {
    type Rewrite = <StreamSchema as Schema>::Rewrite;

    fn migrator() -> &'static Migrator {
        &GLOSSED
    }

    fn rewrite(pending: Version) -> Option<Self::Rewrite> {
        StreamSchema::rewrite(pending)
    }
}

#[derive(Op)]
struct Cited;

impl Cited {
    const CITED: &str = "00000000-0000-4000-8000-000000000001";
    const CITING: &str = "00000000-0000-4000-8000-000000000002";
}

impl Batch for Cited {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let messages = statements.written(store_ports::query!(
            r#"
INSERT INTO
  messages (uuid, session, parent, at, author, body)
VALUES
  (
    '00000000-0000-4000-8000-000000000001',
    's',
    NULL,
    '2026-01-01T00:00:00Z',
    'owner',
    'Only one.'
  ),
  (
    '00000000-0000-4000-8000-000000000002',
    's',
    '00000000-0000-4000-8000-000000000001',
    '2026-01-01T00:00:01Z',
    'owner',
    'First point. See 00000000-0000-4000-8000-000000000001.'
  )
"#
        ))?;
        let segments = statements.written(store_ports::query!(
            r#"
INSERT INTO
  segments (message, ordinal, shape, body)
VALUES
  (
    '00000000-0000-4000-8000-000000000001',
    0,
    'sentence',
    'Only one.'
  ),
  (
    '00000000-0000-4000-8000-000000000002',
    0,
    'question',
    'First point. See 00000000-0000-4000-8000-000000000001.'
  )
"#
        ))?;
        let link = statements.written(store_ports::query!(
            r#"
INSERT INTO
  mechanical_links (
    source,
    relation,
    target,
    source_message,
    target_message
  )
VALUES
  (
    '00000000-0000-4000-8000-000000000002#0',
    'cites',
    '00000000-0000-4000-8000-000000000001',
    '00000000-0000-4000-8000-000000000002',
    '00000000-0000-4000-8000-000000000001'
  )
"#
        ))?;
        Ok(messages.plus(segments).plus(link))
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
fn a_store_whose_applied_migration_was_edited_is_refused_before_any_rewrite_and_left_as_it_was()
-> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    let older = run.port(place.opened_as::<Older>());
    older.write(Stale)?;
    older.write(Edited)?;
    assert!(matches!(run.port(place.opened()).read(Applied), Err(StoreFailure::Migrate(_))));
    assert_eq!(place.applied()?, Older::versions());
    assert_eq!(place.bodies()?, ["kept"]);
    Ok(())
}

#[test]
fn a_crash_after_the_migrations_and_the_rewrite_leaves_the_store_as_it_was_and_the_next_open_does_both()
-> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened_as::<Older>()).write(Stale)?;
    assert!(run.port(place.opened_as::<Faulted>()).read(Applied).is_err());
    assert_eq!(place.applied()?, Older::versions());
    assert_eq!(place.bodies()?, ["kept"]);
    let reopened = run.port(place.opened());
    let cursors = reopened.read(SessionCursors)?;
    assert_eq!(cursors.resuming(&SessionId::new("s".to_owned())), Cursor::default());
    assert_eq!(reopened.read(Applied)?.len(), StreamSchema::migrator().iter().count());
    assert_eq!(place.bodies()?, ["kept forever"]);
    Ok(())
}

#[test]
fn a_migration_failing_after_a_due_rewrite_leaves_the_store_as_it_was() -> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened_as::<Older>()).write(Stale)?;
    assert!(matches!(run.port(place.opened_as::<Broken>()).read(Applied), Err(StoreFailure::Migrate(_))));
    assert_eq!(place.applied()?, Older::versions());
    assert_eq!(place.bodies()?, ["kept"]);
    Ok(())
}

#[test]
fn a_rederivation_that_would_drop_an_annotated_segment_refuses_and_leaves_the_store_as_it_was()
-> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened_as::<Older>()).write(Annotated)?;
    let refused = run.port(place.opened()).read(Applied).err().ok_or("the open was not refused")?;
    assert!(matches!(&refused, StoreFailure::Refused(_)));
    assert_eq!(
        refused.to_string(),
        "the store's rewrite refuses to open it: re-deriving the stored segments would change these segments that stored rows refer to: o1#1"
    );
    assert_eq!(place.applied()?, Older::versions());
    assert_eq!(place.bodies()?, ["kept forever", "gone"]);
    Ok(())
}

#[test]
fn a_rederivation_that_would_change_any_annotated_segment_refuses_naming_it_and_leaves_the_store_as_it_was()
-> Result<(), Box<dyn Error>> {
    let refusals = Remark::VARIANTS.iter().map(|remark| Place::refusal(*remark)).collect::<Result<Vec<_>, _>>()?;
    let refused: Vec<Refusal> = Remark::VARIANTS.iter().map(|remark| Place::refused(*remark)).collect();
    assert_eq!(refusals, refused);
    Ok(())
}

#[test]
fn a_rederivation_rewrites_a_changed_segment_that_carries_no_annotation() -> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    let older = run.port(place.opened_as::<Older>());
    older.write(Told::Stale)?;
    older.write(Remarked { remark: Remark::Conversational, on: Uuid::new("o2".to_owned()) })?;
    assert_eq!(run.port(place.opened()).read(Applied)?.len(), StreamSchema::migrator().iter().count());
    assert_eq!(place.shown("o1")?, ["0 sentence First point.", "1 sentence Second point."]);
    assert_eq!(place.shown("o2")?, ["0 sentence Only one."]);
    Ok(())
}

#[test]
fn a_rederivation_opens_a_store_whose_annotated_segments_it_does_not_change() -> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    let older = run.port(place.opened_as::<Older>());
    older.write(Told::Current)?;
    let remarks = Remark::VARIANTS.iter().map(|remark| Remarked { remark: *remark, on: Uuid::new("o1".to_owned()) });
    remarks.map(|remarked| older.write(remarked)).collect::<Result<Vec<Written>, StoreFailure>>()?;
    assert_eq!(run.port(place.opened()).read(Applied)?.len(), StreamSchema::migrator().iter().count());
    assert_eq!(place.shown("o1")?, ["0 sentence First point.", "1 sentence Second point."]);
    Ok(())
}

#[test]
fn a_rederivation_that_would_renumber_a_cited_segment_refuses_and_leaves_the_link_on_the_text_it_was_made_on()
-> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened_as::<Older>()).write(Cited)?;
    let refused = run.port(place.opened()).read(Applied).err().ok_or("the open was not refused")?;
    assert!(matches!(&refused, StoreFailure::Refused(_)));
    assert_eq!(
        refused.to_string(),
        format!(
            "the store's rewrite refuses to open it: re-deriving the stored segments would change these segments that stored rows refer to: {}#0",
            Cited::CITING
        )
    );
    assert_eq!(place.applied()?, Older::versions());
    assert_eq!(place.shown(Cited::CITING)?, [format!("0 question First point. See {}.", Cited::CITED)]);
    assert_eq!(place.linked(Cited::CITING)?, [format!("{}#0 cites {}", Cited::CITING, Cited::CITED)]);
    Ok(())
}

#[test]
fn a_rederivation_refuses_to_change_a_segment_that_a_table_added_later_refers_to_by_foreign_key()
-> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened_as::<Older>()).write(Told::Stale)?;
    let refused = run.port(place.opened_as::<Glossed>()).read(Applied).err().ok_or("the open was not refused")?;
    assert!(matches!(&refused, StoreFailure::Refused(_)));
    assert_eq!(refused.to_string(), Place::REFERENCED);
    assert_eq!(place.applied()?, Older::versions());
    assert_eq!(place.shown("o1")?, ["0 question First point. Second point."]);
    Ok(())
}

#[test]
fn a_rewrite_due_with_the_first_migration_runs_on_the_schema_its_queries_are_checked_against()
-> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<Early>());
    let cursors = store.read(SessionCursors)?;
    assert_eq!(cursors.resuming(&SessionId::new("staged".to_owned())), Cursor::new(Offset::new(7)));
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
