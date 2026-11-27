use std::{iter, thread};

use design_messages::{SessionCursors, StreamSchema};
use dispatch_doubles::{Count, Stepped};
use dispatch_ports::{Clock, Pause, Wall};
use fixture::{Cases, Debugged, Failed, Fixture};
use newtype::Wrapping;
use port_core::{Counted, Op, Released, Resource, Run};
use serde::Deserialize;
use sqlite_ports::SqliteOpener;
use store_ports::{Opens, Store, StoreLock};

#[derive(Fixture, Deserialize, Debug)]
struct Clocked {
    start: Wall,
    pauses: Vec<Pause>,
    threads: Count,
}

#[derive(Fixture, Deserialize, Debug)]
struct Stored {
    reads: Count,
}

#[test]
fn every_call_counts_once_under_its_op_in_name_order_across_threads() -> Result<(), Failed> {
    let drifted = Cases::<Clocked>::drifted(|case| {
        let Clocked { start, pauses, threads } = case.into_row();
        let run = Run::root();
        let walls = {
            let clock = run.port(Stepped::starting(start));
            let paused: Vec<Wall> = pauses.iter().map(|pause| clock.paused(*pause)).collect();
            thread::scope(|scope| {
                iter::repeat_with(|| scope.spawn(|| clock.now())).take(threads.into_inner()).for_each(drop);
            });
            (paused, clock.now())
        };
        let counted: Vec<Counted> = run.finish().iter().collect();
        Ok(Debugged((walls, counted)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn a_per_type_op_counts_under_its_own_name_and_a_held_lock_is_released_once_when_dropped() -> Result<(), Failed> {
    let drifted = Cases::<Stored>::drifted(|case| {
        let run = Run::root();
        let observed = {
            let store = run.port(SqliteOpener.in_memory::<StreamSchema>());
            let reads = case.row().reads.into_inner();
            let cursors =
                iter::repeat_with(|| store.read(SessionCursors)).take(reads).collect::<Result<Vec<_>, _>>()?;
            let held = store.lock()?;
            let refused = store.lock().map(|_second| ());
            drop(held);
            let again = store.lock().map(|_third| ());
            (cursors, refused, again)
        };
        let names = [SessionCursors::NAME, StoreLock::NAME, Released::<StoreLock>::NAME];
        let counted: Vec<Counted> = run.finish().iter().collect();
        Ok(Debugged((observed, names, counted)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
