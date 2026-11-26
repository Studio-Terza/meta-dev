use crate::cli::command::Told;
use console_ports::Shown;
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};

use file_ports::Files;
use text_lines::Lines;

mod boundary;

use super::super::boundary::Exact;
use super::super::kept::{Mutant, Outcomes};
use super::verdict::Verdict;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::journal::Journal;
use crate::cli::command::step::mutation::cargo_mutants::CargoMutants;
use crate::cli::command::step::mutation::shard_summary::outcome::Outcome;
use crate::cli::ports::{Exit, Invocation, Ports, Text};
use boundary::Narrowed;
use strum::VariantArray;

#[derive(Debug)]
pub(super) struct Pass {
    output: PathBuf,
}

#[derive(Debug)]
pub(super) struct Survivors(Vec<Text>);

impl From<Vec<Text>> for Survivors {
    fn from(names: Vec<Text>) -> Survivors {
        Survivors(names)
    }
}

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
    const OUTCOMES_FILE: &'static str = "outcomes.json";
    const LISTED: [&'static str; 4] = ["mutants", "--list", "--json", "--in-diff"];

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

    pub(super) fn timed_out(&self, files: &impl Files) -> Result<Shown, Failure<'static>> {
        let timeouts = self.listed(files, Outcome::Timeout)?;
        Ok(Shown::said(format_args!("mutants: {} timed out on their own crate's tests\n", timeouts.len())))
    }

    pub(super) fn survivors(&self, files: &impl Files) -> Result<Option<Survivors>, Failure<'static>> {
        let missed = self.listed(files, Outcome::Missed);
        Ok(Some(missed?).filter(|names| names.is_empty().then_some(()).is_none()).map(Survivors))
    }

    pub(super) fn judged(
        &self,
        files: &impl Files,
        expected: &Survivors,
    ) -> Result<Vec<Told<'static>>, Failure<'static>> {
        let journals: Vec<Journal> = Outcome::VARIANTS.iter().copied().map(|outcome| self.journal(outcome)).collect();
        let read: Vec<Option<Text>> =
            Journal::read_present(files, journals).into_iter().map(Option::transpose).collect::<Result<_, _>>()?;
        let tested: BTreeSet<Text> = read.iter().flatten().flat_map(Self::names).collect();
        expected.selected(&tested)?;
        let unproven = self.listed(files, Outcome::Missed)?.into_iter().chain(self.listed(files, Outcome::Timeout)?);
        let missed: Vec<String> = unproven.map(|name| [name.as_ref(), "\n"].concat()).collect();
        Ok(Verdict::from(&Text::new(missed.concat())).told())
    }

    pub(super) fn chosen(ports: &impl Ports, diff: &Path, names: &[Text]) -> Result<Text, Failure<'static>> {
        let invocation =
            Invocation::new("cargo", Self::LISTED.map(OsString::from).into_iter().chain([diff.as_os_str().to_owned()]));
        let listed = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let mutants = Mutant::listed(&Text::new(listed.as_ref().to_owned())).map_err(Failure::Outcomes)?;
        mutants.iter().filter(|mutant| names.contains(&mutant.name())).collect::<Narrowed>().diff(ports.files())
    }

    pub(super) fn narrowed(&self, files: &impl Files, survivors: &Survivors) -> Result<Text, Failure<'static>> {
        let path = self.outcomes();
        let read = files.read_text(&path).map_err(|failure| Failure::unusable(path.clone(), failure))?;
        let missed = Outcomes::read(&read).map_err(Failure::Outcomes)?.missed();
        missed.iter().filter(|mutant| survivors.0.contains(&mutant.name())).collect::<Narrowed>().diff(files)
    }

    pub(super) fn outcomes(&self) -> PathBuf {
        self.output.join(Self::OUTCOMES).join(Self::OUTCOMES_FILE)
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
