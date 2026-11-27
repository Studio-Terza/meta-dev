#![deny(clippy::wildcard_enum_match_arm)]

mod ignored;
mod ports;

use test_failure::Failed;

use std::fs;
use std::iter;
use std::ops::Range;
use std::path::Path;

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use proptest::prelude::{Strategy, prop_assert_eq, proptest};
use proptest::sample::select;
use proptest::test_runner::{Config, RngSeed};

use ::ports::dispatch::{
    AgentRun, Bell, ClockFiller, Deadline, Dispatch, DispatchFailure, DispatchFiller, Due, Ended, Errored, Fault,
    Finished, Job, Model, Output, Pause, Pid, Prompt, Reply, Stall, Stalled, Started, Stderr, Timeout, Told,
    Unreadable, Usage, Wall, Watch, Watched, Woke, clock_ops, dispatch_ops,
};
use invariants::{Ignoring, Unpoisoned, invariant, invariants};
use port_core::{Calls, Pass, Port, Released, Run, Snapshot};

use ignored::Ignored;

invariants! {
    enum Invariant {
        Endless => "a watch over the fake clock ends within ten thousand reads of it",
    }
}

#[test]
fn every_agent_result_reads_as_its_golden_reply_usage_or_error() -> Result<(), Failed> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/behaviour");
    let texts: Vec<String> = serde_json::from_str(&fs::read_to_string(directory.join("envelopes.json"))?)?;
    let read: String = texts
        .into_iter()
        .flat_map(|text| [format!("{:?}", Output::read(&text, Stderr::Silent)), "\n".to_owned()])
        .collect();
    let golden = directory.join("envelopes.txt");
    std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| fs::write(&golden, &read))?;
    assert_eq!(read, fs::read_to_string(golden)?);
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct Sleep {
    after: usize,
    slept: Pause,
}

impl Sleep {
    const STILL: Sleep = Sleep { after: 0, slept: Pause::new(0) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Wait {
    pause: Pause,
    woke: Woke,
}

impl Wait {
    fn granted(self) -> Pause {
        match self.woke {
            Woke::Lapsed => self.pause,
            Woke::Rung => Pause::new(0),
        }
    }
}

#[derive(Debug, Default)]
struct Waits(Mutex<Vec<Wait>>);

impl Waits {
    fn recorded(self) -> Vec<Wait> {
        self.0.into_inner().unpoisoned()
    }
}

#[derive(Debug)]
struct FakeClock<'waits> {
    now: Mutex<Wall>,
    reads: AtomicUsize,
    sleep: Sleep,
    waits: &'waits Waits,
}

impl<'waits> FakeClock<'waits> {
    const READS: usize = 10_000;

    fn new(sleep: Sleep, waits: &'waits Waits) -> FakeClock<'waits> {
        FakeClock { now: Mutex::new(Watching::START), reads: AtomicUsize::new(0), sleep, waits }
    }
}

impl ClockFiller for FakeClock<'_> {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        invariant!(self.reads.fetch_add(1, Ordering::Relaxed) < FakeClock::READS, Invariant::Endless);
        *self.now.lock().unpoisoned()
    }

    fn moment(&self, _pass: Pass<'_, clock_ops::Moment>) -> ::ports::dispatch::Moment {
        ::ports::dispatch::Moment::from(*self.now.lock().unpoisoned())
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause, bell: &Bell) -> Wall {
        let wait = Wait { pause, woke: bell.waited(Duration::ZERO) };
        let mut waits = self.waits.0.lock().unpoisoned();
        let slept =
            Some(self.sleep).filter(|sleep| sleep.after == waits.len()).map_or(Pause::new(0), |sleep| sleep.slept);
        waits.push(wait);
        let mut now = self.now.lock().unpoisoned();
        *now = now.after(wait.granted()).after(slept);
        *now
    }
}

#[derive(Debug)]
struct FakeDispatch {
    finishing: Option<usize>,
    polls: Mutex<usize>,
}

impl FakeDispatch {
    fn finished() -> Finished {
        Finished::new(Reply::new("conversational(a.0)".to_owned()), Usage::default())
    }
}

