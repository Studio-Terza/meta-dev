use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use test_failure::Failed;

use dispatch_ports::{
    Agent, Call, Clock, Deadline, Dispatch, Ended, Finished, HeadlessClaude, Job, Model, Pause, Pid, Printed, Prompt,
    Ran, Reply, Shell, Stall, Started, SystemClock, SystemShell, Timeout, Tokens, Usage, Watch, Watched, Woke,
    shell_ops,
};
use invariants::Ignoring;
use port_core::{Calls, Run};

use super::{FakeClock, Ignored, Sleep, Wait, Waits, Watching};

#[test]
fn a_live_process_group_has_a_start_time_until_it_is_killed() -> Result<(), Failed> {
    let mut sleeper = Command::new("sleep").arg("30").process_group(0).spawn()?;
    let pid = Pid::new(sleeper.id());
    let run = Run::root();
    let claude = run.port(HeadlessClaude::new(PathBuf::new(), Agent::CLAUDE));
    let before = claude.started(pid)?.is_some();
    let killed = claude.kill(pid)?;
    sleeper.wait()?;
    let after = claude.started(pid)?;
    let again = claude.kill(pid)?;
    assert_eq!((before, killed, after, again), (true, Ended::Killed, None, Ended::Absent));
    Ok(())
}

#[test]
fn an_agent_that_prints_its_result_is_watched_to_that_reply_and_usage() -> Result<(), Failed> {
    let printed = r#"{"subtype":"success","result":"conversational(a.0)","usage":{"input_tokens":4,"output_tokens":9,"cache_creation_input_tokens":2,"cache_read_input_tokens":7}}"#;
    let run = Run::root();
    let watched = {
        let echo =
            run.port(HeadlessClaude::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")), Agent::new("sh", &["-c", "cat"])));
        let clock = run.port(SystemClock);
        let deadline = Deadline::after(clock.now(), Timeout::new(2));
        let held = echo.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::new(printed.to_owned()))?;
        Watch::over(&echo, &clock).watched(held)?
    };
    run.finish().settled()?;
    let usage = Usage::new(Tokens::new(4), Tokens::new(9), Tokens::new(2), Tokens::new(7));
    assert_eq!(watched, Watched::Finished(Finished::new(Reply::new("conversational(a.0)".to_owned()), usage)));
    Ok(())
}

const MODELLED: [&str; 2] = [
    "-c",
    r#"cat >/dev/null; printf '{"subtype":"success","result":"%s %s","usage":{"input_tokens":1,"output_tokens":1}}' "$0" "$1""#,
];

struct Modelled;

impl Modelled {
    fn watched(model: Model) -> Option<Watched> {
        let run = Run::root();
        let watched = {
            let echo =
                run.port(HeadlessClaude::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")), Agent::new("sh", &MODELLED)));
            let clock = run.port(SystemClock);
            let deadline = Deadline::after(clock.now(), Timeout::new(2));
            let held = echo
                .dispatch(&Job::due_by(deadline, model), &Prompt::new(String::new()))
                .ignoring(Ignored::Unstalled)?;
            Watch::over(&echo, &clock).watched(held).ignoring(Ignored::Unstalled)?
        };
        run.finish().settled().ignoring(Ignored::Unstalled)?;
        Some(watched)
    }

    fn told(said: &str) -> Watched {
        let usage = Usage::new(Tokens::new(1), Tokens::new(1), Tokens::new(0), Tokens::new(0));
        Watched::Finished(Finished::new(Reply::new(said.to_owned()), usage))
    }
}

#[test]
fn an_agent_is_told_the_model_its_job_names() {
    assert_eq!(
        [Modelled::watched(Model::Haiku), Modelled::watched(Model::Sonnet)],
        [Some(Modelled::told("--model haiku")), Some(Modelled::told("--model sonnet"))]
    );
}

struct Stalling;

#[derive(Debug, PartialEq, Eq)]
struct Stalled {
    watched: Watched,
    left: Option<Started>,
}

impl Stalling {
    const PROMPT: usize = 256 * 1024;

