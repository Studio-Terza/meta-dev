#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use console_ports::{Answer, Complaint, Heard, Shown, Variables};
use design::{Agents, Cli};
use dispatch_ports::{AnsweringShell, MemoryClock, Output, Ran, ReplayedDispatch, Spoken, Stderr, Wall};
use file_ports::Disk;
use grammar_slug::Letters;
use serde_json::Value;
use sqlite_ports::SqliteOpener;
use tempfile::TempDir;
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
    let (header, steps): (Vec<Vec<String>>, Vec<Vec<String>>) = serde_json::from_slice(&fs::read(found)?)?;
    let [variables, layout, kept, opening, closing] = header.as_slice() else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let (
        [home_variable, pwd_variable, flag, work],
        [projects, placeholder, temporary, transcript_name],
        [home_base, tmp_base, queue_path, golden],
        [scenario_mark, step_mark, shown_mark, complaint_mark],
        [asked_mark, prompt_mark, queue_mark, newline],
    ) = (variables.as_slice(), layout.as_slice(), kept.as_slice(), opening.as_slice(), closing.as_slice())
    else {
        return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
    };
    let at = SystemTime::now();
    let shown = steps
        .chunk_by(|first, second| first.first() == second.first())
        .map(|scenario| {
            let (home, tmp) = (TempDir::new()?, TempDir::new_in(temporary)?);
            let workspace = home.path().join(work);
            fs::create_dir_all(&workspace)?;
            let slug = OsString::from(Letters::slugged(&workspace.to_string_lossy()));
            let transcript = home.path().join(projects).join(&slug).join(transcript_name);
            let roots = [(home_base, home.path()), (tmp_base, tmp.path())];
            let stepped = scenario.iter().map(|step| {
                let [_, files, stdin, replies, answers, transcript_key, elapsed, words @ ..] = step.as_slice() else {
                    return Err(Failed::from(io::Error::from(io::ErrorKind::InvalidData)));
                };
                let placed: Vec<(String, String, String)> = serde_json::from_slice(&fs::read(files)?)?;
                placed.iter().try_for_each(|(origin, base, destination)| {
                    let (_, root) = roots.iter().find(|(named, _)| *named == base).ok_or(io::ErrorKind::InvalidData)?;
                    let parts = Path::new(destination).components().map(|part| part.as_os_str().to_owned());
                    let target: PathBuf = [root.as_os_str().to_owned()]
                        .into_iter()
                        .chain(parts.map(|part| if part == OsStr::new(placeholder) { slug.clone() } else { part }))
                        .collect();
                    fs::create_dir_all(target.parent().unwrap_or(root))?;
                    fs::write(&target, fs::read(origin)?)
                })?;
                let mut heard = fs::read_to_string(stdin)?;
                if let Some(Value::Object(mut fields)) = Some(transcript_key)
                    .filter(|key| !key.is_empty())
                    .map(|_| serde_json::from_str(&heard))
                    .transpose()?
                {
                    fields.insert(transcript_key.clone(), Value::from(transcript.to_string_lossy().into_owned()));
                    heard = Value::Object(fields).to_string();
                }
                let envelopes: Vec<String> = serde_json::from_slice(&fs::read(replies)?)?;
                let replayed =
                    ReplayedDispatch::replaying(envelopes.iter().map(|text| Some(Output::read(text, Stderr::Silent))));
                let shell =
                    AnsweringShell::answering(serde_json::from_slice::<BTreeMap<Spoken, Ran>>(&fs::read(answers)?)?);
                let environment: Variables = [home_variable, pwd_variable]
                    .map(|variable| (OsString::from(variable), home.path().as_os_str().to_owned()))
                    .into_iter()
                    .collect::<Variables>()
                    .hearing(Heard::new(heard));
                let arguments = [flag, work].into_iter().chain(words).map(OsString::from);
                let clock = MemoryClock::starting(Wall::from(at + Duration::from_secs(elapsed.parse()?)));
                let agents = Agents::new(|_kept: PathBuf| &replayed, clock, &shell);
                let uttered = Cli::new(&environment, Disk, SqliteOpener, agents).rooted(arguments);
                let printed = AsRef::<Shown>::as_ref(&uttered).clone();
                let answer = Answer::reported(uttered.ended()).showing(printed);
                let asked =
                    shell.asked().into_iter().map(|call| [Spoken::from(&call).to_string(), newline.clone()].concat());
                let prompts = replayed.prompts().into_iter().map(|prompt| String::from_utf8(prompt.as_ref().to_vec()));
                let prompted =
                    prompts.map(|text| text.map(|asking| [prompt_mark.clone(), asking, newline.clone()].concat()));
                let complaint = AsRef::<str>::as_ref(AsRef::<Complaint>::as_ref(&answer)).to_owned();
                let heading = [step_mark.clone(), serde_json::to_string(words)?, newline.clone()];
                let answered =
                    [shown_mark.clone(), AsRef::<Shown>::as_ref(&answer).to_string(), complaint_mark.clone()];
                Ok(heading
                    .into_iter()
                    .chain(answered)
                    .chain([complaint, asked_mark.clone()])
                    .chain(asked)
                    .chain(prompted.collect::<Result<Vec<String>, _>>()?)
                    .collect::<String>())
            });
            let said = stepped.collect::<Result<String, Failed>>()?;
            let queue: String = fs::read_to_string(home.path().join(queue_path)).into_iter().collect();
            let name = scenario.first().and_then(|step| step.first()).cloned().unwrap_or_default();
            Ok([scenario_mark.clone(), name, newline.clone(), said, queue_mark.clone(), queue].concat())
        })
        .collect::<Result<String, Failed>>()?;
    assert_eq!(shown, fs::read_to_string(golden)?);
    Ok(())
}
