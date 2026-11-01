mod listing;
mod unit;

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use serde_json::Deserializer;

use super::Step;
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};
use listing::Listing;
use unit::Unit;

#[derive(Debug)]
pub struct CompileTimes {
    page: Journal,
    report: Journal,
}

impl CompileTimes {
    const UNIT_DATA: &'static str = "const UNIT_DATA = ";

    fn units(&self, page: &str) -> Result<Vec<Unit>, Failure> {
        page.match_indices(Self::UNIT_DATA)
            .next()
            .and_then(|(at, found)| page.get(at + found.len()..))
            .and_then(|rest| Deserializer::from_str(rest).into_iter::<Vec<Unit>>().next())
            .and_then(Result::ok)
            .ok_or_else(|| Failure::NoTimings { path: self.page.as_ref().to_path_buf() })
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
        let listing = Listing::totalled(&self.units(&self.page.read()?)?);
        write!(out, "{listing}")?;
        self.report.emptied()?.append(&listing).map(|()| Exit::Success)
    }
}
