use std::collections::VecDeque;
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, PoisonError};

use design::{Agents, Attempted, Cli, Count, Failure, Lapsed, Line, Outcome};
use design_model::{JobId, Provenance, RunId};
use design_store::{JobOpened, RunSpawned, RunState, Spawned, Unstructured};
use dispatch_ports::{
    AgentRun, ClockFiller, Deadline, DispatchFailure, DispatchFiller, Ended, Finished, Job, Pause, Pid, Prompt, Reply,
    Started, Tokens, Usage, Wall, clock_ops, dispatch_ops,
};
use newtype::Newtype;
use port_core::{Pass, Run};
use store_ports::{Store, Written};
use structurer::{Briefing, Directives, Lapse};
use text_lines::Lines;

use super::{Applying, FakeLent, Invocation};

#[derive(Debug, Default)]
pub struct Script {
    replies: Mutex<VecDeque<Option<Finished>>>,
    current: Mutex<Option<Finished>>,
    prompts: Mutex<Vec<Prompt>>,
    living: Vec<Living>,
    kills: Mutex<Vec<Pid>>,
    spawned: Mutex<u32>,
}

#[derive(Clone, Debug)]
struct Living {
    pid: Pid,
    started: Started,
}

#[derive(Debug)]
pub struct FakeDispatch<'script>(&'script Script);

#[derive(Debug)]
pub struct FakeClock {
    now: Mutex<Wall>,
    sleep: Pause,
}

pub struct Staff;

static IDLE: LazyLock<Script> = LazyLock::new(Script::default);

impl Staff {
    const START: Wall = Wall::new(1_800_000_000);

    pub fn idle() -> Agents<impl FnOnce(PathBuf) -> FakeDispatch<'static>, FakeClock> {
        Agents::new(|_scratch| FakeDispatch(&IDLE), FakeClock::asleep_for(Pause::new(0)))
    }
}

impl FakeClock {
    fn asleep_for(sleep: Pause) -> FakeClock {
        FakeClock { now: Mutex::new(Staff::START), sleep }
    }
}

impl ClockFiller for FakeClock {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause) -> Wall {
        let mut now = self.now.lock().unwrap_or_else(PoisonError::into_inner);
        *now = now.after(pause).after(self.sleep);
        *now
    }
}

impl DispatchFiller for FakeDispatch<'_> {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        job: &Job,
        prompt: &Prompt,
    ) -> Result<AgentRun, DispatchFailure> {
        let mut spawned = self.0.spawned.lock().unwrap_or_else(PoisonError::into_inner);
        *spawned = spawned.saturating_add(1);
        let reply = self.0.replies.lock().unwrap_or_else(PoisonError::into_inner).pop_front().flatten();
        *self.0.current.lock().unwrap_or_else(PoisonError::into_inner) = reply;
        self.0.prompts.lock().unwrap_or_else(PoisonError::into_inner).push(prompt.clone());
        Ok(AgentRun::new(Pid::new(spawned.saturating_add(100)), *job.as_ref()))
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        _run: &mut AgentRun,
    ) -> Result<Option<Finished>, DispatchFailure> {
        Ok(self.0.current.lock().unwrap_or_else(PoisonError::into_inner).take())
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        Ok(self.0.living.iter().find(|living| living.pid == pid).map(|living| living.started.clone()))
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, pid: Pid) -> Result<Ended, DispatchFailure> {
        self.0.kills.lock().unwrap_or_else(PoisonError::into_inner).push(pid);
        Ok(self.0.living.iter().find(|living| living.pid == pid).map_or(Ended::Absent, |_living| Ended::Killed))
    }
}

impl Script {
    const COVERING: &str = "conversational(aaaaaaaa-0000-4000-8000-000000000003.0)";
    const MISREAD: &str = "place(";

    fn replying(replies: &[Option<&str>]) -> Script {
        Script {
            replies: Mutex::new(replies.iter().map(|reply| reply.map(Script::finished)).collect()),
            ..Script::default()
        }
    }

