use std::error::Error;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use dispatch_ports::{Dispatch, Ended, HeadlessClaude, Pid};
use port_core::Run;

#[test]
fn a_live_process_group_has_a_start_time_until_it_is_killed() -> Result<(), Box<dyn Error>> {
    let mut sleeper = Command::new("sleep").arg("30").process_group(0).spawn()?;
    let pid = Pid::new(u64::from(sleeper.id()));
    let run = Run::root();
    let claude = run.port(HeadlessClaude::from(PathBuf::new()));
    let before = claude.started(pid)?.is_some();
    let killed = claude.kill(pid)?;
    sleeper.wait()?;
    let after = claude.started(pid)?;
    let again = claude.kill(pid)?;
    assert_eq!((before, killed, after, again), (true, Ended::Killed, None, Ended::Absent));
    Ok(())
}
