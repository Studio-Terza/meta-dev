use std::sync::{Mutex, PoisonError};

use laws::proptest::prelude::{any, prop_assert, prop_assert_eq};
use laws::proptest::sample::select;
use newtype::Counter;
use port_core::{
    Calls, Counted, Held, Measured, Metered, Op, Pass, Port, Release, ReleaseReceipt, Released, Resource, Run,
    Snapshot, port,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Session(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the session lock was already gone")]
struct Gone;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lockable {
    Intact,
    Broken,
}

#[derive(Debug, Resource)]
struct SessionLock {
    lockable: Lockable,
}

impl Release for SessionLock {
    type Failure = Gone;

    fn release(&mut self, pass: Pass<'_, Released<SessionLock>>) -> Result<ReleaseReceipt, Gone> {
        (self.lockable == Lockable::Intact).then_some(pass).map(Pass::released).ok_or(Gone)
    }
}

#[port]
trait Transcripts {
    fn sessions(&self) -> Vec<Session>;
    fn read_from(&self, session: &Session, at: usize) -> Vec<String>;
    fn lock(&self, session: &Session) -> Result<Held<'_, SessionLock>, Gone>;
    fn try_lock(&self, session: &Session) -> Option<Held<'_, SessionLock>>;
    fn lock_bare(&self, session: &Session) -> Held<'_, SessionLock>;

    fn read_all(&self) -> Vec<Vec<String>> {
        self.sessions().iter().map(|session| self.read_from(session, 0)).collect()
    }
}

#[port]
trait SessionCache {
    fn clear(&self);
}

impl SessionCacheFiller for Memory {
    fn clear(&self, _pass: Pass<'_, session_cache_ops::Clear>) {}
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

    fn lock(&self, _pass: Pass<'_, transcripts_ops::Lock>, session: &Session) -> Result<SessionLock, Gone> {
        Ok(SessionLock::of(session))
    }

    fn try_lock(&self, _pass: Pass<'_, transcripts_ops::TryLock>, session: &Session) -> Option<SessionLock> {
        Some(SessionLock::of(session))
    }

    fn lock_bare(&self, _pass: Pass<'_, transcripts_ops::LockBare>, session: &Session) -> SessionLock {
        SessionLock::of(session)
    }
}

impl SessionLock {
    fn of(session: &Session) -> SessionLock {
        let lockable = Lockable::ALL.into_iter().find(|lockable| lockable.session() == *session);
        SessionLock { lockable: lockable.unwrap_or(Lockable::Intact) }
    }
}

impl Lockable {
    const ALL: [Lockable; 2] = [Lockable::Intact, Lockable::Broken];

    fn session(self) -> Session {
        match self {
            Lockable::Intact => Session("a".to_owned()),
            Lockable::Broken => Session("broken".to_owned()),
        }
    }

    fn ended(self, receipt: Result<ReleaseReceipt, Gone>) -> Result<ReleaseReceipt, Gone> {
        match self {
            Lockable::Intact => receipt,
            Lockable::Broken => Err(Gone),
        }
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

    fn over<F>(run: &Run, filler: F) -> Port<'_, F> {
        run.port(filler)
    }

    fn metered<P: Metered>(port: &P) -> &P {
        port
    }

    fn counts(counted: Counted) -> usize {
        counted.calls().count()
    }

    fn finished(run: Run) -> Measured {
        run.finish()
    }
}

laws::law!(an_op_is_counted_once_per_call_of_its_own, 0usize..24, calls, {
    let run = Run::root();
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
    let run = Run::root();
    let port = run.port(Memory::holding(&names.iter().map(String::as_str).collect::<Vec<_>>()));
    prop_assert_eq!(Transcripts::read_all(Tally::metered(&port)).len(), sessions);
    drop(port);
    let measured = run.finish();
    prop_assert_eq!(measured.snapshot().calls::<transcripts_ops::Sessions>(), Calls::new(1));
    prop_assert_eq!(measured.snapshot().calls::<transcripts_ops::ReadFrom>(), Calls::new(sessions));
});

laws::law!(calls_on_scoped_threads_are_all_counted, 1usize..6, threads, {
    let run = Run::root();
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
    let (first, second) = (Run::root(), Run::root());
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
    let run = Run::root();
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
fn a_resource_and_a_derived_op_are_named_by_module_path() {
    assert_eq!(SessionLock::NAME.to_string(), "laws::SessionLock");
    assert_eq!(SessionLock::RELEASE.to_string(), "laws::SessionLock::release");
    assert_eq!(Tally::named::<Released<SessionLock>>(), "laws::SessionLock::release");
    assert_eq!(Tally::named::<Count>(), "laws::Count");
}

laws::law!(per_type_ops_count_each_query_type_apart, (0usize..6, 0usize..6), asked, {
    let run = Run::root();
    let port = run.port(Memory::holding(&["a", "b"]));
    (0..asked.0).for_each(|_| {
        let _ = Queries::ask(&port, &Count);
    });
    (0..asked.1).for_each(|_| {
        let _ = Queries::ask(&port, &Names);
    });
    prop_assert_eq!(Queries::ask(&port, &Names), ["a", "b"]);
    drop(port);
    let measured = run.finish();
    prop_assert_eq!(measured.snapshot().calls::<Count>(), Calls::new(asked.0));
    prop_assert_eq!(measured.snapshot().calls::<Names>(), Calls::new(asked.1.saturating_add(1)));
});

#[derive(Clone, Copy, Debug)]
enum Acquisition {
    Fallible,
    Optional,
    Bare,
}

impl Acquisition {
    const ALL: [Acquisition; 3] = [Acquisition::Fallible, Acquisition::Optional, Acquisition::Bare];

    fn held<'port>(self, port: &'port Port<'_, Memory>, session: &Session) -> Option<Held<'port, SessionLock>> {
        match self {
            Acquisition::Fallible => Transcripts::lock(port, session).ok(),
            Acquisition::Optional => Transcripts::try_lock(port, session),
            Acquisition::Bare => Some(Transcripts::lock_bare(port, session)),
        }
    }
}

laws::law!(acquiring_counts_the_op_and_dropping_counts_one_release, 0usize..3, shape, {
    let acquisition = Acquisition::ALL.get(shape).copied().unwrap_or(Acquisition::Bare);
    let run = Run::root();
    let port = run.port(Memory::holding(&["a"]));
    let held = acquisition.held(&port, &Session("a".to_owned()));
    prop_assert!(held.is_some());
    drop(held);
    drop(port);
    let measured = run.finish();
    let acquired: usize = [
        measured.snapshot().calls::<transcripts_ops::Lock>(),
        measured.snapshot().calls::<transcripts_ops::TryLock>(),
        measured.snapshot().calls::<transcripts_ops::LockBare>(),
    ]
    .into_iter()
    .map(Calls::count)
    .sum();
    prop_assert_eq!(acquired, 1);
    prop_assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
    prop_assert!(measured.settled().is_ok());
});

laws::law!(
    ending_a_held_resource_releases_once_and_returns_its_receipt_or_failure,
    select(Lockable::ALL.to_vec()),
    lockable,
    {
        let run = Run::root();
        let port = run.port(Memory::default());
        let ended = Transcripts::lock(&port, &lockable.session()).map(Held::end);
        drop(port);
        let measured = run.finish();
        let other = Run::root();
        let receipt = Transcripts::lock_bare(&other.port(Memory::default()), &Session("a".to_owned())).end();
        prop_assert_eq!(ended, Ok(lockable.ended(receipt)));
        prop_assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
        prop_assert!(measured.settled().is_ok());
    }
);

laws::law!(a_failed_release_on_drop_shows_in_the_measured_failures, select(Lockable::ALL.to_vec()), lockable, {
    let run = Run::root();
    let port = run.port(Memory::default());
    drop(Transcripts::lock(&port, &lockable.session()));
    drop(port);
    let measured = run.finish();
    prop_assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
    prop_assert_eq!(measured.settled().is_err(), lockable == Lockable::Broken);
});

laws::law!(a_resource_is_released_exactly_once_whether_ended_or_dropped, any::<bool>(), ending, {
    let run = Run::root();
    let port = run.port(Memory::holding(&["a"]));
    let held = Transcripts::lock(&port, &Session("a".to_owned()));
    let ended = ending.then_some(held).map(|held| held.map(Held::end));
    prop_assert!(ended.iter().all(|result| matches!(result, Ok(Ok(_)))));
    drop(port);
    let measured = run.finish();
    prop_assert_eq!(measured.snapshot().calls::<Released<SessionLock>>(), Calls::new(1));
    prop_assert!(measured.settled().is_ok());
});

#[test]
fn a_two_word_trait_names_its_ops_module_in_snake_case() {
    let run = Run::root();
    let port = Tally::over(&run, Memory::default());
    SessionCache::clear(Tally::metered(&port));
    drop(port);
    assert_eq!(Tally::names(run.finish().snapshot()), ["laws::session_cache_ops::SessionCache::clear"]);
}
