#![deny(clippy::wildcard_enum_match_arm)]

use std::ffi::OsString;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use test_failure::Failed;

use console_ports::{Heard, Shown, Variables};
use design::{Agents, Cli, Failure, HookEvent, Outcome};
use dispatch_ports::{
    AgentRun, DispatchFailure, DispatchFiller, Ended, Finished, Job, Output, Pid, Prompt, Reply, Started, SystemClock,
    Tokens, Usage, dispatch_ops,
};
use file_ports::Disk;
use grammar_slug::Letters;
use invariants::Unpoisoned;
use port_core::Pass;
use serde_json::Value;
use sqlite_ports::SqliteOpener;
use tempfile::TempDir;

const OWNER: &str = "Merge only when CI is green.";
const QUESTION: &str = "Shall I merge the branch now?";
const STALLED: &str = "Nothing is running: no background shell or agent will wake you, so the turn has stalled.";
const OVERRUN: &str =
    "OVERRUN: these background tasks ran 20 minutes without finishing or writing output; check and stop them:";
const TRAIL_HEADER: &str = "session\tshells\tagents\thalt\tread\tquoted\tkept\tfate\ttokens\tmillis";
const LONG_AGO: Duration = Duration::from_secs(3600);
const TRIMMING: u32 = 1;
const READING: u32 = 2;

#[derive(Debug, Default)]
struct Replies {
    quotes: String,
    kept: String,
    asked: Mutex<Vec<String>>,
}

impl Replies {
    fn quoting(quotes: &str, kept: &str) -> Replies {
        Replies { quotes: quotes.to_owned(), kept: kept.to_owned(), asked: Mutex::new(Vec::new()) }
    }

    fn prompts(&self) -> Vec<String> {
        self.asked.lock().unpoisoned().clone()
    }
}

#[derive(Debug)]
struct Agent<'replies> {
    replies: &'replies Replies,
}

impl DispatchFiller for Agent<'_> {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        job: &Job,
        prompt: &Prompt,
    ) -> Result<AgentRun, DispatchFailure> {
        let text: String = String::from_utf8(AsRef::<[u8]>::as_ref(prompt).to_vec()).into_iter().collect();
        let pid = Pid::new(if text.contains("numbered") { TRIMMING } else { READING });
        self.replies.asked.lock().unpoisoned().push(text);
        Ok(AgentRun::new(pid, *job.as_ref()))
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        run: &mut AgentRun,
    ) -> Result<Option<Output>, DispatchFailure> {
        let reply = if run.pid() == Pid::new(TRIMMING) { &self.replies.kept } else { &self.replies.quotes };
        let usage = Usage::new(Tokens::new(3), Tokens::new(4), Tokens::new(0), Tokens::new(0));
        Ok(Some(Output::Finished(Finished::new(Reply::new(reply.clone()), usage))))
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, _pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        Ok(None)
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, _pid: Pid) -> Result<Ended, DispatchFailure> {
        Ok(Ended::Absent)
    }
}

#[derive(Debug, PartialEq)]
struct Refusal(String);

#[derive(Debug)]
struct Said {
    shown: String,
    ended: Result<Outcome, Refusal>,
}

impl Said {
    fn reason(&self) -> Result<Option<String>, Failed> {
        let decision: Value = serde_json::from_str(&self.shown)?;
        Ok(decision.get("reason").and_then(Value::as_str).map(str::to_owned))
    }
}

#[derive(Debug)]
struct Home {
    directory: TempDir,
}

impl Home {
    fn new() -> Result<Home, Failed> {
        let home = Home { directory: TempDir::new()? };
        fs::create_dir_all(home.workspace())?;
        Ok(home)
    }

    fn holding(lines: &[String]) -> Result<Home, Failed> {
        let home = Home::new()?;
        let transcript = home.transcript();
        fs::create_dir_all(transcript.parent().unwrap_or(home.path()))?;
        fs::write(&transcript, lines.concat())?;
        Ok(home)
    }

    fn path(&self) -> &Path {
        self.directory.path()
    }

    fn workspace(&self) -> PathBuf {
        self.path().join("work")
    }

    fn transcript(&self) -> PathBuf {
        let slug = Letters::slugged(&self.workspace().to_string_lossy());
        self.path().join(".claude/projects").join(slug).join("s.jsonl")
    }

    fn line(kind: &str, uuid: &str, text: &str) -> String {
        let (origin, content) = match kind {
            "user" => ("{\"kind\":\"human\"}", format!("\"{text}\"")),
            _ => ("null", format!("[{{\"type\":\"text\",\"text\":\"{text}\"}}]")),
        };
        format!(
            "{{\"type\":\"{kind}\",\"uuid\":\"{uuid}\",\"parentUuid\":null,\"sessionId\":\"s\",\"timestamp\":\"2026-01-01T00:00:00Z\",\"origin\":{origin},\"message\":{{\"model\":\"claude\",\"content\":{content}}}}}\n"
        )
    }

    fn conversation() -> [String; 2] {
        [Home::line("user", "o1", OWNER), Home::line("assistant", "l1", "Understood.")]
    }

