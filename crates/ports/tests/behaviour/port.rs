use std::path::Path;
use std::sync::Mutex;

use invariants::{Ignoring, Unpoisoned, ignored};
use newtype::Counter;
use ports::port::{
    Calls, Counted, Held, Measured, Metered, Op, Pass, Port, Release, ReleaseReceipt, Released, Resource, Run, port,
};

ignored! {
    enum Ignored {
        Unlocked => "a fallible lock that fails holds nothing, as an optional lock that finds none",
    }
}

const GOLDEN: &str = "tests/behaviour/laws.txt";

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

#[port]
trait Queries {
    #[op(Q)]
    fn ask<Q: Query>(&self, query: &Q) -> Q::Answer;
}

trait Query: Op {
    type Answer;

    fn answered(&self, rows: &[Session]) -> Self::Answer;
}

#[derive(Op)]
struct Count;

#[derive(Op)]
struct Names;

#[derive(Debug, Default)]
struct Memory {
    sessions: Mutex<Vec<Session>>,
}

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

impl SessionCacheFiller for Memory {
    fn clear(&self, _pass: Pass<'_, session_cache_ops::Clear>) {}
}

impl QueriesFiller for Memory {
    fn ask<Q: Query>(&self, _pass: Pass<'_, Q>, query: &Q) -> Q::Answer {
        query.answered(&self.sessions.lock().unpoisoned())
    }
}

impl TranscriptsFiller for Memory {
    fn sessions(&self, _pass: Pass<'_, transcripts_ops::Sessions>) -> Vec<Session> {
        self.sessions.lock().unpoisoned().clone()
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
        SessionLock { lockable: if session.0 == "broken" { Lockable::Broken } else { Lockable::Intact } }
    }
}

struct Scenario(&'static str, &'static [&'static str], for<'run, 'port> fn(&'port Port<'run, Memory>) -> String);

impl Scenario {
    const ALL: [Scenario; 15] = [
        Scenario("no calls", &[], |_| String::new()),
        Scenario("each call counts once", &["a"], |port| format!("{:?}", [(); 3].map(|()| port.sessions().len()))),
        Scenario("a provided method counts its calls", &["a", "b"], |port| {
            format!("{:?}", Scenario::metered(port).read_all())
        }),
        Scenario("scoped threads", &["a"], |port| {
            format!("{:?}", std::thread::scope(|scope| [(); 4].map(|()| scope.spawn(|| port.sessions())).len()))
        }),
        Scenario("runs keep their own counts", &["a"], |port| {
            format!("{} {:?}", port.sessions().len(), Scenario::ALL.get(1).map(Scenario::shown))
        }),
        Scenario("ops list in name order", &["a"], |port| {
            format!(
                "{:?} {:?} {:?}",
                port.read_from(&Session::a(), 1),
                port.sessions().len(),
                port.lock(&Session::a()).map(Held::end)
            )
        }),
        Scenario("per-type ops", &["a", "b"], |port| {
            format!("{} {} {:?}", port.ask(&Count), port.ask(&Count), port.ask(&Names))
        }),
        Scenario("a fallible lock dropped", &[], |port| {
            format!("{:?}", port.lock(&Session::a()).ignoring(Ignored::Unlocked).map(|held| held.lockable))
        }),
        Scenario("an optional lock dropped", &[], |port| {
            format!("{:?}", port.try_lock(&Session::a()).map(|held| held.lockable))
        }),
        Scenario("a bare lock dropped", &[], |port| format!("{:?}", port.lock_bare(&Session::a()).lockable)),
        Scenario("a broken lock ended", &[], |port| {
            format!("{:?}", port.lock(&Session("broken".to_owned())).map(Held::end))
        }),
        Scenario("a broken lock dropped", &[], |port| {
            format!("{:?}", port.lock(&Session("broken".to_owned())).map(|held| held.lockable))
        }),
        Scenario("a lock changed through its hold", &[], |port| Scenario::changed(port.lock_bare(&Session::a()))),
        Scenario("a two-word trait", &[], |port| format!("{:?}", SessionCache::clear(port))),
        Scenario("names", &[], |_| format!("{} {} {}", SessionLock::NAME, Released::<SessionLock>::NAME, Count::NAME)),
    ];

    fn metered<Ported: Metered>(port: &Ported) -> &Ported {
        port
    }

    fn changed(mut held: Held<'_, SessionLock>) -> String {
        let reached = held.lockable;
        held.lockable = Lockable::Broken;
        format!("{reached:?} {:?}", held.end())
    }

    fn shown(&self) -> String {
        let run = Run::root();
        let port =
            run.port(Memory { sessions: Mutex::new(self.1.iter().map(|name| Session((*name).to_owned())).collect()) });
        let said = (self.2)(&port);
        drop(port);
        let measured: Measured = run.finish();
        let counted: Vec<String> = measured
            .snapshot()
            .iter()
            .map(|counted: Counted| format!("{} {}", counted.name(), counted.calls().count()))
            .collect();
        let sessions: Calls = measured.snapshot().calls::<transcripts_ops::Sessions>();
        let settled = match measured.settled() {
            Ok(_snapshot) => "settled".to_owned(),
            Err(failures) => failures.to_string(),
        };
        format!("{}: {said}; {} sessions in [{}]; {settled}\n", self.0, sessions.count(), counted.join(", "))
    }
}

impl Session {
    fn a() -> Session {
        Session("a".to_owned())
    }
}

#[test]
fn every_scenario_counts_and_releases_as_its_golden_says() -> Result<(), std::io::Error> {
    let shown: String = Scenario::ALL.iter().map(Scenario::shown).collect();
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDEN);
    std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| std::fs::write(&golden, &shown))?;
    assert_eq!(shown, std::fs::read_to_string(golden)?);
    Ok(())
}
