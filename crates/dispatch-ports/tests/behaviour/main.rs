#![deny(clippy::wildcard_enum_match_arm)]

mod ports;

use std::error::Error;
use std::iter;
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use dispatch_ports::{
    AgentRun, Bell, ClockFiller, Deadline, Dispatch, DispatchFailure, DispatchFiller, Due, Ended, Errored, Finished,
    Job, Output, Pause, Pid, Prompt, Reply, Stall, Started, Stderr, Timeout, Tokens, Told, Unreadable, Usage, Wall,
    Watch, Watched, Woke, clock_ops, dispatch_ops,
};
use laws::proptest::prelude::prop_assert_eq;
use laws::proptest::sample::select;
use laws::proptest::strategy::Strategy;
use laws::proptest::test_runner::TestCaseError;
use port_core::{Calls, Pass, Port, Released, Run, Snapshot};

#[derive(Debug, PartialEq, Eq)]
enum Envelope {
    Read(Output),
    Garbled,
}

impl Envelope {
    fn read(text: &str) -> Envelope {
        match Output::read(text, Stderr::Silent) {
            Output::Unreadable(Unreadable::Json { .. }) => Envelope::Garbled,
            output @ (Output::Finished(_)
            | Output::Errored(_)
            | Output::Unreadable(Unreadable::Unsuccessful { .. })) => Envelope::Read(output),
        }
    }

    fn replied(reply: &str, [input, output, written, read]: [u32; 4]) -> Envelope {
        let usage = Usage::new(Tokens::new(input), Tokens::new(output), Tokens::new(written), Tokens::new(read));
        Envelope::Read(Output::Finished(Finished::new(Reply::new(reply.to_owned()), usage)))
    }

    fn errored(told: &str) -> Envelope {
        Told::try_new(told.to_owned())
            .map_or(Envelope::Garbled, |inked| Envelope::Read(Output::Errored(Errored::new(inked, Usage::default()))))
    }

    fn unsuccessful() -> Envelope {
        Envelope::Read(Output::Unreadable(Unreadable::Unsuccessful { stderr: Stderr::Silent }))
    }
}

laws::table!(
    an_agent_result_is_read_as_its_reply_and_usage_or_its_error,
    |text: &&'static str| Envelope::read(text),
    [
        r#"{"type":"result","subtype":"success","is_error":false,"result":"conversational(a.0)","usage":{"input_tokens":4,"output_tokens":9,"cache_creation_input_tokens":2,"cache_read_input_tokens":7,"service_tier":"standard"}}"# => Envelope::replied("conversational(a.0)", [4, 9, 2, 7]),
        r#"{"subtype":"success","result":"x","usage":{"input_tokens":1,"output_tokens":2}}"# => Envelope::replied("x", [1, 2, 0, 0]),
        r#"{"type":"result","subtype":"success","is_error":true,"result":"Invalid API key","usage":{"input_tokens":0,"output_tokens":0}}"# => Envelope::errored("Invalid API key"),
        r#"{"subtype":"error_during_execution","is_error":true,"usage":{"input_tokens":1,"output_tokens":2}}"# => Envelope::unsuccessful(),
        r#"{"subtype":"error_max_turns","result":"x","usage":{"input_tokens":1,"output_tokens":2}}"# => Envelope::unsuccessful(),
        r#"{"subtype":"success","usage":{"input_tokens":1,"output_tokens":2}}"# => Envelope::unsuccessful(),
        r#"{"subtype":"success","result":"  ","usage":{"input_tokens":1,"output_tokens":2}}"# => Envelope::Garbled,
        r#"{"subtype":"success","result":"x"}"# => Envelope::Garbled,
        "not json" => Envelope::Garbled,
    ]
);

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
        self.0.into_inner().unwrap_or_else(PoisonError::into_inner)
    }
}

#[derive(Debug)]
struct FakeClock<'waits> {
    now: Mutex<Wall>,
    sleep: Sleep,
    waits: &'waits Waits,
}

impl<'waits> FakeClock<'waits> {
    fn new(sleep: Sleep, waits: &'waits Waits) -> FakeClock<'waits> {
        FakeClock { now: Mutex::new(Watching::START), sleep, waits }
    }
}

impl ClockFiller for FakeClock<'_> {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause, bell: &Bell) -> Wall {
        let wait = Wait { pause, woke: bell.waited(Duration::ZERO) };
        let mut waits = self.waits.0.lock().unwrap_or_else(PoisonError::into_inner);
        let slept =
            Some(self.sleep).filter(|sleep| sleep.after == waits.len()).map_or(Pause::new(0), |sleep| sleep.slept);
        waits.push(wait);
        let mut now = self.now.lock().unwrap_or_else(PoisonError::into_inner);
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
        let mut polls = self.polls.lock().unwrap_or_else(PoisonError::into_inner);
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

