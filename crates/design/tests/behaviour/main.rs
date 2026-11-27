#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{Duration, SystemTime};
use test_failure::Failed;

use console_ports::{Heard, Shown, Variables};
use design::{Agents, Cli, Failure, HookEvent, Outcome};
use dispatch_ports::{
    AgentRun, Bell, Call, ClockFiller, Deadline, DispatchFailure, DispatchFiller, Ended, Errored, Finished, Job,
    Moment, Output, Pause, Pid, Printed, Prompt, Ran, Reply, ShellFiller, Started, Timeout, Tokens, Told, Usage, Wall,
    clock_ops, dispatch_ops, shell_ops,
};
use file_ports::{Bytes, Disk, FileFailure, Files, Kind, Offset, Size, Stamps, Text};
use grammar_slug::Letters;
use invariants::Unpoisoned;
use newtype::Wrapping;
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
const STOPPED: &str = "{\"type\":\"user\"}\n{\"type\":\"api-request\"}\n";
const GOLDENS: &str = "tests/goldens";
const TEMPORARY: &str = "/private/tmp";
const TRIMMING: u32 = 1;
const READING: u32 = 2;
const STEP: i64 = 5;
const AGENT: &str = "/work/agent";
const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";
const HEAD: &str = "pr view 12 --json headRefOid";
const WATCH: &str = "pr checks 12 --watch --fail-fast";
const CHECKS: &str = "pr checks 12 --json name,bucket,link";
const LISTED: &str = "pr checks 12 --json name,state";
const LOG: &str = "run view --job 456 --log-failed";
const UNTYPED: &str = "End your turn with exactly one line and nothing else: DONE <branch> <full sha> once that \
                       commit is the pushed head of a branch with no pull request yet, DONE #<pr> <full sha> once the \
                       pull request's head is that commit, or BLOCKED #<pr> <full sha> when you cannot go on.";
const BRANCH: &str = "design/open-pr";
const REMOTE: &str = "api graphql -F owner={owner} -F name={repo} -f ref=refs/heads/design/open-pr -f \
                      query=query($owner:String!,$name:String!,$ref:String!){repository(owner:$owner,name:$name){ref(\
                      qualifiedName:$ref){target{oid}}}}";
const FIRST: &str = "log --reverse --format=%(trailers:key=Spec,valueonly,separator=%x2c) \
                     origin/main..0123456789abcdef0123456789abcdef01234567";
const CREATE: &str = "pr create --head design/open-pr --base main --title D2 Typed agent outcomes --body spec: \
                      tools-retirement#D2\n\n🤖 Generated with [Claude Code](https://claude.com/claude-code)";
const SPEC: &str = "# Tools retirement\n\nD1 Dropped until missed (D2 later): waits.\n\nD2 Typed agent outcomes (owner \
                    de7a042f): the hook refuses (with D3) other endings.\nD20 Later work (owner 1).\n";
const VIEW: &str = "pr view design/open-pr --json number";

#[derive(Debug)]
struct Stepping {
    now: SystemTime,
    moments: AtomicI64,
    slept: AtomicU64,
}

impl Stepping {
    fn at(now: SystemTime) -> Stepping {
        Stepping { now, moments: AtomicI64::default(), slept: AtomicU64::default() }
    }

    fn wall(&self) -> Wall {
        Wall::from(self.now + Duration::from_secs(self.slept.load(Ordering::Relaxed)))
    }
}

