//! The process boundary: a failed command's stderr is lossy text from another process, shown as written.
#![allow(clippy::disallowed_methods)]

use crate::PortError;
use nix::errno::Errno;
use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// How long one port command may run before it is killed and reported as a port error.
pub const PORT_TIMEOUT: Duration = Duration::from_secs(30);

/// How often a running command is checked for having finished.
const POLL: Duration = Duration::from_millis(10);

/// How long a timed-out command's group has, after SIGTERM, to end before SIGKILL.
const GRACE: Duration = Duration::from_millis(200);

fn drain(pipe: Option<impl Read + Send + 'static>) -> mpsc::Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        let _ = sender.send(bytes);
    });
    receiver
}

/// How long a group killed with SIGKILL has to be gone before it is reported as still there.
const CONFIRM: Duration = Duration::from_secs(2);

fn refused(group: Pid, error: Errno) -> PortError {
    PortError::new(format!("process group {group}: {error}"))
}

/// SIGTERM, a short grace, then SIGKILL, with no look at the outcome after it. False when the
/// group was already gone; an error when the first signal is refused (no permission, say). After
/// that first signal the group is ours to signal, so a later refusal is the process table's way of
/// saying that only zombies are left (macOS answers EPERM for them).
fn signal_end(group: Pid) -> Result<bool, PortError> {
    match killpg(group, Signal::SIGTERM) {
        Ok(()) => {}
        Err(Errno::ESRCH) => return Ok(false),
        Err(error) => return Err(refused(group, error)),
    }
    thread::sleep(GRACE);
    match killpg(group, Signal::SIGKILL) {
        Ok(()) | Err(Errno::ESRCH | Errno::EPERM) => Ok(true),
        Err(error) => Err(refused(group, error)),
    }
}

/// Whether the group still has a live member: signal 0 finds a member, but a leader that is a
/// zombie (nobody has waited for it) is gone for our purposes, and so is a group only zombies
/// remain in.
fn remains(group: Pid, leader: u32) -> Result<bool, PortError> {
    match killpg(group, None) {
        Ok(()) => {
            let state = run_found("ps", &["-o", "stat=", "-p", &leader.to_string()])?;
            Ok(state.is_some_and(|state| !state.trim_start().starts_with('Z')))
        }
        Err(Errno::ESRCH | Errno::EPERM) => Ok(false),
        Err(error) => Err(refused(group, error)),
    }
}

/// Ends a whole process group, so that what its leader started goes too: SIGTERM, a short grace,
/// then SIGKILL. A group that is already gone is not an error. Ok means the group is gone: a kill
/// that fails, or a group that is still there after `CONFIRM`, is an error.
pub fn end_group(leader: u32) -> Result<(), PortError> {
    let group = i32::try_from(leader)
        .ok()
        .filter(|id| *id > 1)
        .map(Pid::from_raw)
        .ok_or_else(|| PortError::new(format!("process group {leader}: not a group to kill")))?;
    if !signal_end(group)? {
        return Ok(());
    }
    let deadline = Instant::now() + CONFIRM;
    while remains(group, leader)? {
        if Instant::now() >= deadline {
            return Err(PortError::new(format!("process group {leader}: still there after SIGKILL")));
        }
        thread::sleep(POLL);
    }
    Ok(())
}

/// Ends the command and its process group. The leader is this process's child and stays a zombie
/// until waited for, so the group is not confirmed gone here.
fn terminate(child: &mut Child) {
    let group = i32::try_from(child.id()).ok().map(Pid::from_raw);
    group.into_iter().for_each(|group| drop(signal_end(group)));
    let _ = child.kill();
    let _ = child.wait();
}

