use std::ffi::OsString;
use std::path::Path;

use console_ports::{Heard, Variables};
use design::{Agents, Failure, HookEvent, Outcome};
use design_store::{StopTrail, StreamSchema};
use dispatch_fakes::{Scripted, Stepped};
use dispatch_ports::{Finished, Model, Output, Pause, Prompt, Reply, Tokens, Usage, Wall};
use file_ports::{Bytes, Epoch, Files, Memory, Stamps};
use newtype::Wrapping;
use port_core::Run;
use serde_json::Value;
use store_ports::Store;
use text_lines::Lines;

use crate::{Opening, Sqlite};

use super::{Answered, FakeLent, HOME, Invocation, Session, Speaker};

struct Asking {
    store: Sqlite,
    files: Memory,
    script: Scripted,
}

struct Asked {
    outcome: Result<Outcome, Failure>,
    text: String,
    prompts: Vec<String>,
}

impl Asking {
    const RULED: &str = "the lint wall time limit stays at three seconds";
    const QUEUED: &str = "yes keep it first in first out";
    const QUOTED: &str = "\"the lint wall time limit stays at three seconds\"";
    const PICKED: &str = "[{\"uuid\":\"o1\",\"quote\":\"the lint wall time limit stays at three seconds\"}]";
    const KEPT: &str = "[1]";
    const NONE: &str = "[]";

    fn over(replies: &[&str]) -> Result<Asking, Failure> {
        Asking::spoken(
            [
                Session::line(Speaker::Owner, "o1", "null", Asking::RULED),
                Session::line(Speaker::Lead, "l1", "\"o1\"", "should the queue keep its order"),
                Session::line(Speaker::Owner, "o2", "\"l1\"", Asking::QUEUED),
                Session::line(Speaker::Lead, "l2", "\"o2\"", "noted"),
            ]
            .concat(),
            replies,
        )
    }

    fn spoken(lines: String, replies: &[&str]) -> Result<Asking, Failure> {
        let session = Session::working(lines);
        let asking = Asking {
            store: Sqlite::in_memory::<StreamSchema>(),
            files: session.files(),
            script: Scripted::replying(replies.iter().map(|reply| Some(Output::Finished(Asking::finished(reply))))),
        };
        Invocation::within(&asking.store, FakeLent::new(&asking.files), &["ingest"])?;
        Ok(asking)
    }

    fn finished(reply: &str) -> Finished {
        Finished::new(
            Reply::new((*reply).to_owned()),
            Usage::new(Tokens::new(10), Tokens::new(20), Tokens::new(3), Tokens::new(4)),
        )
    }

    fn asked(&self, heard: &str, arguments: &[&str]) -> Asked {
        let environment = [("HOME", HOME), ("PWD", "/work")]
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .into_iter()
            .collect::<Variables>()
            .hearing(Heard::new(heard.to_owned()));
        let uttered = design::Cli::new(
            &environment,
            FakeLent::new(&self.files),
            Opening(|_path| self.store.clone()),
            Agents::new(|_scratch| &self.script, Stepped::starting(Wall::new(Tasked::NOW), Pause::new(0))),
        )
        .report(&Run::root(), arguments.iter().map(OsString::from));
        let text = Answered::said(&uttered);
        let prompts = self.script.prompts().into_iter().map(Prompt::into_inner).collect();
        Asked { outcome: uttered.ended(), text, prompts }
    }

    fn answered(replies: &[&str], question: &str) -> Result<Asked, Failure> {
        Ok(Asking::over(replies)?.asked("", &["answer", question]))
    }

    fn crowded(replies: &[&str]) -> Result<Asking, Failure> {
        let crowd = (1..=300_u16).map(|index| Session::line(Speaker::Owner, &format!("m{index:03}"), "null", "noted"));
        Asking::spoken(
            crowd.chain([Session::stamped(Speaker::Owner, "o1", "null", Asking::RULED, "2026-01-02")]).collect(),
            replies,
        )
    }
}

struct Tasked;