impl ClockFiller for Stepping {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        self.wall()
    }

    fn moment(&self, _pass: Pass<'_, clock_ops::Moment>) -> Moment {
        Moment::new(self.moments.fetch_add(STEP, Ordering::Relaxed))
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause, _bell: &Bell) -> Wall {
        self.slept.fetch_add(u64::from(pause.into_inner()), Ordering::Relaxed);
        self.wall()
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

#[derive(Debug, Default)]
struct GitHub {
    answers: BTreeMap<String, Vec<Ran>>,
    asked: Mutex<Vec<Asked>>,
}

#[derive(Clone, Debug)]
struct Answer {
    asked: String,
    ran: Ran,
}

#[derive(Debug)]
struct Asked {
    spoken: String,
    deadline: Deadline,
}

impl Answer {
    fn new(asked: &str, ran: Ran) -> Answer {
        Answer { asked: format!("gh {asked}"), ran }
    }

    fn git(asked: &str, ran: Ran) -> Answer {
        Answer { asked: format!("git {asked}"), ran }
    }
}

impl GitHub {
    fn answering(given: &[Answer]) -> GitHub {
        let answers = given.iter().fold(BTreeMap::<String, Vec<Ran>>::new(), |mut answers, answer| {
            answers.entry(answer.asked.clone()).or_default().push(answer.ran.clone());
            answers
        });
        GitHub { answers, asked: Mutex::default() }
    }

    fn asked(&self) -> Vec<String> {
        self.asked.lock().unpoisoned().iter().map(|asked| asked.spoken.clone()).collect()
    }

    fn deadlines(&self) -> Vec<Deadline> {
        self.asked.lock().unpoisoned().iter().map(|asked| asked.deadline).collect()
    }
}

#[derive(Debug)]
struct Gh<'answers> {
    github: &'answers GitHub,
}

impl ShellFiller for Gh<'_> {
    fn ran(&self, _pass: Pass<'_, shell_ops::Ran>, call: &Call) -> Result<Ran, DispatchFailure> {
        let spoken =
            [call.program().to_owned()].into_iter().chain(call.arguments().iter().cloned()).collect::<Vec<_>>();
        let asked = Asked { spoken: spoken.join(" "), deadline: call.deadline() };
        let mut log = self.github.asked.lock().unpoisoned();
        let earlier = log.iter().filter(|before| before.spoken == asked.spoken).count();
        let answers = self.github.answers.get(&asked.spoken);
        let answer =
            answers.and_then(|rans| rans.get(earlier).or(rans.last())).cloned().ok_or(DispatchFailure::Unjoined);
        log.push(asked);
        answer
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
        [Home::line("user", "o1", OWNER), Home::line("assistant", "p1", "Understood.")]
    }

    fn answered() -> Result<Home, Failed> {
        Home::holding(&Home::conversation())
    }

    fn said(&self, arguments: &[&str], input: &str, replies: &Replies) -> Said {
        self.heard(arguments, input, replies, &GitHub::default())
    }

    fn finished(&self, said: &str, reminded: bool, github: &GitHub) -> Said {
        let input = serde_json::json!({
            "session_id": "s",
            "hook_event_name": "SubagentStop",
            "stop_hook_active": reminded,
            "cwd": AGENT,
            "last_assistant_message": said,
        });
        self.heard(&["hook", "subagent-stop"], &input.to_string(), &Replies::default(), github)
    }

    fn heard(&self, arguments: &[&str], input: &str, replies: &Replies, github: &GitHub) -> Said {
        let environment: Variables = [
            (OsString::from("HOME"), self.path().as_os_str().to_owned()),
            (OsString::from("PWD"), self.path().as_os_str().to_owned()),
        ]
        .into_iter()
        .collect::<Variables>()
        .hearing(Heard::new(input.to_owned()));
        let words = ["--workspace", "work"].iter().chain(arguments).map(OsString::from);
        let files = Rerooted { temporary: self.path().join("tmp") };
        let clock = Stepping::at(self.at);
        let agents = Agents::new(|_kept: PathBuf| Agent { replies }, clock, Gh { github });
        let uttered = Cli::new(&environment, files, SqliteOpener, agents).rooted(words);
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

    fn queued(&self, text: &str) -> Result<(), Failed> {
        let folder = self.path().join("studio-meta-dev/transcripts");
        fs::create_dir_all(&folder)?;
        fs::write(folder.join("queue.md"), text)?;
        Ok(())
    }

    fn specified(&self, text: &str) -> Result<(), Failed> {
        let folder = self.path().join("studio-meta-dev/transcripts/specs");
        fs::create_dir_all(&folder)?;
        fs::write(folder.join("tools-retirement.md"), text)?;
        Ok(())
    }

    fn golden(name: &str) -> Result<String, Failed> {
        Ok(fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDENS).join(name))?)
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
    assert_eq!(settings.get("hooks").and_then(Value::as_object).map(serde_json::Map::len), Some(2));
    let subagent = settings.pointer("/hooks/SubagentStop/0/hooks/0").cloned().unwrap_or_default();
    assert_eq!(
        subagent.get("command").and_then(Value::as_str),
        Some("design --workspace \"$CLAUDE_PROJECT_DIR\" hook subagent-stop")
    );
    assert_eq!(subagent.get("timeout").and_then(Value::as_u64), Some(1500));
    Ok(())
}

