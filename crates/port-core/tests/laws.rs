use std::fmt;
use std::sync::{Mutex, PoisonError};

use laws::proptest::prelude::{any, prop_assert, prop_assert_eq};
use newtype::Counter;
use port_core::{
    Calls, Counted, Held, Holds, Measured, Metered, Op, Pass, Port, Release, ReleaseFailure, Released, Resource,
    RootRefused, Rooting, Run, Snapshot, port,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Session(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Busy;

impl std::error::Error for Busy {}

impl fmt::Display for Busy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("busy")
    }
}

#[derive(Debug, Resource)]
struct SessionLock {
    session: Session,
    locks: Option<()>,
}

impl Release for SessionLock {
    type Failure = Busy;

    fn release(&mut self, _pass: Pass<'_, Released<SessionLock>>) -> Result<(), Busy> {
        self.locks.take().ok_or(Busy)
    }
}

#[port]
trait Transcripts {
    fn sessions(&self) -> Vec<Session>;
    fn read_from(&self, session: &Session, at: usize) -> Vec<String>;
    fn lock(&self, session: &Session) -> Result<Held<'_, SessionLock>, Busy>;

    fn read_all(&self) -> Vec<Vec<String>> {
        self.sessions().iter().map(|session| self.read_from(session, 0)).collect()
    }
}

trait Query: Op {
    type Answer;

    fn answered(&self, rows: &[Session]) -> Self::Answer;
}

#[derive(Op)]
struct Count;

#[derive(Op)]
struct Names;

impl Query for Count {
    type Answer = usize;

    fn answered(&self, rows: &[Session]) -> usize {
        rows.len()
    }
}

impl Query for Names {
    type Answer = Vec<String>;

    fn answered(&self, rows: &[Session]) -> Vec<String> {
        rows.iter().map(|session| session.0.clone()).collect()
    }
}

#[port]
trait Queries {
    #[op(Q)]
    fn ask<Q: Query>(&self, query: &Q) -> Q::Answer;
}

#[derive(Debug, Default)]
struct Memory {
    sessions: Mutex<Vec<Session>>,
}

impl Memory {
    fn holding(names: &[&str]) -> Memory {
        Memory { sessions: Mutex::new(names.iter().map(|name| Session((*name).to_owned())).collect()) }
    }

    fn listed(&self) -> Vec<Session> {
        self.sessions.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

impl TranscriptsFiller for Memory {
    fn sessions(&self, _pass: Pass<'_, transcripts_ops::Sessions>) -> Vec<Session> {
        self.listed()
    }

    fn read_from(&self, _pass: Pass<'_, transcripts_ops::ReadFrom>, session: &Session, at: usize) -> Vec<String> {
        vec![session.0.clone(); at.saturating_add(1)]
    }

    fn lock(&self, _pass: Pass<'_, transcripts_ops::Lock>, session: &Session) -> Result<SessionLock, Busy> {
        let held = (session.0 != "broken").then_some(());
        Ok(SessionLock { session: session.clone(), locks: held })
    }
}

impl QueriesFiller for Memory {
    fn ask<Q: Query>(&self, _pass: Pass<'_, Q>, query: &Q) -> Q::Answer {
        query.answered(&self.listed())
    }
}

struct Tally;

impl Tally {
    fn names(snapshot: &Snapshot) -> Vec<String> {
        snapshot.iter().map(|counted| counted.name().to_string()).collect()
    }

    fn named<O: Op>() -> String {
        O::NAME.to_string()
    }

    fn shared() -> Result<Run, RootRefused> {
        Run::root(Rooting::Shared)
    }

    fn over<F>(run: &Run, filler: F) -> Port<'_, F> {
        run.port(filler)
    }

    fn metered<P: Metered>(port: &P) -> &P {
        port
    }

    fn failures(failures: &[ReleaseFailure]) -> Vec<String> {
        failures.iter().map(|failure| format!("{}: {}", failure.resource(), failure.failure())).collect()
    }

    fn counts(counted: Counted) -> usize {
        counted.calls().count()
    }

    fn finished(run: Run) -> Measured {
        run.finish()
    }

    fn acquired<'port, T: Holds>(
        port: &'port Port<'_, Memory>,
        body: impl FnOnce(&Memory, Pass<'_, transcripts_ops::Lock>) -> T,
    ) -> T::Held<'port> {
        port.counted_acquire(body)
    }
}

