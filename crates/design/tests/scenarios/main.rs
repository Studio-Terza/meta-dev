mod ports;

use std::error::Error;

use design::{Failure, HookEvent, Line, Unmatched};
use design_model::Prefix;
use ports::Design;
use serde_json::Value;
use strum::VariantArray;
use text_lines::Lines;

#[test]
fn an_unknown_command_exits_with_failure_and_prints_the_usage() -> Result<(), Box<dyn Error>> {
    let output = Design::with_a_session_in_work()?.run(&["frobnicate"])?;
    assert_ne!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stderr)?.contains("usage: design"));
    Ok(())
}

#[test]
fn ingest_of_a_workspace_exits_successfully_with_its_report() -> Result<(), Box<dyn Error>> {
    let output = Design::with_a_session_in_work()?.in_work(&["ingest"], "")?;
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stdout)?.contains("authorship owner 1\n"));
    Ok(())
}

#[test]
fn a_store_kept_on_disk_answers_show_and_a_second_ingest_writes_nothing() -> Result<(), Box<dyn Error>> {
    let design = Design::with_a_session_in_work()?;
    let first = design.in_work(&["ingest"], "")?;
    let second = design.in_work(&["ingest"], "")?;
    let shown = design.in_work(&["show", "o1"], "")?;
    assert!(String::from_utf8(first.stdout)?.contains("written 4\n"));
    assert!(String::from_utf8(second.stdout)?.contains("written 0\n"));
    assert!(String::from_utf8(shown.stdout)?.contains("o1 owner session \"s\" at \"2026-01-01T00:00:00Z\"\n"));
    Ok(())
}

#[test]
fn the_hook_settings_are_json_naming_for_every_event_a_command_that_runs_its_hook() -> Result<(), Box<dyn Error>> {
    let output = Design::with_a_session_in_work()?.run(&["hook", "--settings"])?;
    let settings: Value = serde_json::from_slice(&output.stdout)?;
    let commanded = HookEvent::VARIANTS
        .iter()
        .filter_map(|event| {
            let named = serde_json::to_value(event).ok()?;
            let command = settings.get("hooks")?.get(named.as_str()?)?.get(0)?.get("hooks")?.get(0)?.get("command")?;
            command.as_str()?.contains(&format!(" hook {event}")).then_some(*event)
        })
        .collect::<Vec<HookEvent>>();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(commanded, HookEvent::VARIANTS);
    Ok(())
}

#[test]
fn a_prompt_hook_ingests_and_prints_nothing_into_the_conversation() -> Result<(), Box<dyn Error>> {
    let design = Design::with_a_session_in_work()?;
    let hooked = design.in_work(&["hook", "user-prompt-submit"], "")?;
    let shown = design.in_work(&["show", "o1"], "")?;
    assert_eq!((hooked.status.code(), hooked.stdout, hooked.stderr), (Some(0), Vec::new(), Vec::new()));
    assert_eq!(shown.status.code(), Some(0));
    Ok(())
}

#[test]
fn a_stop_whose_agent_flails_blocks_once_with_a_json_decision_at_exit_zero_and_a_stop_already_continuing_only_notices()
-> Result<(), Box<dyn Error>> {
    let stopping = "{\"session_id\":\"s\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":false}";
    let continuing = "{\"session_id\":\"s\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":true}";
    let design = Design::with_a_session_in_work()?;
    let blocked = design.in_work(&["hook", "stop"], stopping)?;
    let passed = design.in_work(&["hook", "stop"], stopping)?;
    let prompted = design.in_work(&["hook", "user-prompt-submit"], "")?;
    let noticed = Design::with_a_session_in_work()?.in_work(&["hook", "stop"], continuing)?;
    let decision: Value = serde_json::from_slice(&blocked.stdout)?;
    let notice: Value = serde_json::from_slice(&noticed.stdout)?;
    assert_eq!((blocked.status.code(), blocked.stderr), (Some(0), Vec::new()));
    assert_eq!(decision.get("decision").and_then(Value::as_str), Some("block"));
    assert!(decision.get("reason").is_some_and(Value::is_string));
    assert_eq!((passed.status.code(), passed.stdout, passed.stderr), (Some(0), Vec::new(), Vec::new()));
    assert_eq!((prompted.status.code(), prompted.stdout, prompted.stderr), (Some(0), Vec::new(), Vec::new()));
    assert_eq!((noticed.status.code(), noticed.stderr), (Some(0), Vec::new()));
    assert_eq!(notice.get("decision"), None);
    assert_eq!(notice.get("systemMessage"), decision.get("reason"));
    Ok(())
}