impl GitHub {
    fn head(sha: &str) -> Answer {
        Answer::new(HEAD, Ran::Succeeded(Printed::new(format!("{{\"headRefOid\":\"{sha}\"}}"))))
    }

    fn watched() -> Answer {
        Answer::new(WATCH, Ran::Succeeded(Printed::default()))
    }

    fn listed(names: &[&str]) -> Answer {
        let listed: Vec<Value> =
            names.iter().map(|name| serde_json::json!({"name": name, "state": "IN_PROGRESS"})).collect();
        Answer::new(LISTED, Ran::Failed(Printed::new(Value::from(listed).to_string())))
    }

    fn unlisted() -> Answer {
        Answer::new(LISTED, Ran::Failed(Printed::default()))
    }

    fn checks(buckets: &[[&str; 3]]) -> Answer {
        let listed: Vec<Value> = buckets
            .iter()
            .map(|[name, bucket, link]| serde_json::json!({"name": name, "bucket": bucket, "link": link}))
            .collect();
        Answer::new(CHECKS, Ran::Failed(Printed::new(Value::from(listed).to_string())))
    }

    fn red() -> GitHub {
        GitHub::answering(&[
            GitHub::head(SHA),
            GitHub::listed(&["test"]),
            Answer::new(WATCH, Ran::Failed(Printed::default())),
            GitHub::checks(&[
                ["test", "fail", "https://github.com/o/r/actions/runs/9/job/456"],
                ["build", "cancel", "https://github.com/o/r/actions/runs/9/job/458?pr=12"],
                ["lint", "pass", "https://github.com/o/r/actions/runs/9/job/457"],
            ]),
            Answer::new(
                LOG,
                Ran::Succeeded(Printed::new(
                    (1..=42).map(|line| format!("test step line {line}")).collect::<Vec<_>>().join("\n"),
                )),
            ),
        ])
    }

    fn green() -> GitHub {
        GitHub::answering(&[
            GitHub::head(SHA),
            GitHub::listed(&["test"]),
            GitHub::watched(),
            GitHub::checks(&[
                ["test", "pass", "https://github.com/o/r/actions/runs/9/job/456"],
                ["mutants", "skipping", "https://github.com/o/r/actions/runs/9/job/459"],
            ]),
        ])
    }
}

impl GitHub {
    fn remote(held: &str) -> Answer {
        let found = format!("{{\"data\":{{\"repository\":{{\"ref\":{held}}}}}}}");
        Answer::new(REMOTE, Ran::Succeeded(Printed::new(found)))
    }

    fn pushed(sha: &str) -> Answer {
        GitHub::remote(&format!("{{\"target\":{{\"oid\":\"{sha}\"}}}}"))
    }

    fn first(trailer: &str) -> Answer {
        let log = format!("{trailer}\nother#D9\n");
        Answer::git(FIRST, Ran::Succeeded(Printed::new(log)))
    }

    fn opening(after: &[Answer]) -> GitHub {
        let created = Answer::new(CREATE, Ran::Succeeded(Printed::new("https://github.com/o/r/pull/12\n".to_owned())));
        let before = [GitHub::pushed(SHA), GitHub::first("tools-retirement#D2"), created];
        GitHub::answering(&before.into_iter().chain(after.iter().cloned()).collect::<Vec<_>>())
    }
}

impl Home {
    fn queue(&self) -> Result<String, Failed> {
        Ok(fs::read_to_string(self.path().join("studio-meta-dev/transcripts/queue.md"))?)
    }
}

