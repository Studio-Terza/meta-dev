use std::error::Error;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use dispatch_ports::{
    Agent, Clock, Deadline, Dispatch, Ended, Finished, HeadlessClaude, Job, Pid, Prompt, Reply, SystemClock, Timeout,
    Tokens, Usage, Watch, Watched,
};
use port_core::Run;

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
