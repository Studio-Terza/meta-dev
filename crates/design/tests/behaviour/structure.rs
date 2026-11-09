use std::collections::VecDeque;
use std::error::Error;
use std::ffi::OsString;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::slice;
use std::sync::{LazyLock, Mutex, PoisonError};

use design::{
    Agents, Attempted, Cli, Count, Failure, Flail, Flailing, Flails, HookEvent, Lapsed, Line, Outcome, Sequel,
    Unresolved,
};
use design_model::{Entered, Entry, Facet, JobId, Node, NodeId, Ordinal, Prefix, Provenance, RunId};
use design_store::{JobOpened, Planting, Ran, RunSpawned, RunState, RunsOf, Spawned, Unstructured};
use dispatch_ports::{
    Agent, AgentRun, ClockFiller, Deadline, DispatchFailure, DispatchFiller, Ended, Fault, Finished, HeadlessClaude,
    Job, Output, Pause, Pid, Prompt, Reply, Stall, Started, SystemClock, Tokens, Usage, Wall, clock_ops, dispatch_ops,
};
use file_ports::{Bytes, Files};
use newtype::Newtype;
use port_core::{Calls, Snapshot};
use port_core::{Pass, Run};
use serde_json::Value;
use store_ports::{Store, Written};
use structurer::{Briefing, Directives, Lapse, SegmentRef, Uncovered};
use text_lines::Lines;
use transcript_ports::transcripts_ops::ReadFrom;

use super::{Applying, FakeLent, Invocation, Session, Speaker};

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
    const COVERING: &str = "place(aaaaaaaa-0000-4000-8000-000000000001.0, n1, mentions)\nsense(aaaaaaaa-0000-4000-8000-000000000003.0, decision, specific, firm, explicit)\nlink(aaaaaaaa-0000-4000-8000-000000000003.1, aaaaaaaa-0000-4000-8000-000000000005.0, answers)\nsense(aaaaaaaa-0000-4000-8000-000000000005.0, question, specific, firm, explicit)";
    const LISTED: &str = "bbbbbbbb-0000-4000-8000-000000000001";
    const SPANNING: &str = "conversational(bbbbbbbb-0000-4000-8000-000000000001.0)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.1)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.2)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.3)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.4)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.5)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.6)\nlink(bbbbbbbb-0000-4000-8000-000000000001.8, bbbbbbbb-0000-4000-8000-000000000001.7, refers-to)";
    const FORWARD: &str = "conversational(bbbbbbbb-0000-4000-8000-000000000001.0)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.1)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.2)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.3)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.4)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.5)\nconversational(bbbbbbbb-0000-4000-8000-000000000001.6)\nlink(bbbbbbbb-0000-4000-8000-000000000001.7, bbbbbbbb-0000-4000-8000-000000000001.8, refers-to)";
    const UNCOVERING: &str = "propose(new a, refines(n1), feature, \"x\")";
    const MISREAD: &str = "place(";
    const NINE: &str = "1. a\\n2. b\\n3. c\\n4. d\\n5. e\\n6. f\\n7. g\\n8. h\\n9. i";
    const SEVENTEEN: &str = "1. a\\n2. b\\n3. c\\n4. d\\n5. e\\n6. f\\n7. g\\n8. h\\n9. i\\n10. j\\n11. k\\n12. l\\n13. m\\n14. n\\n15. o\\n16. p\\n17. q";

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
    counted: Snapshot,
}

impl Structuring {
    fn over(script: Script, sleep: Pause) -> Result<Structuring, Failure> {
        Ok(Structuring { applying: Applying::ready()?, script, sleep })
    }

    fn structured(&self, arguments: &[&str]) -> Structured {
        self.structured_by(
            Agents::new(|_scratch| FakeDispatch(&self.script), FakeClock::asleep_for(self.sleep)),
            arguments,
        )
    }

