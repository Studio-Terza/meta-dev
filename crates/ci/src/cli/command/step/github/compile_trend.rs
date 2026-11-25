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
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};
use build::Build;
use fortnight::Fortnight;
use notice::{Notice, Report};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pace {
    Slower,
    Steady,
}

#[derive(Debug)]
pub struct CompileTrend {
    today: PathBuf,
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
        let mut arguments = Arguments::from(given);
        let today = &arguments.taken()?;
        let downloads = &arguments.taken()?;
        arguments.ended()?;
        Ok(CompileTrend { today: PathBuf::from(today), fortnight: Fortnight::downloading_to(PathBuf::from(downloads)) })
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let today = Build::measured_in(ports.files(), &self.today)?;
        let earlier = self.fortnight.downloaded(ports)?;
        Self::concluded(&Notice::new(&today, &earlier, &self.fortnight), ports)
    }
}