    fn answered() -> Result<Home, Failed> {
        Home::holding(&Home::conversation())
    }

    fn said(&self, arguments: &[&str], input: &str, replies: &Replies) -> Said {
        let environment: Variables = [
            (OsString::from("HOME"), self.path().as_os_str().to_owned()),
            (OsString::from("PWD"), self.path().as_os_str().to_owned()),
        ]
        .into_iter()
        .collect::<Variables>()
        .hearing(Heard::new(input.to_owned()));
        let words = ["--workspace", "work"].iter().chain(arguments).map(OsString::from);
        let uttered =
            Cli::new(&environment, Disk, SqliteOpener, Agents::new(|_kept: PathBuf| Agent { replies }, SystemClock))
                .rooted(words);
        let shown = AsRef::<Shown>::as_ref(&uttered).to_string();
        Said { shown, ended: uttered.ended().map_err(|failure| Refusal(failure.to_string())) }
    }

    fn stop(&self, extra: &str) -> String {
        format!(
            "{{\"session_id\":\"s\",\"hook_event_name\":\"Stop\",\"stop_hook_active\":false,\"transcript_path\":{}{extra}}}",
            Value::from(self.transcript().to_string_lossy().into_owned())
        )
    }

    fn asking(&self) -> String {
        self.stop(&format!(",\"last_assistant_message\":\"{QUESTION}\""))
    }

    fn agent_task(&self, name: &str, touched: SystemTime) -> Result<(), Failed> {
        let folder = self.transcript().with_extension("").join("subagents");
        fs::create_dir_all(&folder)?;
        let task = folder.join(format!("{name}.jsonl"));
        fs::write(&task, "{\"type\":\"user\"}\n")?;
        File::options().write(true).open(&task)?.set_modified(touched)?;
        Ok(())
    }
}

#[test]
fn ingest_reports_each_session_read_the_authors_and_the_rows_written_and_a_second_ingest_writes_none()
-> Result<(), Failed> {
    let home = Home::answered()?;
    let replies = Replies::default();
    let first = home.said(&["ingest"], "", &replies);
    let second = home.said(&["ingest"], "", &replies);
    let read = Home::conversation().concat().len();
    assert_eq!(
        (first.shown.as_str(), first.ended),
        (
            format!("session s read to {read}\nauthorship owner 1\nauthorship lead 1\nwritten 3\n").as_str(),
            Ok(Outcome::Ingested)
        )
    );
    assert_eq!((second.shown.as_str(), second.ended), ("written 0\n", Ok(Outcome::Ingested)));
    Ok(())
}

#[test]
fn a_dry_run_ingest_reports_the_same_rows_but_keeps_none() -> Result<(), Failed> {
    let home = Home::answered()?;
    let replies = Replies::default();
    let dry = home.said(&["ingest", "--dry-run"], "", &replies);
    let real = home.said(&["ingest"], "", &replies);
    assert!(dry.shown.contains("written 3\n"), "{}", dry.shown);
    assert_eq!(dry.shown, real.shown);
    Ok(())
}

#[test]
fn an_unrecognised_line_is_counted_by_its_kind() -> Result<(), Failed> {
    let home = Home::holding(&[Home::line("user", "o1", OWNER), Home::line("brand-new-kind", "x1", "odd")])?;
    let said = home.said(&["ingest"], "", &Replies::default());
    assert!(said.shown.contains("unrecognised brand-new-kind 1\n"), "{}", said.shown);
    Ok(())
}

#[test]
fn the_hook_settings_name_a_command_running_each_hook_in_the_project() -> Result<(), Failed> {
    let said = Home::new()?.said(&["hook", "--settings"], "", &Replies::default());
    let settings: Value = serde_json::from_str(&said.shown)?;
    let handler = settings.pointer("/hooks/Stop/0/hooks/0").cloned().unwrap_or_default();
    assert_eq!(said.ended, Ok(Outcome::Settings));
    assert_eq!(handler.get("type").and_then(Value::as_str), Some("command"));
    assert_eq!(
        handler.get("command").and_then(Value::as_str),
        Some("design --workspace \"$CLAUDE_PROJECT_DIR\" hook stop")
    );
    assert_eq!(handler.get("timeout").and_then(Value::as_u64), Some(3600));
    assert_eq!(settings.get("hooks").and_then(Value::as_object).map(serde_json::Map::len), Some(1));
    Ok(())
}

#[test]
fn a_continuing_stop_passes_silently() -> Result<(), Failed> {
    let home = Home::answered()?;
    let replies = Replies::default();
    let said = home.said(&["hook", "stop"], "{\"stop_hook_active\":true,\"session_id\":\"s\"}", &replies);
    assert_eq!((said.shown.as_str(), said.ended), ("", Ok(Outcome::Hooked(HookEvent::Stop))));
    assert_eq!(replies.prompts(), Vec::<String>::new());
    Ok(())
}