    fn structured_by<D: DispatchFiller>(
        &self,
        agents: Agents<impl FnOnce(PathBuf) -> D, impl ClockFiller>,
        arguments: &[&str],
    ) -> Structured {
        let run = Run::root();
        let mut out = Vec::new();
        let outcome = Cli::new(
            &Invocation::environment(),
            FakeLent::new(&self.applying.files),
            |_path| self.applying.store.clone(),
            agents,
        )
        .report(&run, arguments.iter().map(OsString::from), &mut out);
        let counted = run.finish().settled().unwrap_or_default();
        let text = String::from_utf8(out).unwrap_or_default();
        let lines = Structured::parsed(&text);
        Structured { outcome, lines, text, counted }
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
        text.parse::<Lines>().into_iter().flatten().filter_map(|line| line.parse::<Line>().ok()).collect()
    }

    fn has(&self, wanted: &[Line]) -> Vec<Line> {
        wanted.iter().filter(|line| self.lines.iter().all(|had| had != *line)).cloned().collect()
    }
}

#[test]
fn a_reply_structuring_outside_its_job_or_repeating_a_job_is_retried_but_one_linking_to_an_earlier_job_applies()
-> Result<(), Failure> {
    let said = Script::NINE;
    let later = format!("link({}.8, {}.0, refers-to)", Script::LISTED, Script::LISTED);
    let structuring = Structuring {
        applying: Applying::over(&Session::working(Session::line(Speaker::Owner, Script::LISTED, "null", said)))?,
        script: Script::replying(&[Some(Script::SPANNING), Some(Script::FORWARD), Some(Script::FORWARD), Some(&later)]),
        sleep: Pause::new(0),
    };
    let before = structuring.unstructured()?;
    let structured = structuring.structured(&["structure"]);
    let after = structuring.unstructured()?;
    let outside = SegmentRef::new(Prefix::new(Script::LISTED.to_owned()), Ordinal::new(8));
    let retried = Line::Retried(Lapsed::new(JobId::new(1), Lapse::refused(&Unresolved::Outside(outside.clone()))));
    let repeated = Line::Retried(Lapsed::new(JobId::new(2), Lapse::from(Uncovered::new(outside, Vec::new()))));
    let ran = Line::Ran(Attempted::new(RunId::new(4), JobId::new(2), RunState::Replied, Some(Script::usage())));
    assert_eq!(structured.has(&[retried, repeated, ran]), []);
    assert_eq!(structured.counted.calls::<Unstructured>(), Calls::new(1));
    assert_eq!(structured.outcome?, Outcome::Structured { jobs: Count::new(2) });
    assert_eq!((before, after), (9, 0));
    let shown = structuring.applying.run(&["show", Script::LISTED])?;
    let entered = [RunId::new(2), RunId::new(4)].map(|run| {
        Structured::parsed(&shown)
            .into_iter()
            .filter(|line| matches!(line, Line::Annotated { entered, .. } if *entered == Provenance::Agent(run)))
            .count()
    });
    assert_eq!(entered, [8, 1]);
    Ok(())
}

#[test]
fn a_job_is_shown_the_questions_still_open_earlier_in_its_session_and_may_link_an_answer_to_one_but_not_place_one()
-> Result<(), Box<dyn Error>> {
    let asked = format!("{}.0", Session::ASKED);
    let [decided, other] = [Applying::answer(0), Applying::answer(1)];
    let later = "aaaaaaaa-0000-4000-8000-000000000005.0";
    let rest = format!("conversational({other})\nconversational({later})");
    let placing = format!("place({asked}, n1, mentions)\nconversational({decided})\n{rest}");
    let answering = format!("link({decided}, {asked}, answers)\n{rest}");
    let structuring = Structuring::over(Script::replying(&[Some(&placing), Some(&answering)]), Pause::new(0))?;
    structuring
        .applying
        .applied(&format!("place({asked}, n1, questions)\nsense({asked}, question, specific, firm, explicit)"))?;
    let open = structuring.applying.run(&["view", "open-questions"])?;
    let structured = structuring.structured(&["structure"]);
    let outside = SegmentRef::new(Prefix::new(Session::ASKED.to_owned()), Ordinal::new(0));
    let retried = Line::Retried(Lapsed::new(JobId::new(1), Lapse::refused(&Unresolved::Outside(outside))));
    let shown = format!("asked {asked} \"what of aaaaaaaa-0000-4000-8000-000000000004\"\n");
    let briefs = structuring.script.prompts().iter().flatten().map(ToString::to_string).collect::<Vec<String>>();
    assert_eq!(open.parse::<Lines>().into_iter().flatten().count(), 1);
    assert_eq!(briefs.iter().map(|brief| brief.matches(&shown).count()).collect::<Vec<usize>>(), [1, 1]);
    assert_eq!(structured.has(&[retried]), []);
    assert_eq!(structured.outcome?, Outcome::Structured { jobs: Count::new(1) });
    assert_eq!(structuring.applying.run(&["view", "open-questions"])?, "");
    Ok(())
}