/// Runs the command, as the leader of its own process group, to its end and the end of its
/// output, or ends the group at `limit` and fails. A process that left the group while holding a
/// pipe leaves its reader behind, not waited for, so that a hang cannot pass on.
fn execute(
    program: &str,
    args: &[&str],
    limit: Duration,
    prepare: impl FnOnce(&mut Command),
) -> Result<Output, PortError> {
    let shown = format!("{program} {}", args.join(" "));
    let failed = |error: std::io::Error| PortError::new(format!("{shown}: {error}"));
    let timed_out = || PortError::new(format!("{shown}: timed out after {} s", limit.as_secs_f64()));
    let mut command = Command::new(program);
    prepare(&mut command);
    let mut child = command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(failed)?;
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let deadline = Instant::now() + limit;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(failed)? {
            break status;
        }
        if Instant::now() >= deadline {
            terminate(&mut child);
            return Err(timed_out());
        }
        thread::sleep(POLL);
    };
    let mut read = |reader: mpsc::Receiver<Vec<u8>>| match reader
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
    {
        Ok(bytes) => Ok(bytes),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            terminate(&mut child);
            Err(timed_out())
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(PortError::new(format!("{shown}: unreadable output"))),
    };
    Ok(Output { status, stdout: read(stdout)?, stderr: read(stderr)? })
}

pub fn run(program: &str, args: &[&str]) -> Result<String, PortError> {
    run_within(program, args, PORT_TIMEOUT, |_| ())
}

/// Like `run`, with the command prepared first, as a worker's narrowed environment is.
pub fn run_prepared(program: &str, args: &[&str], prepare: impl FnOnce(&mut Command)) -> Result<String, PortError> {
    run_within(program, args, PORT_TIMEOUT, prepare)
}

fn run_within(
    program: &str,
    args: &[&str],
    limit: Duration,
    prepare: impl FnOnce(&mut Command),
) -> Result<String, PortError> {
    let shown = format!("{program} {}", args.join(" "));
    let output = execute(program, args, limit, prepare)?;
    match output.status.success() {
        true => String::from_utf8(output.stdout).map_err(|error| PortError::new(format!("{shown}: {error}"))),
        false => Err(PortError::new(format!("{shown}: {}", String::from_utf8_lossy(&output.stderr)))),
    }
}

/// Like `run`, for a lookup whose "not found" is the command's quiet exit status 1 with nothing on
/// stderr, as with `git rev-parse --verify --quiet`: that is `None`; any other failure is an error.
pub fn run_found(program: &str, args: &[&str]) -> Result<Option<String>, PortError> {
    found_with(program, args, |_| ())
}

fn found_with(program: &str, args: &[&str], prepare: impl FnOnce(&mut Command)) -> Result<Option<String>, PortError> {
    let shown = format!("{program} {}", args.join(" "));
    let output = execute(program, args, PORT_TIMEOUT, prepare)?;
    match (output.status.success(), output.status.code(), output.stderr.is_empty()) {
        (true, ..) => {
            String::from_utf8(output.stdout).map(Some).map_err(|error| PortError::new(format!("{shown}: {error}")))
        }
        (false, Some(1), true) => Ok(None),
        (false, ..) => Err(PortError::new(format!("{shown}: {}", String::from_utf8_lossy(&output.stderr)))),
    }
}

/// A yes-or-no command, such as `git merge-base --is-ancestor`: exit 0 is yes and a quiet exit 1
/// is no; anything else is an error.
pub fn decides_with(program: &str, args: &[&str], prepare: impl FnOnce(&mut Command)) -> Result<bool, PortError> {
    found_with(program, args, prepare).map(|found| found.is_some())
}

#[cfg(test)]
mod tests {
    use super::{decides_with, end_group, run, run_found, run_within};
    use meta_testkit::Scratch;
    use std::fs;
    use std::process::Command;
    use std::time::{Duration, Instant};

    /// Whether the process is gone; a zombie that nobody has reaped yet is gone too.
    fn gone(pid: &str) -> bool {
        let listed = Command::new("ps").args(["-o", "stat=", "-p", pid]).output().unwrap();
        listed.stdout.first().is_none_or(|stat| *stat == b'Z')
    }

