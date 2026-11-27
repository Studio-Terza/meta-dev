use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::LazyLock;
use test_failure::Failed;

use design_messages::{Fate, Halt, SessionCursors, StreamSchema};
use dispatch_ports::{Pid, Tokens, Wall};
use file_ports::Offset;
use invariants::{Ignoring, ignored};
use newtype::Enumerated;
use port_core::{Op, Run};
use proptest::option;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};
use sqlite_ports::SqliteOpener;
use store_ports::{
    Batch, Busy, Column, ColumnFailure, Columns, Execute, Migrator, Query, Schema, Store, StoreFailure, Version,
};
use tempfile::TempDir;
use transcript_ports::{Authorship, Body, Cursor, Instant, SessionId, Uuid};

ignored! {
    enum Ignored {
        Unread => "a raw value that does not read back is None, which the roundtrip reports",
    }
}

#[derive(Op)]
struct Resumed;

#[derive(Op)]
struct HalfWritten;

#[derive(Op)]
struct Applied;

impl Batch for Resumed {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
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

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
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

#[derive(Columns)]
struct Migrated {
    version: Version,
}

impl Query for Applied {
    type Rows = Vec<Version>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Version>, StoreFailure> {
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

    fn applied(&self) -> Result<BTreeSet<Version>, StoreFailure> {
        Ok(Run::root().port(self.opened_as::<Untouched>()).read(Applied)?.into_iter().collect())
    }

    fn cursor(store: &impl Store) -> Result<Cursor, Failed> {
        Ok(store.read(SessionCursors)?.resuming(&SessionId::try_new("s".to_owned())?))
    }
}

struct Older;

static OLDER: LazyLock<Migrator> = LazyLock::new(|| {
    Migrator::from(
        StreamSchema::migrator()
            .iter()
            .filter(|migration| migration.version() < Older::LAST)
            .copied()
            .collect::<Vec<_>>(),
    )
});

impl Older {
    const LAST: i64 = 20_261_126_000_004;
}

impl Schema for Older {
    fn migrator() -> &'static Migrator {
        &OLDER
    }
}

struct Untouched;

static UNTOUCHED: LazyLock<Migrator> = LazyLock::new(|| Migrator::from(Vec::new()).ignoring_missing());

impl Schema for Untouched {
    fn migrator() -> &'static Migrator {
        &UNTOUCHED
    }
}

struct Broken;

static BREAKS: LazyLock<Migrator> = LazyLock::new(|| store_ports::migrate!("tests/broken"));

static BROKEN: LazyLock<Migrator> =
    LazyLock::new(|| Migrator::from(StreamSchema::migrator().iter().chain(BREAKS.iter()).copied().collect::<Vec<_>>()));

impl Schema for Broken {
    fn migrator() -> &'static Migrator {
        &BROKEN
    }
}

#[test]
fn a_store_opens_lazily_and_creates_its_directory_on_the_first_op() -> Result<(), Failed> {
    let place = Place::new()?;
    let run = Run::root();
    let store = run.port(place.opened());
    assert!(place.store().metadata().is_err());
    store.write(Resumed)?;
    assert!(place.store().metadata().is_ok());
    Ok(())
}

#[test]
fn reopening_a_store_keeps_its_rows_and_runs_no_migration_again() -> Result<(), Failed> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened()).write(Resumed)?;
    let reopened = run.port(place.opened());
    assert_eq!(Place::cursor(&reopened)?, Cursor::new(Offset::try_new(7)?));
    assert_eq!(reopened.read(Applied)?.len(), StreamSchema::migrator().iter().count());
    Ok(())
}

#[test]
fn an_older_store_takes_the_migrations_it_lacks_and_keeps_its_rows() -> Result<(), Failed> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened_as::<Older>()).write(Resumed)?;
    let reopened = run.port(place.opened());
    assert_eq!(Place::cursor(&reopened)?, Cursor::new(Offset::try_new(7)?));
    assert_eq!(place.applied()?.len(), StreamSchema::migrator().iter().count());
    Ok(())
}

#[test]
fn a_failing_statement_rolls_back_the_whole_batch() -> Result<(), Failed> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<StreamSchema>());
    assert!(store.write(HalfWritten).is_err());
    assert_eq!(Place::cursor(&store)?, Cursor::default());
    Ok(())
}

