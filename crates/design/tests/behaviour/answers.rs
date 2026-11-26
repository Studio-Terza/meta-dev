use std::error::Error;
use std::ffi::OsString;

use console_ports::{Heard, Variables};
use design::{Agents, Failure, HookEvent, Outcome};
use design_store::StreamSchema;
use dispatch_fakes::{Scripted, Stepped};
use dispatch_ports::{Finished, Model, Output, Pause, Prompt, Reply, Tokens, Usage, Wall};
use file_ports::Memory;
use newtype::Wrapping;
use port_core::Run;
use serde_json::Value;

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
    const LINT: &str = "{\"kind\":\"question\",\"question\":\"should the lint limit change\"}";
    const PICKED: &str = "[{\"uuid\":\"o1\",\"quote\":\"the lint wall time limit stays at three seconds\"}]";
    const KEPT: &str = "[1]";
    const STATEMENT: &str = "{\"kind\":\"statement\",\"question\":\"\"}";
    const COVERED: &str = "conversational(o1.0)\nconversational(o2.0)";
    const STOPPING: &str = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false,\"last_assistant_message\":\"Should the lint limit change?\"}";

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
            Agents::new(|_scratch| &self.script, Stepped::starting(Wall::new(1_800_000_000), Pause::new(0))),
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

#[test]
fn haiku_reads_every_earlier_owner_message_under_its_header_and_sonnet_trims_the_quotes() -> Result<(), Failure> {
    let asking = Asking::over(&[Asking::LINT, Asking::PICKED, Asking::KEPT])?;
    let asked = asking.asked("", &["answer", "is the lint limit still three seconds"]);
    let classifying = asked.prompts.first().cloned().unwrap_or_default();
    let selecting = asked.prompts.get(1).cloned().unwrap_or_default();
    let trimming = asked.prompts.get(2).cloned().unwrap_or_default();
    assert!(classifying.contains("is the lint limit still three seconds"), "{classifying}");
    assert!(selecting.contains("should the lint limit change"), "{selecting}");
    assert!(selecting.contains(&format!("[o1 2026-01-01T00:00:00Z] {}", Asking::RULED)), "{selecting}");
    assert!(selecting.contains(&format!("[o2 2026-01-01T00:00:00Z] {}", Asking::QUEUED)), "{selecting}");
    assert!(trimming.contains(&format!("1. o1 2026-01-01T00:00:00Z: {}", Asking::QUOTED)), "{trimming}");
    assert_eq!(asking.script.models(), [Model::Haiku, Model::Haiku, Model::Sonnet]);
    Ok(())
}

#[test]
fn the_owner_messages_are_read_in_chunks_of_300_and_each_quote_is_checked_against_its_own_chunk() -> Result<(), Failure>
{
    let asking = Asking::crowded(&[Asking::LINT, Asking::PICKED, Asking::PICKED, Asking::KEPT])?;
    let asked = asking.asked("", &["answer", "should the lint limit change"]);
    let first = asked.prompts.get(1).cloned().unwrap_or_default();
    let second = asked.prompts.get(2).cloned().unwrap_or_default();
    assert_eq!(
        (first.contains("[m300 "), first.contains("[o1 "), second.contains("[m001 "), second.contains("[o1 ")),
        (true, false, false, true)
    );
    assert!(asked.text.contains("read: 301 owner messages\nquoted: 1\n"), "{}", asked.text);
    assert!(asked.text.contains(&format!("o1 2026-01-02T00:00:00Z: {}", Asking::QUOTED)), "{}", asked.text);
    Ok(())
}

#[test]
fn a_stop_without_its_last_message_reads_the_sessions_last_lead_message() -> Result<(), Failure> {
    let asking = Asking::over(&[Asking::COVERED, Asking::STATEMENT])?;
    let stopping = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false,\"session_id\":\"s\"}";
    let stopped = asking.asked(stopping, &["hook", "stop"]);
    let classifying = stopped.prompts.get(1).cloned().unwrap_or_default();
    assert!(classifying.contains("Message:\nnoted"), "{classifying}");
    assert_eq!(stopped.outcome?, Outcome::Hooked(HookEvent::Stop));
    Ok(())
}

#[test]
fn an_answer_quotes_the_owner_messages_the_trim_keeps() -> Result<(), Failure> {
    let asked = Asking::answered(&[Asking::LINT, Asking::PICKED, Asking::KEPT], "should the lint limit change")?;
    assert_eq!(asked.outcome?, Outcome::Answered);
    assert!(asked.text.contains("read: 2 owner messages\nquoted: 1\n"), "{}", asked.text);
    assert!(asked.text.contains(&format!("o1 2026-01-01T00:00:00Z: {}", Asking::QUOTED)), "{}", asked.text);
    Ok(())
}

#[test]
fn an_answer_drops_the_quotes_the_trim_rejects() -> Result<(), Failure> {
    let asked = Asking::answered(&[Asking::LINT, Asking::PICKED, "[]"], "should the lint limit change")?;
    assert!(asked.text.contains("quoted: 1\nanswers: none"), "{}", asked.text);
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
    let asked =
        Asking::answered(&[Asking::LINT, &picks, "[1,2,3,4,5,6,7,8,9,10]"], "what order should the queue keep")?;
    assert_eq!(asked.text.matches("o2 2026-01-01T00:00:00Z: ").count(), 8, "{}", asked.text);
    Ok(())
}

#[test]
fn an_answer_drops_a_quote_the_owner_never_wrote_and_sends_no_trim() -> Result<(), Failure> {
    let invented = "[{\"uuid\":\"o1\",\"quote\":\"the limit is gone\"},{\"uuid\":\"o2\",\"quote\":\"the lint wall\"}]";
    let asked = Asking::answered(&[Asking::LINT, invented], "should the lint limit change")?;
    assert!(asked.text.contains("quoted: 0\nanswers: none"), "{}", asked.text);
    assert_eq!(asked.prompts.len(), 2);
    Ok(())
}

#[test]
fn a_statement_reads_no_owner_messages() -> Result<(), Failure> {
    let asked = Asking::answered(&[Asking::STATEMENT], "the build is green")?;
    assert!(asked.text.contains("statement: asks the owner nothing"), "{}", asked.text);
    assert_eq!((asked.text.contains("read:"), asked.prompts.len()), (false, 1));
    Ok(())
}

#[test]
fn a_stop_that_asks_the_owner_blocks_with_the_quotes_that_answer_it() -> Result<(), Box<dyn Error>> {
    let asking = Asking::over(&[Asking::COVERED, Asking::LINT, Asking::PICKED, Asking::KEPT])?;
    let stopped = asking.asked(Asking::STOPPING, &["hook", "stop"]);
    let decision: Value = serde_json::from_str(&stopped.text)?;
    let reason = decision.get("reason").and_then(Value::as_str).unwrap_or_default();
    assert_eq!(stopped.outcome?, Outcome::Blocked);
    assert!(reason.contains("The transcript may already answer this; reread before asking:"), "{reason}");
    assert!(reason.contains(Asking::QUOTED), "{reason}");
    Ok(())
}

#[test]
fn a_stop_already_held_by_its_hook_asks_nothing() -> Result<(), Failure> {
    let asking = Asking::over(&[Asking::COVERED, Asking::LINT, Asking::PICKED])?;
    let held = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":true,\"last_assistant_message\":\"Should the lint limit change?\"}";
    let stopped = asking.asked(held, &["hook", "stop"]);
    assert_eq!((stopped.outcome?, stopped.prompts.len()), (Outcome::Hooked(HookEvent::Stop), 1));
    Ok(())
}
