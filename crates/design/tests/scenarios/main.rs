mod ports;

use std::error::Error;

use ports::Design;

#[test]
fn an_unknown_command_exits_with_failure_and_prints_the_usage() -> Result<(), Box<dyn Error>> {
    let output = Design::with_a_session_in_work()?.run(&["frobnicate"])?;
    assert_ne!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stderr)?.contains("usage: design"));
    Ok(())
}

#[test]
fn ingest_of_a_workspace_exits_successfully_with_its_report() -> Result<(), Box<dyn Error>> {
    let output = Design::with_a_session_in_work()?.in_work(&["ingest"])?;
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stdout)?.contains("authorship owner 1\n"));
    Ok(())
}

#[test]
fn a_store_kept_on_disk_answers_show_and_a_second_ingest_writes_nothing() -> Result<(), Box<dyn Error>> {
    let design = Design::with_a_session_in_work()?;
    let first = design.in_work(&["ingest"])?;
    let second = design.in_work(&["ingest"])?;
    let shown = design.in_work(&["show", "o1"])?;
    assert!(String::from_utf8(first.stdout)?.contains("written 4\n"));
    assert!(String::from_utf8(second.stdout)?.contains("written 0\n"));
    assert!(String::from_utf8(shown.stdout)?.contains("o1 owner session \"s\" at \"2026-01-01T00:00:00Z\"\n"));
    Ok(())
}
