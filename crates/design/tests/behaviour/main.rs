#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime};
use test_failure::Failed;

use console_ports::{Complaint, Heard, Shown, Variables};
use design::{Agents, Cli, Failure, HookEvent, Outcome};
use dispatch_ports::{
    AgentRun, Bell, Call, ClockFiller, Deadline, DispatchFailure, DispatchFiller, Ended, Errored, Finished, Job,
    Moment, Output, Pause, Pid, Printed, Prompt, Ran, Reply, ShellFiller, Started, Tokens, Told, Usage, Wall,
    clock_ops, dispatch_ops, shell_ops,
};
use file_ports::{Bytes, Disk, FileFailure, Files, Kind, Offset, Size, Stamps, Text};
use grammar::Letters;
use invariants::{Unpoisoned, invariant, invariants};
use newtype::Wrapping;
use port_core::Pass;
use serde::Deserialize;
use serde_json::Value;
use sqlite_ports::SqliteOpener;
use tempfile::TempDir;

use ports::Design;

const OWNER: &str = "Merge only when CI is green.";
const QUESTION: &str = "Shall I merge the branch now?";
const NONE: fn(&Home) -> Result<(), Failed> = |_| Ok(());
const QUIET: Duration = Duration::from_mins(20);
const BUSY: Duration = Duration::from_mins(19);
const ASKED: &str = "{\"type\":\"user\"}\n";
const STOPPED: &str = "{\"type\":\"user\"}\n{\"type\":\"api-request\"}\n";
const GOLDENS: &str = "tests/goldens";
const AGENT: &str = "/work/agent";
const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const LOCAL: &str = "project = \"meta-dev\"\nrepository = \"o/r\"\n";
const SHORT_REPLY: [[&str; 3]; 8] = [
    ["assistant", "a1", "I checked CI\\nShall I merge #12 now?"],
    ["user", "a2", "yes"],
    ["assistant", "a3", "Merged."],
    ["user", "a4", "Next, port the hook and keep every quote it reads exactly as it is today."],
    ["assistant", "a5", "Done. Ship it?"],
    ["user", "a6", "sure"],
    ["assistant", "a7", "Anything else?"],
    ["user", "a8", "yes, merge it and then port the hook"],
];
const TEMPORARY: &str = "/private/tmp";
const TRIMMING: u32 = 1;
const READING: u32 = 2;
const STEP: i64 = 5;
const READS: i64 = 10_000;
const STORE: &str = ".meta/design.sqlite";
const PLACED: &str = "placed";
const MERGEABLE: &str = "pr view 12 --json mergeable -R o/r";

static MIGRATED: LazyLock<Result<Vec<u8>, Failed>> = LazyLock::new(|| {
    let home = Home::new()?;
    home.said(&["stops"], "", &Replies::default());
    Ok(fs::read(home.workspace().join(STORE))?)
});

#[derive(Debug)]
struct Stepping {
    now: SystemTime,
    moments: AtomicI64,
    slept: AtomicU64,
    reads: AtomicI64,
}

impl Stepping {
    fn at(now: SystemTime) -> Stepping {
        Stepping { now, moments: AtomicI64::default(), slept: AtomicU64::default(), reads: AtomicI64::default() }
    }

    fn wall(&self) -> Wall {
        Wall::from(self.now + Duration::from_secs(self.slept.load(Ordering::Relaxed)))
    }
}

invariants! {
    enum Invariant {
        Endless => "a stop reads the fixed test clock fewer than ten thousand times",
    }
}

impl ClockFiller for Stepping {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        invariant!(self.reads.fetch_add(1, Ordering::Relaxed) < READS, Invariant::Endless);
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
    directory: PathBuf,
}

impl Answer {
    fn new(asked: &str, ran: Ran) -> Answer {
        Answer { asked: format!("gh {asked}"), ran }
    }
}

