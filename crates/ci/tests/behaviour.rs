use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs};

use ci::Cli;
use fixture::{Failed, Fixture, Rows, Spelled};
use serde::Deserialize;
use tempfile::TempDir;

#[derive(Fixture, Deserialize, Debug)]
struct Run {
    arguments: Vec<Spelled>,
    #[serde(default)]
    files: BTreeMap<PathBuf, Spelled>,
    exit: Exit,
    #[serde(default)]
    out: Spelled,
    #[serde(default)]
    err: Spelled,
    #[serde(default)]
    written: BTreeMap<PathBuf, Spelled>,
}

#[derive(Deserialize, Clone, Copy, Debug)]
enum Exit {
    Passed,
    Failed,
}

#[test]
fn every_run_exits_prints_and_writes_as_its_row_says() -> Result<(), Failed> {
    Rows::<Run>::read()?.iter().try_for_each(|run| {
        let directory = TempDir::new()?;
        run.files.iter().try_for_each(|(path, text)| fs::write(directory.path().join(path), text.as_str()))?;
        env::set_current_dir(directory.path())?;
        let arguments: Vec<OsString> = run.arguments.iter().map(|argument| OsString::from(argument.as_str())).collect();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = Cli::from_args(&arguments).exit(&mut out, &mut err);
        let printed = String::from_utf8(out)?;
        let complained = String::from_utf8(err)?;
        let written: BTreeMap<PathBuf, String> = run
            .written
            .keys()
            .map(|path| Ok((path.clone(), fs::read_to_string(directory.path().join(path))?)))
            .collect::<Result<_, Failed>>()?;
        let expected = match run.exit {
            Exit::Passed => ExitCode::SUCCESS,
            Exit::Failed => ExitCode::FAILURE,
        };
        let wanted: BTreeMap<PathBuf, String> =
            run.written.iter().map(|(path, text)| (path.clone(), text.as_str().to_owned())).collect();
        assert_eq!(
            (
                &run.arguments,
                code,
                printed.get(..run.out.as_str().len()),
                complained.get(..run.err.as_str().len()),
                written
            ),
            (&run.arguments, expected, Some(run.out.as_str()), Some(run.err.as_str()), wanted)
        );
        Ok(())
    })
}
