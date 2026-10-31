//! The process boundary: a failed command's stderr is lossy text from another process, shown as written.
#![allow(clippy::disallowed_methods)]

use crate::PortError;
use meta_core::Stopped;
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

/// SIGTERM, a short grace, then SIGKILL, with no look at the outcome after it. Says whether the
/// group took a signal, and how the kernel refused one, if it did. ESRCH on the first signal is a
/// group already gone; on the second, one that went in the grace. A refusal proves nothing yet:
/// macOS answers EPERM for a group of zombies and for a member that may not be signalled alike, so
/// only the process table can tell them apart.
fn signal_end(signal: impl Fn(Signal) -> Result<(), Errno>) -> (Stopped, Option<Errno>) {
    match signal(Signal::SIGTERM) {
        Ok(()) => {}
        Err(Errno::ESRCH) => return (Stopped::AlreadyGone, None),
        Err(error) => return (Stopped::AlreadyGone, Some(error)),
    }
    thread::sleep(GRACE);
    match signal(Signal::SIGKILL) {
        Ok(()) | Err(Errno::ESRCH) => (Stopped::Killed, None),
        Err(error) => (Stopped::Killed, Some(error)),
    }
}

/// The whole process table, one `<process group> <state> <command>` line per process. A flag per
/// column, for in BSD `ps` a header runs to the end of its argument. Decoded lossily: the command
/// line of some other process must not stop a kill.
fn process_table() -> Result<String, PortError> {
    let args = ["-A", "-ww", "-o", "pgid=", "-o", "stat=", "-o", "command="];
    let output = execute("ps", &args, PORT_TIMEOUT, |_| ())?;
    match output.status.success() {
        true => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
        false => Err(PortError::new(format!("ps: {}", String::from_utf8_lossy(&output.stderr)))),
    }
}

/// The lines of `table` for members of `group` that are not zombies, leader or not. A zombie has
/// exited and only waits for its parent to read its status. A line that cannot be read is an
/// error, so that a table nobody can read never counts as a group gone.
fn live_members(table: &str, group: Pid) -> Result<Vec<&str>, PortError> {
    let mut live = Vec::new();
    for line in table.lines() {
        let mut columns = line.split_whitespace();
        let member = columns.next().and_then(|id| id.parse::<i32>().ok());
        let (Some(member), Some(state)) = (member, columns.next()) else {
            return Err(PortError::new(format!("process group {group}: unreadable ps line: {line}")));
        };
        if member == group.as_raw() && !state.starts_with('Z') {
            live.push(line);
        }
    }
    Ok(live)
}

fn has_live_member(table: &str, group: Pid) -> Result<bool, PortError> {
    live_members(table, group).map(|live| !live.is_empty())
}

/// The process group a leader's id names, unless it is none that may be killed.
fn group_of(leader: u32) -> Result<Pid, PortError> {
    i32::try_from(leader)
        .ok()
        .filter(|id| *id > 1)
        .map(Pid::from_raw)
        .ok_or_else(|| PortError::new(format!("process group {leader}: not a group to kill")))
}

/// Whether a live member of the group has `mark` on its command line. A leader can die before the
/// rest of its group, and an id can be reused, so this is what tells such a group from a stranger's.
pub fn group_holds(leader: u32, mark: &str) -> Result<bool, PortError> {
    let table = process_table()?;
    Ok(live_members(&table, group_of(leader)?)?.iter().any(|member| member.contains(mark)))
}

/// Ends `group` through `signal`, and confirms the end through `table`, for up to `confirm`. Only a
/// table with no live member confirms it. The wait covers a refused signal too, for a member that
/// is exiting may still be listed while the kernel already refuses to signal it.
fn end_through(
    group: Pid,
    signal: impl Fn(Signal) -> Result<(), Errno>,
    table: impl Fn() -> Result<String, PortError>,
    confirm: Duration,
) -> Result<Stopped, PortError> {
    let (stopped, refusal) = signal_end(signal);
    match refusal {
        None if stopped == Stopped::AlreadyGone => return Ok(stopped),
        None | Some(Errno::EPERM) => {}
        Some(error) => return Err(refused(group, error)),
    }
    let deadline = Instant::now() + confirm;
    while has_live_member(&table()?, group)? {
        if Instant::now() >= deadline {
            return Err(match refusal {
                Some(error) => refused(group, error),
                None => PortError::new(format!("process group {group}: still there after SIGKILL")),
            });
        }
        thread::sleep(POLL);
    }
    Ok(stopped)
}