impl GitHub {
    fn answering(given: &[Answer]) -> GitHub {
        let clean = Answer::new(MERGEABLE, Ran::Succeeded(Printed::new("{\"mergeable\":\"MERGEABLE\"}".to_owned())));
        let mut answers = given.iter().fold(BTreeMap::<String, Vec<Ran>>::new(), |mut answers, answer| {
            answers.entry(answer.asked.clone()).or_default().push(answer.ran.clone());
            answers
        });
        answers.entry(clean.asked).or_insert(vec![clean.ran]);
        GitHub { answers, asked: Mutex::default() }
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
        let asked =
            Asked { spoken: spoken.join(" "), deadline: call.deadline(), directory: call.directory().to_path_buf() };
        let mut log = self.github.asked.lock().unpoisoned();
        let earlier = log.iter().filter(|before| before.spoken == asked.spoken).count();
        let answers = self.github.answers.get(&asked.spoken);
        let answer =
            answers.and_then(|rans| rans.get(earlier).or(rans.last())).cloned().ok_or(DispatchFailure::Unjoined);
        log.push(asked);
        answer
    }
}

impl GitHub {
    fn timeline(&self, at: SystemTime) -> String {
        let start = Wall::from(at).into_inner();
        let log = self.asked.lock().unpoisoned();
        let lines: Vec<String> = log
            .iter()
            .map(|asked| {
                let elsewhere = Some(&asked.directory).filter(|directory| *directory != Path::new(AGENT));
                let place = elsewhere.map_or_else(String::new, |directory| format!(" in {}", directory.display()));
                format!("+{} {}{place}\n", asked.deadline.into_inner().into_inner() - start, asked.spoken)
            })
            .collect();
        lines.concat()
    }
}

#[derive(Debug, PartialEq)]
struct Refusal(String);

