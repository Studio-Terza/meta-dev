#![deny(clippy::wildcard_enum_match_arm)]

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime};
use test_failure::Failed;

use console_ports::{Heard, Shown, Variables};
use design::{Agents, Cli, Failure, HookEvent, Outcome};
use dispatch_ports::{
    AgentRun, Bell, ClockFiller, DispatchFailure, DispatchFiller, Ended, Errored, Finished, Job, Moment, Output, Pause,
    Pid, Prompt, Reply, Started, Tokens, Told, Usage, Wall, clock_ops, dispatch_ops,
};
use file_ports::{Bytes, Disk, FileFailure, Files, Kind, Offset, Size, Stamps, Text};
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
const QUIET: Duration = Duration::from_mins(20);
const BUSY: Duration = Duration::from_mins(19);
const ASKED: &str = "{\"type\":\"user\"}\n";
const TEMPORARY: &str = "/private/tmp";
const TRIMMING: u32 = 1;
const READING: u32 = 2;
const STEP: i64 = 5;

#[derive(Debug)]
struct Stepping {
    now: SystemTime,
    moments: AtomicI64,
}

impl ClockFiller for Stepping {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        Wall::from(self.now)
    }

    fn moment(&self, _pass: Pass<'_, clock_ops::Moment>) -> Moment {
        Moment::new(self.moments.fetch_add(STEP, Ordering::Relaxed))
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, _pause: Pause, _bell: &Bell) -> Wall {
        Wall::from(SystemTime::now())
    }
}

#[derive(Debug, Default)]
struct Replies {
    quotes: String,
    kept: String,
    refused: Option<Told>,
    asked: Mutex<Vec<String>>,
}

impl Replies {
    fn quoting(quotes: &str, kept: &str) -> Replies {
        Replies { quotes: quotes.to_owned(), kept: kept.to_owned(), refused: None, asked: Mutex::new(Vec::new()) }
    }

    fn refusing() -> Result<Replies, Failed> {
        Ok(Replies { refused: Some(Told::try_new("refused".to_owned())?), ..Replies::default() })
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
        Ok(Some(self.replies.refused.clone().map_or_else(
            || Output::Finished(Finished::new(Reply::new(reply.clone()), usage)),
            |told| Output::Errored(Errored::new(told, usage)),
        )))
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
struct Rerooted {
    temporary: PathBuf,
}

impl Rerooted {
    fn real(&self, path: &Path) -> PathBuf {
        match path.strip_prefix(TEMPORARY) {
            Ok(below) => self.temporary.join(below),
            Err(_elsewhere) => path.to_path_buf(),
        }
    }
}

impl Files for Rerooted {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        Disk.read(&self.real(path))
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        Disk.read_text(&self.real(path))
    }

    fn read_from(&self, path: &Path, offset: Offset) -> Result<Bytes, FileFailure> {
        Disk.read_from(&self.real(path), offset)
    }

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        Disk.write(&self.real(path), bytes)
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        Disk.append(&self.real(path), bytes)
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        Disk.create_directories(&self.real(path))
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        Disk.copy(&self.real(from), &self.real(to))
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        Disk.entries(&self.real(directory))
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        Disk.canonical(&self.real(path))
    }

    fn kind(&self, path: &Path) -> Kind {
        Disk.kind(&self.real(path))
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        Disk.size(&self.real(path))
    }

    fn stamps(&self, path: &Path) -> Result<Stamps, FileFailure> {
        Disk.stamps(&self.real(path))
    }
}

#[derive(Debug)]
struct Home {
    directory: TempDir,
    at: SystemTime,
}

