#![deny(clippy::wildcard_enum_match_arm)]

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};
use test_failure::Failed;

use console_ports::{Complaint, Heard, Shown, Variables};
use design::{Agents, Cli, Failure};
use dispatch_ports::{
    Call, ClockFiller, DispatchFailure, Pause, Printed, Ran, ShellFiller, Wall, clock_ops, shell_ops,
};
use file_ports::{Bytes, Disk, FileFailure, Files, Kind, Offset, Size, Stamps, Text};
use grammar_letters::Letters;
use invariants::{Unpoisoned, invariant, invariants};
use newtype::Wrapping;
use port_core::Pass;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlite_ports::SqliteOpener;
use store_ports::{Opens, Schema};
use tempfile::TempDir;

use ports::Design;

const STORIES: &str = "tests/behaviour/stories.toml";
const GOLDEN: &str = "tests/behaviour/stories.txt";
const AGENT: &str = "/work/agent";
const TEMPORARY: &str = "/private/tmp";
const ROOT: &str = "/home";
const TEMPORARY_REAL: &str = "tmp";
const LOCAL: &str = "queue = \"/home/placed/queue.toml\"\nspecs = \"/home/placed/specs\"\nrepository = \"o/r\"\n";
const READS: i64 = 10_000;

invariants! {
    enum Invariant {
        Endless => "a story step reads the fixed test clock fewer than ten thousand times",
    }
}

#[derive(Debug)]
struct Stepping {
    now: SystemTime,
    slept: AtomicU64,
    reads: AtomicI64,
}

impl Stepping {
    fn wall(&self) -> Wall {
        Wall::from(self.now + Duration::from_secs(self.slept.load(Ordering::Relaxed)))
    }
}

impl ClockFiller for Stepping {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        invariant!(self.reads.fetch_add(1, Ordering::Relaxed) < READS, Invariant::Endless);
        self.wall()
    }

    fn paused(&self, _pass: Pass<'_, clock_ops::Paused>, pause: Pause) -> Wall {
        self.slept.fetch_add(u64::from(pause.into_inner()), Ordering::Relaxed);
        self.wall()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Tool {
    #[default]
    Gh,
    Git,
    Claude,
}

#[derive(Clone, Copy, Debug, Deserialize)]
enum Answered {
    Succeeded,
    Failed,
    Overdue,
}

#[derive(Clone, Debug, Deserialize)]
struct Scripted {
    #[serde(default)]
    tool: Tool,
    asked: String,
    ran: Answered,
    #[serde(default)]
    printed: String,
}

#[derive(Debug)]
struct Script<'story> {
    answers: Vec<&'story Scripted>,
    asked: Mutex<Vec<Asked>>,
    at: SystemTime,
    files: Rerooted,
}

#[derive(Debug)]
struct Asked {
    spoken: String,
    line: String,
}

impl Scripted {
    fn ran(&self) -> Ran {
        let printed = Printed::new(self.printed.clone());
        match self.ran {
            Answered::Succeeded => Ran::Succeeded(printed),
            Answered::Failed => Ran::Failed(printed),
            Answered::Overdue => Ran::Overdue,
        }
    }

    fn answers(&self, call: &Call, spoken: &str) -> Option<()> {
        match self.tool {
            Tool::Gh | Tool::Git => (spoken == format!("{} {}", self.tool.program(), self.asked)).then_some(()),
            Tool::Claude => (call.program() == Tool::Claude.program()
                && call.input().is_some_and(|fed| fed.contains(&self.asked)))
            .then_some(()),
        }
    }
}

impl Tool {
    fn program(self) -> &'static str {
        match self {
            Tool::Gh => "gh",
            Tool::Git => "git",
            Tool::Claude => "claude",
        }
    }
}

