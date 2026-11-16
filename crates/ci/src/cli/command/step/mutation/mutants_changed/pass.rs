use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::num::NonZeroU8;
use std::path::PathBuf;

use file_ports::Files;
use text_lines::Lines;

use super::super::boundary::Exact;
use super::verdict::Verdict;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::journal::Journal;
use crate::cli::command::step::mutation::cargo_mutants::CargoMutants;
use crate::cli::command::step::mutation::shard_summary::outcome::Outcome;
use crate::cli::ports::{Exit, Ports, Text};

#[derive(Debug)]
pub(super) struct Pass {
    output: PathBuf,
}

#[derive(Debug)]
pub(super) struct Survivors(Vec<Text>);

impl Survivors {
    const EXAMINE: &'static str = "--re";

    pub(super) fn examined(&self) -> Vec<OsString> {
        self.0.iter().flat_map(|name| [OsString::from(Self::EXAMINE), OsString::from(Exact::from(name))]).collect()
    }

    fn selected(&self, tested: &BTreeSet<Text>) -> Result<(), Failure<'static>> {
        let untested: Vec<&str> =
            self.0.iter().filter(|name| tested.contains(*name).then_some(()).is_none()).map(AsRef::as_ref).collect();
        untested.first().map_or(Ok(()), |_| Err(Failure::Unselected(Text::new(untested.join("\n")))))
    }
}

impl Pass {
    const OUTCOMES: &'static str = "mutants.out";

    pub(super) fn within(output: PathBuf) -> Pass {
        Pass { output }
    }

    pub(super) fn ran(
        &self,
        ports: &impl Ports,
        selection: &[&OsStr],
        excused: &[NonZeroU8],
    ) -> Result<Exit, Failure<'static>> {
        let invocation = CargoMutants::invocation(&self.output, selection, &[]);
        let run = ports.run(&invocation).map_err(Failure::starting(&invocation))?;
        Ok(run.exit().excusing(excused))
    }

    pub(super) fn survivors(
        &self,
        files: &impl Files,
        out: &mut impl Write,
    ) -> Result<Option<Survivors>, Failure<'static>> {
        let [timeouts, missed] = [Outcome::Timeout, Outcome::Missed].map(|outcome| self.listed(files, outcome));
        writeln!(out, "mutants: {} timed out on their own crate's tests", timeouts?.len())?;
        Ok(Some(missed?).filter(|names| names.is_empty().then_some(()).is_none()).map(Survivors))
    }

    pub(super) fn judged(
        &self,
        files: &impl Files,
        out: &mut impl Write,
        expected: &Survivors,
    ) -> Result<Exit, Failure<'static>> {
        let journals: Vec<Journal> = Outcome::ALL.into_iter().map(|outcome| self.journal(outcome)).collect();
        let read: Vec<Option<Text>> =
            Journal::read_present(files, journals).into_iter().map(Option::transpose).collect::<Result<_, _>>()?;
        let tested: BTreeSet<Text> = read.iter().flatten().flat_map(Self::names).collect();
        expected.selected(&tested)?;
        let missed: Vec<String> =
            self.listed(files, Outcome::Missed)?.iter().map(|name| [name.as_ref(), "\n"].concat()).collect();
        Verdict::from(&Text::new(missed.concat())).reported(out)
    }

    fn listed(&self, files: &impl Files, outcome: Outcome) -> Result<Vec<Text>, Failure<'static>> {
        let read = Journal::read_present(files, vec![self.journal(outcome)]).into_iter().flatten();
        read.map(|text| text.map(|found| Self::names(&found)))
            .collect::<Result<Vec<Vec<Text>>, _>>()
            .map(|lists| lists.concat())
    }

    fn journal(&self, outcome: Outcome) -> Journal {
        Journal::from(self.output.join(Self::OUTCOMES).join(outcome.file_name()))
    }

    fn names(text: &Text) -> Vec<Text> {
        text.as_ref().parse::<Lines>().into_iter().flatten().map(Text::new).collect()
    }
}
