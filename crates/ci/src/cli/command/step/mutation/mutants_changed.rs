use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::super::journal::Journal;
use super::Step;
use super::cargo_mutants::CargoMutants;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct MutantsChanged {
    diff: Journal,
    output: PathBuf,
    scope: Vec<OsString>,
}

impl MutantsChanged {
    const IN_DIFF: &'static str = "--in-diff";
    const DIFF: [&'static str; 6] = ["diff", "--no-renames", "HEAD^1", "HEAD", "--", "*.rs"];
    const MISSED_LIST: [&'static str; 2] = ["mutants.out", "missed.txt"];

    fn changed_lines(ports: &impl Ports) -> Result<String, Failure> {
        let invocation = Invocation::new("git", Self::DIFF);
        ports.output(&invocation).map_err(Failure::starting(&invocation))
    }

    fn missed_list(&self) -> Journal {
        Journal::file(&Self::MISSED_LIST.iter().fold(self.output.clone(), |path, part| path.join(part)))
    }

    fn judged(&self, out: &mut impl Write) -> Result<Exit, Failure> {
        let list = self.missed_list();
        let missed = if list.exists() { list.read()? } else { String::new() };
        if missed.is_empty() {
            writeln!(out, "mutants: none missed")?;
            Ok(Exit::Success)
        } else {
            write!(out, "mutants missed:\n{missed}")?;
            Ok(Exit::Failure(CargoMutants::MISSED))
        }
    }

    fn tested(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let changed = Self::changed_lines(ports)?;
        self.diff.replace(&changed)?;
        let diff: &Path = self.diff.as_ref();
        let selection: Vec<&OsStr> = [OsStr::new(Self::IN_DIFF), diff.as_os_str()]
            .into_iter()
            .chain(self.scope.iter().map(OsString::as_os_str))
            .collect();
        let invocation = CargoMutants::invocation(&self.output, &selection, &[]);
        let run = ports.run(&invocation).map_err(Failure::starting(&invocation))?;
        run.exit().excusing(&[CargoMutants::TIMEOUT]).followed_by(|| self.judged(out))
    }
}

impl Step for MutantsChanged {
    fn from_arguments(arguments: &[OsString]) -> Result<MutantsChanged, Failure> {
        match arguments {
            [diff, output, scope @ ..] => Ok(MutantsChanged {
                diff: Journal::file(Path::new(diff)),
                output: PathBuf::from(output),
                scope: scope.to_vec(),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        if self.scope.is_empty() {
            writeln!(out, "mutants: skipped, no crate is reached by this change, so nothing was tested")?;
            Ok(Exit::Success)
        } else {
            self.tested(ports, out)
        }
    }
}