impl Script<'_> {
    fn answered(&self, call: &Call) -> Result<Ran, DispatchFailure> {
        let spoken = [call.program().to_owned()]
            .into_iter()
            .chain(call.arguments().iter().cloned())
            .collect::<Vec<_>>()
            .join(" ");
        let start = Wall::from(self.at).into_inner();
        let elsewhere = Some(call.directory()).filter(|directory| *directory != Path::new(AGENT));
        let place = elsewhere.map_or_else(String::new, |directory| format!(" in {}", directory.display()));
        let fed = call.input().map_or_else(String::new, |prompt| format!(" fed:\n{prompt}\n"));
        let line = format!("+{} {spoken}{place}{fed}\n", call.deadline().into_inner().into_inner() - start);
        let mut log = self.asked.lock().unpoisoned();
        let matching: Vec<&&Scripted> =
            self.answers.iter().filter(|answer| answer.answers(call, &spoken).is_some()).collect();
        let key = format!("{spoken}{fed}");
        let earlier = log.iter().filter(|before| before.spoken == key).count();
        log.push(Asked { spoken: key, line });
        let homeless = call.input().filter(|_fed| self.files.kind(call.directory()) == Kind::Missing);
        homeless.map_or(Ok(()), |_fed| Err(DispatchFailure::Spawn(io::Error::from(io::ErrorKind::NotFound))))?;
        matching.get(earlier).or(matching.last()).map(|answer| answer.ran()).ok_or(DispatchFailure::Unjoined)
    }

    fn timeline(&self) -> String {
        let log = self.asked.lock().unpoisoned();
        let shown: Vec<&Asked> = log
            .iter()
            .enumerate()
            .filter(|(at, asked)| log.iter().take(*at).filter(|before| before.spoken == asked.spoken).count() < 2)
            .map(|(_at, asked)| asked)
            .collect();
        let hidden = log.len() - shown.len();
        let lines: String = shown.iter().map(|asked| asked.line.as_str()).collect();
        let repeated = log.last().filter(|_last| hidden > 0).map_or_else(String::new, |last| {
            format!("  ...and {hidden} more calls repeating those above, the last:\n{}", last.line)
        });
        format!("{lines}{repeated}")
    }
}

impl ShellFiller for &Script<'_> {
    fn ran(&self, _pass: Pass<'_, shell_ops::Ran>, call: &Call) -> Result<Ran, DispatchFailure> {
        self.answered(call)
    }

    fn fanned(&self, _pass: Pass<'_, shell_ops::Fanned>, calls: &[Call]) -> Vec<Result<Ran, DispatchFailure>> {
        calls.iter().map(|call| self.answered(call)).collect()
    }
}

#[derive(Clone, Debug)]
struct Rerooted {
    root: PathBuf,
}

#[derive(Debug)]
struct Rehomed(Rerooted);

impl Rerooted {
    fn real(&self, path: &Path) -> PathBuf {
        let temporary = path.strip_prefix(TEMPORARY).map(|below| self.root.join(TEMPORARY_REAL).join(below));
        let homed = path.strip_prefix(ROOT).map(|below| self.root.join(below));
        match (temporary, homed) {
            (Ok(real), _) | (Err(_), Ok(real)) => real,
            (Err(_), Err(_)) => path.to_path_buf(),
        }
    }
}

impl Opens for Rehomed {
    type Store = <SqliteOpener as Opens>::Store;

    fn file<S: Schema>(&self, path: PathBuf) -> Self::Store {
        SqliteOpener.file::<S>(self.0.real(&path))
    }

