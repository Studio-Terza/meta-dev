use std::num::NonZeroU32;
use std::path::PathBuf;
use test_failure::Failed;

use design_messages::{CheckName, PullNumber, SessionCursors, StreamSchema};
use file_ports::Offset;
use invariants::{Ignoring, ignored};
use port_core::{Op, Run};
use proptest::option;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::test_runner::{Config, RngSeed};
use sqlite_ports::SqliteOpener;
use store_ports::{Batch, Busy, Column, ColumnFailure, Execute, Opens, Schema, Store, StoreFailure};
use tempfile::TempDir;
use transcript_ports::{Authorship, Body, Cursor, Instant, SessionId, TaskId, Uuid};

ignored! {
    enum Ignored {
        Unread => "a raw value that does not read back is None, which the roundtrip reports",
    }
}

#[derive(Op)]
struct Resumed;

#[derive(Op)]
struct HalfWritten;

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

struct Altered;

impl Schema for Altered {
    const SQL: &'static str = include_str!("altered.sql");
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
        SqliteOpener.file::<S>(self.store())
    }

    fn cursor(store: &impl Store) -> Result<Cursor, Failed> {
        Ok(store.read(SessionCursors)?.resuming(&SessionId::try_new("s".to_owned())?))
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
    assert!(place.store().with_file_name("design.sqlite-wal").metadata().is_ok());
    Ok(())
}

#[test]
fn reopening_a_store_under_its_own_schema_keeps_its_rows() -> Result<(), Failed> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened()).write(Resumed)?;
    let reopened = run.port(place.opened());
    assert_eq!(Place::cursor(&reopened)?, Cursor::new(Offset::try_new(7)?));
    Ok(())
}

#[test]
fn a_store_built_for_another_schema_is_deleted_and_rebuilt_empty() -> Result<(), Failed> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened()).write(Resumed)?;
    let altered = run.port(place.opened_as::<Altered>());
    assert_eq!(Place::cursor(&altered)?, Cursor::default());
    altered.write(Resumed)?;
    let restored = run.port(place.opened());
    assert_eq!(Place::cursor(&restored)?, Cursor::default());
    Ok(())
}

#[test]
fn a_failing_statement_rolls_back_the_whole_batch() -> Result<(), Failed> {
    let run = Run::root();
    let store = run.port(SqliteOpener.in_memory::<StreamSchema>());
    assert!(store.write(HalfWritten).is_err());
    assert_eq!(Place::cursor(&store)?, Cursor::default());
    Ok(())
}

#[test]
fn a_held_lock_refuses_other_file_handles_until_it_ends_and_a_second_memory_lock_until_dropped() -> Result<(), Failed> {
    let place = Place::new()?;
    let run = Run::root();
    let (first, second, memory) =
        (run.port(place.opened()), run.port(place.opened()), run.port(SqliteOpener.in_memory::<StreamSchema>()));
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
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn every_column_reads_back_and_refuses_what_it_cannot_hold(uuid in any::<Uuid>(), session in any::<SessionId>(), instant in any::<Instant>(), body in any::<Body>(), offset in any::<Offset>(), optional in option::of(any::<Body>()), task in any::<TaskId>(), misspelled in "[a-z]{0,3}[A-Z ]{1,3}", negative in i64::MIN..0) {
        prop_assert_eq!(Stored::back::<Uuid>(&uuid.raw()), Some(uuid));
        prop_assert_eq!(Stored::back::<SessionId>(&session.raw()), Some(session));
        prop_assert_eq!(Stored::back::<Instant>(&instant.raw()), Some(instant));
        prop_assert_eq!(Stored::back::<Body>(&body.raw()), Some(body));
        prop_assert_eq!(Stored::back::<Offset>(&offset.raw()), Some(offset));
        prop_assert_eq!(Stored::back::<Option<Body>>(&optional.raw()), Some(optional));
        prop_assert_eq!(Stored::back::<TaskId>(&task.raw()), Some(task));
        prop_assert!(matches!(Authorship::from_raw(misspelled), Err(ColumnFailure::Misspelled(_))));
        prop_assert!(matches!(Offset::from_raw(negative), Err(ColumnFailure::Refused(_))));
    }
}

#[test]
fn every_named_column_reads_back_as_its_variant() {
    for author in [Authorship::Owner, Authorship::Lead] {
        assert_eq!(Stored::back::<Authorship>(&author.raw()), Some(author));
    }
}

#[test]
fn a_pull_number_and_a_check_name_read_back_and_refuse_zero_blank_and_control_characters() -> Result<(), Failed> {
    let pull = NonZeroU32::new(u32::MAX).map(PullNumber::new);
    let check = CheckName::try_new("CI / test (macos)".to_owned())?;
    assert_eq!(pull.and_then(|number| Stored::back::<PullNumber>(&number.raw())), pull);
    assert_eq!(Stored::back::<CheckName>(&check.raw()), Some(check));
    assert!(matches!(PullNumber::from_raw(0), Err(ColumnFailure::Unfit(_))));
    assert!(matches!(PullNumber::from_raw(i64::from(u32::MAX) + 1), Err(ColumnFailure::Unfit(_))));
    for refused in ["", "ci\ntest", "\u{7f}"] {
        assert!(matches!(CheckName::from_raw(refused.to_owned()), Err(ColumnFailure::Refused(_))), "{refused:?}");
    }
    Ok(())
}

type Sqlite = <SqliteOpener as Opens>::Store;