#[test]
fn a_flail_first_met_by_a_continuing_stop_reaches_the_lead_once_through_the_next_prompt_or_the_next_stop()
-> Result<(), Box<dyn Error>> {
    let stopping = "{\"session_id\":\"s\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":false}";
    let continuing = "{\"session_id\":\"s\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":true}";
    let prompted = Design::with_a_session_in_work()?;
    let noticed = prompted.in_work(&["hook", "stop"], continuing)?;
    let continued = prompted.in_work(&["hook", "stop"], continuing)?;
    let told = prompted.in_work(&["hook", "user-prompt-submit"], "")?;
    let after =
        [prompted.in_work(&["hook", "user-prompt-submit"], "")?, prompted.in_work(&["hook", "stop"], stopping)?];
    let blocking = Design::with_a_session_in_work()?;
    let renoticed = blocking.in_work(&["hook", "stop"], continuing)?;
    let blocked = blocking.in_work(&["hook", "stop"], stopping)?;
    let passed = blocking.in_work(&["hook", "stop"], stopping)?;
    let messages: [Value; 2] = [serde_json::from_slice(&noticed.stdout)?, serde_json::from_slice(&renoticed.stdout)?];
    let context: Value = serde_json::from_slice(&told.stdout)?;
    let decision: Value = serde_json::from_slice(&blocked.stdout)?;
    let specific = context.get("hookSpecificOutput");
    assert_eq!(specific.and_then(|specific| specific.get("hookEventName")), Some(&Value::from("UserPromptSubmit")));
    assert_eq!(specific.and_then(|specific| specific.get("additionalContext")), messages[0].get("systemMessage"));
    assert_eq!(decision.get("decision").and_then(Value::as_str), Some("block"));
    assert_eq!(decision.get("reason"), messages[1].get("systemMessage"));
    assert_eq!(
        [&continued, &told, &blocked].map(|output| (output.status.code(), output.stderr.as_slice())),
        [(Some(0), &[][..]), (Some(0), &[][..]), (Some(0), &[][..])]
    );
    assert_eq!([continued, passed].map(|output| output.stdout), [Vec::<u8>::new(), Vec::new()]);
    assert_eq!(after.map(|output| output.stdout), [Vec::<u8>::new(), Vec::new()]);
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Answer {
    code: Option<i32>,
    stderr: String,
    unparsed: Vec<String>,
}

impl Answer {
    fn given(arguments: &&[&str]) -> Option<Answer> {
        let output = Design::with_a_session_in_work().ok()?.in_work(arguments, "").ok()?;
        let stdout = String::from_utf8(output.stdout).ok()?;
        let unparsed = Lines::from(stdout.as_str()).filter(|line| line.parse::<Line>().is_err()).map(str::to_owned);
        Some(Answer {
            code: output.status.code(),
            stderr: String::from_utf8(output.stderr).ok()?,
            unparsed: unparsed.collect(),
        })
    }

    fn failed(failure: &Failure) -> Answer {
        Answer { code: Some(1), stderr: format!("{failure}\n"), unparsed: Vec::new() }
    }

    fn printed() -> Answer {
        Answer { code: Some(0), stderr: String::new(), unparsed: Vec::new() }
    }
}

laws::table!(
    the_binary_writes_failures_as_their_message_and_output_only_as_lines_its_parser_reads,
    Answer::given,
    [
        &["frobnicate"][..] => Some(Answer::failed(&Failure::Usage)),
        &["hook"][..] => Some(Answer::failed(&Failure::Usage)),
        &["hook", "stop", "--settings"][..] => Some(Answer::failed(&Failure::Usage)),
        &["show", "nobody"][..] => Some(Answer::failed(&Failure::Unmatched(Unmatched::NoMatch(Prefix::new("nobody".to_owned()))))),
        &["ingest"][..] => Some(Answer::printed()),
        &["search", "go"][..] => Some(Answer::printed()),
    ]
);