#[test]
fn a_store_migrated_past_what_the_code_declares_is_refused_and_left_as_it_was() -> Result<(), Failed> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened()).read(Applied)?;
    let applied = place.applied()?;
    assert!(matches!(run.port(place.opened_as::<Older>()).read(Applied), Err(StoreFailure::Migrate(_))));
    assert_eq!(place.applied()?, applied);
    Ok(())
}

#[test]
fn a_failing_migration_leaves_the_store_as_it_was() -> Result<(), Failed> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened()).write(Resumed)?;
    let applied = place.applied()?;
    assert!(matches!(run.port(place.opened_as::<Broken>()).read(Applied), Err(StoreFailure::Migrate(_))));
    assert_eq!(place.applied()?, applied);
    assert_eq!(Place::cursor(&run.port(place.opened()))?, Cursor::new(Offset::try_new(7)?));
    Ok(())
}

#[test]
fn a_held_lock_refuses_other_file_handles_until_it_ends_and_a_second_memory_lock_until_dropped() -> Result<(), Failed> {
    let place = Place::new()?;
    let run = Run::root();
    let (first, second, memory) =
        (run.port(place.opened()), run.port(place.opened()), run.port(Sqlite::in_memory::<StreamSchema>()));
    let (held, claimed) = (first.lock()?, memory.lock()?);
    assert!(matches!((second.lock(), memory.lock()), (Err(Busy::Taken(_)), Err(Busy::Claimed))));
    held.end()?;
    drop(claimed);
    assert!(second.lock().is_ok() && memory.lock().is_ok());
    Ok(())
}

struct Stored;

impl Stored {
    fn back<Value: Column<Raw: Clone>>(raw: &Value::Raw) -> Option<Value> {
        Value::from_raw(raw.clone()).ignoring(Ignored::Unread)
    }

    fn unread<Named: Enumerated + Column<Raw: Clone> + PartialEq + Copy>() -> Vec<Named> {
        Named::VARIANTS.iter().copied().filter(|named| Stored::back::<Named>(&named.raw()) != Some(*named)).collect()
    }
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn every_column_reads_back_and_refuses_what_it_cannot_hold(uuid in any::<Uuid>(), session in any::<SessionId>(), instant in any::<Instant>(), body in any::<Body>(), offset in any::<Offset>(), optional in option::of(any::<Body>()), pid in any::<Pid>(), tokens in any::<Tokens>(), wall in any::<Wall>(), beyond in i64::from(u32::MAX) + 1..i64::MAX, misspelled in "[a-z]{0,3}[A-Z ]{1,3}", negative in i64::MIN..0) {
        prop_assert_eq!(Stored::back::<Uuid>(&uuid.raw()), Some(uuid));
        prop_assert_eq!(Stored::back::<SessionId>(&session.raw()), Some(session));
        prop_assert_eq!(Stored::back::<Instant>(&instant.raw()), Some(instant));
        prop_assert_eq!(Stored::back::<Body>(&body.raw()), Some(body));
        prop_assert_eq!(Stored::back::<Offset>(&offset.raw()), Some(offset));
        prop_assert_eq!(Stored::back::<Option<Body>>(&optional.raw()), Some(optional));
        prop_assert_eq!(Stored::back::<Pid>(&pid.raw()), Some(pid));
        prop_assert_eq!(Stored::back::<Tokens>(&tokens.raw()), Some(tokens));
        prop_assert_eq!(Stored::back::<Wall>(&wall.raw()), Some(wall));
        prop_assert!(matches!(Pid::from_raw(beyond), Err(ColumnFailure::Unfit(_))));
        prop_assert!(matches!(Halt::from_raw(misspelled.clone()), Err(ColumnFailure::Misspelled(_))));
        prop_assert!(matches!(Authorship::from_raw(misspelled), Err(ColumnFailure::Misspelled(_))));
        prop_assert!(matches!(Offset::from_raw(negative), Err(ColumnFailure::Refused(_))));
    }
}

#[test]
fn every_named_column_reads_back_as_its_variant() {
    assert_eq!(Stored::unread::<Halt>(), []);
    assert_eq!(Stored::unread::<Fate>(), []);
    for author in [Authorship::Owner, Authorship::Lead] {
        assert_eq!(Stored::back::<Authorship>(&author.raw()), Some(author));
    }
}

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;
