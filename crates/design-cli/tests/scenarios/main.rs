#![deny(clippy::wildcard_enum_match_arm)]

mod ports;

use test_failure::Failed;

use design::Failure;
use ports::Design;
use serde_json::Value;
use tempfile::TempDir;

#[test]
fn an_unknown_command_exits_with_failure_and_prints_the_usage() -> Result<(), Failed> {
    let home = TempDir::new()?;
    let output = Design::run(home.path(), &["frobnicate"])?;
    assert_eq!(
        (output.status.code(), String::from_utf8(output.stdout)?, String::from_utf8(output.stderr)?),
        (Some(1_i32), String::new(), format!("{}\n", Failure::Usage))
    );
    Ok(())
}

#[test]
fn the_hook_settings_are_printed_as_json_at_exit_zero() -> Result<(), Failed> {
    let home = TempDir::new()?;
    let output = Design::run(home.path(), &["hook", "--settings"])?;
    let settings: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!((output.status.code(), output.stderr), (Some(0_i32), Vec::new()));
    assert!(settings.pointer("/hooks/Stop/0/hooks/0/command").and_then(Value::as_str).is_some());
    Ok(())
}

#[test]
fn ingest_of_a_workspace_without_transcripts_fails_naming_the_transcripts() -> Result<(), Failed> {
    let home = TempDir::new()?;
    let workspace = home.path().join("work");
    std::fs::create_dir_all(&workspace)?;
    let output = Design::run(home.path(), &["--workspace", &workspace.to_string_lossy(), "ingest"])?;
    assert_eq!((output.status.code(), output.stdout), (Some(1_i32), Vec::new()));
    assert!(String::from_utf8(output.stderr)?.contains("cannot reach the transcripts: "));
    Ok(())
}
