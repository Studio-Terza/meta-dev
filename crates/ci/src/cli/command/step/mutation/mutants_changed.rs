mod verdict;

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Path, PathBuf};

use file_ports::Files;

use super::super::journal::Journal;
use super::Step;
use super::cargo_mutants::CargoMutants;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports, Text};
use verdict::Verdict;

#[derive(Debug)]
pub struct MutantsChanged {
    diff: Journal,
    missed: Journal,
    output: PathBuf,
    scope: Vec<OsString>,
}

impl MutantsChanged {
    const IN_DIFF: &'static str = "--in-diff";
    const DIFF: [&'static str; 6] = ["diff", "--no-renames", "HEAD^1", "HEAD", "--", "*.rs"];
    const MISSED_LIST: [&'static str; 2] = ["mutants.out", "missed.txt"];

    fn changed_lines(ports: &impl Ports) -> Result<Text, Failure<'static>> {
        let invocation = Invocation::new("git", Self::DIFF);
        ports.output(&invocation).map_err(Failure::starting(&invocation))
    }

    fn new(diff: PathBuf, output: PathBuf, scope: Vec<OsString>) -> MutantsChanged {
        MutantsChanged {
            missed: Journal::from(
                [output.as_path()].into_iter().chain(Self::MISSED_LIST.map(Path::new)).collect::<PathBuf>(),
            ),
            diff: Journal::from(diff),
            output,
            scope,
        }
    }

    fn judged(&self, files: &impl Files, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let missed = self.missed.read_or(files, Text::default())?;
        Verdict::from(&missed).reported(out)
    }

    fn tested(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let changed = Self::changed_lines(ports)?;
        self.diff.replace(ports.files(), &changed)?;
        let diff: &Path = self.diff.as_ref();
        let selection: Vec<&OsStr> = [OsStr::new(Self::IN_DIFF), diff.as_os_str()]
            .into_iter()
            .chain(self.scope.iter().map(OsString::as_os_str))
            .collect();
        let invocation = CargoMutants::invocation(&self.output, &selection, &[]);
        let run = ports.run(&invocation).map_err(Failure::starting(&invocation))?;
        run.exit().excusing(&[CargoMutants::TIMEOUT]).followed_by(|| self.judged(ports.files(), out))
    }
}

impl Step for MutantsChanged {
    fn from_arguments(arguments: Vec<OsString>) -> Result<MutantsChanged, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let diff = &arguments.taken()?;
        let output = &arguments.taken()?;
        let scope = arguments.rest();
        Ok(MutantsChanged::new(PathBuf::from(diff), PathBuf::from(output), scope))
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        match self.scope.first() {
            None => Verdict::Skipped.reported(out),
            Some(_) => self.tested(ports, out),
        }
    }
}
