use crate::PortError;
use std::process::Command;

pub fn run(program: &str, args: &[&str]) -> Result<String, PortError> {
    let shown = args.iter().fold(program.to_owned(), |shown, arg| format!("{shown} {arg}"));
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
    fn a_failed_command_reports_its_stderr_without_the_trailing_newline() {
        let error = run("/bin/sh", &["-c", "echo boom >&2; exit 3"]).unwrap_err();
        assert_eq!(error.to_string(), "/bin/sh -c echo boom >&2; exit 3: boom");
    }
}
