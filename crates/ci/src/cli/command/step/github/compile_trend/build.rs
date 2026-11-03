use std::collections::BTreeMap;
use std::path::Path;

use text_lines::OneLine;

use crate::cli::command::failure::Failure;
use crate::cli::command::step::CompileTimes;
use crate::cli::command::step::journal::Journal;
use crate::cli::command::step::seconds::Seconds;

#[derive(Debug)]
pub struct Build {
    total: Seconds,
    crates: BTreeMap<OneLine, Seconds>,
}

impl Build {
    const MEASURED: &'static str = "measured.toml";
    const PAGES: [&'static str; 2] =
        ["clippy/cargo-timings/cargo-timing.html", "test-compile/cargo-timings/cargo-timing.html"];

    pub(super) fn total(&self) -> Seconds {
        self.total
    }

    pub(super) fn crates(&self) -> &BTreeMap<OneLine, Seconds> {
        &self.crates
    }

    pub(super) fn of_crate(&self, name: &OneLine) -> Option<Seconds> {
        self.crates.get(name).copied()
    }
}

impl TryFrom<&Path> for Build {
    type Error = Failure;

    fn try_from(directory: &Path) -> Result<Build, Failure> {
        let measured = Journal::file(&directory.join(Build::MEASURED));
        let stages: BTreeMap<OneLine, Seconds> = toml::from_str(&measured.read()?)
            .map_err(|source| Failure::Measured { path: measured.as_ref().to_path_buf(), source })?;
        let pages = Build::PAGES
            .iter()
            .map(|page| CompileTimes::per_crate(&Journal::file(&directory.join(page))))
            .collect::<Result<Vec<BTreeMap<OneLine, Seconds>>, Failure>>()?;
        let crates = pages.into_iter().flatten().fold(BTreeMap::new(), |mut crates, (name, seconds)| {
            let total: &mut Seconds = crates.entry(name).or_default();
            *total = [*total, seconds].into_iter().sum();
            crates
        });
        Ok(Build { total: stages.into_values().sum(), crates })
    }
}
