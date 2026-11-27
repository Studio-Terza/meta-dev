mod boundary;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use console_ports::Shown;
use file_ports::{Bytes, Files};
use text_lines::Lines;

use super::Step;
use crate::cli::command::Telling;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Location, Word};
use crate::cli::ports::{Exit, Invocation, Ports, Text};
use boundary::{Complete, Found, Run};
use grammar_output::Record;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub(crate) struct MutantsPrior {
    #[field(Word)]
    repository: OsString,
    #[field(Word)]
    branch: OsString,
    #[field(Location)]
    directory: PathBuf,
}

impl MutantsPrior {
    const HEAD: [&'static str; 2] = ["rev-parse", "HEAD^2"];
    const TESTED: &'static str = "tested";

    fn looked(&self, ports: &impl Ports) -> Result<Found, Failure<'static>> {
        let heads = Invocation::new("git", Self::HEAD);
        let printed = ports.output(&heads).map_err(Failure::starting(&heads))?;
        let head = Text::new(printed.as_ref().parse::<Lines>().into_iter().flatten().next().unwrap_or_default());
        let listing = Run::listing(&self.repository, &self.branch);
        let listed = ports.output(&listing).map_err(Failure::starting(&listing))?;
        let runs = Run::listed(&listed).map_err(Failure::Runs)?;
        let ancestries: Vec<Invocation> = runs.iter().map(|run| run.ancestry(&head)).collect();
        let answered = ports.outputs(&ancestries);
        let ancestors: Vec<Run> =
            runs.into_iter().zip(answered).filter(|(_, answer)| answer.is_ok()).map(|(run, _)| run).collect();
        let uploads: Vec<Invocation> = ancestors.iter().map(|run| run.uploads(&self.repository)).collect();
        let uploaded = Failure::started(ports.outputs(&uploads), &uploads)?;
        let complete = ancestors
            .into_iter()
            .zip(&uploaded)
            .map(|(run, text)| run.complete(text))
            .find_map(Result::transpose)
            .transpose()
            .map_err(Failure::Runs)?;
        complete.map_or(Ok(Found::None), |found| self.reused(ports, &found))
    }

    fn reused(&self, ports: &impl Ports, complete: &Complete) -> Result<Found, Failure<'static>> {
        let downloads = complete.downloads(&self.repository, &self.directory);
        Failure::started(ports.outputs(&downloads), &downloads)?;
        let paths = complete.tested(&self.directory);
        let files = ports.files();
        let read: Vec<Text> = files
            .read_text_many(&paths)
            .into_iter()
            .zip(&paths)
            .map(|(text, path)| text.map_err(|failure| Failure::unusable(path.clone(), failure)))
            .collect::<Result<_, _>>()?;
        let found = complete.found(&read);
        let marker: &Path = &self.directory.join(Self::TESTED);
        found
            .tested()
            .map(|tested| files.write(marker, Bytes::from(tested.as_ref())))
            .transpose()
            .map_err(|failure| Failure::unusable(marker.to_path_buf(), failure))?;
        Ok(found)
    }
}

impl Step for MutantsPrior {
    fn from_arguments(given: Vec<OsString>) -> Result<MutantsPrior, Failure<'static>> {
        MutantsPrior::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let found = Found::reported(self.looked(ports));
        Ok(Telling::reported(vec![Shown::said(found)], Ok(Exit::Success)))
    }
}
