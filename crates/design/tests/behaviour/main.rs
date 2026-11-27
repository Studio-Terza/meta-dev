#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use console_ports::{Answer, Complaint, Heard, Shown, Variables};
use design::{Agents, Arguments, Cli};
use dispatch_ports::{
    AnsweringShell, MemoryClock, Output, Pause, Prompt, Ran, ReplayedDispatch, Spoken, Timeout, Wall,
};
use file_ports::Disk;
use newtype::Wrapping;
use serde::Deserialize;
use sqlite_ports::SqliteOpener;
use tempfile::Builder;
use test_failure::Failed;

#[test]
fn each_scenario_answers_with_its_golden_output() -> Result<(), Failed> {
    let source = Path::new(file!());
    let table = source.with_file_name(source.file_stem().unwrap_or_default());
    let found = std::env::current_dir()?
        .ancestors()
        .map(|directory| directory.join(&table))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let bytes = fs::read(found)?;
    let mut rows = serde_json::Deserializer::from_slice(&bytes);
    let base = PathBuf::deserialize(&mut rows)?;
    let golden = PathBuf::deserialize(&mut rows)?;
    let scenarios = Vec::<(
        PathBuf,
        Variables,
        PathBuf,
        Vec<(Vec<(PathBuf, PathBuf)>, Heard, PathBuf, PathBuf, Pause, Arguments)>,
    )>::deserialize(&mut rows)?;
    fs::create_dir_all(&base)?;
    fs::remove_dir_all(&base)?;
    let scratch = Builder::new()
        .prefix(base.file_name().unwrap_or_default())
        .rand_bytes(usize::default())
        .tempdir_in(base.parent().unwrap_or(&base))?;
    let at = SystemTime::now();
    let answered = scenarios
        .into_iter()
        .map(|(workspace, environment, queue, steps)| {
            fs::create_dir_all(workspace)?;
            let said = steps
                .into_iter()
                .map(|(placed, heard, replies, answers, elapsed, arguments)| {
                    placed.iter().try_for_each(|(origin, target)| {
                        fs::create_dir_all(target.parent().unwrap_or(&base))?;
                        fs::write(target, fs::read(origin)?)
                    })?;
                    let replayed = ReplayedDispatch::replaying(
                        serde_json::from_slice::<Vec<Output>>(&fs::read(replies)?)?.into_iter().map(Some),
                    );
                    let shell = AnsweringShell::answering(serde_json::from_slice::<BTreeMap<Spoken, Vec<Ran>>>(
                        &fs::read(answers)?,
                    )?);
                    let hearing = environment.clone().hearing(heard);
                    let start = Wall::from(at + Duration::from(elapsed));
                    let clock = MemoryClock::starting(start);
                    let agents = Agents::new(|_kept: PathBuf| &replayed, clock, &shell);
                    let uttered = Cli::new(&hearing, Disk, SqliteOpener, agents).rooted(arguments);
                    let printed = AsRef::<Shown>::as_ref(&uttered).clone();
                    let answer = Answer::reported(uttered.ended()).showing(printed);
                    let complaint = AsRef::<str>::as_ref(AsRef::<Complaint>::as_ref(&answer)).to_owned();
                    let asked: Vec<(Spoken, Timeout)> = shell
                        .asked()
                        .iter()
                        .map(|call| {
                            let offset = call.deadline().into_inner().into_inner() - start.into_inner();
                            Ok((Spoken::from(call), Timeout::new(u32::try_from(offset)?)))
                        })
                        .collect::<Result<_, Failed>>()?;
                    let prompts: Vec<Prompt> = replayed.prompts();
                    Ok((AsRef::<Shown>::as_ref(&answer).to_string(), complaint, asked, prompts))
                })
                .collect::<Result<Vec<_>, Failed>>()?;
            let kept: String = fs::read_to_string(queue).into_iter().collect();
            Ok((said, kept))
        })
        .collect::<Result<Vec<_>, Failed>>()?;
    scratch.close()?;
    assert_eq!(answered, serde_json::from_slice::<Vec<_>>(&fs::read(golden)?)?);
    Ok(())
}
