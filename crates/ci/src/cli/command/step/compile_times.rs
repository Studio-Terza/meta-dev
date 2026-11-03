mod listing;
mod unit;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use serde_json::Deserializer;

use super::Step;
use super::journal::Journal;
use super::seconds::Seconds;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};
use listing::Listing;
use text_lines::OneLine;
use unit::Unit;

#[derive(Debug)]
pub struct CompileTimes {
    page: Journal,
    report: Journal,
}

impl CompileTimes {
    const UNIT_DATA: &'static str = "const UNIT_DATA = ";

    pub(crate) fn per_crate(page: &Journal) -> Result<BTreeMap<OneLine, Seconds>, Failure> {
        let read = page.read()?;
        let text = read.as_ref();
        let units: Vec<Unit> = text
            .match_indices(Self::UNIT_DATA)
            .next()
            .and_then(|(at, found)| text.get(at + found.len()..))
            .and_then(|rest| Deserializer::from_str(rest).into_iter::<Vec<Unit>>().next())
            .and_then(Result::ok)
            .ok_or_else(|| Failure::NoTimings { path: page.as_ref().to_path_buf() })?;
        Ok(units.iter().fold(BTreeMap::new(), |mut totals, unit| {
            let total = totals.entry(unit.label().clone()).or_default();
            *total = [*total, unit.took()].into_iter().sum();
            totals
        }))
    }
}

impl Step for CompileTimes {
    fn from_arguments(arguments: &[OsString]) -> Result<CompileTimes, Failure> {
        match arguments {
            [page, report] => {
                Ok(CompileTimes { page: Journal::file(Path::new(page)), report: Journal::file(Path::new(report)) })
            }
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, _ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let listing = Listing::totalled(Self::per_crate(&self.page)?);
        write!(out, "{listing}")?;
        self.report.emptied()?.append(&listing).map(|()| Exit::Success)
    }
}
