#![deny(clippy::wildcard_enum_match_arm)]

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};
use test_failure::{Cases, Failed, Golden};

use console_ports::{Complaint, Heard, Shown, Variables};
use design::{Answer, Asked, Cli, Failure};
use dispatch_ports::Wall;
use grammar::letters::Letters;
use newtype::Wrapping;
use serde::Deserialize;
use tempfile::TempDir;

use ports::Design;

const STORIES: &str = "tests/behaviour/stories.toml";
const GOLDEN: &str = "tests/behaviour/stories.txt";
const WORKSPACE: &str = "work";
const HARNESS: &str = "tmp";
const AGENT: &str = "work/agent";
const PLACED: &str = "placed";
const LOCAL: &str = "queue = \"placed/queue.toml\"\nspecs = \"placed/specs\"\nrepository = \"o/r\"\n";

#[derive(Debug, Deserialize)]
struct Stories {
    story: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct Story {
    name: String,
    local: Option<String>,
    queue: Option<String>,
    settings: Option<String>,
    spec: Option<String>,
    #[serde(default)]
    records: Vec<String>,
    #[serde(default)]
    shells: Vec<Task>,
    #[serde(default)]
    agents: Vec<Task>,
    #[serde(default)]
    repositories: Vec<PathBuf>,
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
    words: Vec<String>,
    #[serde(default)]
    input: String,
    #[serde(default)]
    heard: Vec<Answer>,
    #[serde(default)]
    append: Vec<String>,
}

#[derive(Debug)]
struct Home {
    directory: TempDir,
    at: SystemTime,
}

impl Home {
    fn new() -> Result<Home, Failed> {
        let home = Home { directory: TempDir::new()?, at: SystemTime::now() };
        fs::create_dir_all(home.root().join(WORKSPACE).join(".meta"))?;
        fs::create_dir_all(home.root().join(PLACED))?;
        Ok(home)
    }

    fn root(&self) -> &Path {
        self.directory.path()
    }

    fn settings(&self) -> PathBuf {
        self.root().join(WORKSPACE).join(".claude/settings.local.json")
    }

    fn slug(&self) -> String {
        Letters::slugged(&self.root().join(WORKSPACE).to_string_lossy())
    }

    fn transcript(&self) -> PathBuf {
        self.root().join(".claude/projects").join(self.slug()).join("s.jsonl")
    }

    fn appended(&self, records: &[String]) -> Result<(), Failed> {
        let transcript = self.transcript();
        fs::create_dir_all(transcript.with_file_name(""))?;
        let lines: String = records.iter().flat_map(|record| [record.as_str(), "\n"]).collect();
        File::options().create(true).append(true).open(transcript)?.write_all(lines.as_bytes())?;
        Ok(())
    }

    fn set(&self, story: &Story) -> Result<(), Failed> {
        let placed = self.root().join(PLACED);
        let local = self.root().join(WORKSPACE).join(".meta/local.toml");
        fs::write(local, story.local.as_deref().unwrap_or(LOCAL))?;
        story.queue.as_ref().map_or(Ok(()), |queue| fs::write(placed.join("queue.toml"), queue))?;
        story.settings.as_ref().map_or(Ok(()), |settings| {
            let file = self.settings();
            fs::create_dir_all(file.with_file_name(""))?;
            fs::write(file, settings)
        })?;
        story.spec.as_ref().map_or(Ok(()), |spec| {
            fs::create_dir_all(placed.join("specs"))?;
            fs::write(placed.join("specs/tools-retirement.md"), spec)
        })?;
        self.appended(&story.records)?;
        story
            .repositories
            .iter()
            .try_for_each(|repository| fs::create_dir_all(self.root().join(repository).join(".git")))?;
        story.agents.iter().try_for_each(|agent| self.agent_task(agent))?;
        story.shells.iter().try_for_each(|shell| self.shell_task(shell))
    }

    fn agent_task(&self, task: &Task) -> Result<(), Failed> {
        let folder = self.transcript().with_extension("").join("subagents");
        fs::create_dir_all(&folder)?;
        let written = folder.join(&task.file);
        fs::write(&written, &task.text)?;
        File::options().write(true).open(&written)?.set_modified(self.at - Duration::from_mins(task.quiet))?;
        Ok(())
    }

