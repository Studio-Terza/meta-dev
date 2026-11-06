mod ports;

use std::error::Error;

use ports::Design;

#[test]
fn an_unknown_command_exits_with_failure_and_prints_the_usage() -> Result<(), Box<dyn Error>> {
    let output = Design::with_a_session_in_work()?.run(&["frobnicate"])?;
    assert_ne!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stderr)?.contains("usage: design [--workspace <dir>] (ingest | show <uuid>)"));
    Ok(())
}

#[test]
fn ingest_of_a_workspace_exits_successfully_with_its_report() -> Result<(), Box<dyn Error>> {
    let output = Design::with_a_session_in_work()?.run(&["ingest", "--workspace", "/work"])?;
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stdout)?.contains("authorship owner 1\n"));
    Ok(())
}