    fn watched(self) -> Result<Observed, Box<dyn Error>> {
        let waits = Waits::default();
        let run = Run::root();
        let watched = {
            let slept = Pause::new(self.timeout.saturating_add(1));
            let clock = run.port(FakeClock::new(Sleep { after: self.sleeping_after, slept }, &waits));
            let dispatch = run.port(FakeDispatch { finishing: self.finishing, polls: Mutex::new(0) });
            let deadline = Deadline::after(Watching::START, Timeout::new(self.timeout));
            let held = dispatch.dispatch(&Job::due_by(deadline), &Prompt::new(String::new()))?;
            Watch::over(&dispatch, &clock).watched(held)?
        };
        Ok(Observed::read(watched, run.finish().snapshot(), waits))
    }
}

laws::law!(a_deadline_that_passes_during_a_sleep_kills_the_run_on_the_next_check, (20..200_u32, 0..9_usize), drawn, {
    let (timeout, sleeping_after) = drawn;
    let watching = Watching { timeout, sleeping_after, finishing: None };
    let checks = Calls::new(sleeping_after.saturating_add(1));
    let watched = watching.watched().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(
        watched,
        Observed {
            watched: Watched::Stalled(Stall::Overdue),
            calls: [checks, checks, Calls::new(1)],
            waits: vec![Watching::LAPSED; sleeping_after.saturating_add(1)],
        }
    );
});

laws::law!(a_run_that_replies_before_its_deadline_is_finished_and_released_once, (20..200_u32, 1..9_usize), drawn, {
    let (timeout, finishing) = drawn;
    let watching = Watching { timeout, sleeping_after: 9, finishing: Some(finishing) };
    let watched = watching.watched().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(
        watched,
        Observed {
            watched: Watched::Finished(FakeDispatch::finished()),
            calls: [Calls::new(finishing), Calls::new(finishing.saturating_sub(1)), Calls::new(1)],
            waits: vec![Watching::LAPSED; finishing.saturating_sub(1)],
        }
    );
});

struct Lingering;

impl Lingering {
    fn watched(timeout: Timeout) -> Result<Observed, Box<dyn Error>> {
        let waits = Waits::default();
        let run = Run::root();
        let watched = {
            let clock = run.port(FakeClock::new(Sleep::STILL, &waits));
            let dispatch = run.port(FakeDispatch { finishing: None, polls: Mutex::new(0) });
            let deadline = Deadline::after(Watching::START, timeout);
            let held = dispatch.dispatch(&Job::due_by(deadline), &Prompt::new(String::new()))?;
            held.bell().ringer().ring();
            Watch::over(&dispatch, &clock).watched(held)?
        };
        Ok(Observed::read(watched, run.finish().snapshot(), waits))
    }

    fn observed(timeout: Timeout) -> Observed {
        let deadline = Deadline::after(Watching::START, timeout);
        let pauses = iter::successors(Some(Watching::START), |now| Some(now.after(Watching::PAUSE)))
            .take_while(|now| deadline.due(*now) == Due::Pending)
            .count();
        let checks = Calls::new(pauses.saturating_add(1));
        Observed {
            watched: Watched::Stalled(Stall::Overdue),
            calls: [checks, checks, Calls::new(1)],
            waits: iter::once(Watching::RUNG).chain(iter::repeat_n(Watching::LAPSED, pauses)).collect(),
        }
    }
}

laws::law!(a_ring_left_on_the_bell_does_not_cut_a_run_short, 1..200_u32, drawn, {
    let timeout = Timeout::new(drawn);
    let watched = Lingering::watched(timeout).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(watched, Lingering::observed(timeout));
});

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
                format!("{weekday} {month} {day:>2} {hour:02}:{minute:02}:{second:02} {year}").parse().ok()
            })
    }
}

laws::roundtrip!(
    a_start_time_reads_back_as_it_is_rendered,
    Starts::any(),
    |started: &Started| started.to_string(),
    |text: &String| text.parse::<Started>().ok(),
    Some
);