#[test]
fn a_job_spanning_two_sessions_is_shown_each_session_s_earlier_open_questions() -> Result<(), Box<dyn Error>> {
    let [first, second] = ["aaaaaaaa-0000-4000-8000-000000000011", "aaaaaaaa-0000-4000-8000-000000000012"];
    let [first_answer, second_answer] =
        ["aaaaaaaa-0000-4000-8000-000000000013", "aaaaaaaa-0000-4000-8000-000000000014"];
    let lines = [
        Session::recorded("x", Speaker::Owner, first, ["null", "which crate", "2026-01-01"]),
        Session::recorded("y", Speaker::Owner, second, ["null", "which store", "2026-01-02"]),
        Session::recorded("x", Speaker::Owner, first_answer, ["null", "the first", "2026-01-03"]),
        Session::recorded("y", Speaker::Owner, second_answer, ["null", "the second", "2026-01-04"]),
    ];
    let applying = Applying::over(&Session::working(lines.concat()))?;
    let questions = [first, second]
        .map(|asked| format!("place({asked}.0, n1, questions)\nsense({asked}.0, question, specific, firm, explicit)"))
        .join("\n");
    applying.applied(&questions)?;
    let covering = [first_answer, second_answer].map(|said| format!("conversational({said}.0)")).join("\n");
    let structuring = Structuring { applying, script: Script::replying(&[Some(&covering)]), sleep: Pause::new(0) };
    let structured = structuring.structured(&["structure"]);
    let briefs = structuring.script.prompts().iter().flatten().map(ToString::to_string).collect::<Vec<String>>();
    let shown =
        [(first, "which crate"), (second, "which store")].map(|(asked, body)| format!("asked {asked}.0 \"{body}\"\n"));
    assert_eq!(structured.outcome?, Outcome::Structured { jobs: Count::new(1) });
    let counts =
        briefs.iter().map(|brief| shown.iter().map(|line| brief.matches(line).count()).collect::<Vec<usize>>());
    assert_eq!(counts.collect::<Vec<Vec<usize>>>(), [[1, 1]]);
    Ok(())
}

#[test]
fn a_flailed_job_holds_its_segments_and_the_run_goes_on_and_the_next_run_does_not_redispatch_them()
-> Result<(), Box<dyn Error>> {
    let u = Some(Script::UNCOVERING);
    let last = format!("conversational({}.8)", Script::LISTED);
    let structuring = Structuring {
        applying: Applying::over(&Session::working(Session::line(
            Speaker::Owner,
            Script::LISTED,
            "null",
            Script::NINE,
        )))?,
        script: Script::replying(&[u, u, Some(&last), u]),
        sleep: Pause::new(0),
    };
    let first = structuring.structured(&["structure"]);
    let told = first.outcome.as_ref().err().map(ToString::to_string).unwrap_or_default();
    assert!(matches!(first.outcome, Err(Failure::Flailed { sequel: Sequel::WentOn, .. })));
    let second = structuring.structured(&["structure"]);
    assert_eq!(structuring.script.prompts().len(), 3, "the second run re-sent the flailed job's segments");
    let shown = structuring.applying.run(&["show", "job", "1"])?;
    assert!(
        matches!(structuring.applying.run(&["show", "job", "3"]), Err(Failure::Unrun(job)) if job == JobId::new(3))
    );
    golden::golden!(
        "structure-flailed.txt",
        &format!("design structure\n{}{told}\ndesign structure\n{}design show job 1\n{shown}", first.text, second.text)
    )?;
    assert_eq!(second.outcome?, Outcome::Structured { jobs: Count::new(0) });
    Ok(())
}

