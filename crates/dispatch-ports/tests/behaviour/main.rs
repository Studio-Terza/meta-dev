mod ports;

use std::error::Error;
use std::sync::{Mutex, PoisonError};

use dispatch_ports::{
    AgentRun, ClockFiller, Deadline, Dispatch, DispatchFailure, DispatchFiller, Ended, Finished, Job, Pause, Pid,
    Prompt, Reply, Started, Timeout, Tokens, Usage, Wall, Watch, Watched, clock_ops, dispatch_ops,
};
use laws::proptest::prelude::prop_assert_eq;
use laws::proptest::sample::select;
use laws::proptest::strategy::Strategy;
use laws::proptest::test_runner::TestCaseError;
use newtype::Newtype;
use port_core::{Calls, Pass, Released, Run};

#[derive(Debug, PartialEq, Eq)]
struct Envelope {
    reply: String,
    spent: [u64; 4],
}

impl Envelope {
    fn read(text: &str) -> Option<Envelope> {
        text.parse::<Finished>().ok().map(|finished| Envelope {
            reply: finished.reply().clone().into_inner(),
            spent: finished.usage().spent().map(Tokens::into_inner),
        })
    }

    fn of(reply: &str, spent: [u64; 4]) -> Envelope {
        Envelope { reply: reply.to_owned(), spent }
    }
}

laws::table!(
    an_agent_result_is_read_as_its_reply_and_usage,
    |text: &&'static str| Envelope::read(text),
    [
        r#"{"type":"result","subtype":"success","is_error":false,"result":"conversational(a.0)","usage":{"input_tokens":4,"output_tokens":9,"cache_creation_input_tokens":2,"cache_read_input_tokens":7,"service_tier":"standard"}}"# => Some(Envelope::of("conversational(a.0)", [4, 9, 2, 7])),
        r#"{"subtype":"success","result":"x","usage":{"input_tokens":1,"output_tokens":2}}"# => Some(Envelope::of("x", [1, 2, 0, 0])),
        r#"{"subtype":"success","result":"  ","usage":{"input_tokens":1,"output_tokens":2}}"# => None,
        r#"{"subtype":"error_max_turns","result":"x","usage":{"input_tokens":1,"output_tokens":2}}"# => None,
        r#"{"subtype":"success","usage":{"input_tokens":1,"output_tokens":2}}"# => None,
        r#"{"subtype":"success","result":"x"}"# => None,
        "not json" => None,
    ]
);

#[derive(Clone, Copy, Debug)]
struct Sleep {
    after: usize,
    slept: Pause,
}

#[derive(Debug)]
struct FakeClock {
    now: Mutex<Wall>,
    sleep: Sleep,
    waits: Mutex<usize>,
}

impl ClockFiller for FakeClock {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause) -> Wall {
        let mut waits = self.waits.lock().unwrap_or_else(PoisonError::into_inner);
        let slept = Some(self.sleep).filter(|sleep| sleep.after == *waits).map_or(Pause::new(0), |sleep| sleep.slept);
        *waits = waits.saturating_add(1);
        let mut now = self.now.lock().unwrap_or_else(PoisonError::into_inner);
        *now = now.after(pause).after(slept);
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
        Ok(AgentRun::new(Pid::new(7), job.deadline()))
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        _run: &mut AgentRun,
    ) -> Result<Option<Finished>, DispatchFailure> {
        let mut polls = self.polls.lock().unwrap_or_else(PoisonError::into_inner);
        *polls = polls.saturating_add(1);
        Ok(self.finishing.filter(|finishing| *finishing == *polls).map(|_| FakeDispatch::finished()))
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
}

#[derive(Clone, Copy, Debug)]
struct Watching {
    timeout: u32,
    sleeping_after: usize,
    finishing: Option<usize>,
}

impl Watching {
    const START: Wall = Wall::new(1_800_000_000);

    fn watched(self) -> Result<Observed, Box<dyn Error>> {
        let run = Run::root();
        let slept = Pause::new(self.timeout.saturating_add(1));
        let clock = run.port(FakeClock {
            now: Mutex::new(Watching::START),
            sleep: Sleep { after: self.sleeping_after, slept },
            waits: Mutex::new(0),
        });
        let dispatch = run.port(FakeDispatch { finishing: self.finishing, polls: Mutex::new(0) });
        let deadline = Deadline::after(Watching::START, Timeout::new(self.timeout));
        let held = dispatch.dispatch(&Job::due_by(deadline), &Prompt::new(String::new()))?;
        let watched = Watch::over(&dispatch, &clock).watched(held)?;
        drop((dispatch, clock));
        let measured = run.finish();
        let snapshot = measured.snapshot();
        Ok(Observed {
            watched,
            calls: [
                snapshot.calls::<dispatch_ops::Output>(),
                snapshot.calls::<clock_ops::Waited>(),
                snapshot.calls::<Released<AgentRun>>(),
            ],
        })
    }
}

laws::law!(a_deadline_that_passes_during_a_sleep_kills_the_run_on_the_next_check, (20..200_u32, 0..9_usize), drawn, {
    let (timeout, sleeping_after) = drawn;
    let watching = Watching { timeout, sleeping_after, finishing: None };
    let checks = Calls::new(sleeping_after.saturating_add(1));
    let watched = watching.watched().map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(watched, Observed { watched: Watched::Overdue, calls: [checks, checks, Calls::new(1)] });
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
        }
    );
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
