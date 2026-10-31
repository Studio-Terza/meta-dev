use super::JsonlLog;
use serde_json::Value;
use std::process::{Command, Stdio};

const SETTINGS: &str = include_str!("../../../../self/claude-settings.json");
const BLOCKING: i32 = 2;
const MAX_TIMEOUT_SECONDS: u64 = 15;

const EVENTS: [&str; 2] = ["PreToolUse", "PostToolUse"];

fn entry(event: &str) -> (String, String) {
    let settings: Value = serde_json::from_str(SETTINGS).unwrap();
    let hook = &settings["hooks"][event][0];
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
fn the_hook_entries_gate_agent_task_and_send_message_before_and_bind_agent_and_task_after() {
    let (before, command) = entry("PreToolUse");
    assert_eq!(before, "Agent|Task|SendMessage");
    assert_eq!(command, "meta hook pre-tool-use || exit 2");
    let (after, command) = entry("PostToolUse");
    assert_eq!(after, "Agent|Task");
    assert_eq!(command, "meta hook post-tool-use || exit 2");
}

#[test]
fn the_hook_entries_block_on_failure_and_time_out_after_the_log_lock_deadline() {
    let settings: Value = serde_json::from_str(SETTINGS).unwrap();
    EVENTS.into_iter().for_each(|event| {
        let hook = &settings["hooks"][event][0]["hooks"][0];
        let timeout = hook["timeout"].as_u64().unwrap();
        assert_eq!(hook["onFailure"].as_str(), Some("block"), "{event}");
        assert!(timeout > JsonlLog::PATIENCE.as_secs() && timeout <= MAX_TIMEOUT_SECONDS, "{event}: {timeout}");
    });
}

#[test]
fn the_hook_entries_block_when_meta_is_missing_or_crashes() {
    EVENTS.into_iter().for_each(|event| {
        let (_, command) = entry(event);
        let crashing = "meta() { return 101; }; ";
        let denying = "meta() { return 0; }; ";
        assert_eq!(
            [exit_code("", &command), exit_code(crashing, &command), exit_code(denying, &command)],
            [Some(BLOCKING), Some(BLOCKING), Some(0)],
            "{event}"
        );
    });
}

#[test]
fn the_subagent_stop_entry_records_every_stop_and_never_blocks_one() {
    let settings: Value = serde_json::from_str(SETTINGS).unwrap();
    let group = &settings["hooks"]["SubagentStop"][0];
    let hook = &group["hooks"][0];
    let command = hook["command"].as_str().unwrap();
    assert_eq!(command, "meta hook subagent-stop || true");
    assert!(group.get("matcher").is_none(), "internal agents are ignored by meta, so no matcher");
    assert!(hook["timeout"].as_u64().unwrap() > JsonlLog::PATIENCE.as_secs());
    let crashing = "meta() { return 101; }; ";
    let outcomes = ["", crashing].map(|prelude| exit_code(prelude, command));
    assert_eq!(outcomes, [Some(0), Some(0)], "exit 2 would keep the agent from stopping");
}