impl Tasked {
    const NOW: i64 = 1_800_000_000;
    const MINUTE: i64 = 60;
    const FOLDER: &str = "/private/tmp/claude-501/-work/s/tasks";
    const TRANSCRIPT: &str = "/home/tester/.claude/projects/-work/s.jsonl";
    const EXITED: &str = "done\n[exited with code 0]\n";
    const ENDED: &str = "{\"type\":\"assistant\",\"message\":{\"stop_reason\":\"end_turn\",\"content\":[]}}\n";
    const WORKING: &str = "{\"type\":\"user\",\"message\":{\"content\":[{\"type\":\"tool_result\"}]}}\n";
    const CALLING: &str = "{\"type\":\"assistant\",\"message\":{\"stop_reason\":\"tool_use\",\"content\":[]}}\n";

    fn placed(asking: &Asking, name: &str, text: &str, [born, touched]: [i64; 2]) -> Result<(), laws::Unmet> {
        let path = Path::new(Tasked::FOLDER).join(name);
        asking.files.create_directories(Path::new(Tasked::FOLDER))?;
        asking.files.write(&path, Bytes::from(text))?;
        let ago = |minutes: i64| Epoch::new(Tasked::NOW - minutes * Tasked::MINUTE);
        asking.files.stamped(&path, Stamps::new(ago(born), ago(touched)));
        Ok(())
    }

    fn stopping(said: &str) -> String {
        serde_json::json!({"hook_event_name": "Stop", "stop_hook_active": false, "session_id": "s",
            "transcript_path": Tasked::TRANSCRIPT, "last_assistant_message": said})
        .to_string()
    }

    fn stopped(asking: &Asking) -> Result<Stop, laws::Unmet> {
        let asked = asking.asked(&Tasked::stopping("I'll merge it once CI is green."), &["hook", "stop"]);
        let listed = asking.asked("", &["stops"]);
        let run = Run::root();
        let rows = run.port(asking.store.clone()).read(StopTrail)?;
        run.finish().settled()?;
        Ok(Stop {
            reason: Tasked::reason(&asked),
            asked,
            trail: rows.iter().map(ToString::to_string).collect(),
            listed,
        })
    }

    fn reason(asked: &Asked) -> String {
        serde_json::from_str::<Value>(&asked.text)
            .into_iter()
            .find_map(|decision| decision.get("reason").and_then(Value::as_str).map(str::to_owned))
            .unwrap_or_default()
    }
}

struct Stop {
    asked: Asked,
    reason: String,
    trail: Vec<String>,
    listed: Asked,
}

#[test]
fn haiku_reads_every_earlier_owner_message_under_its_header_and_a_trim_keeps_the_quotes() -> Result<(), Failure> {
    let asking = Asking::over(&[Asking::PICKED, Asking::KEPT])?;
    let asked = asking.asked("", &["answer", "is the lint limit still three seconds"]);
    let selecting = asked.prompts.first().cloned().unwrap_or_default();
    let trimming = asked.prompts.get(1).cloned().unwrap_or_default();
    assert!(selecting.contains("is the lint limit still three seconds"), "{selecting}");
    assert!(selecting.contains(&format!("[o1 2026-01-01T00:00:00Z] {}", Asking::RULED)), "{selecting}");
    assert!(selecting.contains(&format!("[o2 2026-01-01T00:00:00Z] {}", Asking::QUEUED)), "{selecting}");
    assert!(trimming.contains(&format!("1. o1 2026-01-01T00:00:00Z: {}", Asking::QUOTED)), "{trimming}");
    assert_eq!(asking.script.models(), [Model::Haiku, Model::Haiku]);
    Ok(())
}

#[test]
fn the_owner_messages_are_read_in_chunks_of_300_and_each_quote_is_checked_against_its_own_chunk() -> Result<(), Failure>
{
    let asking = Asking::crowded(&[Asking::PICKED, Asking::PICKED, Asking::KEPT])?;
    let asked = asking.asked("", &["answer", "should the lint limit change"]);
    let first = asked.prompts.first().cloned().unwrap_or_default();
    let second = asked.prompts.get(1).cloned().unwrap_or_default();
    assert_eq!(
        (first.contains("[m300 "), first.contains("[o1 "), second.contains("[m001 "), second.contains("[o1 ")),
        (true, false, false, true)
    );
    assert!(asked.text.contains("read: 301 owner messages\nquoted: 1\n"), "{}", asked.text);
    assert!(asked.text.contains(&format!("o1 2026-01-02T00:00:00Z: {}", Asking::QUOTED)), "{}", asked.text);
    Ok(())
}

