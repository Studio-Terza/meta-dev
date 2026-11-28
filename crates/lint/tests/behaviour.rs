use std::env;
use std::path::PathBuf;

use dispatch_ports::{Call, Clock, Deadline, Shell, SystemClock, SystemShell, Timeout};
use file_doubles::Memory;
use fixture::{Cases, Failed, Fixture, Rows, Spelled};
use lint::{Cli, Verdict};
use port_core::Run;
use serde::Deserialize;
use tempfile::TempDir;
use timer_ports::Stopped;

#[derive(Fixture, Deserialize, Debug)]
struct Lint {
    arguments: Vec<PathBuf>,
    verdict: Verdict,
    files: Memory,
    #[serde(default)]
    out: Out,
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
fn the_lint_binary_exits_as_its_verdict_says() -> Result<(), Failed> {
    Rows::<Binary>::read()?.iter().try_for_each(|binary| {
        let directory = TempDir::new()?;
        let run = Run::root();
        let clock = run.port(SystemClock);
        let shell = run.port(SystemShell);
        let arguments = binary.arguments.iter().map(ToString::to_string).collect();
        let program: &'static str = Box::leak(env::var(binary.program.as_str())?.into_boxed_str());
        let call = Call::new(
            program,
            arguments,
            directory.path().to_path_buf(),
            Deadline::after(clock.now(), binary.timeout),
        );
        let exit = shell.ran(&call)?.succeeded().map_or(Exit::Failed, |_printed| Exit::Passed);
        assert_eq!((&binary.arguments, exit), (&binary.arguments, binary.exit));
        Ok(())
    })
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Out {
    #[default]
    Kept,
    Full,
}

#[test]
fn every_case_prints_its_golden_output_and_reaches_its_verdict() -> Result<(), Failed> {
    let drifted = Cases::<Lint>::drifted(|case| {
        let name = case.name().clone();
        let Lint { arguments, verdict: expected, files, out } = case.into_row();
        let cli = Cli::from_args(arguments.into_iter().map(PathBuf::into_os_string));
        let mut kept = Vec::new();
        let mut err = Vec::new();
        let verdict = match out {
            Out::Kept => cli.verdict(files, Stopped::default(), &mut kept, &mut err),
            Out::Full => {
                let mut full: &mut [u8] = &mut [];
                cli.verdict(files, Stopped::default(), &mut full, &mut err)
            }
        };
        assert_eq!((&name, verdict), (&name, expected));
        Ok(String::from_utf8([kept, err].concat())?)
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