#[test]
fn a_deadline_passed_during_a_sleep_stalls_the_run_loudly_before_its_next_job_and_the_next_run_sends_its_segments_again()
-> Result<(), Failure> {
    let chatter = |ordinals: Range<usize>| {
        ordinals.map(|ordinal| format!("conversational({}.{ordinal})", Script::LISTED)).collect::<Vec<String>>()
    };
    let [first, last] = [0..8, 8..9].map(|ordinals| chatter(ordinals).join("\n"));
    let structuring = Structuring {
        applying: Applying::over(&Session::working(Session::line(
            Speaker::Owner,
            Script::LISTED,
            "null",
            Script::NINE,
        )))?,
        script: Script::replying(&[None, Some(&first), Some(&last)]),
        sleep: Pause::new(3600),
    };
    let stalled = structuring.structured(&["structure"]);
    let owed = (structuring.unstructured()?, structuring.script.prompts().len());
    let again = structuring.structured(&["structure"]);
    let overdue = Line::Ran(Attempted::new(RunId::new(1), JobId::new(1), RunState::Overdue, None));
    let replied = [(2, 2), (3, 3)].map(|(run, job)| {
        Line::Ran(Attempted::new(RunId::new(run), JobId::new(job), RunState::Replied, Some(Script::usage())))
    });
    assert_eq!([stalled.has(&[overdue]), again.has(&replied)], [[], []]);
    assert!(matches!(stalled.outcome, Err(Failure::Stalled { stall: Stall::Overdue, .. })));
    assert_eq!((owed, structuring.script.prompts().len()), ((9, 1), 3));
    assert_eq!(again.outcome?, Outcome::Structured { jobs: Count::new(2) });
    Ok(())
}

#[test]
fn a_flail_then_a_stall_halts_the_stop_before_its_third_job_and_the_stop_tells_the_flail_and_the_stall_it_halted_on()
-> Result<(), Box<dyn Error>> {
    let u = Some(Script::UNCOVERING);
    let structuring = Structuring {
        applying: Applying::over(&Session::working(Session::line(
            Speaker::Owner,
            Script::LISTED,
            "null",
            Script::SEVENTEEN,
        )))?,
        script: Script::replying(&[u, u, None]),
        sleep: Pause::new(3600),
    };
    let stopped = structuring.structured(&["hook", "stop"]);
    let decision: Value = serde_json::from_str(&stopped.text)?;
    let segment = |ordinal| SegmentRef::new(Prefix::new(Script::LISTED.to_owned()), Ordinal::new(ordinal));
    let lapse = Lapse::from(Uncovered::new(segment(0), (1..8).map(segment).collect()));
    let flailed = Failure::Flailed {
        flails: Flails::new(Flail::new(JobId::new(1), lapse), Vec::new()),
        sequel: Sequel::Halted { job: JobId::new(2) },
    };
    let stalled = Failure::Stalled { job: JobId::new(2), stall: Stall::Overdue };
    let told = decision
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .parse::<Lines>()
        .into_iter()
        .flatten()
        .collect::<Vec<String>>();
    assert_eq!(told, [flailed.to_string(), stalled.to_string()]);
    assert_eq!(stopped.outcome?, Outcome::Blocked);
    assert_eq!((structuring.unstructured()?, structuring.script.prompts().len()), (9, 3));
    let overdue = Line::Ran(Attempted::new(RunId::new(3), JobId::new(2), RunState::Overdue, None));
    assert_eq!(Structured::parsed(&structuring.applying.run(&["show", "job", "2"])?), [overdue]);
    assert!(
        matches!(structuring.applying.run(&["show", "job", "3"]), Err(Failure::Unrun(job)) if job == JobId::new(3))
    );
    Ok(())
}

