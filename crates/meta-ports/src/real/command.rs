use crate::PortError;
use std::process::Command;

pub fn run(program: &str, args: &[&str]) -> Result<String, PortError> {
    let shown = format!("{program} {}", args.join(" "));
    let output =
        Command::new(program).args(args).output().map_err(|error| PortError::new(format!("{shown}: {error}")))?;
    match output.status.success() {
        true => String::from_utf8(output.stdout).map_err(|error| PortError::new(format!("{shown}: {error}"))),
        false => Err(PortError::new(format!("{shown}: {}", String::from_utf8_lossy(&output.stderr).trim()))),
    }
}