impl Home {
    fn new() -> Result<Home, Failed> {
        let home = Home { directory: TempDir::new()?, at: SystemTime::now() };
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
        let files = Rerooted { temporary: self.path().join("tmp") };
        let clock = Stepping { now: self.at, moments: AtomicI64::default() };
        let uttered =
            Cli::new(&environment, files, SqliteOpener, Agents::new(|_kept: PathBuf| Agent { replies }, clock))
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

    fn appended(&self, line: &str) -> Result<(), Failed> {
        File::options().append(true).open(self.transcript())?.write_all(line.as_bytes())?;
        Ok(())
    }

    fn agent_task(&self, file: &str, text: &str, quiet: Duration) -> Result<(), Failed> {
        let folder = self.transcript().with_extension("").join("subagents");
        fs::create_dir_all(&folder)?;
        let task = folder.join(file);
        fs::write(&task, text)?;
        File::options().write(true).open(&task)?.set_modified(self.at - quiet)?;
        Ok(())
    }

    fn shell_task(&self, file: &str, text: &str) -> Result<(), Failed> {
        let slug = Letters::slugged(&self.workspace().to_string_lossy());
        let folder = self.path().join("tmp/claude-501").join(slug).join("s/tasks");
        fs::create_dir_all(&folder)?;
        fs::write(folder.join(file), text)?;
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
fn a_stop_with_an_agent_quiet_under_twenty_minutes_passes_and_trails_it_as_live() -> Result<(), Failed> {
    let home = Home::answered()?;
    home.agent_task("agent-busy.jsonl", ASKED, BUSY)?;
    let replies = Replies::default();
    let said = home.said(&["hook", "stop"], &home.stop(""), &replies);
    let trail = home.said(&["stops"], "", &replies);
    assert_eq!((said.shown.as_str(), said.ended), ("", Ok(Outcome::Hooked(HookEvent::Stop))));
    assert!(trail.shown.contains("\ns\t0\t1\tlive\t"), "{}", trail.shown);
    Ok(())
}

#[test]
fn a_stop_with_a_shell_still_running_passes_and_trails_it_as_live() -> Result<(), Failed> {
    let home = Home::answered()?;
    home.shell_task("b1.output", "building\n")?;
    let replies = Replies::default();
    let said = home.said(&["hook", "stop"], &home.stop(""), &replies);
    let trail = home.said(&["stops"], "", &replies);
    assert_eq!((said.shown.as_str(), said.ended), ("", Ok(Outcome::Hooked(HookEvent::Stop))));
    assert!(trail.shown.contains("\ns\t1\t0\tlive\t"), "{}", trail.shown);
    Ok(())
}

#[test]
fn finished_tasks_and_files_that_are_not_tasks_leave_the_stop_stalled() -> Result<(), Failed> {
    let home = Home::answered()?;
    let ended = "{\"type\":\"assistant\",\"message\":{\"stop_reason\":\"end_turn\"}}\n";
    home.shell_task("b1.output", &["x".repeat(1024), "\n[exited with code 0]\n".to_owned()].concat())?;
    home.shell_task("b2.log", "building\n")?;
    home.agent_task("agent-done.jsonl", &[ASKED, ended].concat(), QUIET)?;
    home.agent_task("agent-done.meta.json", "{}\n", QUIET)?;
    let said = home.said(&["hook", "stop"], &home.stop(""), &Replies::default());
    assert_eq!(said.reason()?.as_deref(), Some(STALLED));
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
    home.agent_task("agent-hung.jsonl", ASKED, QUIET)?;
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
        trail.shown.contains(&format!("{TRAIL_HEADER}\ns\t0\t0\tstalled\t1\t1\t1\tblocked\t14\t{STEP}\n")),
        "{}",
        trail.shown
    );
    assert_eq!(trail.shown.matches('\n').count(), 2);
    Ok(())
}

#[test]
fn a_refused_reading_still_counts_its_tokens_in_the_trail() -> Result<(), Failed> {
    let home = Home::answered()?;
    let replies = Replies::refusing()?;
    let said = home.said(&["hook", "stop"], &home.asking(), &replies);
    let trail = home.said(&["stops"], "", &replies);
    assert_eq!(said.reason()?.as_deref(), Some(STALLED));
    assert_eq!(trail.shown, format!("{TRAIL_HEADER}\ns\t0\t0\tstalled\t1\t0\t0\tblocked\t7\t{STEP}\n"));
    Ok(())
}

#[test]
fn a_stop_without_its_last_message_reads_with_the_leads_stored_words() -> Result<(), Failed> {
    let home = Home::answered()?;
    let replies = Replies::quoting(&format!("[{{\"uuid\": \"o1\", \"quote\": \"{OWNER}\"}}]"), "[1]");
    home.said(&["hook", "stop"], &home.stop(""), &replies);
    let prompts = replies.prompts();
    assert_eq!(prompts.len(), 2);
    assert!(prompts.iter().all(|prompt| prompt.contains("Understood.")), "{prompts:?}");
    Ok(())
}

#[test]
fn an_ingest_after_the_transcript_grows_moves_the_cursor_once() -> Result<(), Failed> {
    let home = Home::answered()?;
    let replies = Replies::default();
    home.said(&["ingest"], "", &replies);
    let added = Home::line("user", "o2", "Ship it.");
    home.appended(&added)?;
    let grown = home.said(&["ingest"], "", &replies);
    let settled = home.said(&["ingest"], "", &replies);
    let read = Home::conversation().concat().len() + added.len();
    assert_eq!(grown.shown, format!("session s read to {read}\nauthorship owner 1\nwritten 2\n"));
    assert_eq!(settled.shown, "written 0\n");
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