#[test]
fn an_answer_quotes_the_owner_messages_the_trim_keeps() -> Result<(), Failure> {
    let asked = Asking::answered(&[Asking::PICKED, Asking::KEPT], "should the lint limit change")?;
    assert_eq!(asked.outcome?, Outcome::Answered);
    assert!(asked.text.contains("read: 2 owner messages\nquoted: 1\n"), "{}", asked.text);
    assert!(asked.text.contains(&format!("o1 2026-01-01T00:00:00Z: {}", Asking::QUOTED)), "{}", asked.text);
    Ok(())
}

#[test]
fn an_answer_drops_the_quotes_the_trim_rejects() -> Result<(), Failure> {
    let asked = Asking::answered(&[Asking::PICKED, Asking::NONE], "should the lint limit change")?;
    assert!(asked.text.contains("quoted: 1\nquotes kept: none"), "{}", asked.text);
    Ok(())
}

#[test]
fn the_trim_keeps_at_most_eight_quotes() -> Result<(), Failure> {
    let picked: Vec<String> = (4..=13)
        .map(|count| {
            let quote: String = Asking::QUEUED.chars().take(count).collect();
            format!("{{\"uuid\":\"o2\",\"quote\":\"{quote}\"}}")
        })
        .collect();
    let picks = format!("[{}]", picked.join(","));
    let asked = Asking::answered(&[&picks, "[1,2,3,4,5,6,7,8,9,10]"], "what order should the queue keep")?;
    assert_eq!(asked.text.matches("o2 2026-01-01T00:00:00Z: ").count(), 8, "{}", asked.text);
    Ok(())
}

#[test]
fn an_answer_drops_a_quote_the_owner_never_wrote_and_sends_no_trim() -> Result<(), Failure> {
    let invented = "[{\"uuid\":\"o1\",\"quote\":\"the limit is gone\"},{\"uuid\":\"o2\",\"quote\":\"the lint wall\"}]";
    let asked = Asking::answered(&[invented], "should the lint limit change")?;
    assert!(asked.text.contains("quoted: 0\nquotes kept: none"), "{}", asked.text);
    assert_eq!(asked.prompts.len(), 1);
    Ok(())
}

#[test]
fn a_stop_without_its_last_message_searches_with_the_sessions_last_lead_message() -> Result<(), Failure> {
    let asking = Asking::over(&[Asking::NONE])?;
    let stopping = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false,\"session_id\":\"s\"}";
    let stopped = asking.asked(stopping, &["hook", "stop"]);
    let selecting = stopped.prompts.first().cloned().unwrap_or_default();
    assert!(selecting.contains("message to its owner:\nnoted\n"), "{selecting}");
    assert_eq!(stopped.outcome?, Outcome::Hooked(HookEvent::Stop));
    Ok(())
}

#[test]
fn a_stop_already_held_by_its_hook_reads_nothing() -> Result<(), Failure> {
    let asking = Asking::over(&[Asking::PICKED])?;
    let held = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":true,\"last_assistant_message\":\"Should the lint limit change?\"}";
    let stopped = asking.asked(held, &["hook", "stop"]);
    assert_eq!((stopped.outcome?, stopped.prompts.len()), (Outcome::Hooked(HookEvent::Stop), 0));
    Ok(())
}

#[test]
fn a_stop_with_nothing_running_blocks_as_stalled_with_the_quotes_that_bear_on_it() -> Result<(), laws::Unmet> {
    let asking = Asking::over(&[Asking::PICKED, Asking::KEPT])?;
    Tasked::placed(&asking, "b1.output", Tasked::EXITED, [5, 4])?;
    Tasked::placed(&asking, "a1.output", Tasked::ENDED, [5, 1])?;
    let Stop { asked, reason, trail, listed } = Tasked::stopped(&asking)?;
    assert_eq!(asked.outcome?, Outcome::Blocked);
    assert!(reason.contains("Nothing is running: no background shell or agent will wake you"), "{reason}");
    assert!(
        reason.contains(&format!("Earlier owner words bearing on this:\no1 2026-01-01T00:00:00Z: {}", Asking::QUOTED)),
        "{reason}"
    );
    assert_eq!(trail, ["s\t0\t0\tstalled\t2\t1\t1\tblocked\t74\t0"]);
    assert_eq!(listed.outcome?, Outcome::Trailed);
    golden::golden!("stops.txt", &listed.text);
    Ok(())
}

