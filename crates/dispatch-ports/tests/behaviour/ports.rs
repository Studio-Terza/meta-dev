use std::error::Error;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use dispatch_ports::{
    Agent, AgentRun, Clock, Deadline, Dispatch, Ended, Finished, HeadlessClaude, Job, Pause, Pid, Prompt, Reply, Stall,
    Started, SystemClock, Timeout, Tokens, Usage, Watch, Watched, Woke,
};
use port_core::Run;

use super::{FakeClock, Sleep, Wait, Waits, Watching};

#[test]
fn a_live_process_group_has_a_start_time_until_it_is_killed() -> Result<(), Box<dyn Error>> {
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
fn an_agent_that_prints_its_result_is_watched_to_that_reply_and_usage() -> Result<(), Box<dyn Error>> {
    let printed = r#"{"subtype":"success","result":"conversational(a.0)","usage":{"input_tokens":4,"output_tokens":9,"cache_creation_input_tokens":2,"cache_read_input_tokens":7}}"#;
    let run = Run::root();
    let watched = {
        let echo =
            run.port(HeadlessClaude::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")), Agent::new("sh", &["-c", "cat"])));
        let clock = run.port(SystemClock);
        let deadline = Deadline::after(clock.now(), Timeout::new(5));
        let held = echo.dispatch(&Job::due_by(deadline), &Prompt::new(printed.to_owned()))?;
        Watch::over(&echo, &clock).watched(held)?
    };
    run.finish().settled()?;
    let usage = Usage::new(Tokens::new(4), Tokens::new(9), Tokens::new(2), Tokens::new(7));
    assert_eq!(watched, Watched::Finished(Finished::new(Reply::new("conversational(a.0)".to_owned()), usage)));
    Ok(())
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
            let held = agent.dispatch(&Job::due_by(deadline), &Prompt::new("x".repeat(Stalling::PROMPT))).ok()?;
            let pid = held.pid();
            let watched = Watch::over(&agent, &clock).watched(held).ok()?;
            Some(Stalled { watched, left: agent.started(pid).ok()? })
        };
        run.finish().settled().ok()?;
        ended
    }
}

laws::table!(
    an_agent_that_leaves_a_large_prompt_unread_is_killed_and_reaped_at_its_deadline,
    Stalling::watched,
    [
        &["-c", "exec sleep 30"][..] => Some(Stalled { watched: Watched::Stalled(Stall::Overdue), left: None }),
        &["-c", "exec 0<&-; exec sleep 30"][..] => Some(Stalled { watched: Watched::Stalled(Stall::Overdue), left: None }),
    ]
);

#[test]
fn a_wait_nobody_rings_lasts_its_full_pause() -> Result<(), Box<dyn Error>> {
    let pause = Pause::new(1);
    let waits = Waits::default();
    let run = Run::root();
    let waited = {
        let agent = run.port(HeadlessClaude::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            Agent::new("sh", &["-c", "exec sleep 30"]),
        ));
        let clock = run.port(FakeClock::new(Sleep::STILL, &waits));
        let silent = agent
            .dispatch(&Job::due_by(Deadline::after(clock.now(), Timeout::new(60))), &Prompt::new("x".to_owned()))?;
        let waited = clock.waited(pause, silent.bell());
        silent.end()?;
        waited
    };
    run.finish().settled()?;
    assert_eq!((waited, waits.recorded()), (Watching::START.after(pause), vec![Wait { pause, woke: Woke::Lapsed }]));
    Ok(())
}

#[test]
fn quiet_bell_waits_out_its_pause() -> Result<(), Box<dyn Error>> {
    let pause = Duration::from_millis(50);
    let run = Run::root();
    let quiet = {
        let clock = run.port(SystemClock);
        AgentRun::new(Pid::new(1), Deadline::after(clock.now(), Timeout::new(5)))
    };
    run.finish().settled()?;
    let started = Instant::now();
    let woke = quiet.bell().waited(pause);
    assert_eq!((woke, started.elapsed() >= pause), (Woke::Lapsed, true));
    Ok(())
}

#[test]
fn a_ring_wakes_one_wait_and_is_then_spent() -> Result<(), Box<dyn Error>> {
    let waits = Waits::default();
    let run = Run::root();
    let woke = {
        let agent = run.port(HeadlessClaude::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            Agent::new("sh", &["-c", "exec sleep 30"]),
        ));
        let clock = run.port(FakeClock::new(Sleep::STILL, &waits));
        let held = agent
            .dispatch(&Job::due_by(Deadline::after(clock.now(), Timeout::new(60))), &Prompt::new("x".to_owned()))?;
        let before = held.bell().waited(Duration::ZERO);
        held.bell().ringer().ring();
        let rung = held.bell().waited(Duration::ZERO);
        let after = held.bell().waited(Duration::ZERO);
        held.end()?;
        [before, rung, after]
    };
    run.finish().settled()?;
    assert_eq!(woke, [Woke::Lapsed, Woke::Rung, Woke::Lapsed]);
    Ok(())
}