/// Ends a whole process group, so that what its leader started goes too: SIGTERM, a short grace,
/// then SIGKILL. A group that is already gone is not an error. Ok means no live member is left: a
/// kill that fails, or a live member still there after `CONFIRM`, is an error.
pub fn end_group(leader: u32) -> Result<Stopped, PortError> {
    let group = group_of(leader)?;
    end_through(group, |signal| killpg(group, signal), process_table, CONFIRM)
}

/// Ends the command and its process group. The leader is this process's child and stays a zombie
/// until waited for, so the group is not confirmed gone here.
fn terminate(child: &mut Child) {
    let group = i32::try_from(child.id()).ok().map(Pid::from_raw);
    if let Some(group) = group {
        let _ = signal_end(|signal| killpg(group, signal));
    }
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
    use super::{
        Stopped, decides_with, end_group, end_through, group_holds, has_live_member, process_table, run, run_found,
        run_within,
    };
    use crate::PortError;
    use meta_testkit::Scratch;
    use nix::errno::Errno;
    use nix::sys::signal::Signal;
    use nix::unistd::Pid;
    use std::cell::RefCell;
    use std::fs;
    use std::process::Command;
    use std::time::{Duration, Instant};

    /// A group id for injected signals and tables: nothing is ever sent to it.
    const GROUP: i32 = 4242;

    fn group() -> Pid {
        Pid::from_raw(GROUP)
    }

    /// `end_through` with the kernel's answers to SIGTERM and SIGKILL, and a table that reads the
    /// same on every look.
    fn ended_on(term: Result<(), Errno>, kill: Result<(), Errno>, table: &str) -> Result<Stopped, String> {
        let signal = |signal| if signal == Signal::SIGTERM { term } else { kill };
        end_through(group(), signal, || Ok(table.to_owned()), Duration::ZERO).map_err(|error| error.to_string())
    }

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
        assert_eq!(end_group(leader), Ok(Stopped::AlreadyGone));
    }

    #[test]
    fn a_real_leader_left_a_zombie_counts_as_gone_and_so_does_its_group_on_a_second_stop() {
        use std::os::unix::process::CommandExt;
        let mut child = Command::new("sleep").arg("30").process_group(0).spawn().unwrap();
        let leader = child.id();
        // Not waited for until the end, so that the leader stays a zombie holding its group.
        assert_eq!(end_group(leader), Ok(Stopped::Killed));
        let table = process_table().unwrap();
        let group = Pid::from_raw(i32::try_from(leader).unwrap());
        assert_eq!(has_live_member(&table, group), Ok(false), "{table}");
        // Linux lets a zombie be signalled and macOS answers EPERM. Neither is unstoppable.
        assert!(end_group(leader).is_ok());
        child.wait().unwrap();
    }

    #[test]
    fn a_live_member_keeps_the_group_there_whatever_became_of_its_leader() {
        let cases = [
            ("  4242 Z (sh)\n  4242 S /bin/sh -c x\n", true),
            ("  4242 Ss+ sleep 30\n", true),
            ("  4242 ?E sleep 30\n", true),
            ("  4242 Z+ (sh)\n 4242 Z (sleep)\n", false),
            ("  4243 S sleep 30\n  424 R+ sleep 30\n", false),
            ("", false),
        ];
        for (table, live) in cases {
            assert_eq!(has_live_member(table, group()), Ok(live), "{table}");
        }
    }

    #[test]
    fn a_group_holds_a_mark_only_on_a_live_member_of_that_group() {
        use std::os::unix::process::CommandExt;
        let scratch = Scratch::new("group-holds");
        let mark = scratch.join("the-mark").display().to_string();
        let script = format!("sleep 30; : {mark}");
        let mut held = Command::new("/bin/sh").args(["-c", &script]).process_group(0).spawn().unwrap();
        let mut other = Command::new("sleep").arg("30").process_group(0).spawn().unwrap();
        assert_eq!(group_holds(held.id(), &mark), Ok(true));
        assert_eq!(group_holds(held.id(), "/nothing/has/this"), Ok(false));
        assert_eq!(group_holds(other.id(), &mark), Ok(false));
        assert_eq!(end_group(held.id()), Ok(Stopped::Killed));
        assert_eq!(group_holds(held.id(), &mark), Ok(false), "a zombie holds nothing");
        assert_eq!(end_group(other.id()), Ok(Stopped::Killed));
        held.wait().unwrap();
        other.wait().unwrap();
    }

    #[test]
    fn a_process_table_that_cannot_be_read_never_counts_as_a_group_gone() {
        for table in ["4242\n", "pgid S\n", "\n"] {
            let error = has_live_member(table, group()).unwrap_err();
            assert!(error.to_string().contains("unreadable ps line"), "{error}");
        }
        let unread = end_through(group(), |_| Ok(()), || Err(PortError::new("ps failed")), Duration::ZERO);
        assert_eq!(unread, Err(PortError::new("ps failed")));
    }

    #[test]
    fn a_member_still_there_after_sigkill_is_an_error_even_when_the_leader_is_a_zombie() {
        let still = format!("process group {GROUP}: still there after SIGKILL");
        assert_eq!(ended_on(Ok(()), Ok(()), "4242 Z\n4242 D\n"), Err(still));
    }

    #[test]
    fn a_group_confirmed_gone_was_killed_when_only_zombies_are_left() {
        assert_eq!(ended_on(Ok(()), Ok(()), "4242 Z\n"), Ok(Stopped::Killed));
        assert_eq!(ended_on(Ok(()), Err(Errno::ESRCH), ""), Ok(Stopped::Killed));
        assert_eq!(ended_on(Ok(()), Err(Errno::EPERM), "4242 Z\n"), Ok(Stopped::Killed));
    }

    #[test]
    fn a_real_member_that_refuses_a_signal_is_reported_and_a_group_of_zombies_is_not() {
        let refused = format!("process group {GROUP}: {}", Errno::EPERM);
        assert_eq!(ended_on(Err(Errno::EPERM), Ok(()), "4242 S\n"), Err(refused.clone()));
        assert_eq!(ended_on(Ok(()), Err(Errno::EPERM), "4242 Z\n4242 S\n"), Err(refused));
        assert_eq!(ended_on(Err(Errno::EPERM), Ok(()), "4242 Z\n"), Ok(Stopped::AlreadyGone));
    }

    #[test]
    fn a_group_with_no_such_process_is_already_gone_and_no_table_is_read() {
        let unread = || -> Result<String, PortError> { panic!("the table was read") };
        let ended = end_through(group(), |_| Err(Errno::ESRCH), unread, Duration::ZERO);
        assert_eq!(ended, Ok(Stopped::AlreadyGone));
    }

    #[test]
    fn a_signal_refused_for_another_reason_is_an_error_with_no_table_read() {
        let unread = || -> Result<String, PortError> { panic!("the table was read") };
        let error = end_through(group(), |_| Err(Errno::EINVAL), unread, Duration::ZERO).unwrap_err();
        assert_eq!(error.to_string(), format!("process group {GROUP}: {}", Errno::EINVAL));
    }

    #[test]
    fn a_member_that_is_slow_to_go_is_looked_at_again_until_it_has_gone() {
        let looks = RefCell::new(vec!["4242 S\n".to_owned(), "4242 R\n".to_owned(), String::new()]);
        let table = || Ok(looks.borrow_mut().remove(0));
        let ended = end_through(group(), |_| Ok(()), table, Duration::from_secs(10));
        assert_eq!(ended, Ok(Stopped::Killed));
        assert!(looks.borrow().is_empty());
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
