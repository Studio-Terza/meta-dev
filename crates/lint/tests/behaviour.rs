use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::{ascii, env, iter, mem};

use dispatch_ports::{Call, Clock, Deadline, Shell, SystemClock, SystemShell, Timeout};
use file_doubles::Memory;
use file_ports::{Files, Kind};
use fixture::{Cases, Debugged, Failed, Fixture, Properties, Rows, Spelled};
use lint::{Cli, Verdict};
use port_core::Run;
use proptest::prelude::any;
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;
use serde::Deserialize;
use shapes::{Campaign, Input};
use tempfile::TempDir;
use timer_ports::Stopped;

#[derive(Fixture, Deserialize, Debug)]
struct Sampled {
    drawn: NonZeroUsize,
    kept: NonZeroUsize,
}

#[test]
fn the_fuzz_generator_draws_the_workspaces_its_golden_lists() -> Result<(), Failed> {
    let drifted = Cases::<Sampled>::drifted(|case| {
        let mut runner = TestRunner::new(Properties::settled());
        let pool = iter::repeat_with(|| any::<Input>().new_tree(&mut runner).map(|tree| tree.current()))
            .take(case.row().drawn.get())
            .collect::<Result<Vec<Input>, _>>()?;
        let shapes = pool
            .iter()
            .map(mem::discriminant)
            .fold(Vec::new(), |seen, shape| if seen.contains(&shape) { seen } else { [seen, vec![shape]].concat() });
        let drawn = shapes.iter().flat_map(|shape| {
            pool.iter().filter(move |input| mem::discriminant(*input) == *shape).take(case.row().kept.get())
        });
        let rendered = drawn
            .map(|input| {
                let run = Run::root();
                let files = run.port(input.files());
                let placed = files
                    .entries(&Campaign::root())?
                    .into_iter()
                    .map(|path| match files.kind(&path) {
                        Kind::Directory => files.entries(&path),
                        Kind::File | Kind::Link | Kind::Missing => Ok(vec![path]),
                    })
                    .collect::<Result<Vec<Vec<PathBuf>>, _>>()?
                    .concat();
                placed
                    .into_iter()
                    .map(|path| {
                        let bytes = files.read(&path)?;
                        let text: String =
                            bytes.as_ref().iter().copied().flat_map(ascii::escape_default).map(char::from).collect();
                        Ok((path, text))
                    })
                    .collect::<Result<Vec<(PathBuf, String)>, Failed>>()
            })
            .collect::<Result<Vec<_>, Failed>>()?;
        Ok(Debugged(rendered).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}

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
        let call =
            Call::new(program, arguments, directory.path().to_path_buf(), Deadline::after(clock.now(), binary.timeout));
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