#[test]
fn a_retried_flail_owes_its_segments_again_and_the_next_run_sends_and_applies_them_while_a_job_that_did_not_flail_or_never_ran_is_refused()
-> Result<(), Box<dyn Error>> {
    let u = Some(Script::UNCOVERING);
    let structuring = Structuring::over(Script::replying(&[u, u, Some(Script::COVERING)]), Pause::new(0))?;
    let flailed = structuring.structured(&["structure"]);
    let held = structuring.unstructured()?;
    let retried = structuring.structured(&["retry", "1"]);
    let owed = structuring.unstructured()?;
    let applied = structuring.structured(&["structure"]);
    let again = structuring.structured(&["retry", "1"]);
    let unflailed = structuring.structured(&["retry", "2"]);
    let unrun = structuring.structured(&["retry", "99"]);
    let segment = |message, ordinal| {
        SegmentRef::new(Prefix::new(format!("aaaaaaaa-0000-4000-8000-00000000000{message}")), Ordinal::new(ordinal))
    };
    let released = [(1, 0), (3, 0), (3, 1), (5, 0)]
        .map(|(message, ordinal)| Line::Owed(Flailing::new(segment(message, ordinal), JobId::new(1), RunId::new(2))));
    assert!(matches!(flailed.outcome, Err(Failure::Flailed { sequel: Sequel::NoMoreJobs, .. })));
    assert_eq!((held, owed, structuring.unstructured()?), (0, 4, 0));
    assert_eq!(retried.lines, released);
    assert_eq!(retried.outcome?, Outcome::Retried { segments: Count::new(4) });
    assert_eq!(
        applied.has(&[Line::Ran(Attempted::new(
            RunId::new(3),
            JobId::new(2),
            RunState::Replied,
            Some(Script::usage())
        ))]),
        []
    );
    assert_eq!(applied.outcome?, Outcome::Structured { jobs: Count::new(1) });
    assert_eq!(structuring.script.prompts().len(), 3);
    assert!(matches!(again.outcome, Err(Failure::Unflailed(job)) if job == JobId::new(1)));
    assert!(matches!(unflailed.outcome, Err(Failure::Unflailed(job)) if job == JobId::new(2)));
    assert!(matches!(unrun.outcome, Err(Failure::Unrun(job)) if job == JobId::new(99)));
    assert_eq!((again.text.as_str(), unflailed.text.as_str(), unrun.text.as_str()), ("", "", ""));
    Ok(())
}

#[test]
fn a_misread_reply_is_retried_once_with_its_misread_in_the_prompt_then_flails() -> Result<(), Failure> {
    let structuring =
        Structuring::over(Script::replying(&[Some(Script::MISREAD), Some(Script::MISREAD)]), Pause::new(0))?;
    let structured = structuring.structured(&["structure"]);
    let lapse = Script::MISREAD.parse::<Directives>().err().map(Lapse::from);
    let prompts = structuring.script.prompts();
    let retried = prompts.first().cloned().flatten().zip(lapse.clone()).map(|(first, lapse)| first.retried(&lapse));
    assert_eq!(prompts.get(1).cloned().flatten(), retried);
    assert_eq!(prompts.len(), 2);
    let flailed = lapse.map(|lapse| Flails::new(Flail::new(JobId::new(1), lapse), Vec::new()));
    assert!(matches!(
        structured.outcome,
        Err(Failure::Flailed { flails, sequel: Sequel::NoMoreJobs }) if flailed.as_ref() == Some(&flails)
    ));
    Ok(())
}

#[test]
fn a_restarted_structure_kills_a_stored_run_past_its_deadline_whose_pid_and_start_time_match() -> Result<(), Failure> {
    let started = "Sat Nov  7 10:00:00 2026";
    let structuring =
        Structuring::over(Script::replying(&[Some(Script::COVERING)]).living(41, started), Pause::new(0))?;
    structuring.stored_run(41, started)?;
    let structured = structuring.structured(&["structure"]);
    let again = structuring.structured(&["structure"]);
    let reaped = [Line::Reaped { run: RunId::new(1), state: RunState::Killed }, Line::Interrupted(JobId::new(1))];
    assert_eq!(structured.has(&reaped), []);
    assert_eq!(again.has(&reaped), reaped);
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
    let structured = structuring.structured(&["structure"]);
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
    let store = run.port(structuring.applying.store.clone());
    let [other, gone] = [
        Entry::new("other".parse()?, Facet::Project, Entered::Init.into()),
        Entry::new("gone".parse()?, Facet::Project, Entered::Init.into()),
    ];
    store.write(Planting::sown(Node::new(NodeId::new(2), other)))?;
    store.write(Planting::sown(Node::new(NodeId::new(3), gone)))?;
    structuring
        .applying
        .applied("propose(new g, refines(n3), feature, \"g\")\npropose(new h, refines(n1), feature, \"h\")")?;
    structuring.applying.run(&["drop", "n3", "--reason", "abandoned"])?;
    structuring.applying.run(&["refine", "n5"])?;
    structuring.applying.run(&["specify", "n5", "--artifact", "behaviour:h", "--condition", "h holds"])?;
    golden::golden!("structure.txt", &format!("design structure\n{}", structuring.structured(&["structure"]).text))?;
    let briefs: String = structuring.script.prompts().iter().flatten().map(ToString::to_string).collect();
    golden::golden!("structure-briefs.txt", &briefs)?;
    assert!(matches!(structuring.applying.run(&["show", "n6"]), Err(Failure::Unplaced(_))));
    Ok(())
}

