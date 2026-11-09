mod listing;
mod unit;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde_json::Deserializer;

use file_ports::Text;

use super::Step;
use super::journal::Journal;
use super::seconds::Seconds;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
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

    pub(crate) fn per_crate<'path>(
        page: impl Into<Cow<'path, Path>>,
        read: &Text,
    ) -> Result<BTreeMap<OneLine, Seconds>, Failure<'path>> {
        let units: Vec<Unit> = Self::units(page, read)?;
        Ok(units.into_iter().fold(BTreeMap::new(), |totals, unit| unit.tallied(totals)))
    }

    pub(crate) fn units<'path, Timed: DeserializeOwned>(
        page: impl Into<Cow<'path, Path>>,
        read: &Text,
    ) -> Result<Vec<Timed>, Failure<'path>> {
        let text = read.as_ref();
        text.match_indices(Self::UNIT_DATA)
            .next()
            .and_then(|(at, found)| text.get(at + found.len()..))
            .and_then(|rest| Deserializer::from_str(rest).into_iter::<Vec<Timed>>().next())
            .and_then(Result::ok)
            .ok_or_else(|| Failure::NoTimings { path: page.into() })
    }
}

impl Step for CompileTimes {
    fn from_arguments(arguments: Vec<OsString>) -> Result<CompileTimes, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let page = &arguments.taken()?;
        let report = &arguments.taken()?;
        arguments.ended()?;
        Ok(CompileTimes { page: Journal::file(Path::new(page)), report: Journal::file(Path::new(report)) })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let listing = Listing::totalled(Self::per_crate(&self.page, &self.page.read(ports.files())?)?);
        write!(out, "{listing}")?;
        self.report.emptied(ports.files())?.append(ports.files(), &listing).map(|()| Exit::Success)
    }
}