#[test]
fn a_stop_while_a_ci_wait_shell_runs_is_not_stalled_and_passes_without_quotes() -> Result<(), laws::Unmet> {
    let asking = Asking::over(&[Asking::NONE])?;
    Tasked::placed(&asking, "b1.output", "", [4, 4])?;
    let Stop { asked, trail, .. } = Tasked::stopped(&asking)?;
    assert_eq!(asked.outcome?, Outcome::Hooked(HookEvent::Stop));
    assert_eq!(trail, ["s\t1\t0\tlive\t2\t0\t0\tpassed\t37\t0"]);
    Ok(())
}

#[test]
fn a_stop_while_an_agent_works_is_not_stalled_whether_it_waits_on_a_tool_result_or_calls_one() -> Result<(), laws::Unmet>
{
    [Tasked::WORKING, Tasked::CALLING].iter().try_for_each(|last| {
        let asking = Asking::over(&[Asking::NONE])?;
        Tasked::placed(&asking, "a1.output", last, [40, 2])?;
        let Stop { asked, trail, .. } = Tasked::stopped(&asking)?;
        assert_eq!(asked.outcome?, Outcome::Hooked(HookEvent::Stop));
        assert_eq!(trail, ["s\t0\t1\tlive\t2\t0\t0\tpassed\t37\t0"]);
        Ok::<(), laws::Unmet>(())
    })
}

#[test]
fn a_live_stop_with_quotes_blocks_once_with_only_the_quotes() -> Result<(), laws::Unmet> {
    let asking = Asking::over(&[Asking::PICKED, Asking::KEPT])?;
    Tasked::placed(&asking, "b1.output", "", [4, 4])?;
    let Stop { asked, reason, trail, .. } = Tasked::stopped(&asking)?;
    assert_eq!(asked.outcome?, Outcome::Blocked);
    assert_eq!(
        reason.parse::<Lines>().into_iter().flatten().collect::<Vec<String>>(),
        ["Earlier owner words bearing on this:".to_owned(), format!("o1 2026-01-01T00:00:00Z: {}", Asking::QUOTED)]
    );
    assert_eq!(trail, ["s\t1\t0\tlive\t2\t1\t1\tblocked\t74\t0"]);
    Ok(())
}

#[test]
fn a_shell_silent_for_twenty_minutes_overruns_once_and_an_attended_one_does_not_excuse_the_stop()
-> Result<(), laws::Unmet> {
    let asking = Asking::over(&[Asking::NONE, Asking::NONE])?;
    Tasked::placed(&asking, "bhung.output", "", [50, 21])?;
    Tasked::placed(&asking, "bwatched.output", "progress\n", [50, 3])?;
    let Stop { asked, reason, trail, .. } = Tasked::stopped(&asking)?;
    let again = Tasked::stopped(&asking)?;
    assert_eq!(asked.outcome?, Outcome::Blocked);
    assert!(reason.contains("OVERRUN: these background tasks ran 20 minutes without finishing or writing output; check and stop them: bhung"), "{reason}");
    assert!(again.reason.contains("Nothing is running"), "{}", again.reason);
    assert_eq!(again.trail, ["s\t0\t0\toverrun\t2\t0\t0\tblocked\t37\t0", "s\t0\t0\tstalled\t2\t0\t0\tblocked\t37\t0"]);
    assert_eq!(trail.len(), 1);
    Ok(())
}

#[test]
fn a_shell_started_exactly_twenty_minutes_ago_no_longer_excuses_the_stop() -> Result<(), laws::Unmet> {
    let asking = Asking::over(&[Asking::NONE])?;
    Tasked::placed(&asking, "b1.output", "", [20, 1])?;
    let Stop { trail, .. } = Tasked::stopped(&asking)?;
    assert_eq!(trail, ["s\t0\t0\tstalled\t2\t0\t0\tblocked\t37\t0"]);
    Ok(())
}

#[test]
fn a_stop_with_no_transcript_path_makes_no_stall_claim() -> Result<(), laws::Unmet> {
    let asking = Asking::over(&[Asking::NONE])?;
    let stopping = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false,\"session_id\":\"s\",\"last_assistant_message\":\"done\"}";
    let stopped = asking.asked(stopping, &["hook", "stop"]);
    assert_eq!(stopped.outcome?, Outcome::Hooked(HookEvent::Stop));
    Ok(())
}
