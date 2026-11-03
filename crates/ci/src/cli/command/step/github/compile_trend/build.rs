use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use file_ports::Files;
use text_lines::OneLine;

use crate::cli::command::failure::Failure;
use crate::cli::command::step::CompileTimes;
use crate::cli::command::step::journal::Journal;
use crate::cli::command::step::seconds::Seconds;
use crate::cli::ports::Text;

#[derive(Debug)]
enum Source {
    Measured(Journal),
    Page(Journal),
}

#[derive(Debug)]
enum Part {
    Stages(BTreeMap<OneLine, Seconds>),
    Crates(BTreeMap<OneLine, Seconds>),
}

impl Source {
    fn journals(sources: &[Source]) -> Vec<Journal> {
        sources
            .iter()
            .map(|source| match source {
                Source::Measured(journal) | Source::Page(journal) => journal.clone(),
            })
            .collect()
    }

    fn parsed(&self, text: &Text) -> Result<Part, Failure> {
        match self {
            Source::Measured(measured) => toml::from_str(text.as_ref())
                .map(Part::Stages)
                .map_err(|source| Failure::Measured { path: measured.as_ref().to_path_buf(), source }),
            Source::Page(page) => CompileTimes::per_crate(page, text).map(Part::Crates),
        }
    }
}

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

impl Build {
    pub(super) fn measured_in(files: &impl Files, directory: &Path) -> Result<Build, Failure> {
        let sources = Build::sources(directory);
        Build::tallied(&sources, Journal::read_all(files, &Source::journals(&sources)))
    }

    pub(super) fn measured_in_all(files: &impl Files, directories: &[PathBuf]) -> Result<Vec<Build>, Failure> {
        let sources: Vec<Vec<Source>> = directories.iter().map(|directory| Build::sources(directory)).collect();
        let journals: Vec<Vec<Journal>> = sources.iter().map(|group| Source::journals(group)).collect();
        sources
            .iter()
            .zip(Journal::read_groups(files, &journals))
            .map(|(group, read)| Build::tallied(group, read))
            .collect()
    }

    fn sources(directory: &Path) -> Vec<Source> {
        [Source::Measured(Journal::file(&directory.join(Build::MEASURED)))]
            .into_iter()
            .chain(Build::PAGES.iter().map(|page| Source::Page(Journal::file(&directory.join(page)))))
            .collect()
    }

    fn tallied(sources: &[Source], read: Vec<Result<Text, Failure>>) -> Result<Build, Failure> {
        let parts = sources
            .iter()
            .zip(read)
            .map(|(source, text)| source.parsed(&text?))
            .collect::<Result<Vec<Part>, Failure>>()?;
        Ok(parts.into_iter().fold(Build { total: Seconds::default(), crates: BTreeMap::new() }, Build::adding))
    }

    fn adding(self, part: Part) -> Build {
        match part {
            Part::Stages(stages) => {
                Build { total: [self.total, stages.into_values().sum()].into_iter().sum(), crates: self.crates }
            }
            Part::Crates(crates) => Build {
                total: self.total,
                crates: crates.into_iter().fold(self.crates, |mut all, (name, seconds)| {
                    let total: &mut Seconds = all.entry(name).or_default();
                    *total = [*total, seconds].into_iter().sum();
                    all
                }),
            },
        }
    }
}
