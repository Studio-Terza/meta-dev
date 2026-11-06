use std::sync::{Mutex, PoisonError};

use laws::proptest::prelude::{any, prop_assert_eq};
use newtype::Counter;
use port_core::{Calls, Counted, Measured, Metered, Op, Pass, Port, Run, Snapshot, port};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Session(String);

#[port]
trait Transcripts {
    fn sessions(&self) -> Vec<Session>;
    fn read_from(&self, session: &Session, at: usize) -> Vec<String>;

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

laws::law!(a_snapshot_lists_ops_in_name_order, any::<(bool, bool)>(), which, {
    let run = Run::root();
    let port = Tally::over(&run, Memory::holding(&["a"]));
    let session = Session("a".to_owned());
    let _ = which.0.then(|| Transcripts::read_from(&port, &session, 0));
    let _ = which.1.then(|| Transcripts::sessions(&port));
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
}

#[test]
fn a_two_word_trait_names_its_ops_module_in_snake_case() {
    let run = Run::root();
    let port = Tally::over(&run, Memory::default());
    SessionCache::clear(Tally::metered(&port));
    drop(port);
    assert_eq!(Tally::names(run.finish().snapshot()), ["laws::session_cache_ops::SessionCache::clear"]);
}
