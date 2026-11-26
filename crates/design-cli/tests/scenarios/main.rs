#![deny(clippy::wildcard_enum_match_arm)]

mod ignored;
mod ports;

use design::{Failure, Flail, Flails, HookEvent, Line, Sequel, Unmatched};
use design_model::{JobId, Prefix};
use invariants::Ignoring;
use newtype::Enumerated;
use ports::Design;
use serde_json::Value;
use structurer::{Directives, Lapse};
use text_lines::Lines;

use ignored::Ignored;

#[test]
fn an_unknown_command_exits_with_failure_and_prints_the_usage() -> Result<(), laws::Unmet> {
    let output = Design::with_a_session_in_work()?.run(&["frobnicate"])?;
    assert_ne!(output.status.code(), Some(0_i32));
    assert!(String::from_utf8(output.stderr)?.contains("usage: design"));
    Ok(())
}

#[test]
fn ingest_of_a_workspace_exits_successfully_with_its_report() -> Result<(), laws::Unmet> {
    let output = Design::with_a_session_in_work()?.in_work(&["ingest"], "")?;
    assert_eq!(output.status.code(), Some(0_i32));
    assert!(String::from_utf8(output.stdout)?.contains("authorship owner 1\n"));
    Ok(())
}

#[test]
fn a_store_kept_on_disk_answers_show_and_a_second_ingest_writes_nothing() -> Result<(), laws::Unmet> {
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
fn the_hook_settings_are_json_naming_for_every_event_a_command_that_runs_its_hook() -> Result<(), laws::Unmet> {
    let output = Design::with_a_session_in_work()?.run(&["hook", "--settings"])?;
    let settings: Value = serde_json::from_slice(&output.stdout)?;
    let commanded = HookEvent::VARIANTS
        .iter()
        .filter_map(|event| {
            let named = serde_json::to_value(event).ignoring(Ignored::Unnamed)?;
            let command = settings.get("hooks")?.get(named.as_str()?)?.get(0)?.get("hooks")?.get(0)?.get("command")?;
            command.as_str()?.contains(&format!(" hook {event}")).then_some(*event)
        })
        .collect::<Vec<HookEvent>>();
    assert_eq!(output.status.code(), Some(0_i32));
    assert_eq!(commanded, HookEvent::VARIANTS);
    Ok(())
}

#[test]
fn a_prompt_hook_ingests_and_prints_nothing_into_the_conversation() -> Result<(), laws::Unmet> {
    let design = Design::with_a_session_in_work()?;
    let hooked = design.in_work(&["hook", "user-prompt-submit"], "")?;
    let shown = design.in_work(&["show", "o1"], "")?;
    assert_eq!((hooked.status.code(), hooked.stdout, hooked.stderr), (Some(0_i32), Vec::new(), Vec::new()));
    assert_eq!(shown.status.code(), Some(0_i32));
    Ok(())
}

#[test]
fn a_flail_from_structuring_blocks_the_next_stop_once_with_a_json_decision_at_exit_zero_and_a_continuing_stop_prints_nothing()
-> Result<(), laws::Unmet> {
    let stopping = "{\"session_id\":\"s\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":false}";
    let continuing = "{\"session_id\":\"s\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":true}";
    let design = Design::with_a_session_in_work()?;
    let structured = [design.in_work(&["ingest"], "")?, design.in_work(&["structure"], "")?];
    let blocked = design.in_work(&["hook", "stop"], stopping)?;
    let passed = design.in_work(&["hook", "stop"], stopping)?;
    let prompted = design.in_work(&["hook", "user-prompt-submit"], "")?;
    let continued = design.in_work(&["hook", "stop"], continuing)?;
    let decision: Value = serde_json::from_slice(&blocked.stdout)?;
    assert_eq!(structured.map(|output| output.status.code()), [Some(0_i32), Some(1_i32)]);
    assert_eq!((blocked.status.code(), blocked.stderr), (Some(0_i32), Vec::new()));
    assert_eq!(decision.get("decision").and_then(Value::as_str), Some("block"));
    assert!(decision.get("reason").is_some_and(Value::is_string));
    assert_eq!(
        [passed, prompted, continued].map(|output| (output.status.code(), output.stdout, output.stderr)),
        [
            (Some(0_i32), Vec::new(), Vec::new()),
            (Some(0_i32), Vec::new(), Vec::new()),
            (Some(0_i32), Vec::new(), Vec::new())
        ]
    );
    Ok(())
}