#[derive(Debug)]
struct Said {
    shown: String,
    ended: Result<Outcome, Refusal>,
    answer: console_ports::Answer,
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
        fs::create_dir_all(home.workspace().join(".meta"))?;
        home.configured(LOCAL)?;
        Ok(home)
    }

    fn configured(&self, local: &str) -> Result<(), Failed> {
        self.unplaced(&format!("queue = {}\nspecs = {}\n{local}", self.placed("queue.md"), self.placed("specs")))
    }

    fn placed(&self, kept: &str) -> Value {
        Value::from(self.path().join(PLACED).join(kept).to_string_lossy())
    }

    fn unplaced(&self, local: &str) -> Result<(), Failed> {
        Ok(fs::write(self.workspace().join(".meta/local.toml"), local)?)
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

    fn used(&self, command: &str, github: &GitHub) -> Said {
        let input = serde_json::json!({
            "session_id": "s",
            "hook_event_name": "PostToolUse",
            "cwd": AGENT,
            "tool_name": "Bash",
            "tool_input": {"command": command},
            "tool_response": {"stdout": ""},
        });
        self.heard(&["hook", "post-tool-use"], &input.to_string(), &Replies::default(), github)
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
        let shown = AsRef::<Shown>::as_ref(&uttered).clone();
        let ended = uttered.ended();
        let answer = console_ports::Answer::reported(ended.as_ref()).showing(&shown);
        Said { shown: shown.to_string(), ended: ended.map_err(|failure: Failure| Refusal(failure.to_string())), answer }
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

#[derive(Clone, Copy, Debug)]
enum Input {
    Nothing,
    Grown,
    Live,
    Stop,
    Asking,
    Text(&'static str),
}

struct Step(&'static [&'static str], Input);

struct Scenario {
    name: &'static str,
    setup: fn(&Home) -> Result<(), Failed>,
    replies: fn() -> Result<Replies, Failed>,
    steps: &'static [Step],
}

const STOP: Step = Step(&["hook", "stop"], Input::Stop);
const ASK: Step = Step(&["hook", "stop"], Input::Asking);
const INGEST: Step = Step(&["ingest"], Input::Nothing);
const STOPS: Step = Step(&["stops"], Input::Nothing);
const GOLDEN: &str = "tests/behaviour/scenarios.txt";

impl Scenario {
    const ALL: [Scenario; 20] = [
        Scenario::of("ingest twice", NONE, &[INGEST, INGEST]),
        Scenario::of("a dry run, then a real one", NONE, &[Step(&["ingest", "--dry-run"], Input::Nothing), INGEST]),
        Scenario::of("an unrecognised line", |home| home.wrote(&Home::line("brand-new-kind", "x1", "odd")), &[INGEST]),
        Scenario::of("hook settings", NONE, &[Step(&["hook", "--settings"], Input::Nothing)]),
        Scenario::of(
            "a continuing stop",
            NONE,
            &[Step(&["hook", "stop"], Input::Text("{\"stop_hook_active\":true,\"session_id\":\"s\"}"))],
        ),
        Scenario::of("nothing running", NONE, &[STOP]),
        Scenario::of(
            "an agent quiet under twenty minutes",
            |home| home.agent_task("agent-busy.jsonl", ASKED, BUSY),
            &[STOP, STOPS],
        ),
        Scenario::of("a shell still running", |home| home.shell_task("b1.output", "building\n"), &[STOP, STOPS]),
        Scenario::of("finished tasks and files that are not tasks", Home::ended_tasks, &[STOP]),
        Scenario::of("no transcript", NONE, &[Step(&["hook", "stop"], Input::Text("{\"stop_hook_active\":false}"))]),
        Scenario::of(
            "an agent quiet for twenty minutes",
            |home| home.agent_task("agent-hung.jsonl", STOPPED, QUIET),
            &[STOP, STOP],
        ),
        Scenario { replies: KEPT, ..Scenario::of("quoted owner words", NONE, &[ASK, STOPS]) },
        Scenario {
            replies: || Ok(Replies::quoting("[{\"uuid\": \"o1\", \"quote\": \"Never said this.\"}]", "[1]")),
            ..Scenario::of("a quote the owner never wrote", NONE, &[ASK])
        },
        Scenario { replies: Replies::refusing, ..Scenario::of("a refused reading", NONE, &[ASK, STOPS]) },
        Scenario { replies: KEPT, ..Scenario::of("a stop without its last message", NONE, &[STOP]) },
        Scenario::of("a transcript that grows", NONE, &[INGEST, Step(&["ingest"], Input::Grown), INGEST]),
        Scenario::of(
            "words the cli does not know",
            NONE,
            &[
                Step(&["frobnicate"], Input::Nothing),
                Step(&["hook"], Input::Nothing),
                Step(&["hook", "stop", "--settings"], Input::Nothing),
                Step(&["ingest", "--wet"], Input::Nothing),
                Step(&[], Input::Nothing),
            ],
        ),
        Scenario::of("an unreadable hook input", NONE, &[Step(&["hook", "stop"], Input::Text("not json"))]),
        Scenario::of(
            "the open queue, stalled and then live",
            |home| home.queued(&Home::golden("queue.md")?),
            &[STOP, Step(&["hook", "stop"], Input::Live)],
        ),
        Scenario {
            replies: || {
                let picked = [
                    ["a2", "yes"],
                    ["a4", "Next, port the hook"],
                    ["a6", "sure"],
                    ["a8", "yes, merge it and then port the hook"],
                ]
                .map(|[uuid, quote]| format!("{{\"uuid\": \"{uuid}\", \"quote\": \"{quote}\"}}"))
                .join(", ");
                Ok(Replies::quoting(&format!("[{picked}]"), "[1, 2, 3, 4]"))
            },
            ..Scenario::of("short owner replies with the lead sentences they answered", Home::short_replies, &[ASK])
        },
    ];

    const fn of(name: &'static str, setup: fn(&Home) -> Result<(), Failed>, steps: &'static [Step]) -> Scenario {
        Scenario { name, setup, replies: || Ok(Replies::default()), steps }
    }

    fn shown(&self) -> Result<String, Failed> {
        let home = Home::answered()?;
        let migrated = MIGRATED.as_ref().map_err(|failed| Failed::from(format!("{failed:?}")))?;
        fs::create_dir_all(home.workspace().join(".meta"))?;
        fs::write(home.workspace().join(STORE), migrated)?;
        let replies = (self.replies)()?;
        (self.setup)(&home)?;
        let ran: Vec<String> = self
            .steps
            .iter()
            .map(|Step(words, input)| {
                let given = match input {
                    Input::Nothing => String::new(),
                    Input::Grown => home.appended(&Home::line("user", "o2", "Ship it.")).map(|()| String::new())?,
                    Input::Live => home.shell_task("b1.output", "building\n").map(|()| home.stop(""))?,
                    Input::Stop => home.stop(""),
                    Input::Asking => home.asking(),
                    Input::Text(text) => (*text).to_owned(),
                };
                let said = home.said(words, &given, &replies);
                let stopped = said.ended == Ok(Outcome::Hooked(HookEvent::Stop));
                Ok(format!("$ {} < {input:?}\n{}{:?} {stopped}\n", words.join(" "), said.shown, said.ended))
            })
            .collect::<Result<_, Failed>>()?;
        let prompts: Vec<String> = replies
            .prompts()
            .iter()
            .map(|prompt| {
                let held = [
                    QUESTION,
                    OWNER,
                    "Understood.",
                    "[o1 ",
                    "1. o1 2026-01-01T00:00:00Z: \"",
                    "[a2 2026-01-01T00:00:00Z] yes\n",
                ];
                format!("prompt holds {:?}\n", held.map(|part| prompt.contains(part)))
            })
            .collect();
        Ok(format!("## {}\n{}{}", self.name, ran.concat(), prompts.concat()))
    }
}

impl Home {
    fn wrote(&self, line: &str) -> Result<(), Failed> {
        self.appended(line)
    }

    fn ended_tasks(&self) -> Result<(), Failed> {
        let ended = "{\"type\":\"assistant\",\"message\":{\"stop_reason\":\"end_turn\"}}\n";
        self.shell_task("b1.output", &["x".repeat(1024), "\n[exited with code 0]\n".to_owned()].concat())?;
        self.shell_task("b2.log", "building\n")?;
        self.shell_task("b3.output", "building\n\n[killed]")?;
        self.agent_task("agent-done.jsonl", &[ASKED, ended].concat(), QUIET)?;
        self.agent_task("agent-done.meta.json", "{}\n", QUIET)
    }

    fn queued(&self, text: &str) -> Result<(), Failed> {
        let folder = self.path().join(PLACED);
        fs::create_dir_all(&folder)?;
        fs::write(folder.join("queue.md"), text)?;
        Ok(())
    }

    fn queue(&self) -> Result<String, Failed> {
        Ok(fs::read_to_string(self.path().join(PLACED).join("queue.md"))?)
    }

    fn specified(&self, text: &str) -> Result<(), Failed> {
        let folder = self.path().join(PLACED).join("specs");
        fs::create_dir_all(&folder)?;
        fs::write(folder.join("tools-retirement.md"), text)?;
        Ok(())
    }

    fn golden(name: &str) -> Result<String, Failed> {
        Ok(fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDENS).join(name))?)
    }

    fn short_replies(&self) -> Result<(), Failed> {
        let lines: String = SHORT_REPLY.iter().map(|[kind, uuid, text]| Home::line(kind, uuid, text)).collect();
        fs::write(self.transcript(), lines)?;
        Ok(())
    }
}