laws::law!(an_op_is_counted_once_per_call_of_its_own, 0usize..24, calls, {
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&["a"]));
    (0..calls).for_each(|_| {
        let _ = Transcripts::sessions(&port);
    });
    drop(port);
    let measured = run.finish();
    prop_assert_eq!(measured.snapshot().calls::<transcripts_ops::Sessions>(), Calls::new(calls));
    prop_assert_eq!(measured.snapshot().calls::<transcripts_ops::ReadFrom>(), Calls::zero());
    prop_assert_eq!(measured.snapshot().iter().map(Tally::counts).sum::<usize>(), calls);
});

laws::law!(a_provided_method_counts_the_required_calls_it_makes, 0usize..8, sessions, {
    let names: Vec<String> = (0..sessions).map(|number| number.to_string()).collect();
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&names.iter().map(String::as_str).collect::<Vec<_>>()));
    prop_assert_eq!(Transcripts::read_all(Tally::metered(&port)).len(), sessions);
    drop(port);
    let measured = run.finish();
    prop_assert_eq!(measured.snapshot().calls::<transcripts_ops::Sessions>(), Calls::new(1));
    prop_assert_eq!(measured.snapshot().calls::<transcripts_ops::ReadFrom>(), Calls::new(sessions));
});

laws::law!(calls_on_scoped_threads_are_all_counted, 1usize..6, threads, {
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&["a"]));
    std::thread::scope(|scope| {
        (0..threads).for_each(|_| {
            scope.spawn(|| Transcripts::sessions(&port));
        });
    });
    drop(port);
    let measured = Tally::finished(run);
    prop_assert_eq!(measured.snapshot().calls::<transcripts_ops::Sessions>(), Calls::new(threads));
});

laws::law!(runs_never_share_their_counts, (0usize..8, 0usize..8), pair, {
    let (first, second) = (Tally::shared()?, Tally::shared()?);
    let (left, right) = (first.port(Memory::default()), second.port(Memory::default()));
    (0..pair.0).for_each(|_| {
        let _ = Transcripts::sessions(&left);
    });
    (0..pair.1).for_each(|_| {
        let _ = Transcripts::sessions(&right);
    });
    drop((left, right));
    let (counted, other) = (first.finish(), second.finish());
    prop_assert_eq!(counted.snapshot().calls::<transcripts_ops::Sessions>(), Calls::new(pair.0));
    prop_assert_eq!(other.snapshot().calls::<transcripts_ops::Sessions>(), Calls::new(pair.1));
});

laws::law!(a_snapshot_lists_ops_in_name_order, any::<(bool, bool, bool)>(), which, {
    let run = Tally::shared()?;
    let port = Tally::over(&run, Memory::holding(&["a"]));
    let session = Session("a".to_owned());
    let _ = which.0.then(|| Transcripts::read_from(&port, &session, 0));
    let _ = which.1.then(|| Transcripts::sessions(&port));
    let _ = which.2.then(|| Transcripts::lock(&port, &session));
    drop(port);
    let names = Tally::names(run.finish().snapshot());
    let mut sorted = names.clone();
    sorted.sort();
    prop_assert_eq!(names, sorted);
});

#[test]
fn op_names_are_the_module_path_then_the_trait_and_method() {
    assert_eq!(Tally::named::<transcripts_ops::Sessions>(), "laws::transcripts_ops::Transcripts::sessions");
    assert_eq!(Tally::named::<transcripts_ops::ReadFrom>(), "laws::transcripts_ops::Transcripts::read_from");
    assert_eq!(Tally::named::<transcripts_ops::Lock>(), "laws::transcripts_ops::Transcripts::lock");
}

