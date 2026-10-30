use super::JsonlLog;
use serde_json::Value;
use std::process::{Command, Stdio};

const SETTINGS: &str = include_str!("../../../../self/claude-settings.json");
const BLOCKING: i32 = 2;
const MAX_TIMEOUT_SECONDS: u64 = 15;

fn entry() -> (String, String) {
    let settings: Value = serde_json::from_str(SETTINGS).unwrap();
    let hook = &settings["hooks"]["PreToolUse"][0];
    let command = hook["hooks"][0]["command"].as_str().unwrap().to_owned();
    (hook["matcher"].as_str().unwrap().to_owned(), command)
}

fn exit_code(prelude: &str, command: &str) -> Option<i32> {
    let script = format!("{prelude}{command}");
    let status =
        Command::new("/bin/sh").args(["-c", &script]).env("PATH", "/nonexistent").stdin(Stdio::null()).status();
    status.unwrap().code()
}

#[test]
fn the_hook_entry_gates_agent_task_and_send_message_only() {
    let (matcher, _) = entry();
    assert_eq!(matcher.split('|').collect::<Vec<_>>(), ["Agent", "Task", "SendMessage"]);
}

#[test]
fn the_hook_entry_blocks_on_failure_and_times_out_after_the_log_lock_deadline() {
    let settings: Value = serde_json::from_str(SETTINGS).unwrap();
    let hook = &settings["hooks"]["PreToolUse"][0]["hooks"][0];
    let timeout = hook["timeout"].as_u64().unwrap();
    assert_eq!(hook["onFailure"].as_str(), Some("block"));
    assert!(timeout > JsonlLog::PATIENCE.as_secs() && timeout <= MAX_TIMEOUT_SECONDS, "{timeout}");
}

#[test]
fn the_hook_entry_blocks_when_meta_is_missing_or_crashes() {
    let (_, command) = entry();
    let crashing = "meta() { return 101; }; ";
    let denying = "meta() { return 0; }; ";
    assert_eq!(
        [exit_code("", &command), exit_code(crashing, &command), exit_code(denying, &command)],
        [Some(BLOCKING), Some(BLOCKING), Some(0)]
    );
}