const KEPT: fn() -> Result<Replies, Failed> =
    || Ok(Replies::quoting(&format!("[{{\"uuid\": \"o1\", \"quote\": \"{OWNER}\"}}]"), "[1]"));

#[test]
fn every_scenario_says_what_its_golden_says() -> Result<(), Failed> {
    let shown: String = Scenario::ALL.iter().map(Scenario::shown).collect::<Result<_, Failed>>()?;
    let stories: String = Story::all()?.iter().map(Story::shown).collect::<Result<_, Failed>>()?;
    [(GOLDEN, shown), (STORIES, stories)].into_iter().try_for_each(|(name, said)| {
        let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| fs::write(&golden, &said))?;
        assert_eq!(said, fs::read_to_string(golden)?, "{name}");
        Ok(())
    })
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

const STORIES: &str = "tests/behaviour/subagent-stops.txt";
const SCRIPTS: &str = "tests/behaviour/stories.toml";

#[derive(Debug, Deserialize)]
struct Stories {
    story: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct Story {
    name: String,
    queue: Option<String>,
    spec: Option<String>,
    local: Option<Local>,
    finish: Vec<Finish>,
}

#[derive(Debug, Deserialize)]
enum Local {
    Written(String),
    Removed,
    Unplaced { queue: Option<Placing>, specs: Option<Placing>, rest: String },
}

#[derive(Debug, Deserialize)]
enum Placing {
    Placed,
    Relative(String),
}

#[derive(Debug, Deserialize)]
struct Finish {
    said: String,
    #[serde(default)]
    hook: Hooked,
    #[serde(default)]
    turn: Turn,
    #[serde(default)]
    answers: Vec<Scripted>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
enum Hooked {
    #[default]
    SubagentStop,
    PostToolUse,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
enum Tool {
    #[default]
    Gh,
    Git,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
enum Turn {
    #[default]
    First,
    Reminded,
}

#[derive(Debug, Deserialize)]
struct Scripted {
    #[serde(default)]
    tool: Tool,
    asked: String,
    ran: Answered,
    printed: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
enum Answered {
    Succeeded,
    Failed,
    Overdue,
}

impl Scripted {
    fn answer(&self) -> Answer {
        let printed = Printed::new(self.printed.clone());
        let ran = match self.ran {
            Answered::Succeeded => Ran::Succeeded(printed),
            Answered::Failed => Ran::Failed(printed),
            Answered::Overdue => Ran::Overdue,
        };
        match self.tool {
            Tool::Gh => Answer::new(&self.asked, ran),
            Tool::Git => Answer { asked: format!("git {}", self.asked), ran },
        }
    }
}

impl Placing {
    fn line(&self, key: &str, home: &Home, kept: &str) -> String {
        let value = match self {
            Placing::Placed => home.placed(kept),
            Placing::Relative(path) => Value::from(path.as_str()),
        };
        format!("{key} = {value}\n")
    }
}

impl Story {
    fn all() -> Result<Vec<Story>, Failed> {
        let scripts: Stories =
            toml::from_str(&fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(SCRIPTS))?)?;
        Ok(scripts.story)
    }

