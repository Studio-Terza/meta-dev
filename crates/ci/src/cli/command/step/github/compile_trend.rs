mod build;
mod fortnight;
mod notice;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use super::Step;
use super::issue::{Issue, Title};
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};
use build::Build;
use fortnight::Fortnight;
use notice::{Notice, Report};

#[derive(Debug)]
pub struct CompileTrend {
    today: PathBuf,
    fortnight: Fortnight,
}

impl CompileTrend {
    const TITLE: Title<'static> =
        Title::new("compile-time trend: the cold build is slower than every run of the previous 14 days");

    fn concluded(notice: &Notice<'_>, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'static>> {
        match notice.slower() {
            Some(slower) => Self::filed(&slower.reported(&Fortnight::merged(ports)?), ports, out),
            None => Self::steady(notice, out),
        }
    }

    fn filed(report: &Report<'_>, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'static>> {
        write!(out, "{report}")?;
        let issues = Issue::titled(&[Self::TITLE], ports)?;
        let filings =
            issues.iter().map(|issue| issue.filing(report, out)).collect::<Result<Vec<_>, Failure<'static>>>()?;
        Issue::filed(&filings, ports)
    }

    fn steady(notice: &Notice<'_>, out: &mut impl Write) -> Result<Exit, Failure<'static>> {
        writeln!(out, "{notice}")?;
        Ok(Exit::Success)
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

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let today = Build::measured_in(ports.files(), &self.today)?;
        let earlier = self.fortnight.downloaded(ports)?;
        Self::concluded(&Notice::new(&today, &earlier, &self.fortnight), ports, out)
    }
}
