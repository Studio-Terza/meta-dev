//! The process boundary: a failed command's stderr is lossy text from another process, shown as written.
#![allow(clippy::disallowed_methods)]

use crate::PortError;
use std::process::Command;

pub fn run(program: &str, args: &[&str]) -> Result<String, PortError> {
    let shown = format!("{program} {}", args.join(" "));
    let output =
        Command::new(program).args(args).output().map_err(|error| PortError::new(format!("{shown}: {error}")))?;
    match output.status.success() {
        true => String::from_utf8(output.stdout).map_err(|error| PortError::new(format!("{shown}: {error}"))),
        false => Err(PortError::new(format!("{shown}: {}", String::from_utf8_lossy(&output.stderr)))),
    }
}

/// A yes-or-no command, such as `git merge-base --is-ancestor`: exit 0 is yes and exit 1 is no;
/// anything else is an error.
pub fn decides(program: &str, args: &[&str]) -> Result<bool, PortError> {
    let shown = format!("{program} {}", args.join(" "));
    let output =
        Command::new(program).args(args).output().map_err(|error| PortError::new(format!("{shown}: {error}")))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(PortError::new(format!("{shown}: {}", String::from_utf8_lossy(&output.stderr)))),
    }
}

#[cfg(test)]
mod tests {
    use super::{decides, run};

    #[test]
    fn a_failed_command_shows_its_stderr_as_it_was_written() {
        let error = run("/bin/sh", &["-c", "printf ' boom\\n' >&2; exit 3"]).unwrap_err();
        assert_eq!(error.to_string(), "/bin/sh -c printf ' boom\\n' >&2; exit 3:  boom\n");
    }

    #[test]
    fn a_yes_or_no_command_answers_by_exit_0_or_1_and_anything_else_is_an_error() {
        let decided =
            ["exit 0", "exit 1", "printf 'no repo' >&2; exit 128"].map(|script| decides("/bin/sh", &["-c", script]));
        assert_eq!(decided[..2], [Ok(true), Ok(false)]);
        assert_eq!(decided[2].clone().unwrap_err().to_string(), "/bin/sh -c printf 'no repo' >&2; exit 128: no repo");
    }
}