#[test]
fn an_agent_that_errs_writes_no_result_cannot_start_or_cannot_be_read_stalls_the_run_and_leaves_its_segments_owed()
-> Result<(), Box<dyn Error>> {
    let structuring = Structuring::over(Script::default(), Pause::new(0))?;
    let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let errs = Agent::new(
        "sh",
        &[
            "-c",
            r#"cat >/dev/null; echo '{"type":"result","subtype":"success","is_error":true,"result":"Invalid API key - Please run /login","usage":{"input_tokens":1,"output_tokens":2}}'"#,
        ],
    );
    let refuses = Agent::new("sh", &["-c", "cat >/dev/null; echo 'error: unknown option --tools' >&2; exit 1"]);
    let missing = Agent::new("/nonexistent/claude", &[]);
    let garbled = Agent::new("sh", &["-c", r"cat >/dev/null; printf '\377'"]);
    let reported = [errs, refuses, missing, garbled].map(|agent| {
        let structured = structuring.structured_by(
            Agents::new(|_scratch| HeadlessClaude::new(scratch.clone(), agent), SystemClock),
            &["structure"],
        );
        let failure = structured.outcome.as_ref().err().map(ToString::to_string).unwrap_or_default();
        format!("design structure\n{}{failure}\n", structured.text)
    });
    golden::golden!("structure-unreadable.txt", &reported.concat())?;
    let run = Run::root();
    let store = run.port(structuring.applying.store.clone());
    let jobs = [1, 2, 3, 4].map(|job| store.read(RunsOf::from(JobId::new(job))));
    let states = jobs.map(|ran| ran.map(|ran| ran.iter().map(Ran::state).collect::<Vec<RunState>>()));
    let [errored, unreadable, unstarted, failed] = states;
    assert_eq!(
        [errored?, unreadable?, unstarted?, failed?],
        [vec![RunState::Errored], vec![RunState::Unreadable], Vec::new(), vec![RunState::Failed]]
    );
    assert_eq!(structuring.unstructured()?, 4);
    Ok(())
}

#[test]
fn a_stop_whose_agent_cannot_be_started_blocks_loudly_and_closes_its_job_stalled() -> Result<(), Box<dyn Error>> {
    let structuring = Structuring::over(Script::default(), Pause::new(0))?;
    let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let stopped = [1, 2].map(|_stop| {
        let missing = Agent::new("/nonexistent/claude", &[]);
        structuring.structured_by(
            Agents::new(|_scratch| HeadlessClaude::new(scratch.clone(), missing), SystemClock),
            &["hook", "stop"],
        )
    });
    let [first, second] = stopped.map(|stopped| (serde_json::from_str::<Value>(&stopped.text), stopped.outcome));
    let reasons =
        [first.0?, second.0?].map(|decision| decision.get("reason").and_then(Value::as_str).map(str::to_owned));
    let stalled = [1, 2].map(|job| {
        let unstarted = Stall::Unspawned(Fault::from(DispatchFailure::Spawn(io::Error::from_raw_os_error(2))));
        Some(Failure::Stalled { job: JobId::new(job), stall: unstarted }.to_string())
    });
    let interrupted = Line::Interrupted(JobId::new(1));
    assert_eq!(reasons, stalled);
    assert_eq!([first.1?, second.1?], [Outcome::Blocked, Outcome::Blocked]);
    assert_eq!(structuring.structured(&["structure"]).has(slice::from_ref(&interrupted)), [interrupted]);
    assert_eq!(structuring.unstructured()?, 4);
    Ok(())
}