#[test]
fn a_stop_with_nothing_running_is_blocked_as_stalled() -> Result<(), Failed> {
    let home = Home::answered()?;
    let said = home.said(&["hook", "stop"], &home.stop(""), &Replies::default());
    let decision: Value = serde_json::from_str(&said.shown)?;
    assert_eq!(said.ended, Ok(Outcome::Blocked));
    assert_eq!(decision.get("decision").and_then(Value::as_str), Some("block"));
    assert_eq!(said.reason()?.as_deref(), Some(STALLED));
    Ok(())
}

#[test]
fn a_stop_with_an_agent_still_writing_passes() -> Result<(), Failed> {
    let home = Home::answered()?;
    home.agent_task("agent-busy", SystemTime::now())?;
    let said = home.said(&["hook", "stop"], &home.stop(""), &Replies::default());
    assert_eq!((said.shown.as_str(), said.ended), ("", Ok(Outcome::Hooked(HookEvent::Stop))));
    Ok(())
}

#[test]
fn a_stop_without_a_transcript_sees_no_tasks_and_passes() -> Result<(), Failed> {
    let home = Home::answered()?;
    let said = home.said(&["hook", "stop"], "{\"stop_hook_active\":false}", &Replies::default());
    assert_eq!((said.shown.as_str(), said.ended), ("", Ok(Outcome::Hooked(HookEvent::Stop))));
    Ok(())
}

#[test]
fn an_agent_quiet_for_twenty_minutes_is_reported_once_as_an_overrun() -> Result<(), Failed> {
    let home = Home::answered()?;
    home.agent_task("agent-hung", SystemTime::now() - LONG_AGO)?;
    let replies = Replies::default();
    let first = home.said(&["hook", "stop"], &home.stop(""), &replies);
    let second = home.said(&["hook", "stop"], &home.stop(""), &replies);
    assert_eq!(first.reason()?, Some(format!("{OVERRUN} agent-hung")));
    assert_eq!(second.reason()?.as_deref(), Some(STALLED));
    Ok(())
}

#[test]
fn a_stop_quotes_the_earlier_owner_words_the_reading_agents_keep() -> Result<(), Failed> {
    let home = Home::answered()?;
    let replies = Replies::quoting(&format!("[{{\"uuid\": \"o1\", \"quote\": \"{OWNER}\"}}]"), "[1]");
    let said = home.said(&["hook", "stop"], &home.asking(), &replies);
    let quoted = format!("Earlier owner words bearing on this:\no1 2026-01-01T00:00:00Z: \"{OWNER}\"\n");
    assert_eq!(said.reason()?, Some([STALLED, "\n", &quoted].concat()));
    let prompts = replies.prompts();
    assert_eq!(prompts.len(), 2);
    assert!(prompts.iter().all(|prompt| prompt.contains(QUESTION)));
    assert!(prompts.first().is_some_and(|prompt| prompt.contains(OWNER) && prompt.contains("[o1 ")));
    assert!(prompts.get(1).is_some_and(|prompt| prompt.contains(&format!("1. o1 2026-01-01T00:00:00Z: \"{OWNER}\""))));
    Ok(())
}

#[test]
fn a_quote_the_owner_never_wrote_is_dropped_and_no_trimming_is_asked() -> Result<(), Failed> {
    let home = Home::answered()?;
    let replies = Replies::quoting("[{\"uuid\": \"o1\", \"quote\": \"Never said this.\"}]", "[1]");
    let said = home.said(&["hook", "stop"], &home.asking(), &replies);
    assert_eq!(said.reason()?.as_deref(), Some(STALLED));
    assert_eq!(replies.prompts().len(), 1);
    Ok(())
}

#[test]
fn the_stops_trail_lists_each_stop_with_its_reading() -> Result<(), Failed> {
    let home = Home::answered()?;
    let replies = Replies::quoting(&format!("[{{\"uuid\": \"o1\", \"quote\": \"{OWNER}\"}}]"), "[1]");
    home.said(&["hook", "stop"], &home.asking(), &replies);
    let trail = home.said(&["stops"], "", &replies);
    assert_eq!(trail.ended, Ok(Outcome::Trailed));
    assert!(
        trail.shown.contains(&format!("{TRAIL_HEADER}\ns\t0\t0\tstalled\t1\t1\t1\tblocked\t14\t")),
        "{}",
        trail.shown
    );
    assert_eq!(trail.shown.matches('\n').count(), 2);
    Ok(())
}

#[test]
fn words_the_cli_does_not_know_are_refused_with_the_usage() -> Result<(), Failed> {
    let home = Home::new()?;
    let usage = Failure::Usage.to_string();
    for words in
        [&["frobnicate"][..], &["hook"][..], &["hook", "stop", "--settings"][..], &["ingest", "--wet"][..], &[][..]]
    {
        let said = home.said(words, "", &Replies::default());
        assert_eq!((said.shown.as_str(), said.ended), ("", Err(Refusal(usage.clone()))), "{words:?}");
    }
    Ok(())
}

#[test]
fn a_stop_hook_that_cannot_read_its_input_fails() -> Result<(), Failed> {
    let said = Home::answered()?.said(&["hook", "stop"], "not json", &Replies::default());
    assert!(said.ended.is_err_and(|failure| failure.0.contains("cannot read the hook input: ")));
    Ok(())
}
