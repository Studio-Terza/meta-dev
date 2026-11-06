use std::error::Error;
use std::path::PathBuf;

use design_store::{SessionCursors, StreamSchema};
use file_ports::Offset;
use port_core::{Op, Run};
use store_ports::{Batch, Busy, Query, Sqlite, Statements, Store, StoreFailure, Written};
use tempfile::TempDir;
use transcript_ports::{Cursor, SessionId};

#[derive(Op)]
struct Resumed;

#[derive(Op)]
struct HalfWritten;

#[derive(Op)]
struct Applied;

impl Batch for Resumed {
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!("INSERT INTO cursors (session, read_to) VALUES ('s', 7)"))
    }
}

impl Batch for HalfWritten {
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!("INSERT INTO cursors (session, read_to) VALUES ('s', 1)"))?;
        statements.written(sqlx::query!("INSERT INTO cursors (session, read_to) VALUES ('s', 2)"))
    }
}

impl Query for Applied {
    type Rows = Vec<i64>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<i64>, StoreFailure> {
        let rows = statements.rows(sqlx::query!("SELECT version AS \"version!\" FROM _sqlx_migrations"))?;
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
fn reopening_a_store_keeps_its_rows_and_runs_no_migration_again() -> Result<(), Box<dyn Error>> {
    let place = Place::new()?;
    let run = Run::root();
    run.port(place.opened()).write(Resumed)?;
    let reopened = run.port(place.opened());
    let cursors = reopened.read(SessionCursors)?;
    assert_eq!(cursors.resuming(&SessionId::new("s".to_owned())), Cursor::new(Offset::new(7)));
    assert_eq!(reopened.read(Applied)?.len(), 1);
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