#[test]
fn an_untyped_ending_is_sent_back_once_with_the_two_lines_and_then_escalated() -> Result<(), Failed> {
    let home = Home::new()?;
    home.queued("")?;
    let github = GitHub::default();
    let untyped = [
        "I finished the work; CI should be green.".to_owned(),
        format!("DONE #12 {SHA} and more"),
        format!("done #12 {SHA}"),
        format!("DONE #0 {SHA}"),
        format!("DONE 12 {SHA}"),
        "DONE #12 0123456".to_owned(),
        format!("DONE #12 {SHA}0"),
        format!("DONE main {SHA}"),
        format!("DONE Design/open-pr {SHA}"),
        format!("DONE {BRANCH}/ {SHA}"),
        String::new(),
    ];
    for said in &untyped {
        let sent = home.finished(said, false, &github);
        let reason = sent.reason()?;
        assert_eq!((sent.ended, reason.as_deref()), (Ok(Outcome::Blocked), Some(UNTYPED)), "{said:?}");
    }
    let escalated = home.finished("Still prose.", true, &github);
    assert_eq!((escalated.shown.as_str(), escalated.ended), ("", Ok(Outcome::Escalated)));
    assert_eq!(home.queue()?, format!("- [agent outcome] unread ending, from {AGENT}\n"));
    assert_eq!(github.asked(), Vec::<String>::new());
    Ok(())
}

#[test]
fn a_done_on_a_sha_the_pull_request_does_not_hold_is_sent_back_to_push() -> Result<(), Failed> {
    let github = GitHub::answering(&[GitHub::head(OTHER)]);
    let said = Home::new()?.finished(&format!("DONE #12 {SHA}"), true, &github);
    assert_eq!(said.ended, Ok(Outcome::Blocked));
    assert_eq!(
        said.reason()?,
        Some(format!(
            "Pull request #12 has head {OTHER}, not {SHA}. Push the commit, then end with DONE #12 <full sha of the \
             pushed head>."
        ))
    );
    assert_eq!(github.asked(), [format!("gh {HEAD}")]);
    Ok(())
}

#[test]
fn a_done_on_green_checks_waits_on_ci_for_up_to_twenty_minutes_and_lets_the_agent_stop() -> Result<(), Failed> {
    let home = Home::new()?;
    let github = GitHub::green();
    let said = home.finished("\nDONE #12 0123456789ABCDEF0123456789ABCDEF01234567\n", false, &github);
    let wall = Wall::from(home.at);
    assert_eq!((said.shown.as_str(), said.ended), ("", Ok(Outcome::Hooked(HookEvent::SubagentStop))));
    assert_eq!(github.asked(), [HEAD, LISTED, WATCH, CHECKS].map(|asked| format!("gh {asked}")));
    assert_eq!(github.deadlines(), [60, 60, 1200, 60].map(|seconds| Deadline::after(wall, Timeout::new(seconds))));
    Ok(())
}

#[test]
fn a_done_on_red_checks_is_sent_back_with_each_failed_jobs_log_tail() -> Result<(), Failed> {
    let github = GitHub::red();
    let said = Home::new()?.finished(&format!("DONE #12 {SHA}"), false, &github);
    assert_eq!(said.ended, Ok(Outcome::Blocked));
    assert_eq!(said.reason()?, Some(Home::golden("subagent-failing.txt")?));
    assert_eq!(
        github.asked().get(4..).map(<[String]>::to_vec),
        Some(vec![format!("gh {LOG}"), "gh run view --job 458 --log-failed".to_owned()])
    );
    Ok(())
}

#[test]
fn a_check_failing_on_two_dones_in_a_row_is_escalated_unless_a_green_done_came_between() -> Result<(), Failed> {
    let done = format!("DONE #12 {SHA}");
    let repeated = Home::new()?;
    repeated.queued("")?;
    let first = repeated.finished(&done, false, &GitHub::red());
    let second = repeated.finished(&done, true, &GitHub::red());
    let cleared = Home::new()?;
    let after =
        [GitHub::red(), GitHub::green(), GitHub::red()].map(|github| cleared.finished(&done, true, &github).ended);
    assert_eq!((first.ended, second.ended), (Ok(Outcome::Blocked), Ok(Outcome::Escalated)));
    assert_eq!(repeated.queue()?, format!("- [agent outcome] repeat failure of test on #12 {SHA}, from {AGENT}\n"));
    assert_eq!(after, [Ok(Outcome::Blocked), Ok(Outcome::Hooked(HookEvent::SubagentStop)), Ok(Outcome::Blocked)]);
    Ok(())
}

