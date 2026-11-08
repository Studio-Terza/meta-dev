use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files};
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
    fn parsed(self, read: Result<Text, FileFailure>) -> Result<Part, Failure<'static>> {
        match read {
            Ok(text) => self.part(&text),
            Err(failure) => Err(Failure::unusable(self.into_journal(), failure)),
        }
    }

    fn part(self, text: &Text) -> Result<Part, Failure<'static>> {
        match self {
            Source::Measured(measured) => toml::from_str(text.as_ref())
                .map(Part::Stages)
                .map_err(|source| Failure::Measured { path: measured.into(), source }),
            Source::Page(page) => CompileTimes::per_crate(page, text).map(Part::Crates),
        }
    }

    fn into_journal(self) -> Journal {
        match self {
            Source::Measured(journal) | Source::Page(journal) => journal,
        }
    }
}

impl AsRef<Path> for Source {
    fn as_ref(&self) -> &Path {
        match self {
            Source::Measured(journal) | Source::Page(journal) => journal.as_ref(),
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
    pub(super) fn measured_in(files: &impl Files, directory: &Path) -> Result<Build, Failure<'static>> {
        let sources = Build::sources(directory);
        let read = files.read_text_many(&sources);
        Build::tallied(sources, read)
    }

    pub(super) fn measured_in_all(files: &impl Files, directories: &[PathBuf]) -> Result<Vec<Build>, Failure<'static>> {
        let sources: Vec<Vec<Source>> = directories.iter().map(|directory| Build::sources(directory)).collect();
        let read = files.read_text_many(&sources.iter().flatten().collect::<Vec<&Source>>());
        sources
            .into_iter()
            .scan(read.into_iter(), |unread, group| {
                let size = group.len();
                Some(Build::tallied(group, unread.take(size).collect()))
            })
            .collect()
    }

    fn sources(directory: &Path) -> Vec<Source> {
        [Source::Measured(Journal::file(&directory.join(Build::MEASURED)))]
            .into_iter()
            .chain(Build::PAGES.iter().map(|page| Source::Page(Journal::file(&directory.join(page)))))
            .collect()
    }

    fn tallied(sources: Vec<Source>, read: Vec<Result<Text, FileFailure>>) -> Result<Build, Failure<'static>> {
        let parts = sources
            .into_iter()
            .zip(read)
            .map(|(source, text)| source.parsed(text))
            .collect::<Result<Vec<Part>, Failure<'static>>>()?;
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