#[test]
fn a_flail_from_structuring_reaches_the_lead_once_through_the_next_prompt_or_the_next_stop() -> Result<(), laws::Unmet>
{
    let stopping = "{\"session_id\":\"s\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":false}";
    let prompted = Design::with_a_session_in_work()?;
    prompted.in_work(&["ingest"], "")?;
    prompted.in_work(&["structure"], "")?;
    let told = prompted.in_work(&["hook", "user-prompt-submit"], "")?;
    let after =
        [prompted.in_work(&["hook", "user-prompt-submit"], "")?, prompted.in_work(&["hook", "stop"], stopping)?];
    let blocking = Design::with_a_session_in_work()?;
    blocking.in_work(&["ingest"], "")?;
    blocking.in_work(&["structure"], "")?;
    let blocked = blocking.in_work(&["hook", "stop"], stopping)?;
    let context: Value = serde_json::from_slice(&told.stdout)?;
    let decision: Value = serde_json::from_slice(&blocked.stdout)?;
    let specific = context.get("hookSpecificOutput");
    assert_eq!(specific.and_then(|output| output.get("hookEventName")), Some(&Value::from("UserPromptSubmit")));
    assert!(specific.and_then(|output| output.get("additionalContext")).is_some_and(Value::is_string));
    assert_eq!(decision.get("decision").and_then(Value::as_str), Some("block"));
    assert_eq!(decision.get("reason"), specific.and_then(|output| output.get("additionalContext")));
    assert_eq!(
        [&told, &blocked].map(|output| (output.status.code(), output.stderr.as_slice())),
        [(Some(0_i32), &[][..]), (Some(0_i32), &[][..])]
    );
    assert_eq!(after.map(|output| output.stdout), [Vec::<u8>::new(), Vec::new()]);
    Ok(())
}

#[test]
fn a_run_killed_during_its_second_job_leaves_the_first_job_s_flail_told_and_the_next_run_sends_only_the_second_job_s_segment_again()
-> Result<(), laws::Unmet> {
    let design = Design::with_two_jobs_whose_agent_dies_on_the_second()?;
    design.in_work(&["ingest"], "")?;
    let killed = design.in_work(&["structure"], "")?;
    let told = design.in_work(&["hook", "user-prompt-submit"], "")?;
    let restarted = design.in_work(&["structure"], "")?;
    let unrun = design.in_work(&["show", "job", "4"], "")?;
    let lapse = "place(".parse::<Directives>().map(drop).map_err(Lapse::from);
    let flail = |job, sequel| {
        let flails = lapse.clone().map_err(|flailed| Flails::new(Flail::new(JobId::new(job), flailed), Vec::new()));
        flails.map_err(|gathered| Failure::Flailed { flails: gathered, sequel }.to_string())
    };
    let [first, second] = [flail(1, Sequel::WentOn), flail(3, Sequel::NoMoreJobs)];
    let context: Value = serde_json::from_slice(&told.stdout)?;
    let failed = String::from_utf8(restarted.stderr)?;
    let specific = context.get("hookSpecificOutput").and_then(|specific| specific.get("additionalContext"));
    assert_eq!((killed.status.code(), killed.stdout), (None, Vec::new()));
    assert_eq!(restarted.status.code(), Some(1_i32));
    assert!(matches!((specific.and_then(Value::as_str), &first), (Some(said), Err(expected)) if said == expected));
    assert!(matches!(&second, Err(expected) if failed == format!("{expected}\n")));
    let refused = String::from_utf8(unrun.stderr)?;
    assert_eq!(unrun.status.code(), Some(1_i32));
    assert_eq!(
        refused.parse::<Lines>().into_iter().flatten().collect::<Vec<String>>(),
        [Failure::Unrun(JobId::new(4)).to_string()]
    );
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
        let output = Design::with_a_session_in_work()
            .ignoring(Ignored::Unrun)?
            .in_work(arguments, "")
            .ignoring(Ignored::Unrun)?;
        let stdout = String::from_utf8(output.stdout).ignoring(Ignored::NotUtf8)?;
        let unparsed = stdout.parse::<Lines>().into_iter().flatten().filter(|line| line.parse::<Line>().is_err());
        Some(Answer {
            code: output.status.code(),
            stderr: String::from_utf8(output.stderr).ignoring(Ignored::NotUtf8)?,
            unparsed: unparsed.collect(),
        })
    }

    fn failed(failure: &Failure) -> Answer {
        Answer { code: Some(1), stderr: format!("{failure}\n"), unparsed: Vec::new() }
    }

    fn printed() -> Answer {
        Answer { code: Some(0_i32), stderr: String::new(), unparsed: Vec::new() }
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
