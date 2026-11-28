#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use console_doubles::Variables;
use console_ports::{Answer, Heard, Shown};
use design::{Agents, Cli};
use dispatch_doubles::{Answering, Scripted, Stepped};
use dispatch_ports::{Call, Clock, Deadline, Shell, SystemClock, SystemShell, Timeout, Wall, shell_ops};
use file_doubles::Rerooted;
use fixture::{Cases, Debugged, Failed, Fixture, Rows, Spelled};
use port_core::Run;
use serde::Deserialize;
use tempfile::TempDir;

#[derive(Fixture, Deserialize, Debug)]
struct Home {
    moved: BTreeMap<PathBuf, PathBuf>,
    environment: BTreeMap<Spelled, Spelled>,
    arguments: Vec<Spelled>,
    directories: Vec<PathBuf>,
    files: BTreeMap<PathBuf, Spelled>,
    queue: PathBuf,
}

#[derive(Fixture, Deserialize, Debug)]
struct Story {
    #[serde(default)]
    files: BTreeMap<PathBuf, Spelled>,
    #[serde(default)]
    quiet: BTreeMap<PathBuf, Duration>,
    #[serde(default)]
    directories: Vec<PathBuf>,
    #[serde(default)]
    answers: Vec<Scripted>,
    step: Vec<Step>,
}

#[derive(Deserialize, Debug)]
struct Step {
    words: Vec<Spelled>,
    #[serde(default)]
    input: Spelled,
    #[serde(default)]
    append: BTreeMap<PathBuf, Spelled>,
    #[serde(default)]
    answers: Vec<Scripted>,
}

#[derive(Fixture, Deserialize, Debug)]
struct Binary {
    program: Spelled,
    flag: Spelled,
    workspace: PathBuf,
    directories: Vec<PathBuf>,
    words: Vec<Spelled>,
    input: Spelled,
    timeout: Timeout,
}

#[test]
fn every_story_shows_each_step_its_calls_and_the_queue_it_leaves_as_its_golden_says() -> Result<(), Failed> {
    Rows::<Home>::read()?.iter().try_for_each(|home| {
        let drifted = Cases::<Story>::drifted(|case| {
            let story = case.into_row();
            let root = TempDir::new()?;
            let files: Rerooted = home.moved.iter().map(|(from, to)| (from.clone(), root.path().join(to))).collect();
            home.directories
                .iter()
                .chain(&story.directories)
                .try_for_each(|kept| fs::create_dir_all(files.real(kept)))?;
            home.files.iter().chain(&story.files).try_for_each(|(path, text)| {
                let real = files.real(path);
                real.parent().map_or(Ok(()), fs::create_dir_all)?;
                fs::write(real, text.as_str())
            })?;
            let at = SystemTime::now();
            story.quiet.iter().try_for_each(|(path, quiet)| File::open(files.real(path))?.set_modified(at - *quiet))?;
            let steps = story
                .step
                .iter()
                .map(|step| {
                    step.append.iter().try_for_each(|(path, text)| {
                        let real = files.real(path);
                        let mut held = fs::read_to_string(&real)?;
                        held.push_str(text.as_str());
                        fs::write(&real, held)
                    })?;
                    let environment = home
                        .environment
                        .iter()
                        .map(|(name, value)| (OsString::from(name.as_str()), OsString::from(value.as_str())))
                        .collect::<Variables>()
                        .hearing(Heard::new(step.input.to_string()));
                    let answering =
                        Answering::new(step.answers.iter().chain(&story.answers).cloned().collect(), Wall::from(at));
                    let spoken = home.arguments.iter().chain(&step.words).map(|word| OsString::from(word.as_str()));
                    let agents = Agents::new(Stepped::starting(Wall::from(at)), &answering);
                    let uttered = Cli::new(&environment, files.clone(), files.clone(), agents).rooted(spoken);
                    let shown = AsRef::<Shown>::as_ref(&uttered).clone();
                    let ended = uttered.ended().map_err(|failure| failure.to_string());
                    let answer = Answer::reported(ended.as_ref()).showing(shown);
                    Ok((step.words.clone(), ended, answer, answering))
                })
                .collect::<Result<Vec<_>, Failed>>()?;
            let queue = fs::read_to_string(files.real(&home.queue)).map_err(|unread| unread.kind());
            Ok(Debugged((steps, queue)))
        })?;
        assert_eq!(drifted, Vec::new());
        Ok(())
    })
}

#[test]
fn the_design_binary_wires_the_real_ports_and_reads_its_event_from_standard_input() -> Result<(), Failed> {
    let drifted = Cases::<Binary>::drifted(|case| {
        let binary = case.into_row();
        let home = TempDir::new()?;
        binary.directories.iter().try_for_each(|kept| fs::create_dir_all(home.path().join(kept)))?;
        let program: &'static str = Box::leak(env::var(binary.program.as_str())?.into_boxed_str());
        let workspace = home.path().join(&binary.workspace).to_string_lossy().into_owned();
        let arguments =
            [binary.flag.to_string(), workspace].into_iter().chain(binary.words.iter().map(ToString::to_string));
        let run = Run::root();
        let ran = {
            let clock = run.port(SystemClock);
            let shell = run.port(SystemShell);
            let deadline = Deadline::after(clock.now(), binary.timeout);
            let call = Call::new(program, arguments.collect(), home.path().to_path_buf(), deadline);
            shell.ran(&call.fed(binary.input.to_string())).map_err(|failed| failed.to_string())
        };
        Ok(Debugged((ran, run.finish().calls::<shell_ops::Ran>())))
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
