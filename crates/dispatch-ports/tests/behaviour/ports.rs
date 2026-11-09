use std::error::Error;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use dispatch_ports::{
    Agent, Clock, Deadline, Dispatch, Ended, Finished, HeadlessClaude, Job, Pause, Pid, Prompt, Reply, Stall, Started,
    SystemClock, Timeout, Tokens, Usage, Watch, Watched,
};
use port_core::Run;

use super::{FakeClock, Sleep, Watching};

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
        let deadline = Deadline::after(clock.now(), Timeout::new(60));
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
        let run = Run::root();
        let ended = {
            let agent =
                run.port(HeadlessClaude::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")), Agent::new("sh", arguments)));
            let clock = run.port(FakeClock {
                now: Mutex::new(Watching::START),
                sleep: Sleep { after: 0, slept: Pause::new(0) },
                waits: Mutex::new(0),
            });
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
fn a_bell_rung_by_an_earlier_run_does_not_cut_the_next_run_short() -> Result<(), Box<dyn Error>> {
    let run = Run::root();
    let elapsed = {
        let agent = run.port(HeadlessClaude::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            Agent::new("sh", &["-c", "case \"$(cat)\" in x) exec sleep 30;; *) echo '{\"subtype\":\"success\",\"result\":\"r\",\"usage\":{\"input_tokens\":1,\"output_tokens\":1}}';; esac"]),
        ));
        let clock = run.port(SystemClock);
        let first = agent
            .dispatch(&Job::due_by(Deadline::after(clock.now(), Timeout::new(60))), &Prompt::new("y".to_owned()))?;
        thread::sleep(Duration::from_millis(500));
        let _done = Watch::over(&agent, &clock).watched(first)?;
        let began = Instant::now();
        let second = agent
            .dispatch(&Job::due_by(Deadline::after(clock.now(), Timeout::new(6))), &Prompt::new("x".to_owned()))?;
        let watched = Watch::over(&agent, &clock).watched(second)?;
        assert_eq!(watched, Watched::Stalled(Stall::Overdue));
        began.elapsed()
    };
    run.finish().settled()?;
    assert!(elapsed.as_secs_f64() >= 5.0, "killed after {elapsed:?}, before its 6 s deadline");
    Ok(())
}

#[test]
fn a_wait_nobody_rings_lasts_its_full_pause() -> Result<(), Box<dyn Error>> {
    let pause = Pause::new(1);
    let run = Run::root();
    let elapsed = {
        let agent = run.port(HeadlessClaude::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            Agent::new("sh", &["-c", "exec sleep 30"]),
        ));
        let clock = run.port(SystemClock);
        let silent = agent
            .dispatch(&Job::due_by(Deadline::after(clock.now(), Timeout::new(60))), &Prompt::new("x".to_owned()))?;
        let began = Instant::now();
        clock.waited(pause, silent.bell());
        let elapsed = began.elapsed();
        silent.end()?;
        elapsed
    };
    run.finish().settled()?;
    assert!(elapsed >= Duration::from(pause), "an unrung wait returned after {elapsed:?}, before its {pause:?}");
    Ok(())
}
