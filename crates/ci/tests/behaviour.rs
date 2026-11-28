use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs};

use ci::Cli;
use dispatch_ports::{Call, Clock, Deadline, Shell, SystemClock, SystemShell, Timeout};
use fixture::{Failed, Fixture, Rows, Spelled};
use serde::Deserialize;
use tempfile::TempDir;

#[derive(Fixture, Deserialize, Debug)]
struct Run {
    arguments: Vec<Spelled>,
    #[serde(default)]
    directories: Vec<PathBuf>,
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

#[derive(Fixture, Deserialize, Debug)]
struct Binary {
    program: Spelled,
    arguments: Vec<Spelled>,
    exit: Exit,
    timeout: Timeout,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Exit {
    Passed,
    Failed,
}

#[test]
fn the_ci_binary_exits_as_its_command_line_does() -> Result<(), Failed> {
    Rows::<Binary>::read()?.iter().try_for_each(|binary| {
        let directory = TempDir::new()?;
        let run = port_core::Run::root();
        let clock = run.port(SystemClock);
        let shell = run.port(SystemShell);
        let arguments = binary.arguments.iter().map(ToString::to_string).collect();
        let program: &'static str = Box::leak(env::var(binary.program.as_str())?.into_boxed_str());
        let call =
            Call::new(program, arguments, directory.path().to_path_buf(), Deadline::after(clock.now(), binary.timeout));
        let exit = shell.ran(&call)?.succeeded().map_or(Exit::Failed, |_printed| Exit::Passed);
        assert_eq!((&binary.arguments, exit), (&binary.arguments, binary.exit));
        Ok(())
    })
}

#[test]
fn every_run_exits_prints_and_writes_as_its_row_says() -> Result<(), Failed> {
    Rows::<Run>::read()?.iter().try_for_each(|run| {
        let directory = TempDir::new()?;
        run.directories.iter().try_for_each(|made| fs::create_dir_all(directory.path().join(made)))?;
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
