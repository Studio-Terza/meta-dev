use crate::cli::command::Telling;
use console_ports::Shown;
mod build;
mod fortnight;
mod notice;

use std::ffi::OsString;
use std::path::PathBuf;

use super::Step;
use super::issue::{Issue, Title};
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Location};
use crate::cli::ports::{Exit, Invocation, Ports};
use build::Build;
use fortnight::Fortnight;
use grammar_output::Record;
use notice::{Notice, Report};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pace {
    Slower,
    Steady,
}

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub(crate) struct CompileTrend {
    #[field(Location)]
    today: PathBuf,
    #[field(Location, via = Fortnight::downloading_to)]
    fortnight: Fortnight,
}

impl CompileTrend {
    const TITLE: Title<'static> =
        Title::new("compile-time trend: the cold build is slower than every run of the previous 14 days");

    fn concluded<'step>(notice: &Notice<'_>, ports: &impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        match notice.pace() {
            Pace::Slower => Self::filed(&notice.reported(&Fortnight::merged(ports)?), ports),
            Pace::Steady => Ok(Telling::reported(vec![Shown::said(format_args!("{notice}\n"))], Ok(Exit::Success))),
        }
    }

    fn filed<'step>(report: &Report<'_>, ports: &impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let issues = Issue::titled(&[Self::TITLE], ports)?;
        let said = [Shown::said(report)]
            .into_iter()
            .chain(issues.iter().map(|issue| Shown::said(issue.announcement())))
            .collect();
        let filings: Vec<Invocation> = issues.iter().map(|issue| issue.filing(report)).collect();
        Ok(Telling::reported(said, Issue::filed(&filings, ports)))
    }
}

impl Step for CompileTrend {
    fn from_arguments(given: Vec<OsString>) -> Result<CompileTrend, Failure<'static>> {
        CompileTrend::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let today = Build::measured_in(ports.files(), &self.today)?;
        let earlier = self.fortnight.downloaded(ports)?;
        Self::concluded(&Notice::new(&today, &earlier, &self.fortnight), ports)
    }
}
