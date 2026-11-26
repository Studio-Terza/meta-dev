#![deny(clippy::wildcard_enum_match_arm)]

use std::error::Error;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

struct Design;

impl Design {
    fn run(home: &Path, arguments: &[&str]) -> Result<Output, Box<dyn Error>> {
        Ok(Command::new(env!("CARGO_BIN_EXE_design")).env("HOME", home).current_dir(home).args(arguments).output()?)
    }
}

#[test]
fn an_unknown_command_exits_with_failure_and_prints_the_usage() -> Result<(), Box<dyn Error>> {
    let home = TempDir::new()?;
    let output = Design::run(home.path(), &["frobnicate"])?;
    assert_eq!(output.status.code(), Some(1_i32));
    assert!(String::from_utf8(output.stderr)?.starts_with("usage: design "));
    assert!(output.stdout.is_empty());
    Ok(())
}

#[test]
fn the_hook_settings_are_printed_as_json_at_exit_zero() -> Result<(), Box<dyn Error>> {
    let home = TempDir::new()?;
    let output = Design::run(home.path(), &["hook", "--settings"])?;
    let settings: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!((output.status.code(), output.stderr), (Some(0_i32), Vec::new()));
    assert!(settings.pointer("/hooks/Stop/0/hooks/0/command").and_then(Value::as_str).is_some());
    Ok(())
}

#[test]
fn ingest_of_a_workspace_without_transcripts_fails_naming_the_transcripts() -> Result<(), Box<dyn Error>> {
    let home = TempDir::new()?;
    let workspace = home.path().join("work");
    std::fs::create_dir_all(&workspace)?;
    let output = Design::run(home.path(), &["--workspace", &workspace.to_string_lossy(), "ingest"])?;
    assert_eq!((output.status.code(), output.stdout), (Some(1_i32), Vec::new()));
    assert!(String::from_utf8(output.stderr)?.starts_with("cannot reach the transcripts: "));
    Ok(())
}
