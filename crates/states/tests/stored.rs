use std::sync::LazyLock;

use newtype::Enumerated;
use port_core::{Op, Port, Run};
use sqlite_ports::SqliteOpener;
use store_ports::{
    Batch, Column, Execute, IntoArgument, Migrator, Prepared, Schema, Staged, Store, StoreFailure, Version, Written,
};

states::states! {
    pub stored Door in door {
        start Shut as Leaving { open -> Opened, lock -> Locked }
        state Opened { close -> Shut }
        terminal Locked {}
    }
}

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;

struct Bare;

static BARE: LazyLock<Migrator> = LazyLock::new(|| Migrator::from(Vec::new()).ignoring_missing());

#[derive(Op)]
enum Rewriteless {}

impl Batch for Rewriteless {
    type Failure = StoreFailure;

    fn written(self, _statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        match self {}
    }
}

impl Schema for Bare {
    type Rewrite = Rewriteless;

    fn migrator() -> &'static Migrator {
        &BARE
    }

    fn rewrite(_pending: Version) -> Option<Rewriteless> {
        None
    }
}

impl Staged for Door {
    type Key = i64;

    type State = Door;

    fn moved(key: &i64, from: Door, to: Door) -> Prepared<()> {
        Prepared::written(
            "UPDATE doors SET state = ?3 WHERE id = ?1 AND state = ?2",
            vec![key.into_argument(), from.raw().into_argument(), to.raw().into_argument()],
        )
    }
}

struct Doors;

impl Doors {
    const ONLY: i64 = 1;

    fn placed(store: &Port<'_, Sqlite>, state: Door) -> Result<i64, laws::Unmet> {
        store.write(Prepared::written(
            "CREATE TABLE IF NOT EXISTS doors (id INTEGER PRIMARY KEY, state TEXT NOT NULL)",
            Vec::new(),
        ))?;
        store.write(Prepared::written("DELETE FROM doors", Vec::new()))?;
        store.write(Prepared::written(
            "INSERT INTO doors (id, state) VALUES (?1, ?2)",
            vec![Doors::ONLY.into_argument(), state.raw().into_argument()],
        ))?;
        Ok(Doors::ONLY)
    }
}

laws::lifecycle!(
    a_stored_lifecycle_moves_only_along_its_declared_edges,
    Door,
    Sqlite::in_memory::<Bare>(),
    Doors::placed
);

laws::table!(
    a_stored_state_is_spelled_as_its_name_in_snake_case,
    |state: &Door| state.to_string(),
    [Door::Shut => "shut", Door::Opened => "opened", Door::Locked => "locked"]
);

#[test]
fn every_declared_state_is_stored_and_read_back_by_its_spelling() {
    assert_eq!(Door::VARIANTS, [Door::Shut, Door::Opened, Door::Locked]);
    assert!(Door::VARIANTS.iter().all(|state| state.to_string().parse::<Door>().is_ok_and(|read| read == *state)));
}

#[test]
fn a_held_state_moves_its_row_and_a_stale_one_is_refused() -> Result<(), laws::Unmet> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<Bare>());
    let key = Doors::placed(&store, Door::Shut)?;
    store.write(door::Shut::held(key).open())?;
    let stale = store.write(door::Shut::held(key).lock());
    assert!(matches!(stale, Err(StoreFailure::Unmoved(_))));
    store.write(door::Opened::held(key).close())?;
    store.write(door::Shut::held(key).lock())?;
    assert_eq!(*door::Opened::held(key).key(), key);
    Ok(())
}

#[test]
fn a_held_state_moves_to_a_declared_target_chosen_at_runtime() -> Result<(), laws::Unmet> {
    let run = Run::root();
    let store = run.port(Sqlite::in_memory::<Bare>());
    let chosen = [door::Leaving::Opened, door::Leaving::Locked];
    assert_eq!(chosen.map(Door::from), [Door::Opened, Door::Locked]);
    let key = Doors::placed(&store, Door::Shut)?;
    store.write(door::Shut::held(key).moved(door::Leaving::Opened))?;
    let stale = store.write(door::Shut::held(key).moved(door::Leaving::Locked));
    assert!(matches!(stale, Err(StoreFailure::Unmoved(_))));
    store.write(door::Opened::held(key).close())?;
    store.write(door::Shut::held(key).moved(door::Leaving::Locked))?;
    Ok(())
}