#[test]
fn a_blocked_agent_and_every_unread_step_are_escalated_to_the_queue_and_the_agent_stops() -> Result<(), Failed> {
    let home = Home::new()?;
    home.queued("")?;
    let blocked = format!("BLOCKED #12 {SHA}");
    let done = format!("DONE #12 {SHA}");
    let listed = GitHub::listed(&["test"]);
    let watching =
        |checks: Answer| GitHub::answering(&[GitHub::head(SHA), GitHub::listed(&["test"]), GitHub::watched(), checks]);
    let cases = [
        (&blocked, GitHub::default()),
        (&done, GitHub::default()),
        (
            &done,
            GitHub::answering(&[Answer::new(HEAD, Ran::Failed(Printed::new(format!("{{\"headRefOid\":\"{SHA}\"}}"))))]),
        ),
        (&done, GitHub::answering(&[Answer::new(HEAD, Ran::Succeeded(Printed::new("{}".to_owned())))])),
        (&done, GitHub::answering(&[GitHub::head(SHA), listed.clone()])),
        (&done, GitHub::answering(&[GitHub::head(SHA), listed.clone(), Answer::new(WATCH, Ran::Overdue)])),
        (&done, GitHub::answering(&[GitHub::head(SHA), listed, GitHub::watched()])),
        (&done, watching(Answer::new(CHECKS, Ran::Overdue))),
        (&done, watching(GitHub::checks(&[]))),
        (&done, watching(Answer::new(CHECKS, Ran::Succeeded(Printed::new("[{".to_owned()))))),
        (&done, watching(GitHub::checks(&[["test", "pending", ""], ["lint", "pass", ""]]))),
        (&done, watching(GitHub::checks(&[["status", "fail", "https://ci.example/9"]]))),
    ];
    for (said, github) in &cases {
        let escalated = home.finished(said, false, github);
        assert_eq!((escalated.shown.as_str(), escalated.ended), ("", Ok(Outcome::Escalated)), "{said} {github:?}");
    }
    assert_eq!(home.queue()?, Home::golden("escalated-queue.md")?);
    Ok(())
}

impl GitHub {
    fn timeline(&self, at: SystemTime) -> String {
        let start = Wall::from(at).into_inner();
        let log = self.asked.lock().unpoisoned();
        let lines: Vec<String> = log
            .iter()
            .map(|asked| format!("+{} {}\n", asked.deadline.into_inner().into_inner() - start, asked.spoken))
            .collect();
        lines.concat()
    }
}

#[test]
fn a_done_before_any_check_is_listed_polls_until_checks_appear_then_waits_for_green() -> Result<(), Failed> {
    let home = Home::new()?;
    let green = GitHub::checks(&[["test", "pass", ""], ["lint", "pass", ""]]);
    let github = GitHub::answering(&[
        GitHub::head(SHA),
        GitHub::unlisted(),
        GitHub::listed(&[]),
        GitHub::listed(&["test", "lint"]),
        GitHub::watched(),
        green,
    ]);
    let said = home.finished(&format!("DONE #12 {SHA}"), false, &github);
    assert_eq!((said.shown.as_str(), said.ended), ("", Ok(Outcome::Hooked(HookEvent::SubagentStop))));
    assert_eq!(github.timeline(home.at), Home::golden("checks-appear.txt")?);
    Ok(())
}

#[test]
fn a_done_whose_checks_never_appear_is_escalated_after_five_minutes_of_polling() -> Result<(), Failed> {
    let home = Home::new()?;
    home.queued("")?;
    let github = GitHub::answering(&[GitHub::head(SHA), GitHub::unlisted()]);
    let said = home.finished(&format!("DONE #12 {SHA}"), false, &github);
    assert_eq!((said.shown.as_str(), said.ended), ("", Ok(Outcome::Escalated)));
    assert_eq!(github.timeline(home.at), Home::golden("no-checks.txt")?);
    assert_eq!(home.queue()?, Home::golden("escalated-no-checks-queue.md")?);
    Ok(())
}