    fn shown(&self) -> Result<String, Failed> {
        let home = Home::new()?;
        self.queue.as_deref().map_or(Ok(()), |queued| home.queued(queued))?;
        self.spec.as_deref().map_or(Ok(()), |spec| home.specified(spec))?;
        self.local.as_ref().map_or(Ok(()), |local| match local {
            Local::Written(text) => home.configured(text),
            Local::Removed => home.configured(""),
            Local::Unplaced { queue, specs, rest } => {
                let queued = queue.as_ref().map(|placing| placing.line("queue", &home, "queue.md"));
                let specified = specs.as_ref().map(|placing| placing.line("specs", &home, "specs"));
                home.unplaced(&[queued.unwrap_or_default(), specified.unwrap_or_default(), rest.clone()].concat())
            }
        })?;
        let finished: Vec<String> = self
            .finish
            .iter()
            .map(|finish| {
                let github = GitHub::answering(&finish.answers.iter().map(Scripted::answer).collect::<Vec<_>>());
                let said = match finish.hook {
                    Hooked::SubagentStop => home.finished(&finish.said, finish.turn == Turn::Reminded, &github),
                    Hooked::PostToolUse => home.used(&finish.said, &github),
                };
                let timeline = github.timeline(home.at);
                format!("> {:?} {:?}\n{:?} {}\n{timeline}", finish.said, finish.turn, said.ended, said.shown)
            })
            .collect();
        let queue = match home.queue() {
            Ok(queued) => format!("queue:\n{queued}"),
            Err(_absent) => String::new(),
        };
        Ok(format!("## {}\n{}{queue}", self.name, finished.concat()))
    }
}

mod ports {
    use std::io::Write;
    use std::path::Path;
    use std::process::{Command, Output, Stdio};
    use test_failure::Failed;

