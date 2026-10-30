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

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn a_failed_command_shows_its_stderr_as_it_was_written() {
        let error = run("/bin/sh", &["-c", "printf ' boom\\n' >&2; exit 3"]).unwrap_err();
        assert_eq!(error.to_string(), "/bin/sh -c printf ' boom\\n' >&2; exit 3:  boom\n");
    }
}