    fn living(self, pid: u32, started: &str) -> Script {
        let started = started.parse::<Started>().ok();
        Script { living: started.map(|started| Living { pid: Pid::new(pid), started }).into_iter().collect(), ..self }
    }

    fn finished(reply: &str) -> Finished {
        Finished::new(Reply::new(reply.to_owned()), Script::usage())
    }

    fn usage() -> Usage {
        Usage::new(Tokens::new(10), Tokens::new(20), Tokens::new(3), Tokens::new(4))
    }

    fn prompts(&self) -> Vec<Option<Briefing>> {
        let prompts = self.prompts.lock().unwrap_or_else(PoisonError::into_inner);
        prompts.iter().map(|prompt| prompt.clone().into_inner().parse::<Briefing>().ok()).collect()
    }

    fn kills(&self) -> Vec<Pid> {
        self.kills.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

struct Structuring {
    applying: Applying,
    script: Script,
    sleep: Pause,
}

#[derive(Debug)]
struct Structured {
    outcome: Result<Outcome, Failure>,
    lines: Vec<Line>,
    text: String,
}

impl Structuring {
    fn over(script: Script, sleep: Pause) -> Result<Structuring, Failure> {
        Ok(Structuring { applying: Applying::ready()?, script, sleep })
    }

    fn structured(&self) -> Structured {
        let run = Run::root();
        let mut out = Vec::new();
        let agents = Agents::new(|_scratch| FakeDispatch(&self.script), FakeClock::asleep_for(self.sleep));
        let outcome = Cli::new(
            &Invocation::environment(),
            FakeLent::new(&self.applying.files),
            |_path| self.applying.store.clone(),
            agents,
        )
        .report(&run, ["structure"].iter().map(OsString::from), &mut out);
        let text = String::from_utf8(out).unwrap_or_default();
        let lines = Structured::parsed(&text);
        Structured { outcome, lines, text }
    }

    fn unstructured(&self) -> Result<usize, Failure> {
        let run = Run::root();
        Ok(run.port(self.applying.store.clone()).read(Unstructured)?.len())
    }

    fn stored_run(&self, pid: u32, started: &str) -> Result<Written, Failure> {
        let run = Run::root();
        let store = run.port(self.applying.store.clone());
        store.write(JobOpened::from(JobId::new(1)))?;
        let spawned = Spawned::new(Pid::new(pid), started.parse::<Started>().ok(), Deadline::new(Wall::new(0)));
        Ok(store.write(RunSpawned::new(RunId::new(1), JobId::new(1), spawned))?)
    }
}

impl Structured {
    fn parsed(text: &str) -> Vec<Line> {
        Lines::from(text).filter_map(|line| line.parse::<Line>().ok()).collect()
    }

    fn has(&self, wanted: &[Line]) -> Vec<Line> {
        wanted.iter().filter(|line| self.lines.iter().all(|had| had != *line)).cloned().collect()
    }
}

#[test]
fn a_reply_is_applied_with_agent_provenance_its_usage_recorded_and_the_unstructured_set_shrinks() -> Result<(), Failure>
{
    let structuring = Structuring::over(Script::replying(&[Some(Script::COVERING)]), Pause::new(0))?;
    let before = structuring.unstructured()?;
    let structured = structuring.structured();
    let after = structuring.unstructured()?;
    let ran = Line::Ran(Attempted::new(RunId::new(1), JobId::new(1), RunState::Replied, Some(Script::usage())));
    assert_eq!(structured.has(&[ran]), []);
    assert_eq!(structured.outcome?, Outcome::Structured { jobs: Count::new(1) });
    assert_eq!(after, before.saturating_sub(1));
    let shown = structuring.applying.run(&["show", "aaaaaaaa-0000-4000-8000-000000000003"])?;
    let agent = Provenance::Agent(RunId::new(1));
    let entered = Structured::parsed(&shown)
        .into_iter()
        .filter(|line| matches!(line, Line::Annotated { entered, .. } if *entered == agent))
        .count();
    assert_eq!(entered, 1);
    Ok(())
}

#[test]
fn a_deadline_passed_during_a_sleep_kills_the_run_which_counts_as_a_failure_and_is_retried() -> Result<(), Failure> {
    let structuring = Structuring::over(Script::replying(&[None, Some(Script::COVERING)]), Pause::new(3600))?;
    let structured = structuring.structured();
    let wanted = [
        Line::Retried(Lapsed::new(JobId::new(1), Lapse::overdue())),
        Line::Ran(Attempted::new(RunId::new(1), JobId::new(1), RunState::Overdue, None)),
        Line::Ran(Attempted::new(RunId::new(2), JobId::new(1), RunState::Replied, Some(Script::usage()))),
    ];
    assert_eq!(structured.has(&wanted), []);
    assert_eq!(structured.outcome?, Outcome::Structured { jobs: Count::new(1) });
    Ok(())
}

#[test]
fn a_second_kill_flails_the_job_loudly() -> Result<(), Failure> {
    let structuring = Structuring::over(Script::replying(&[None, None]), Pause::new(3600))?;
    let structured = structuring.structured();
    assert_eq!(structured.has(&[Line::Flailed(Lapsed::new(JobId::new(1), Lapse::overdue()))]), []);
    assert!(matches!(structured.outcome, Err(Failure::Flailed { lapse, .. }) if lapse == Lapse::overdue()));
    Ok(())
}

#[test]
fn a_misread_reply_is_retried_once_with_its_misread_in_the_prompt_then_flails() -> Result<(), Failure> {
    let structuring =
        Structuring::over(Script::replying(&[Some(Script::MISREAD), Some(Script::MISREAD)]), Pause::new(0))?;
    let structured = structuring.structured();
    let lapse = Script::MISREAD.parse::<Directives>().err().map(Lapse::from);
    let prompts = structuring.script.prompts();
    let retried = prompts.first().cloned().flatten().zip(lapse.clone()).map(|(first, lapse)| first.retried(&lapse));
    assert_eq!(prompts.get(1).cloned().flatten(), retried);
    assert_eq!(prompts.len(), 2);
    assert!(
        matches!(structured.outcome, Err(Failure::Flailed { lapse: flailed, .. }) if lapse.as_ref() == Some(&flailed))
    );
    Ok(())
}

#[test]
fn a_restarted_structure_kills_a_stored_run_past_its_deadline_whose_pid_and_start_time_match() -> Result<(), Failure> {
    let started = "Sat Nov  7 10:00:00 2026";
    let structuring =
        Structuring::over(Script::replying(&[Some(Script::COVERING)]).living(41, started), Pause::new(0))?;
    structuring.stored_run(41, started)?;
    let structured = structuring.structured();
    assert_eq!(structured.has(&[Line::Reaped { run: RunId::new(1), state: RunState::Killed }]), []);
    assert_eq!(structuring.script.kills(), [Pid::new(41)]);
    Ok(())
}

#[test]
fn a_reused_pid_with_another_start_time_is_not_killed() -> Result<(), Failure> {
    let structuring = Structuring::over(
        Script::replying(&[Some(Script::COVERING)]).living(41, "Sun Nov  8 09:00:00 2026"),
        Pause::new(0),
    )?;
    structuring.stored_run(41, "Sat Nov  7 10:00:00 2026")?;
    let structured = structuring.structured();
    assert_eq!(structured.has(&[Line::Reaped { run: RunId::new(1), state: RunState::Gone }]), []);
    assert_eq!(structuring.script.kills(), Vec::<Pid>::new());
    Ok(())
}

#[test]
fn a_retried_and_applied_structure_prints_its_lines() -> Result<(), Box<dyn Error>> {
    let structuring =
        Structuring::over(Script::replying(&[Some(Script::MISREAD), Some(Script::COVERING)]), Pause::new(0))?;
    golden::golden!("structure.txt", &format!("design structure\n{}", structuring.structured().text))?;
    let briefs: String = structuring.script.prompts().iter().flatten().map(ToString::to_string).collect();
    golden::golden!("structure-briefs.txt", &briefs)?;
    Ok(())
}