    #[derive(Debug)]
    pub struct Design;

    impl Design {
        pub fn fed(home: &Path, arguments: &[&str], input: &str) -> Result<Output, Failed> {
            let mut child = Command::new(env!("CARGO_BIN_EXE_design"))
                .env("HOME", home)
                .current_dir(home)
                .args(arguments)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            child.stdin.take().ok_or("the child's stdin is not piped")?.write_all(input.as_bytes())?;
            Ok(child.wait_with_output()?)
        }
    }
}

#[derive(Debug, PartialEq)]
struct Exited {
    code: ExitCode,
    out: String,
    err: String,
}

impl Said {
    fn exited(&self) -> Exited {
        let shown: &Shown = self.answer.as_ref();
        let complaint: &Complaint = self.answer.as_ref();
        Exited {
            code: self.answer.code(),
            out: AsRef::<str>::as_ref(shown).to_owned(),
            err: AsRef::<str>::as_ref(complaint).to_owned(),
        }
    }
}

#[test]
fn an_unknown_command_exits_with_failure_and_prints_the_usage() -> Result<(), Failed> {
    let exited = Home::new()?.said(&["frobnicate"], "", &Replies::default()).exited();
    let usage = Exited { code: ExitCode::FAILURE, out: String::new(), err: format!("{}\n", Failure::Usage) };
    assert_eq!(exited, usage);
    Ok(())
}

#[test]
fn the_hook_settings_are_printed_as_json_at_exit_zero() -> Result<(), Failed> {
    let exited = Home::new()?.said(&["hook", "--settings"], "", &Replies::default()).exited();
    let settings: Value = serde_json::from_str(&exited.out)?;
    assert_eq!((exited.code, exited.err), (ExitCode::SUCCESS, String::new()));
    assert!(settings.pointer("/hooks/Stop/0/hooks/0/command").and_then(Value::as_str).is_some());
    Ok(())
}

#[test]
fn the_design_binary_wires_the_real_ports_and_reads_its_event_from_standard_input() -> Result<(), Failed> {
    let home = TempDir::new()?;
    let workspace = home.path().join("work");
    let slug = Letters::slugged(&workspace.to_string_lossy());
    fs::create_dir_all(home.path().join(".claude/projects").join(slug))?;
    let rooted = ["--workspace", &workspace.to_string_lossy(), "hook", "stop"].map(str::to_owned);
    let words: Vec<&str> = rooted.iter().map(String::as_str).collect();
    let continuing = Design::fed(home.path(), &words, "{\"stop_hook_active\":true}")?;
    assert_eq!((continuing.status.code(), continuing.stdout, continuing.stderr), (Some(0_i32), Vec::new(), Vec::new()));
    Ok(())
}

#[test]
fn an_unreadable_stop_hook_input_fails_naming_the_hook_input() -> Result<(), Failed> {
    let exited = Home::new()?.said(&["hook", "stop"], "not json", &Replies::default()).exited();
    assert_eq!((exited.code, exited.out), (ExitCode::FAILURE, String::new()));
    assert!(exited.err.contains("cannot read the hook input: "));
    Ok(())
}

#[test]
fn ingest_of_a_workspace_without_transcripts_fails_naming_the_transcripts() -> Result<(), Failed> {
    let exited = Home::new()?.said(&["ingest"], "", &Replies::default()).exited();
    assert_eq!((exited.code, exited.out), (ExitCode::FAILURE, String::new()));
    assert!(exited.err.contains("cannot reach the transcripts: "));
    Ok(())
}
