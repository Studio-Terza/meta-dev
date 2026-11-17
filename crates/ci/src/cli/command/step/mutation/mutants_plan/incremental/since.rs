use std::ffi::{OsStr, OsString};
use std::path::Path;

use file_ports::{Bytes, Files};
use grammar_hunk::Patch;

use super::super::super::kept::{Commits, Mutant, Places};
use super::super::tested::{Reach, Tested};
use super::boundary::Choice;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports, Text};

const ASKED: usize = 3;

#[derive(Debug)]
pub(super) struct Since<'run> {
    directory: &'run Path,
    reach: &'run Reach,
    commits: &'run Commits,
    diff: &'run Path,
}

impl<'run> Since<'run> {
    const SINCE: &'static str = "since.diff";
    const SELECTED: &'static str = "selected.txt";
    const LISTED: [&'static str; 4] = ["mutants", "--list", "--json", "--in-diff"];

    pub(super) fn new(
        directory: &'run Path,
        reach: &'run Reach,
        commits: &'run Commits,
        diff: &'run Path,
    ) -> Since<'run> {
        Since { directory, reach, commits, diff }
    }

    pub(super) fn chosen(&self, ports: &impl Ports, missed: &[Mutant]) -> Result<Choice, Failure<'static>> {
        let tree = self.commits.tree();
        let fetch = Invocation::new("git", ["fetch", "--no-tags", "--quiet", "origin", tree.as_ref()]);
        ports.output(&fetch).map_or_else(
            |_| Ok(Choice::unfetched(tree)),
            |_| self.named(ports, missed).map(|named| Choice::named(named.as_deref(), missed.len(), self.commits)),
        )
    }

    fn named(&self, ports: &impl Ports, missed: &[Mutant]) -> Result<Option<Vec<Text>>, Failure<'static>> {
        let tree = self.commits.tree().as_ref();
        let since = self.directory.join(Self::SINCE);
        let parent = [tree, "^1"].concat();
        let earlier = Tested::asked(ports, self.reach, OsStr::new(&parent), OsStr::new(tree))?;
        let changed = Tested::asked(ports, self.reach, OsStr::new(tree), OsStr::new(Tested::HEAD))?
            .without(&earlier)
            .diff(ports)?;
        let files = ports.files();
        files
            .write(&since, Bytes::from(changed.as_ref()))
            .map_err(|failure| Failure::unusable(since.clone(), failure))?;
        let asked = [
            Self::listing(self.diff),
            Self::listing(&since),
            Invocation::new("git", ["diff", "-U0", "--no-renames", tree, "HEAD", "--", "*.rs"]),
        ];
        let answers = Failure::started(ports.outputs(&asked), &asked)?;
        let [listed, since_listed, moved] = <[Text; ASKED]>::try_from(answers).map_err(|_unanswered| Failure::Usage)?;
        let mutants = Mutant::listed(&listed).map_err(Failure::Outcomes)?;
        let changed_since = Places::placed(&Mutant::listed(&since_listed).map_err(Failure::Outcomes)?);
        let names = moved
            .as_ref()
            .parse::<Patch>()
            .ok()
            .map(|patch| changed_since.either(&Places::moved(missed, &patch), &mutants));
        names.as_deref().map(|found| self.written(ports, found)).transpose()?;
        Ok(names)
    }

    fn written(&self, ports: &impl Ports, names: &[Text]) -> Result<(), Failure<'static>> {
        let selection = self.directory.join(Self::SELECTED);
        let lines: Vec<&str> = names.iter().map(AsRef::as_ref).collect();
        ports
            .files()
            .write(&selection, Bytes::from(Text::new(lines.join("\n")).as_ref()))
            .map_err(|failure| Failure::unusable(selection, failure))
    }

    fn listing(diff: &Path) -> Invocation {
        Invocation::new("cargo", Self::LISTED.map(OsString::from).into_iter().chain([diff.as_os_str().to_owned()]))
    }
}