    fn shell_task(&self, task: &Task) -> Result<(), Failed> {
        let folder = self.root().join(HARNESS).join(self.slug()).join("s/tasks");
        fs::create_dir_all(&folder)?;
        fs::write(folder.join(&task.file), &task.text)?;
        Ok(())
    }

    fn heard(&self, step: &Step) -> String {
        let root = OsString::from(self.root());
        let environment: Variables = [(OsString::from("HOME"), root.clone()), (OsString::from("PWD"), root)]
            .into_iter()
            .collect::<Variables>()
            .hearing(Heard::new(step.input.clone()));
        let rooted = ["--workspace", WORKSPACE, "--harness", HARNESS].map(str::to_owned);
        let spoken = rooted.into_iter().chain(step.words.iter().cloned()).map(OsString::from);
        let start = Wall::from(self.at);
        let uttered = Cli::new(&environment).recorded(spoken, start, &step.heard);
        let shown = AsRef::<Shown>::as_ref(&uttered).clone();
        let calls: String = AsRef::<[Asked]>::as_ref(&uttered).iter().map(|asked| self.call(start, asked)).collect();
        let ended = uttered.ended().map_err(|failure: Failure| failure.to_string());
        let answer = console_ports::Answer::reported(ended.as_ref()).showing(&shown);
        let complaint: &Complaint = answer.as_ref();
        format!("{shown}{ended:?} {:?} {:?}\n{calls}", answer.code(), AsRef::<str>::as_ref(complaint))
    }

    fn call(&self, start: Wall, asked: &Asked) -> String {
        match asked {
            Asked::Paused(pause) => format!("pause {}\n", pause.into_inner()),
            Asked::Ran(calls) => calls
                .iter()
                .map(|call| {
                    let spoken = [call.program().to_owned()].into_iter().chain(call.arguments().iter().cloned());
                    let shown = match call.directory().strip_prefix(self.root()) {
                        Ok(relative) => relative,
                        Err(_outside) => call.directory(),
                    };
                    let elsewhere = Some(shown).filter(|directory| *directory != Path::new(AGENT));
                    let place = elsewhere.map_or_else(String::new, |directory| format!(" in {}", directory.display()));
                    let fed = call.input().map_or_else(String::new, |prompt| format!(" fed:\n{prompt}\n"));
                    let after = call.deadline().into_inner().into_inner() - start.into_inner();
                    format!("+{after} {}{place}{fed}\n", spoken.collect::<Vec<_>>().join(" "))
                })
                .collect::<Vec<String>>()
                .concat(),
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
        Ok(Cases::read::<Stories>(&Path::new(env!("CARGO_MANIFEST_DIR")).join(STORIES))?.story)
    }

    fn shown(&self) -> Result<String, Failed> {
        let home = Home::new()?;
        home.set(self)?;
        let steps: Vec<String> = self
            .step
            .iter()
            .map(|step| {
                home.appended(&step.append)?;
                let said = home.heard(step);
                Ok(format!("$ {} < {}\n{said}", step.words.join(" "), step.input))
            })
            .collect::<Result<_, Failed>>()?;
        let queue = match fs::read_to_string(home.root().join(PLACED).join("queue.toml")) {
            Ok(queued) => format!("queue:\n{queued}"),
            Err(_absent) => String::new(),
        };
        let settings = match fs::read_to_string(home.settings()) {
            Ok(written) => format!("settings:\n{written}"),
            Err(_absent) => String::new(),
        };
        Ok(format!("## {}\n{}{queue}{settings}\n", self.name, steps.concat()))
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
    Golden::at(Path::new(env!("CARGO_MANIFEST_DIR")), GOLDEN).checked(&shown)
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
                .env("PWD", home)
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
    let slug = Letters::slugged(&home.path().join(WORKSPACE).to_string_lossy());
    fs::create_dir_all(home.path().join(".claude/projects").join(slug))?;
    let unreadable = Design::fed(home.path(), &["--workspace", WORKSPACE, "hook", "stop"], "not json")?;
    assert_eq!((unreadable.status.code(), unreadable.stdout), (Some(1_i32), Vec::new()));
    assert!(String::from_utf8(unreadable.stderr)?.contains("cannot read the hook input: "));
    Ok(())
}