#[test]
fn a_stop_ingests_what_was_said_since_and_structures_it_and_a_stop_after_it_reads_and_dispatches_nothing()
-> Result<(), Box<dyn Error>> {
    let later = "aaaaaaaa-0000-4000-8000-000000000006";
    let covering = format!("{}\nconversational({later}.0)", Script::COVERING);
    let structuring = Structuring::over(Script::replying(&[Some(&covering)]), Pause::new(0))?;
    let linked = Session::linked();
    let said = [linked.lines, Session::line(Speaker::Owner, later, "null", "and later")].concat();
    structuring.applying.files.write(&Path::new(linked.directory).join("s.jsonl"), Bytes::from(&said))?;
    let stopped = structuring.structured(&["hook", "stop"]);
    let again = structuring.structured(&["hook", "stop"]);
    let counted = [&stopped, &again]
        .map(|hooked| [hooked.counted.calls::<ReadFrom>(), hooked.counted.calls::<dispatch_ops::Dispatch>()]);
    assert_eq!(counted, [[Calls::new(1), Calls::new(1)], [Calls::new(0), Calls::new(0)]]);
    assert_eq!(structuring.unstructured()?, 0);
    assert_eq!(again.text, "");
    assert_eq!(
        [stopped.outcome?, again.outcome?],
        [Outcome::Hooked(HookEvent::Stop), Outcome::Hooked(HookEvent::Stop)]
    );
    Ok(())
}

#[test]
fn a_prompt_ingests_silently_and_leaves_what_it_ingested_to_the_stop() -> Result<(), Box<dyn Error>> {
    let structuring = Structuring::over(Script::default(), Pause::new(0))?;
    let before = structuring.unstructured()?;
    let linked = Session::linked();
    let said = [linked.lines, Session::line(Speaker::Owner, "aaaaaaaa-0000-4000-8000-000000000006", "null", "more")];
    structuring.applying.files.write(&Path::new(linked.directory).join("s.jsonl"), Bytes::from(&said.concat()))?;
    let prompted = structuring.structured(&["hook", "user-prompt-submit"]);
    assert_eq!(prompted.counted.calls::<dispatch_ops::Dispatch>(), Calls::new(0));
    assert_eq!((prompted.text.as_str(), structuring.unstructured()?), ("", before + 1));
    assert_eq!(prompted.outcome?, Outcome::Hooked(HookEvent::UserPromptSubmit));
    Ok(())
}

#[test]
fn a_stop_that_flails_blocks_loudly_and_the_stop_after_it_passes_with_the_segments_held() -> Result<(), Box<dyn Error>>
{
    let u = Some(Script::UNCOVERING);
    let structuring = Structuring::over(Script::replying(&[u, u]), Pause::new(0))?;
    let flailed = structuring.structured(&["hook", "stop"]);
    let again = structuring.structured(&["hook", "stop"]);
    let decision: Value = serde_json::from_str(&flailed.text)?;
    let segment = |message, ordinal| {
        SegmentRef::new(Prefix::new(format!("aaaaaaaa-0000-4000-8000-00000000000{message}")), Ordinal::new(ordinal))
    };
    let uncovered = Uncovered::new(segment(1, 0), vec![segment(3, 0), segment(3, 1), segment(5, 0)]);
    let flails = Flails::new(Flail::new(JobId::new(1), Lapse::from(uncovered)), Vec::new());
    let told = Failure::Flailed { flails, sequel: Sequel::NoMoreJobs }.to_string();
    assert_eq!(decision.get("decision").and_then(Value::as_str), Some("block"));
    assert_eq!(decision.get("reason").and_then(Value::as_str), Some(told.as_str()));
    assert_eq!(flailed.outcome?, Outcome::Blocked);
    assert_eq!((again.outcome?, again.text.as_str()), (Outcome::Hooked(HookEvent::Stop), ""));
    Ok(())
}

#[test]
fn a_stop_whose_agent_stalls_blocks_loudly_and_the_stop_after_it_sends_the_segments_again() -> Result<(), Box<dyn Error>>
{
    let structuring = Structuring::over(Script::replying(&[None, Some(Script::COVERING)]), Pause::new(3600))?;
    let stalled = structuring.structured(&["hook", "stop"]);
    let owed = structuring.unstructured()?;
    let again = structuring.structured(&["hook", "stop"]);
    let decision: Value = serde_json::from_str(&stalled.text)?;
    assert_eq!(decision.get("decision").and_then(Value::as_str), Some("block"));
    assert_eq!(stalled.outcome?, Outcome::Blocked);
    assert_eq!((owed, structuring.unstructured()?), (4, 0));
    assert_eq!(again.outcome?, Outcome::Hooked(HookEvent::Stop));
    Ok(())
}

