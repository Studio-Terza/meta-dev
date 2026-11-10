use std::error::Error;
use std::iter;

use design_model::RunId;
use design_runs::{
    Excerpt, JobOpened, LiveRuns, NextJob, NextRun, Ran, RunEnded, RunSpawned, RunState, RunsOf, Spawned, StoredRun,
};
use design_store::StreamSchema;
use dispatch_ports::{Deadline, Finished, Pid, Reply, Started, Tokens, Usage, Wall};
use laws::proptest::option;
use laws::proptest::prelude::{Just, any, prop_assert, prop_assert_eq};
use laws::proptest::sample::select;
use laws::proptest::strategy::Strategy;
use laws::proptest::test_runner::TestCaseError;
use newtype::{Enumerated, Wrapping};
use port_core::Run;
use sqlite_ports::SqliteOpener;
use store_ports::Store;

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;

#[derive(Clone, Debug)]
struct Spawning {
    pid: Pid,
    started: Option<Started>,
    deadline: Wall,
    state: RunState,
    usage: Option<Usage>,
    reply: Option<Reply>,
}

impl Spawning {
    fn any() -> impl Strategy<Value = Spawning> {
        (
            any::<Pid>(),
            option::of(
                Just("Sat Nov  7 10:00:00 2026").prop_filter_map("a start time", |text| text.parse::<Started>().ok()),
            ),
            any::<Wall>(),
            select(RunState::VARIANTS),
            option::of((any::<Tokens>(), any::<Tokens>(), any::<Tokens>(), any::<Tokens>())),
            option::of(("[a-z\"\n€]{1,8}", 0..6000_usize).prop_map(|(unit, times)| Reply::new(unit.repeat(times)))),
        )
            .prop_map(|(pid, started, deadline, state, usage, reply)| Spawning {
                pid,
                started,
                deadline,
                state,
                usage: usage.map(|(input, output, written, read)| Usage::new(input, output, written, read)),
                reply,
            })
    }

    fn stored(&self) -> Result<Lived, Box<dyn Error>> {
        let run = Run::root();
        let store = run.port(Sqlite::in_memory::<StreamSchema>());
        let job = store.read(NextJob)?;
        store.write(JobOpened::from(job))?;
        let id = store.read(NextRun)?;
        let spawned = Spawned::new(self.pid, self.started.clone(), Deadline::new(self.deadline));
        store.write(RunSpawned::new(id, job, spawned))?;
        let live = store.read(LiveRuns)?;
        let finished = self.reply.clone().map(|reply| Finished::new(reply, self.usage.unwrap_or_default()));
        store
            .write(finished.map_or(RunEnded::new(id, self.state, self.usage), |told| RunEnded::finished(id, &told)))?;
        Ok(Lived { live, ran: store.read(RunsOf::from(job))?, after: store.read(LiveRuns)? })
    }
}

#[derive(Debug)]
struct Lived {
    live: Vec<StoredRun>,
    ran: Vec<Ran>,
    after: Vec<StoredRun>,
}

laws::law!(
    a_spawned_run_is_live_until_it_ends_with_its_state_usage_and_reply_bounded_to_sixteen_kibibytes,
    Spawning::any(),
    spawning,
    {
        let lived = spawning.stored().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let spawned: Vec<Spawned> = lived.live.iter().map(|stored| stored.spawned().clone()).collect();
        let wanted = Spawned::new(spawning.pid, spawning.started.clone(), Deadline::new(spawning.deadline));
        prop_assert_eq!(spawned, vec![wanted]);
        let ran: Vec<Ran> = lived.ran;
        prop_assert_eq!(ran.iter().map(|one| [one.identity()]).collect::<Vec<[RunId; 1]>>(), vec![[RunId::new(1)]]);
        let replied = spawning.reply.as_ref().map(|_reply| RunState::Replied);
        let state = replied.unwrap_or(spawning.state);
        let usage = replied.map_or(spawning.usage, |_replied| Some(spawning.usage.unwrap_or_default()));
        prop_assert_eq!(ran.iter().map(Ran::state).collect::<Vec<RunState>>(), vec![state]);
        prop_assert_eq!(ran.iter().map(Ran::usage).collect::<Vec<Option<Usage>>>(), vec![usage]);
        let told = spawning.reply.map(Reply::into_inner);
        let kept = ran.first().and_then(Ran::reply).cloned().map(Excerpt::into_inner);
        prop_assert_eq!(kept.is_some(), told.is_some());
        if let (Some(whole), Some(part)) = (told, kept) {
            prop_assert!(part.len() <= whole.len());
            prop_assert!(iter::zip(part.bytes(), whole.bytes()).all(|(mine, theirs)| mine == theirs));
            prop_assert!(part.len() <= Excerpt::BYTES);
            let next = whole.ceil_char_boundary(part.len().saturating_add(1));
            prop_assert!(part.len() == whole.len() || next > Excerpt::BYTES);
            prop_assert!(whole.len() > Excerpt::BYTES || part == whole);
        }
        let still: Vec<RunId> = lived.after.iter().map(StoredRun::identity).collect();
        let running = [state].into_iter().filter(|now| *now == RunState::Running).map(|_| RunId::new(1));
        prop_assert_eq!(still, running.collect::<Vec<RunId>>());
    }
);

impl Spawning {
    fn replying(reply: String) -> Spawning {
        Spawning {
            pid: Pid::new(7),
            started: None,
            deadline: Wall::new(0),
            state: RunState::Replied,
            usage: None,
            reply: Some(Reply::new(reply)),
        }
    }

    fn kept(self) -> Result<String, Box<dyn Error>> {
        let lived = self.stored()?;
        let reply = lived.ran.first().and_then(Ran::reply).cloned().ok_or("no reply kept")?;
        Ok(Excerpt::into_inner(reply))
    }
}

#[test]
fn an_ascii_reply_of_twenty_thousand_bytes_is_stored_as_exactly_sixteen_kibibytes() -> Result<(), Box<dyn Error>> {
    let kept = Spawning::replying("a".repeat(20_000)).kept()?;
    assert_eq!(kept.len(), 16 * 1024);
    Ok(())
}

#[test]
fn an_ascii_reply_of_exactly_sixteen_kibibytes_is_stored_unchanged() -> Result<(), Box<dyn Error>> {
    let told = "a".repeat(16 * 1024);
    let kept = Spawning::replying(told.clone()).kept()?;
    assert_eq!(kept, told);
    Ok(())
}