#[test]
fn a_done_on_a_pushed_branch_opens_its_pull_request_from_the_first_commit_and_waits_on_ci() -> Result<(), Failed> {
    let viewed = Answer::new(VIEW, Ran::Succeeded(Printed::new("{\"number\":12}".to_owned())));
    let green = GitHub::checks(&[["test", "pass", "https://github.com/o/r/actions/runs/9/job/456"]]);
    let github = GitHub::opening(&[viewed, GitHub::head(SHA), GitHub::listed(&["test"]), GitHub::watched(), green]);
    let home = Home::new()?;
    home.specified(SPEC)?;
    let said = home.finished(&format!("DONE {BRANCH} {SHA}"), false, &github);
    assert_eq!((said.shown.as_str(), said.ended), ("", Ok(Outcome::Hooked(HookEvent::SubagentStop))));
    let asked = [format!("gh {REMOTE}"), format!("git {FIRST}"), format!("gh {CREATE}"), format!("gh {VIEW}")];
    let checked = [HEAD, LISTED, WATCH, CHECKS].map(|step| format!("gh {step}"));
    assert_eq!(github.asked(), [&asked[..], &checked[..]].concat());
    Ok(())
}

#[test]
fn a_done_on_a_branch_whose_remote_head_differs_is_sent_back_to_push() -> Result<(), Failed> {
    let home = Home::new()?;
    let push = format!(" Push the commit, then end with DONE {BRANCH} <full sha of the pushed head>.");
    let cases = [
        (GitHub::pushed(OTHER), format!("Branch {BRANCH} has remote head {OTHER}, not {SHA}.{push}")),
        (GitHub::remote("null"), format!("Branch {BRANCH} is not on the remote.{push}")),
    ];
    for (remote, reason) in cases {
        let github = GitHub::answering(&[remote]);
        let said = home.finished(&format!("DONE {BRANCH} {SHA}"), true, &github);
        assert_eq!((said.reason()?, said.ended), (Some(reason), Ok(Outcome::Blocked)));
        assert_eq!(github.asked(), [format!("gh {REMOTE}")]);
    }
    Ok(())
}

#[test]
fn a_branch_that_cannot_be_read_or_opened_is_escalated_on_the_branch() -> Result<(), Failed> {
    let home = Home::new()?;
    home.queued("")?;
    home.specified(SPEC)?;
    let unviewed = Answer::new(VIEW, Ran::Succeeded(Printed::new("{}".to_owned())));
    let cases = [
        (format!("BLOCKED {BRANCH} {SHA}"), GitHub::default()),
        (format!("DONE {BRANCH} {SHA}"), GitHub::default()),
        (format!("DONE {BRANCH} {SHA}"), GitHub::answering(&[GitHub::remote("{}")])),
        (format!("DONE {BRANCH} {SHA}"), GitHub::answering(&[GitHub::pushed(SHA)])),
        (
            format!("DONE {BRANCH} {SHA}"),
            GitHub::answering(&[GitHub::pushed(SHA), Answer::git(FIRST, Ran::Succeeded(Printed::default()))]),
        ),
        (format!("DONE {BRANCH} {SHA}"), GitHub::answering(&[GitHub::pushed(SHA), GitHub::first("")])),
        (
            format!("DONE {BRANCH} {SHA}"),
            GitHub::answering(&[GitHub::pushed(SHA), GitHub::first("tools-retirement#D2")]),
        ),
        (format!("DONE {BRANCH} {SHA}"), GitHub::opening(&[])),
        (format!("DONE {BRANCH} {SHA}"), GitHub::opening(&[unviewed])),
    ];
    for (said, github) in &cases {
        let escalated = home.finished(said, false, github);
        assert_eq!((escalated.shown.as_str(), escalated.ended), ("", Ok(Outcome::Escalated)), "{said} {github:?}");
    }
    assert_eq!(home.queue()?, Home::golden("escalated-branch-queue.md")?);
    Ok(())
}

