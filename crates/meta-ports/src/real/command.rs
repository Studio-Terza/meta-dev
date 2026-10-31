//! The process boundary: a failed command's stderr is lossy text from another process, shown as written.
#![allow(clippy::disallowed_methods)]

use crate::PortError;
use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// How long one port command may run before it is killed and reported as a port error.
pub const PORT_TIMEOUT: Duration = Duration::from_secs(30);

/// How often a running command is checked for having finished.
const POLL: Duration = Duration::from_millis(10);

fn drain(pipe: Option<impl Read + Send + 'static>) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        bytes
    })
}

/// Runs the command to its end, or kills it at `limit` and fails. A reader of a pipe that a killed
/// command's child still holds is left behind, not waited for, so that a hang cannot pass on.
fn execute(program: &str, args: &[&str], limit: Duration) -> Result<Output, PortError> {
    let shown = format!("{program} {}", args.join(" "));
    let failed = |error: std::io::Error| PortError::new(format!("{shown}: {error}"));
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(failed)?;
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(failed)? {
            break status;
        }
        if started.elapsed() >= limit {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PortError::new(format!("{shown}: timed out after {} s", limit.as_secs_f64())));
        }
        thread::sleep(POLL);
    };
    let read = |reader: thread::JoinHandle<Vec<u8>>| {
        reader.join().map_err(|_| PortError::new(format!("{shown}: unreadable output")))
    };
    Ok(Output { status, stdout: read(stdout)?, stderr: read(stderr)? })
}

pub fn run(program: &str, args: &[&str]) -> Result<String, PortError> {
    run_within(program, args, PORT_TIMEOUT)
}

fn run_within(program: &str, args: &[&str], limit: Duration) -> Result<String, PortError> {
    let shown = format!("{program} {}", args.join(" "));
    let output = execute(program, args, limit)?;
    match output.status.success() {
        true => String::from_utf8(output.stdout).map_err(|error| PortError::new(format!("{shown}: {error}"))),
        false => Err(PortError::new(format!("{shown}: {}", String::from_utf8_lossy(&output.stderr)))),
    }
}

/// Like `run`, for a lookup whose "not found" is the command's quiet exit status 1 with nothing on
/// stderr, as with `git rev-parse --verify --quiet`: that is `None`; any other failure is an error.
pub fn run_found(program: &str, args: &[&str]) -> Result<Option<String>, PortError> {
    let shown = format!("{program} {}", args.join(" "));
    let output = execute(program, args, PORT_TIMEOUT)?;
    match (output.status.success(), output.status.code(), output.stderr.is_empty()) {
        (true, ..) => {
            String::from_utf8(output.stdout).map(Some).map_err(|error| PortError::new(format!("{shown}: {error}")))
        }
        (false, Some(1), true) => Ok(None),
        (false, ..) => Err(PortError::new(format!("{shown}: {}", String::from_utf8_lossy(&output.stderr)))),
    }
}

#[cfg(test)]
mod tests {
    use super::{run, run_found, run_within};
    use std::time::{Duration, Instant};

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
    fn a_command_that_outlasts_the_limit_is_killed_and_fails_with_a_port_error() {
        let started = Instant::now();
        let error = run_within("/bin/sh", &["-c", "sleep 30"], Duration::from_millis(200)).unwrap_err();
        assert_eq!(error.to_string(), "/bin/sh -c sleep 30: timed out after 0.2 s");
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_command_that_ends_within_the_limit_keeps_its_output() {
        assert_eq!(run_within("/bin/sh", &["-c", "printf abc"], Duration::from_secs(10)).as_deref(), Ok("abc"));
    }
}
