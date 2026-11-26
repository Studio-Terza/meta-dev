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
    const LINT: &str =
        "{\"kind\":\"question\",\"question\":\"should the lint limit change\",\"terms\":[\"lint\",\"banana\"]}";
    const QUEUE: &str =
        "{\"kind\":\"question\",\"question\":\"what order should the queue keep\",\"terms\":[\"queue\",\"order\"]}";
    const PICKED: &str = "[{\"uuid\":\"o1\",\"quote\":\"the lint wall time limit stays at three seconds\"}]";
    const COVERED: &str = "conversational(o1.0)\nconversational(o2.0)";
    const STOPPING: &str = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false,\"last_assistant_message\":\"Should the lint limit change?\"}";

    fn over(replies: &[&str]) -> Result<Asking, Failure> {
        let session = Session::working(
            [
                Session::line(Speaker::Owner, "o1", "null", Asking::RULED),
                Session::line(Speaker::Lead, "l1", "\"o1\"", "should the queue keep its order"),
                Session::line(Speaker::Owner, "o2", "\"l1\"", Asking::QUEUED),
                Session::line(Speaker::Lead, "l2", "\"o2\"", "noted"),
            ]
            .concat(),
        );
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

    fn scored(replies: &[&str]) -> Result<Asking, Failure> {
        let crowd = (1..=8_u8).flat_map(|index| {
            [
                Session::line(Speaker::Owner, &format!("a{index}"), "null", "lint alpha beta"),
                Session::line(Speaker::Owner, &format!("b{index}"), "null", "wall alpha beta"),
            ]
        });
        let session = Session::working(
            crowd.chain([Session::line(Speaker::Owner, "x1", "null", "lint wall alpha beta")]).collect::<String>(),
        );
        let asking = Asking {
            store: Sqlite::in_memory::<StreamSchema>(),
            files: session.files(),
            script: Scripted::replying(replies.iter().map(|reply| Some(Output::Finished(Asking::finished(reply))))),
        };
        Invocation::within(&asking.store, FakeLent::new(&asking.files), &["ingest"])?;
        Ok(asking)
    }
}

#[test]
fn an_eval_scores_each_stage_of_a_turn_that_asked_against_the_owners_earlier_answer() -> Result<(), Failure> {
    let session = Session::working(
        [
            Session::stamped(Speaker::Owner, "o1", "null", Asking::RULED, "2026-01-01"),
            Session::stamped(Speaker::Lead, "l1", "\"o1\"", "should the lint limit change", "2026-01-02"),
            Session::stamped(Speaker::Owner, "o2", "\"l1\"", "the lint limit stays", "2026-01-03"),
        ]
        .concat(),
    );
    let replies = [Asking::LINT, Asking::LINT, "[\"o1\",\"o9\"]", Asking::PICKED, Asking::PICKED, "[\"o1\"]", "[]"];
    let asking = Asking {
        store: Sqlite::in_memory::<StreamSchema>(),
        files: session.files(),
        script: Scripted::replying(replies.iter().map(|reply| Some(Output::Finished(Asking::finished(reply))))),
    };
    Invocation::within(&asking.store, FakeLent::new(&asking.files), &["ingest"])?;
    let evaluated = asking.asked("", &["eval", "s", "2026-01-03T00:00:00Z", "2026-01-04T00:00:00Z"]);
    let tallied = "judged=1 answered=1 truths=1 twelve_any=1 all_any=1 twelve_truths=1 all_truths=1 truth_tokens=37 store=[found_any=1 found=1 reachable=1 picks=1 verified=1 tokens=37 seconds=0] base=[found_any=1 found=1 reachable=1 picks=1 verified=0 tokens=37 seconds=0]";
    assert_eq!(evaluated.outcome?, Outcome::Evaluated);
    assert!(
        evaluated.text.contains(&format!("turn l1 2026-01-02T00:00:00Z haiku=Asks sonnet=Asks truth=[o1] {tallied}")),
        "{}",
        evaluated.text
    );
    assert!(evaluated.text.contains("stage1 both=1 haiku_only=0 sonnet_only=0 neither=0"), "{}", evaluated.text);
    assert!(evaluated.text.contains(&format!("total {tallied}")), "{}", evaluated.text);
    assert_eq!(
        asking.script.models(),
        [Model::Haiku, Model::Sonnet, Model::Sonnet, Model::Haiku, Model::Haiku, Model::Sonnet, Model::Sonnet]
    );
    let grounding = evaluated.prompts.get(2).cloned().unwrap_or_default();
    assert!(grounding.contains(&format!("[o1 2026-01-01T00:00:00Z] {}", Asking::RULED)), "{grounding}");
    assert!(grounding.contains("[o2 2026-01-03T00:00:00Z] the lint limit stays"), "{grounding}");
    Ok(())
}