    fn written_pid(file: &std::path::Path) -> String {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(10) {
            if let Ok(pid) = fs::read_to_string(file)
                && !pid.is_empty()
            {
                return pid;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("the grandchild never wrote its pid");
    }

    fn assert_ends_gone(pid: &str) {
        let started = Instant::now();
        while !gone(pid) && started.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(gone(pid), "grandchild {pid} survived");
    }

    #[test]
    fn end_group_returns_ok_only_once_the_group_is_gone_and_a_gone_group_is_not_an_error() {
        use std::os::unix::process::CommandExt;
        let mut child = Command::new("sleep").arg("30").process_group(0).spawn().unwrap();
        let leader = child.id();
        // Reaped as it dies, so that it is gone and not a zombie that still holds its group.
        let reaper = std::thread::spawn(move || child.wait());
        end_group(leader).unwrap();
        reaper.join().unwrap().unwrap();
        end_group(leader).unwrap();
    }

    #[test]
    fn end_group_refuses_ids_that_are_no_group_to_kill() {
        for leader in [0, 1, u32::MAX] {
            let error = end_group(leader).unwrap_err();
            assert!(error.to_string().contains("not a group to kill"), "{error}");
        }
    }

    #[test]
    fn a_timeout_ends_the_commands_own_children_too() {
        let scratch = Scratch::new("grandchild");
        let file = scratch.join("pid");
        let script = format!("sleep 30 & printf %s $! > {}; wait; true", file.display());
        let error = run_within("/bin/sh", &["-c", &script], Duration::from_millis(500), |_| ()).unwrap_err();
        assert_eq!(error.to_string(), format!("/bin/sh -c {script}: timed out after 0.5 s"));
        assert_ends_gone(&written_pid(&file));
    }

    #[test]
    fn a_command_that_ends_but_leaves_a_child_holding_its_output_times_out_and_the_child_goes() {
        let scratch = Scratch::new("holder");
        let file = scratch.join("pid");
        let script = format!("sleep 30 & printf %s $! > {}", file.display());
        let started = Instant::now();
        let error = run_within("/bin/sh", &["-c", &script], Duration::from_millis(500), |_| ()).unwrap_err();
        assert_eq!(error.to_string(), format!("/bin/sh -c {script}: timed out after 0.5 s"));
        assert!(started.elapsed() < Duration::from_secs(10));
        assert_ends_gone(&written_pid(&file));
    }

    fn found(script: &str) -> Result<Option<String>, String> {
        run_found("/bin/sh", &["-c", script]).map_err(|error| error.to_string())
    }

    #[test]
    fn run_found_answers_none_only_for_a_quiet_exit_of_one() {
        assert_eq!(found("exit 1"), Ok(None));
        assert_eq!(found("printf abc"), Ok(Some("abc".to_owned())));
    }

    #[test]
    fn run_found_fails_for_an_exit_other_than_zero_or_one_and_for_a_noisy_one() {
        let other = "printf 'bad\\n' >&2; exit 2";
        let noisy = "printf 'bad\\n' >&2; exit 1";
        assert_eq!(found(other), Err(format!("/bin/sh -c {other}: bad\n")));
        assert_eq!(found(noisy), Err(format!("/bin/sh -c {noisy}: bad\n")));
        assert_eq!(found("exit 2"), Err("/bin/sh -c exit 2: ".to_owned()));
    }

    #[test]
    fn a_failed_command_shows_its_stderr_as_it_was_written() {
        let error = run("/bin/sh", &["-c", "printf ' boom\\n' >&2; exit 3"]).unwrap_err();
        assert_eq!(error.to_string(), "/bin/sh -c printf ' boom\\n' >&2; exit 3:  boom\n");
    }

    #[test]
    fn a_yes_or_no_command_answers_by_exit_0_or_1_and_anything_else_is_an_error() {
        let decided = ["exit 0", "exit 1", "printf 'no repo' >&2; exit 128"]
            .map(|script| decides_with("/bin/sh", &["-c", script], |_| ()));
        assert_eq!(decided[..2], [Ok(true), Ok(false)]);
        assert_eq!(decided[2].clone().unwrap_err().to_string(), "/bin/sh -c printf 'no repo' >&2; exit 128: no repo");
    }

    #[test]
    fn a_command_that_outlasts_the_limit_is_killed_and_fails_with_a_port_error() {
        let started = Instant::now();
        let error = run_within("/bin/sh", &["-c", "sleep 30"], Duration::from_millis(200), |_| ()).unwrap_err();
        assert_eq!(error.to_string(), "/bin/sh -c sleep 30: timed out after 0.2 s");
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_command_that_ends_within_the_limit_keeps_its_output() {
        assert_eq!(run_within("/bin/sh", &["-c", "printf abc"], Duration::from_secs(10), |_| ()).as_deref(), Ok("abc"));
    }
}
