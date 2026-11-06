use std::cell::Cell;
use std::rc::Rc;

use port_core::{Entry, Held, Metered, Op, OpName, ReleaseLog, Resource};

#[derive(Debug)]
struct Lock;

#[derive(Debug)]
struct Vault;

impl Op for Lock {
    type Port = Vault;
    const NAME: OpName = OpName::new("lock");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stuck;

#[derive(Debug)]
struct Slot {
    releases: Rc<Cell<usize>>,
    outcome: Result<(), Stuck>,
}

impl Resource for Slot {
    type Failure = Stuck;

    fn release(&mut self) -> Result<(), Stuck> {
        self.releases.set(self.releases.get() + 1);
        self.outcome
    }
}

#[derive(Debug)]
struct Fixture {
    held: Held<Slot>,
    releases: Rc<Cell<usize>>,
    log: Rc<ReleaseLog<Stuck>>,
}

impl Fixture {
    fn new(outcome: Result<(), Stuck>) -> Fixture {
        let releases = Rc::new(Cell::new(0));
        let log = Rc::new(ReleaseLog::default());
        let slot = Slot { releases: Rc::clone(&releases), outcome };
        let held = Metered::new(Vault).call(|_, entry: Entry<Lock>| Held::new(entry, slot, &log));
        Fixture { held, releases, log }
    }
}

#[test]
fn dropping_releases_exactly_once() {
    let Fixture { held, releases, log } = Fixture::new(Ok(()));
    drop(held);
    assert_eq!((releases.get(), log.drain()), (1, vec![]));
}

#[test]
fn a_held_resource_is_not_released_while_held() {
    let fixture = Fixture::new(Ok(()));
    assert_eq!(fixture.releases.get(), 0);
    drop(fixture.held);
}

#[test]
fn a_failed_release_lands_in_the_log() {
    let Fixture { held, releases, log } = Fixture::new(Err(Stuck));
    drop(held);
    assert_eq!((releases.get(), log.drain()), (1, vec![Stuck]));
}

#[test]
fn the_log_is_empty_after_draining() {
    let Fixture { held, log, .. } = Fixture::new(Err(Stuck));
    drop(held);
    let first = log.drain();
    assert_eq!((first, log.drain()), (vec![Stuck], vec![]));
}

#[test]
fn an_explicit_end_is_not_repeated_on_drop() {
    let Fixture { held, releases, log } = Fixture::new(Ok(()));
    let ended = held.end();
    assert_eq!((ended, releases.get(), log.drain()), (Ok(()), 1, vec![]));
}

#[test]
fn an_explicit_end_returns_its_failure_without_logging_it() {
    let Fixture { held, releases, log } = Fixture::new(Err(Stuck));
    let ended = held.end();
    assert_eq!((ended, releases.get(), log.drain()), (Err(Stuck), 1, vec![]));
}