#[test]
fn the_classifier_reads_the_message_and_the_selector_reads_each_candidate_under_its_header() -> Result<(), Failure> {
    let asked = Asking::answered(&[Asking::LINT, Asking::PICKED], "is the lint limit still three seconds")?;
    let classifying = asked.prompts.first().cloned().unwrap_or_default();
    let selecting = asked.prompts.get(1).cloned().unwrap_or_default();
    assert!(classifying.contains("is the lint limit still three seconds"), "{classifying}");
    assert!(selecting.contains("should the lint limit change"), "{selecting}");
    assert!(selecting.contains(&format!("[o1 2026-01-01T00:00:00Z] {}", Asking::RULED)), "{selecting}");
    Ok(())
}

#[test]
fn a_message_matching_two_terms_outranks_many_that_match_one() -> Result<(), Failure> {
    let wall = "{\"kind\":\"question\",\"question\":\"lint wall\",\"terms\":[\"lint\",\"wall\"]}";
    let asked = Asking::scored(&[wall, "[]"])?.asked("", &["answer", "lint wall"]);
    assert!(asked.text.contains(" x1"), "{}", asked.text);
    Ok(())
}

#[test]
fn a_stop_without_its_last_message_reads_the_sessions_last_lead_message() -> Result<(), Failure> {
    let asking = Asking::over(&[Asking::COVERED, "{\"kind\":\"statement\",\"question\":\"\",\"terms\":[]}"])?;
    let stopping = "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false,\"session_id\":\"s\"}";
    let stopped = asking.asked(stopping, &["hook", "stop"]);
    let classifying = stopped.prompts.get(1).cloned().unwrap_or_default();
    assert!(classifying.contains("Message:\nnoted"), "{classifying}");
    assert_eq!(stopped.outcome?, Outcome::Hooked(HookEvent::Stop));
    Ok(())
}

#[test]
fn an_answer_quotes_the_owner_message_that_shares_any_one_term_with_the_question() -> Result<(), Failure> {
    let asked = Asking::answered(&[Asking::LINT, Asking::PICKED], "should the lint limit change")?;
    assert_eq!(asked.outcome?, Outcome::Answered);
    assert!(asked.text.contains("candidates: o1\n"), "{}", asked.text);
    assert!(asked.text.contains(&format!("o1 2026-01-01T00:00:00Z: {}", Asking::QUOTED)), "{}", asked.text);
    Ok(())
}

#[test]
fn an_answer_reaches_the_owner_reply_to_a_lead_question_that_matched() -> Result<(), Failure> {
    let asked = Asking::answered(&[Asking::QUEUE, "[]"], "what order should the queue keep")?;
    assert!(asked.text.contains("candidates: o2\n"), "{}", asked.text);
    assert!(asked.text.contains("answers: none"), "{}", asked.text);
    Ok(())
}

#[test]
fn an_answer_drops_a_quote_the_owner_never_wrote() -> Result<(), Failure> {
    let invented = "[{\"uuid\":\"o1\",\"quote\":\"the limit is gone\"},{\"uuid\":\"o2\",\"quote\":\"the lint wall\"}]";
    let asked = Asking::answered(&[Asking::LINT, invented], "should the lint limit change")?;
    assert!(asked.text.contains("answers: none"), "{}", asked.text);
    Ok(())
}

#[test]
fn a_statement_is_not_searched_and_sends_no_selection() -> Result<(), Failure> {
    let statement = "{\"kind\":\"statement\",\"question\":\"\",\"terms\":[]}";
    let asked = Asking::answered(&[statement], "the build is green")?;
    assert!(asked.text.contains("statement: asks the owner nothing"), "{}", asked.text);
    assert_eq!((asked.text.contains("candidates"), asked.prompts.len()), (false, 1));
    Ok(())
}

#[test]
fn a_stop_that_asks_the_owner_blocks_with_the_quotes_that_answer_it() -> Result<(), Box<dyn Error>> {
    let asking = Asking::over(&[Asking::COVERED, Asking::LINT, Asking::PICKED])?;
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