#[test]
fn a_branch_whose_spec_detail_has_no_heading_is_escalated_and_never_opened() -> Result<(), Failed> {
    let home = Home::new()?;
    home.queued("")?;
    let done = format!("DONE {BRANCH} {SHA}");
    let cited = |trailer: &str| GitHub::answering(&[GitHub::pushed(SHA), GitHub::first(trailer)]);
    let unspecified = home.finished(&done, false, &cited("tools-retirement#D2"));
    home.specified("D1 Dropped (D2 later).\nD20 Later work (owner 1).\nD2\n D2 Indented (owner 2).\n")?;
    let cases = [cited("tools-retirement#D2"), cited("tools-retirement#D2, other#D3"), cited("owner 3c84f560")];
    let escalated = cases.iter().map(|github| home.finished(&done, false, github)).collect::<Vec<_>>();
    let first = [unspecified];
    let ended = first.iter().chain(&escalated).map(|said| (said.shown.as_str(), said.ended == Ok(Outcome::Escalated)));
    assert_eq!(ended.collect::<Vec<_>>(), [("", true); 4]);
    assert!(cases.iter().all(|github| github.asked().len() == 2));
    assert_eq!(home.queue()?, Home::golden("escalated-detail-queue.md")?);
    Ok(())
}

#[test]
fn a_subagent_stop_hook_that_cannot_read_its_input_or_write_the_queue_fails() -> Result<(), Failed> {
    let home = Home::new()?;
    let unheard = home.heard(&["hook", "subagent-stop"], "not json", &Replies::default(), &GitHub::default());
    let unqueued = home.finished(&format!("BLOCKED #12 {SHA}"), false, &GitHub::default());
    assert!(unheard.ended.is_err_and(|failure| failure.0.contains("cannot read the hook input: ")));
    assert!(
        unqueued.ended.is_err_and(|failure| failure.0.contains("cannot add the escalation to the discussion queue"))
    );
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
    home.shell_task("b3.output", "building\n\n[killed]")?;
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
fn an_agent_stopped_mid_request_and_quiet_for_twenty_minutes_is_reported_once_as_an_overrun() -> Result<(), Failed> {
    let home = Home::answered()?;
    home.agent_task("agent-hung.jsonl", STOPPED, QUIET)?;
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
fn a_stalled_stop_lists_the_open_queue_items_and_a_live_one_does_not() -> Result<(), Failed> {
    let home = Home::answered()?;
    home.queued(&Home::golden("queue.md")?)?;
    let replies = Replies::default();
    let stalled = home.said(&["hook", "stop"], &home.stop(""), &replies);
    home.shell_task("b1.output", "building\n")?;
    let live = home.said(&["hook", "stop"], &home.stop(""), &replies);
    assert_eq!(stalled.reason()?, Some(Home::golden("stalled-with-queue.txt")?));
    assert_eq!((live.shown.as_str(), live.ended), ("", Ok(Outcome::Hooked(HookEvent::Stop))));
    Ok(())
}

#[test]
fn a_short_owner_reply_is_quoted_with_the_lead_sentence_it_answered() -> Result<(), Failed> {
    let home = Home::holding(&[
        Home::line("assistant", "a1", "I checked CI\\nShall I merge #12 now?"),
        Home::line("user", "a2", "yes"),
        Home::line("assistant", "a3", "Merged."),
        Home::line("user", "a4", "Next, port the hook and keep every quote it reads exactly as it is today."),
        Home::line("assistant", "a5", "Done. Ship it?"),
        Home::line("user", "a6", "sure"),
        Home::line("assistant", "a7", "Anything else?"),
        Home::line("user", "a8", "yes, merge it and then port the hook"),
    ])?;
    let picked =
        [("a2", "yes"), ("a4", "Next, port the hook"), ("a6", "sure"), ("a8", "yes, merge it and then port the hook")]
            .map(|(uuid, quote)| format!("{{\"uuid\": \"{uuid}\", \"quote\": \"{quote}\"}}"))
            .join(", ");
    let replies = Replies::quoting(&format!("[{picked}]"), "[1, 2, 3, 4]");
    let said = home.said(&["hook", "stop"], &home.asking(), &replies);
    let prompts = replies.prompts();
    assert_eq!(said.reason()?, Some(Home::golden("short-reply.txt")?));
    assert!(prompts.first().is_some_and(|prompt| prompt.contains("[a2 2026-01-01T00:00:00Z] yes\n")), "{prompts:?}");
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