    fn in_memory<S: Schema>(&self) -> Self::Store {
        SqliteOpener.in_memory::<S>()
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

#[derive(Debug, Deserialize)]
struct Stories {
    story: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct Story {
    name: String,
    local: Option<String>,
    queue: Option<String>,
    spec: Option<String>,
    #[serde(default)]
    transcript: Vec<[String; 3]>,
    #[serde(default)]
    records: Vec<String>,
    #[serde(default)]
    shells: Vec<Task>,
    #[serde(default)]
    agents: Vec<Task>,
    #[serde(default)]
    repositories: Vec<PathBuf>,
    #[serde(default)]
    answers: Vec<Scripted>,
    step: Vec<Step>,
}

#[derive(Debug, Deserialize)]
struct Task {
    file: String,
    text: String,
    #[serde(default)]
    quiet: u64,
}

#[derive(Debug, Deserialize)]
struct Step {
    #[serde(flatten)]
    hooked: Hooked,
    #[serde(default)]
    answers: Vec<Scripted>,
    #[serde(default)]
    append: Vec<[String; 3]>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Hooked {
    Words {
        words: Vec<String>,
        #[serde(default)]
        input: String,
    },
    Stop {
        stop: Stopped,
    },
    Finish {
        finish: String,
        #[serde(default)]
        reminded: Option<Reminded>,
    },
    Ran {
        ran: String,
    },
    Used {
        used: String,
    },
}

#[derive(Debug, Default, Deserialize)]
struct Stopped {
    said: Option<String>,
    #[serde(default)]
    active: Option<Reminded>,
    #[serde(default)]
    untranscribed: Option<Reminded>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
enum Reminded {
    Yes,
}

#[derive(Debug)]
struct Home {
    directory: TempDir,
    at: SystemTime,
}

impl Home {
    fn new() -> Result<Home, Failed> {
        let home = Home { directory: TempDir::new()?, at: SystemTime::now() };
        fs::create_dir_all(home.real(&Home::workspace().join(".meta")))?;
        fs::create_dir_all(home.real(&Home::placed()))?;
        Ok(home)
    }

    fn files(&self) -> Rerooted {
        Rerooted { root: self.directory.path().to_path_buf() }
    }

    fn real(&self, path: &Path) -> PathBuf {
        self.files().real(path)
    }

    fn workspace() -> PathBuf {
        Path::new(ROOT).join("work")
    }

    fn placed() -> PathBuf {
        Path::new(ROOT).join("placed")
    }

    fn slug() -> String {
        Letters::slugged(&Home::workspace().to_string_lossy())
    }

    fn transcript() -> PathBuf {
        Path::new(ROOT).join(".claude/projects").join(Home::slug()).join("s.jsonl")
    }

    fn line([kind, uuid, text]: &[String; 3]) -> String {
        let content = match kind.as_str() {
            "user" => Value::from(text.as_str()),
            _ => json!([{"type": "text", "text": text}]),
        };
        let origin = match kind.as_str() {
            "user" => json!({"kind": "human"}),
            _ => Value::Null,
        };
        let line = json!({"type": kind, "uuid": uuid, "parentUuid": null, "sessionId": "s", "timestamp": "2026-01-01T00:00:00Z", "origin": origin, "message": {"model": "claude", "content": content}});
        format!("{line}\n")
    }

    fn appended(&self, lines: &[String]) -> Result<(), Failed> {
        let transcript = self.real(&Home::transcript());
        fs::create_dir_all(transcript.with_file_name(""))?;
        File::options().create(true).append(true).open(transcript)?.write_all(lines.concat().as_bytes())?;
        Ok(())
    }

    fn set(&self, story: &Story) -> Result<(), Failed> {
        let placed = self.real(&Home::placed());
        fs::write(self.real(&Home::workspace().join(".meta/local.toml")), story.local.as_deref().unwrap_or(LOCAL))?;
        story.queue.as_ref().map_or(Ok(()), |queue| fs::write(placed.join("queue.toml"), queue))?;
        story.spec.as_ref().map_or(Ok(()), |spec| {
            fs::create_dir_all(placed.join("specs"))?;
            fs::write(placed.join("specs/tools-retirement.md"), spec)
        })?;
        let said: Vec<String> = story.transcript.iter().map(Home::line).collect();
        let records: Vec<String> = story.records.iter().map(|record| format!("{record}\n")).collect();
        self.appended(&[said, records].concat())?;
        story.repositories.iter().try_for_each(|repository| fs::create_dir_all(self.real(&repository.join(".git"))))?;
        story.agents.iter().try_for_each(|agent| self.agent_task(agent))?;
        story.shells.iter().try_for_each(|shell| self.shell_task(shell))
    }

    fn agent_task(&self, task: &Task) -> Result<(), Failed> {
        let folder = self.real(&Home::transcript().with_extension("").join("subagents"));
        fs::create_dir_all(&folder)?;
        let written = folder.join(&task.file);
        fs::write(&written, &task.text)?;
        File::options().write(true).open(&written)?.set_modified(self.at - Duration::from_mins(task.quiet))?;
        Ok(())
    }

    fn shell_task(&self, task: &Task) -> Result<(), Failed> {
        let folder = self.real(&Path::new(TEMPORARY).join("claude-501").join(Home::slug()).join("s/tasks"));
        fs::create_dir_all(&folder)?;
        fs::write(folder.join(&task.file), &task.text)?;
        Ok(())
    }

    fn heard(&self, words: &[String], input: &str, script: &Script<'_>) -> String {
        let environment: Variables =
            [(OsString::from("HOME"), OsString::from(ROOT)), (OsString::from("PWD"), OsString::from(ROOT))]
                .into_iter()
                .collect::<Variables>()
                .hearing(Heard::new(input.to_owned()));
        let spoken =
            ["--workspace", "work"].map(str::to_owned).into_iter().chain(words.iter().cloned()).map(OsString::from);
        let clock = Stepping { now: self.at, slept: AtomicU64::default(), reads: AtomicI64::default() };
        let uttered =
            Cli::new(&environment, self.files(), Rehomed(self.files()), Agents::new(clock, script)).rooted(spoken);
        let shown = AsRef::<Shown>::as_ref(&uttered).clone();
        let ended = uttered.ended().map_err(|failure: Failure| failure.to_string());
        let answer = console_ports::Answer::reported(ended.as_ref()).showing(&shown);
        let complaint: &Complaint = answer.as_ref();
        format!("{shown}{ended:?} {:?} {:?}\n", answer.code(), AsRef::<str>::as_ref(complaint))
    }
}

#[derive(Debug)]
struct Spoken {
    words: Vec<String>,
    input: String,
}

impl Spoken {
    fn hook(event: &str, input: &Value) -> Spoken {
        Spoken { words: vec!["hook".to_owned(), event.to_owned()], input: input.to_string() }
    }
}

impl Step {
    fn spoken(&self) -> Spoken {
        match &self.hooked {
            Hooked::Words { words, input } => Spoken { words: words.clone(), input: input.clone() },
            Hooked::Stop { stop } => {
                let transcript = stop
                    .untranscribed
                    .map_or_else(|| Some(Home::transcript().to_string_lossy().into_owned()), |_| None);
                Spoken::hook(
                    "stop",
                    &json!({"session_id": "s", "hook_event_name": "Stop", "stop_hook_active": stop.active.is_some(), "transcript_path": transcript, "last_assistant_message": stop.said}),
                )
            }
            Hooked::Finish { finish, reminded } => Spoken::hook(
                "subagent-stop",
                &json!({"session_id": "s", "hook_event_name": "SubagentStop", "stop_hook_active": reminded.is_some(), "cwd": AGENT, "last_assistant_message": finish}),
            ),
            Hooked::Ran { ran } => Spoken::hook(
                "pre-tool-use",
                &json!({"session_id": "s", "hook_event_name": "PreToolUse", "cwd": AGENT, "tool_name": "Bash", "tool_input": {"command": ran}}),
            ),
            Hooked::Used { used } => Spoken::hook(
                "post-tool-use",
                &json!({"session_id": "s", "hook_event_name": "PostToolUse", "cwd": AGENT, "tool_name": "Bash", "tool_input": {"command": used}, "tool_response": {"stdout": ""}}),
            ),
        }
    }
}

type Told = Result<String, Untold>;

#[derive(Debug, thiserror::Error)]
enum Untold {
    #[error("story {story:?} failed: {failed}")]
    Failed { story: String, failed: String },
    #[error("a story's thread panicked")]
    Panicked,
}

impl Story {
    fn joined(running: Vec<thread::ScopedJoinHandle<'_, Told>>) -> Vec<Told> {
        running
            .into_iter()
            .map(|handle| match handle.join() {
                Ok(told) => told,
                Err(_panicked) => Err(Untold::Panicked),
            })
            .collect()
    }

    fn all() -> Result<Vec<Story>, Failed> {
        let written = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(STORIES))?;
        Ok(toml::from_str::<Stories>(&written)?.story)
    }

    fn shown(&self) -> Result<String, Failed> {
        let home = Home::new()?;
        home.set(self)?;
        let steps: Vec<String> = self
            .step
            .iter()
            .map(|step| {
                home.appended(&step.append.iter().map(Home::line).collect::<Vec<_>>())?;
                let Spoken { words, input } = step.spoken();
                let script = Script {
                    answers: step.answers.iter().chain(&self.answers).collect(),
                    asked: Mutex::default(),
                    at: home.at,
                    files: home.files(),
                };
                let said = home.heard(&words, &input, &script);
                Ok(format!("$ {} < {input}\n{said}{}", words.join(" "), script.timeline()))
            })
            .collect::<Result<_, Failed>>()?;
        let queue = match fs::read_to_string(home.real(&Home::placed().join("queue.toml"))) {
            Ok(queued) => format!("queue:\n{queued}"),
            Err(_absent) => String::new(),
        };
        Ok(format!("## {}\n{}{queue}\n", self.name, steps.concat()))
    }
}

#[test]
fn every_story_says_what_its_golden_says() -> Result<(), Failed> {
    let stories = Story::all()?;
    let told: Vec<Told> = thread::scope(|scope| {
        let running: Vec<thread::ScopedJoinHandle<'_, Told>> = stories
            .iter()
            .map(|story| {
                scope.spawn(move || {
                    story
                        .shown()
                        .map_err(|failed| Untold::Failed { story: story.name.clone(), failed: format!("{failed:?}") })
                })
            })
            .collect();
        Story::joined(running)
    });
    let shown: String = told.into_iter().collect::<Result<_, Untold>>()?;
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDEN);
    std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| fs::write(&golden, &shown))?;
    assert_eq!(shown, fs::read_to_string(golden)?);
    Ok(())
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

#[test]
fn the_design_binary_wires_the_real_ports_and_reads_its_event_from_standard_input() -> Result<(), Failed> {
    let home = TempDir::new()?;
    let workspace = home.path().join("work");
    let slug = Letters::slugged(&workspace.to_string_lossy());
    fs::create_dir_all(home.path().join(".claude/projects").join(slug))?;
    let rooted = ["--workspace", &workspace.to_string_lossy(), "hook", "stop"].map(str::to_owned);
    let words: Vec<&str> = rooted.iter().map(String::as_str).collect();
    let unreadable = Design::fed(home.path(), &words, "not json")?;
    assert_eq!((unreadable.status.code(), unreadable.stdout), (Some(1_i32), Vec::new()));
    assert!(String::from_utf8(unreadable.stderr)?.contains("cannot read the hook input: "));
    Ok(())
}
