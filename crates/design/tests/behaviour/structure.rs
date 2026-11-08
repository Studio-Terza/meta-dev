use std::collections::VecDeque;
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, PoisonError};

use design::{Agents, Attempted, Cli, Count, Failure, Lapsed, Line, Outcome};
use design_model::{Entered, Entry, Facet, JobId, Node, NodeId, Provenance, RunId};
use design_store::{JobOpened, Planting, Ran, RunSpawned, RunState, RunsOf, Spawned, Unstructured};
use dispatch_ports::{
    Agent, AgentRun, ClockFiller, Deadline, DispatchFailure, DispatchFiller, Ended, Errored, Finished, HeadlessClaude,
    Job, Output, Pause, Pid, Prompt, Reply, Started, SystemClock, Tokens, Told, Usage, Wall, clock_ops, dispatch_ops,
};
use newtype::Newtype;
use port_core::{Pass, Run};
use store_ports::{Store, Written};
use structurer::{Briefing, Directives, Lapse};
use text_lines::Lines;

use super::{Applying, FakeLent, Invocation};

#[derive(Debug, Default)]
pub struct Script {
    replies: Mutex<VecDeque<Option<Output>>>,
    current: Mutex<Option<Output>>,
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
    ) -> Result<Option<Output>, DispatchFailure> {
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
    const COVERING: &str = "place(aaaaaaaa-0000-4000-8000-000000000001.0, n1, mentions)\nsense(aaaaaaaa-0000-4000-8000-000000000003.0, decision, specific, firm, explicit)\nlink(aaaaaaaa-0000-4000-8000-000000000003.1, aaaaaaaa-0000-4000-8000-000000000005.0, answers)";
    const UNCOVERING: &str = "propose(new a, refines(n1), feature, \"x\")";
    const MISREAD: &str = "place(";

    fn replying(replies: &[Option<&str>]) -> Script {
        Script {
            replies: Mutex::new(
                replies.iter().map(|reply| reply.map(|reply| Output::Finished(Script::finished(reply)))).collect(),
            ),
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
        self.structured_by(Agents::new(|_scratch| FakeDispatch(&self.script), FakeClock::asleep_for(self.sleep)))
    }

    fn structured_by<D: DispatchFiller>(
        &self,
        agents: Agents<impl FnOnce(PathBuf) -> D, impl ClockFiller>,
    ) -> Structured {
        let run = Run::root();
        let mut out = Vec::new();
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
    assert_eq!((before, after), (4, 0));
    let shown = structuring.applying.run(&["show", "aaaaaaaa-0000-4000-8000-000000000003"])?;
    let agent = Provenance::Agent(RunId::new(1));
    let entered = Structured::parsed(&shown)
        .into_iter()
        .filter(|line| matches!(line, Line::Annotated { entered, .. } if *entered == agent))
        .count();
    assert_eq!(entered, 2);
    Ok(())
}

#[test]
fn a_flailed_job_is_not_redispatched_by_the_next_run() -> Result<(), Box<dyn Error>> {
    let u = Some(Script::UNCOVERING);
    let structuring = Structuring::over(Script::replying(&[u, u, u, u]), Pause::new(0))?;
    let first = structuring.structured();
    assert!(matches!(first.outcome, Err(Failure::Flailed { .. })));
    let second = structuring.structured();
    assert_eq!(structuring.script.prompts().len(), 2, "the second run re-sent the flailed job's segments");
    golden::golden!(
        "structure-flailed.txt",
        &format!("design structure\n{}design structure\n{}", first.text, second.text)
    )?;
    assert_eq!(second.outcome?, Outcome::Structured { jobs: Count::new(0) });
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
fn a_reply_leaving_job_segments_uncovered_writes_nothing_and_is_retried_with_them_in_the_prompt()
-> Result<(), Box<dyn Error>> {
    let structuring =
        Structuring::over(Script::replying(&[Some(Script::UNCOVERING), Some(Script::COVERING)]), Pause::new(0))?;
    let run = Run::root();
    let entry = Entry::new("other".parse()?, Facet::Project, Entered::Init.into());
    run.port(structuring.applying.store.clone()).write(Planting::sown(Node::new(NodeId::new(2), entry)))?;
    golden::golden!("structure.txt", &format!("design structure\n{}", structuring.structured().text))?;
    let briefs: String = structuring.script.prompts().iter().flatten().map(ToString::to_string).collect();
    golden::golden!("structure-briefs.txt", &briefs)?;
    assert!(matches!(structuring.applying.run(&["show", "n3"]), Err(Failure::Unplaced(_))));
    Ok(())
}

#[test]
fn an_agent_that_errs_is_retried_with_its_error_in_the_prompt_then_flails_with_what_it_said_on_stderr()
-> Result<(), Box<dyn Error>> {
    let structuring = Structuring::over(Script::default(), Pause::new(0))?;
    let agent = Agent::new(
        "sh",
        &[
            "-c",
            r#"if grep -c '^retry errored' >/dev/null; then echo 'error: unknown option --tools' >&2; exit 1; else echo '{"type":"result","subtype":"success","is_error":true,"result":"Invalid API key - Please run /login","usage":{"input_tokens":1,"output_tokens":2}}'; fi"#,
        ],
    );
    let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let structured =
        structuring.structured_by(Agents::new(|_scratch| HeadlessClaude::new(scratch, agent), SystemClock));
    let run = Run::root();
    let ran = run.port(structuring.applying.store).read(RunsOf::from(JobId::new(1)))?;
    golden::golden!("structure-unreadable.txt", &format!("design structure\n{}", structured.text))?;
    let first =
        Lapse::from(&Errored::new(Told::new("Invalid API key - Please run /login".to_owned()), Usage::default()));
    assert_eq!(structured.has(&[Line::Retried(Lapsed::new(JobId::new(1), first))]), []);
    assert!(matches!(structured.outcome, Err(Failure::Flailed { .. })));
    assert_eq!(ran.iter().map(Ran::state).collect::<Vec<_>>(), [RunState::Errored, RunState::Unreadable]);
    Ok(())
}

#[test]
fn an_agent_whose_output_cannot_be_collected_fails_structure_and_its_run_ends_failed() -> Result<(), Failure> {
    let structuring = Structuring::over(Script::default(), Pause::new(0))?;
    let agent = Agent::new("sh", &["-c", r"cat >/dev/null; printf '\377'"]);
    let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let structured =
        structuring.structured_by(Agents::new(|_scratch| HeadlessClaude::new(scratch, agent), SystemClock));
    let run = Run::root();
    let ran = run.port(structuring.applying.store).read(RunsOf::from(JobId::new(1)))?;
    assert!(matches!(structured.outcome, Err(Failure::Dispatch(DispatchFailure::Collect(_)))));
    assert_eq!(ran.iter().map(Ran::state).collect::<Vec<_>>(), [RunState::Failed]);
    Ok(())
}