impl DispatchFiller for FakeDispatch {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        job: &Job,
        _prompt: &Prompt,
    ) -> Result<AgentRun, DispatchFailure> {
        Ok(AgentRun::new(Pid::new(7), *job.as_ref()))
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        _run: &mut AgentRun,
    ) -> Result<Option<Output>, DispatchFailure> {
        let mut polls = self.polls.lock().unpoisoned();
        *polls = polls.saturating_add(1);
        Ok(self.finishing.filter(|finishing| *finishing == *polls).map(|_| Output::Finished(FakeDispatch::finished())))
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, _pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        Ok(None)
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, _pid: Pid) -> Result<Ended, DispatchFailure> {
        Ok(Ended::Absent)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Observed {
    watched: Watched,
    calls: [Calls; 3],
    waits: Vec<Wait>,
}

impl Observed {
    fn read(watched: Watched, snapshot: &Snapshot, waits: Waits) -> Observed {
        Observed {
            watched,
            calls: [
                snapshot.calls::<dispatch_ops::Output>(),
                snapshot.calls::<clock_ops::Waited>(),
                snapshot.calls::<Released<AgentRun>>(),
            ],
            waits: waits.recorded(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Watching {
    timeout: u32,
    sleeping_after: usize,
    finishing: Option<usize>,
}

impl Watching {
    const START: Wall = Wall::new(1_800_000_000);
    const PAUSE: Pause = Watch::<Port<'static, FakeDispatch>, Port<'static, FakeClock<'static>>>::PAUSE;
    const LAPSED: Wait = Wait { pause: Watching::PAUSE, woke: Woke::Lapsed };
    const RUNG: Wait = Wait { pause: Watching::PAUSE, woke: Woke::Rung };

    fn watched(self) -> Result<Observed, Failed> {
        let waits = Waits::default();
        let run = Run::root();
        let watched = {
            let slept = Pause::new(self.timeout.saturating_add(1));
            let clock = run.port(FakeClock::new(Sleep { after: self.sleeping_after, slept }, &waits));
            let dispatch = run.port(FakeDispatch { finishing: self.finishing, polls: Mutex::new(0) });
            let deadline = Deadline::after(Watching::START, Timeout::new(self.timeout));
            let held = dispatch.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::new(String::new()))?;
            Watch::over(&dispatch, &clock).watched(held)?
        };
        Ok(Observed::read(watched, run.finish().snapshot(), waits))
    }
}

#[derive(Clone, Copy, Debug)]
struct Drawn {
    timeout: u32,
    count: usize,
}

impl Watching {
    fn drawn(counts: Range<usize>) -> impl Iterator<Item = Drawn> {
        (20..200_u32).step_by(30).flat_map(move |timeout| counts.clone().map(move |count| Drawn { timeout, count }))
    }
}

#[test]
fn a_deadline_that_passes_during_a_sleep_kills_the_run_on_the_next_check() -> Result<(), Failed> {
    Watching::drawn(0..9).try_for_each(|Drawn { timeout, count: sleeping_after }| {
        let watching = Watching { timeout, sleeping_after, finishing: None };
        let checks = Calls::new(sleeping_after.saturating_add(1));
        assert_eq!(
            watching.watched()?,
            Observed {
                watched: Watched::Stalled(Stall::Overdue),
                calls: [checks, checks, Calls::new(1)],
                waits: vec![Watching::LAPSED; sleeping_after.saturating_add(1)],
            }
        );
        Ok(())
    })
}

#[test]
fn a_run_that_replies_before_its_deadline_is_finished_and_released_once() -> Result<(), Failed> {
    Watching::drawn(1..9).try_for_each(|Drawn { timeout, count: finishing }| {
        let watching = Watching { timeout, sleeping_after: 9, finishing: Some(finishing) };
        assert_eq!(
            watching.watched()?,
            Observed {
                watched: Watched::Finished(FakeDispatch::finished()),
                calls: [Calls::new(finishing), Calls::new(finishing.saturating_sub(1)), Calls::new(1)],
                waits: vec![Watching::LAPSED; finishing.saturating_sub(1)],
            }
        );
        Ok(())
    })
}

struct Lingering;

impl Lingering {
    fn watched(timeout: Timeout) -> Result<Observed, Failed> {
        let waits = Waits::default();
        let run = Run::root();
        let watched = {
            let clock = run.port(FakeClock::new(Sleep::STILL, &waits));
            let replying = Lingering::pauses(timeout).saturating_add(2);
            let dispatch = run.port(FakeDispatch { finishing: Some(replying), polls: Mutex::new(0) });
            let deadline = Deadline::after(Watching::START, timeout);
            let held = dispatch.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::new(String::new()))?;
            held.bell().ringer().ring();
            Watch::over(&dispatch, &clock).watched(held)?
        };
        Ok(Observed::read(watched, run.finish().snapshot(), waits))
    }

    fn pauses(timeout: Timeout) -> usize {
        let deadline = Deadline::after(Watching::START, timeout);
        iter::successors(Some(Watching::START), |now| Some(now.after(Watching::PAUSE)))
            .take_while(|now| deadline.due(*now) == Due::Pending)
            .count()
    }

    fn observed(timeout: Timeout) -> Observed {
        let pauses = Lingering::pauses(timeout);
        let checks = Calls::new(pauses.saturating_add(1));
        Observed {
            watched: Watched::Stalled(Stall::Overdue),
            calls: [checks, checks, Calls::new(1)],
            waits: iter::once(Watching::RUNG).chain(iter::repeat_n(Watching::LAPSED, pauses)).collect(),
        }
    }
}

#[test]
fn a_ring_left_on_the_bell_neither_cuts_a_run_short_nor_carries_it_past_its_deadline() -> Result<(), Failed> {
    (1..200_u32).step_by(7).map(Timeout::new).try_for_each(|timeout| {
        assert_eq!(Lingering::watched(timeout)?, Lingering::observed(timeout));
        Ok(())
    })
}

struct Starts;

impl Starts {
    fn any() -> impl Strategy<Value = Started> {
        (
            select(vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
            select(vec!["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]),
            1..32_u8,
            0..24_u8,
            0..60_u8,
            0..60_u8,
            1970..2200_u16,
        )
            .prop_filter_map("a start time", |(weekday, month, day, hour, minute, second, year)| {
                format!("{weekday} {month} {day:>2} {hour:02}:{minute:02}:{second:02} {year}")
                    .parse()
                    .ignoring(Ignored::Unparsed)
            })
    }
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_start_time_reads_back_as_it_is_rendered(started in Starts::any()) {
        prop_assert_eq!(started.to_string().parse::<Started>().ignoring(Ignored::Unparsed), Some(started));
    }
}

#[test]
fn pauses_last_whole_seconds_and_moments_count_milliseconds_either_way() {
    assert_eq!([3, 0].map(|seconds| Duration::from(Pause::new(seconds))), [Duration::from_secs(3), Duration::ZERO]);
    let [early, late] = [Wall::new(2), Wall::new(5)].map(::ports::dispatch::Moment::from);
    assert_eq!([late.since(early), early.since(late), late.since(late)], [3000, -3000, 0]);
}

#[derive(Debug, PartialEq, Eq)]
enum Cause {
    Errored,
    Unreadable,
    Overdue,
    Unspawned,
    Failed,
}

impl Stalled for Cause {
    const ERRORED: Cause = Cause::Errored;
    const UNREADABLE: Cause = Cause::Unreadable;
    const OVERDUE: Cause = Cause::Overdue;
    const UNSPAWNED: Cause = Cause::Unspawned;
    const FAILED: Cause = Cause::Failed;
}

#[test]
fn each_stall_names_its_own_cause() -> Result<(), Failed> {
    let fault = || Fault::from(DispatchFailure::Spawn(std::io::Error::from_raw_os_error(2)));
    let stalls = [
        Stall::Errored(Errored::new(Told::try_new("refused".to_owned())?, Usage::default())),
        Stall::Unreadable(Unreadable::Unsuccessful { stderr: Stderr::Silent }),
        Stall::Overdue,
        Stall::Unspawned(fault()),
        Stall::Failed(fault()),
    ];
    let causes: Vec<Cause> = stalls.iter().map(Stall::stalled).collect();
    assert_eq!(causes, [Cause::Errored, Cause::Unreadable, Cause::Overdue, Cause::Unspawned, Cause::Failed]);
    Ok(())
}
