use std::{iter, thread};

use design_messages::{SessionCursors, StreamSchema};
use dispatch_doubles::{Count, Scripted, Stepped};
use dispatch_ports::{
    AgentRun, Bell, Clock, Deadline, Dispatch, Job, Model, Pause, Pid, Prompt, Reply, Timeout, Wall, dispatch_ops,
};
use fixture::{Cases, Debugged, Failed, Fixture};
use newtype::Wrapping;
use port_core::{Counted, Op, Released, Resource, Run};
use serde::Deserialize;
use sqlite_ports::SqliteOpener;
use store_ports::{Opens, Store};

#[derive(Fixture, Deserialize, Debug)]
struct Clocked {
    start: Wall,
    pauses: Vec<Pause>,
    threads: Count,
}

#[derive(Fixture, Deserialize, Debug)]
struct Dispatched {
    start: Wall,
    timeout: Timeout,
    pid: Pid,
    reply: Reply,
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
            let clock = run.port(Stepped::starting(start, None));
            let bell = Bell::hung();
            let waited: Vec<Wall> = pauses.iter().map(|pause| clock.waited(*pause, &bell)).collect();
            let spawned = usize::try_from(threads.into_inner())?;
            thread::scope(|scope| iter::repeat_with(|| scope.spawn(|| clock.moment())).take(spawned).for_each(drop));
            (waited, clock.now())
        };
        let counted: Vec<Counted> = run.finish().iter().collect();
        Ok(Debugged((walls, counted)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn a_held_resource_is_released_once_when_dropped() -> Result<(), Failed> {
    let drifted = Cases::<Dispatched>::drifted(|case| {
        let Dispatched { start, timeout, pid, reply } = case.into_row();
        let run = Run::root();
        let held = {
            let dispatch = run.port(Scripted::replying(pid, reply.clone(), None));
            let job = Job::due_by(Deadline::after(start, timeout), Model::Haiku);
            let ended = dispatch.dispatch(&job, &Prompt::from(&reply))?;
            let first = ended.pid();
            drop(ended);
            let dropped = dispatch.dispatch(&job, &Prompt::from(&reply))?.pid();
            (first, dropped)
        };
        let names = [AgentRun::NAME, Released::<AgentRun>::NAME, dispatch_ops::Dispatch::NAME];
        let counted: Vec<Counted> = run.finish().iter().collect();
        Ok(Debugged((held, names, counted)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

#[test]
fn a_per_type_op_counts_under_its_own_name_and_a_held_lock_refuses_a_second() -> Result<(), Failed> {
    let drifted = Cases::<Stored>::drifted(|case| {
        let run = Run::root();
        let observed = {
            let store = run.port(SqliteOpener.in_memory::<StreamSchema>());
            let reads = usize::try_from(case.row().reads.into_inner())?;
            let cursors =
                iter::repeat_with(|| store.read(SessionCursors)).take(reads).collect::<Result<Vec<_>, _>>()?;
            let held = store.lock()?;
            let refused = store.lock().map(|_second| ());
            drop(held);
            let again = store.lock().map(|_third| ());
            (cursors, refused, again)
        };
        let counted: Vec<Counted> = run.finish().iter().collect();
        Ok(Debugged((observed, SessionCursors::NAME, counted)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