    fn watched(arguments: &&'static [&'static str]) -> Option<Stalled> {
        let waits = Waits::default();
        let run = Run::root();
        let ended = {
            let agent =
                run.port(HeadlessClaude::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")), Agent::new("sh", arguments)));
            let clock = run.port(FakeClock::new(Sleep::STILL, &waits));
            let deadline = Deadline::after(clock.now(), Timeout::new(1));
            let held = agent
                .dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::new("x".repeat(Stalling::PROMPT)))
                .ignoring(Ignored::Unstalled)?;
            let pid = held.pid();
            let watched = Watch::over(&agent, &clock).watched(held).ignoring(Ignored::Unstalled)?;
            Some(Stalled { watched, left: agent.started(pid).ignoring(Ignored::Unstalled)? })
        };
        run.finish().settled().ignoring(Ignored::Unstalled)?;
        ended
    }
}

#[test]
fn an_agent_that_leaves_a_large_prompt_unread_is_killed_and_reaped_at_its_deadline() {
    for arguments in [&["-c", "exec sleep 30"][..], &["-c", "exec 0<&-; exec sleep 30"][..]] {
        assert_eq!(
            Stalling::watched(&arguments),
            Some(Stalled { watched: Watched::Stalled(Stall::Overdue), left: None }),
            "{arguments:?}"
        );
    }
}

#[test]
fn a_bell_lapses_until_rung_spends_each_ring_once_and_rings_when_its_agent_finishes() -> Result<(), Failed> {
    let (pause, quiet) = (Pause::new(1), Duration::from_millis(50));
    let waits = Waits::default();
    let run = Run::root();
    let woke = {
        let clock = run.port(FakeClock::new(Sleep::STILL, &waits));
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let sleeping = run.port(HeadlessClaude::new(directory.clone(), Agent::new("sh", &["-c", "exec sleep 30"])));
        let speaking = run.port(HeadlessClaude::new(directory, Agent::new("sh", &["-c", "echo spoken"])));
        let job = Job::due_by(Deadline::after(clock.now(), Timeout::new(60)), Model::Haiku);
        let silent = sleeping.dispatch(&job, &Prompt::new("x".to_owned()))?;
        let waited = clock.waited(pause, silent.bell());
        let started = Instant::now();
        let lapsed = (silent.bell().waited(quiet), started.elapsed() >= quiet);
        silent.bell().ringer().ring();
        let [rung, after] = [(); 2].map(|()| silent.bell().waited(Duration::ZERO));
        silent.end()?;
        let finished = speaking.dispatch(&job, &Prompt::new("x".to_owned()))?;
        let spoken = finished.bell().waited(Duration::from_secs(5));
        finished.end()?;
        (waited, lapsed, [rung, after, spoken])
    };
    run.finish().settled()?;
    assert_eq!(woke, (Watching::START.after(pause), (Woke::Lapsed, true), [Woke::Rung, Woke::Lapsed, Woke::Rung]));
    assert_eq!(waits.recorded(), vec![Wait { pause, woke: Woke::Lapsed }]);
    Ok(())
}

#[test]
fn a_shell_call_returns_its_printing_and_whether_it_succeeded_failed_or_ran_past_its_deadline() -> Result<(), Failed> {
    let directory = env!("CARGO_MANIFEST_DIR");
    let run = Run::root();
    let ran = {
        let shell = run.port(SystemShell);
        let clock = run.port(SystemClock);
        let call = |script: &str, seconds: u32| {
            let spoken = vec!["-c".to_owned(), script.to_owned()];
            Call::new("sh", spoken, PathBuf::from(directory), Deadline::after(clock.now(), Timeout::new(seconds)))
        };
        [
            shell.ran(&call("pwd", 5))?,
            shell.ran(&call("printf refused; exit 3", 5))?,
            shell.ran(&call("exec sleep 4", 0))?,
        ]
    };
    let calls = run.finish().settled()?.calls::<shell_ops::Ran>();
    assert_eq!(
        ran,
        [
            Ran::Succeeded(Printed::new(format!("{directory}\n"))),
            Ran::Failed(Printed::new("refused".to_owned())),
            Ran::Overdue
        ]
    );
    assert_eq!(calls, Calls::new(3));
    assert_eq!(ran.iter().filter_map(Ran::succeeded).count(), 1);
    assert_eq!(ran.iter().filter_map(Ran::printed).count(), 2);
    Ok(())
}