#[test]
fn a_resource_names_itself_and_its_release_by_module_path() {
    assert_eq!(SessionLock::NAME.to_string(), "laws::SessionLock");
    assert_eq!(SessionLock::RELEASE.to_string(), "laws::SessionLock::release");
    assert_eq!(Tally::named::<Count>(), "laws::Count");
}

#[test]
fn per_type_ops_count_each_query_type_apart() -> Result<(), RootRefused> {
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&["a", "b"]));
    assert_eq!(Queries::ask(&port, &Count), 2);
    assert_eq!(Queries::ask(&port, &Count), 2);
    assert_eq!(Queries::ask(&port, &Names), ["a", "b"]);
    drop(port);
    let measured = run.finish();
    assert_eq!(measured.snapshot().calls::<Count>(), Calls::new(2));
    assert_eq!(measured.snapshot().calls::<Names>(), Calls::new(1));
    Ok(())
}

#[test]
fn acquiring_counts_the_op_and_dropping_counts_one_release() -> Result<(), RootRefused> {
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&["a"]));
    let held = Transcripts::lock(&port, &Session("a".to_owned()));
    assert_eq!(
        held.as_ref().map(|held| held.resource().map(|lock| lock.session.clone())),
        Ok(Some(Session("a".to_owned())))
    );
    drop(held);
    drop(port);
    let measured = run.finish();
    assert_eq!(measured.snapshot().calls::<transcripts_ops::Lock>(), Calls::new(1));
    assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
    assert!(measured.failures().is_empty());
    Ok(())
}

#[test]
fn ending_a_held_resource_releases_it_once_and_counts_no_second_release() -> Result<(), RootRefused> {
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&["a"]));
    let ended = Transcripts::lock(&port, &Session("a".to_owned())).map(Held::end);
    drop(port);
    let measured = run.finish();
    assert_eq!(ended, Ok(Ok(())));
    assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
    assert!(measured.failures().is_empty());
    Ok(())
}

#[test]
fn ending_a_held_resource_whose_release_fails_returns_the_failure_and_logs_none() -> Result<(), RootRefused> {
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&["broken"]));
    let ended = Transcripts::lock(&port, &Session("broken".to_owned())).map(Held::end);
    drop(port);
    let measured = run.finish();
    assert_eq!(ended, Ok(Err(Busy)));
    assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
    assert!(measured.failures().is_empty());
    Ok(())
}

#[test]
fn a_failed_release_on_drop_is_logged_in_the_measured_failures() -> Result<(), RootRefused> {
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&["broken"]));
    drop(Transcripts::lock(&port, &Session("broken".to_owned())));
    drop(port);
    let measured = run.finish();
    let logged = Tally::failures(measured.failures());
    assert_eq!(logged, ["laws::SessionLock: busy"]);
    assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
    Ok(())
}

laws::law!(a_resource_is_released_exactly_once_whether_ended_or_dropped, any::<Option<()>>(), ending, {
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&["a"]));
    let held = Transcripts::lock(&port, &Session("a".to_owned()));
    let ended = ending.map(|()| held.map(Held::end));
    prop_assert!(ended.iter().all(|result| *result == Ok(Ok(()))));
    drop(port);
    let measured = run.finish();
    prop_assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
    prop_assert!(measured.failures().is_empty());
});

#[test]
fn an_optional_acquisition_is_held_and_released_when_present() -> Result<(), RootRefused> {
    let run = Tally::shared()?;
    let port = run.port(Memory::holding(&["a"]));
    let session = Session("a".to_owned());
    let held = Tally::acquired(&port, |filler, pass| filler.lock(pass, &session).ok());
    assert_eq!(held.as_ref().and_then(Held::resource).map(|lock| lock.session.clone()), Some(session));
    drop(held);
    drop(port);
    let measured = run.finish();
    assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
    Ok(())
}