#[test]
fn probe_a_retried_and_applied_flail_is_not_told_as_held() -> Result<(), Box<dyn Error>> {
    let u = Some(Script::UNCOVERING);
    let structuring = Structuring::over(Script::replying(&[u, u, Some(Script::COVERING)]), Pause::new(0))?;
    let flailed = structuring.structured(&["structure"]);
    let retried = structuring.structured(&["retry", "1"]);
    let applied = structuring.structured(&["structure"]);
    let stopped = structuring.structured(&["hook", "stop"]);
    assert!(matches!(flailed.outcome, Err(Failure::Flailed { sequel: Sequel::NoMoreJobs, .. })));
    assert_eq!(retried.outcome?, Outcome::Retried { segments: Count::new(4) });
    assert_eq!((applied.outcome?, structuring.unstructured()?), (Outcome::Structured { jobs: Count::new(1) }, 0));
    assert_eq!((stopped.outcome?, stopped.text.as_str()), (Outcome::Hooked(HookEvent::Stop), ""));
    Ok(())
}

#[test]
fn probe_review_a_retried_flail_is_not_told_as_held_before_its_retry_runs() -> Result<(), Box<dyn Error>> {
    let u = Some(Script::UNCOVERING);
    let structuring = Structuring::over(Script::replying(&[u, u]), Pause::new(0))?;
    let flailed = structuring.structured(&["structure"]);
    let retried = structuring.structured(&["retry", "1"]);
    let prompted = structuring.structured(&["hook", "user-prompt-submit"]);
    assert!(matches!(flailed.outcome, Err(Failure::Flailed { sequel: Sequel::NoMoreJobs, .. })));
    assert_eq!(retried.outcome?, Outcome::Retried { segments: Count::new(4) });
    assert_eq!((prompted.outcome?, prompted.text.as_str()), (Outcome::Hooked(HookEvent::UserPromptSubmit), ""));
    assert_eq!(structuring.unstructured()?, 4);
    Ok(())
}

#[test]
fn probe_review_b_a_retried_flail_that_flails_again_is_told_once_under_its_new_job() -> Result<(), Box<dyn Error>> {
    let u = Some(Script::UNCOVERING);
    let structuring = Structuring::over(Script::replying(&[u, u, u, u]), Pause::new(0))?;
    let flailed = structuring.structured(&["structure"]);
    let retried = structuring.structured(&["retry", "1"]);
    let again = structuring.structured(&["structure"]);
    let stopped = structuring.structured(&["hook", "stop"]);
    let first = structuring.structured(&["retry", "1"]);
    let second = structuring.structured(&["retry", "2"]);
    let decision: Value = serde_json::from_str(&stopped.text)?;
    let segment = |message, ordinal| {
        SegmentRef::new(Prefix::new(format!("aaaaaaaa-0000-4000-8000-00000000000{message}")), Ordinal::new(ordinal))
    };
    let uncovered = Uncovered::new(segment(1, 0), vec![segment(3, 0), segment(3, 1), segment(5, 0)]);
    let flails = Flails::new(Flail::new(JobId::new(2), Lapse::from(uncovered)), Vec::new());
    let told = Failure::Flailed { flails, sequel: Sequel::NoMoreJobs }.to_string();
    assert!(matches!(flailed.outcome, Err(Failure::Flailed { sequel: Sequel::NoMoreJobs, .. })));
    assert_eq!(retried.outcome?, Outcome::Retried { segments: Count::new(4) });
    assert!(matches!(again.outcome, Err(Failure::Flailed { sequel: Sequel::NoMoreJobs, .. })));
    assert_eq!(decision.get("reason").and_then(Value::as_str), Some(told.as_str()));
    assert_eq!(stopped.outcome?, Outcome::Blocked);
    assert!(matches!(first.outcome, Err(Failure::Unflailed(job)) if job == JobId::new(1)));
    assert_eq!(second.outcome?, Outcome::Retried { segments: Count::new(4) });
    Ok(())
}
