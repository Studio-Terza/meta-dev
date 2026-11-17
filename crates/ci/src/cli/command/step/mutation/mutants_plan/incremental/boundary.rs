use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component, Path, PathBuf};

use grammar_hunk::Patch;
use serde::Deserialize;
use text_lines::{Lines, OneLine};

use super::super::super::kept::{Commits, Tree};
use super::super::boundary::Shards;
use crate::cli::ports::Text;

#[derive(Debug, Deserialize)]
struct Package {
    id: OneLine,
    manifest_path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Dev,
    Build,
}

#[derive(Debug, Deserialize)]
struct Linkage {
    kind: Option<Kind>,
}

#[derive(Debug, Deserialize)]
struct Dependency {
    pkg: OneLine,
    dep_kinds: Vec<Linkage>,
}

#[derive(Debug, Deserialize)]
struct Node {
    deps: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Metadata {
    packages: Vec<Package>,
    workspace_members: Vec<OneLine>,
    workspace_root: PathBuf,
    resolve: Resolve,
}

#[derive(Debug, Default)]
struct Linked(Vec<Option<Kind>>);

#[derive(Debug)]
pub(super) struct TestCode {
    dev_only: Vec<PathBuf>,
    holding: BTreeSet<String>,
    head: String,
}

#[derive(Debug)]
pub(in super::super) enum Choice {
    Full(Reason),
    Chosen { selected: usize, survivors: usize, head: String },
}

#[derive(Debug)]
pub(in super::super) enum Reason {
    NoPrior,
    TestsEdited(PathBuf),
    Unfetched(String),
    Unreadable,
}

impl Linked {
    fn dev_only(&self) -> Option<()> {
        self.0.iter().all(|kind| *kind == Some(Kind::Dev)).then_some(())
    }
}

impl Metadata {
    pub(super) fn read(text: &Text) -> Result<Metadata, serde_json::Error> {
        serde_json::from_str(text.as_ref())
    }

    fn kinds(&self) -> BTreeMap<&OneLine, Linked> {
        let linked = self
            .resolve
            .nodes
            .iter()
            .flat_map(|node| &node.deps)
            .flat_map(|dependency| dependency.dep_kinds.iter().map(|linkage| (&dependency.pkg, linkage.kind)));
        linked.fold(BTreeMap::new(), |kinds, (package, kind)| {
            let mut grown: BTreeMap<&OneLine, Linked> = kinds;
            grown.entry(package).or_default().0.push(kind);
            grown
        })
    }

    fn dev_only(&self) -> Vec<PathBuf> {
        let kinds = self.kinds();
        self.packages
            .iter()
            .filter(|package| self.workspace_members.contains(&package.id))
            .filter(|package| kinds.get(&package.id).and_then(Linked::dev_only).is_some())
            .filter_map(|package| package.manifest_path.parent())
            .filter_map(|directory| directory.strip_prefix(&self.workspace_root).ok())
            .map(Path::to_path_buf)
            .collect()
    }
}

impl TestCode {
    const TESTS: &'static str = "tests";

    pub(super) fn new(metadata: &Metadata, grepped: &Text, commits: &Commits) -> TestCode {
        TestCode {
            dev_only: metadata.dev_only(),
            holding: grepped.as_ref().parse::<Lines>().into_iter().flatten().collect(),
            head: commits.head().as_ref().to_owned(),
        }
    }

    pub(super) fn edited(&self, patch: &Patch) -> Option<Reason> {
        patch
            .changed()
            .iter()
            .filter_map(|changed| changed.removing())
            .find(|path| self.holds(path).is_some())
            .map(|path| Reason::TestsEdited(path.to_path_buf()))
    }

    fn holds(&self, path: &Path) -> Option<()> {
        let tests = path.components().find(|component| *component == Component::Normal(Self::TESTS.as_ref()));
        let dev = self.dev_only.iter().find(|directory| path.starts_with(directory));
        tests
            .map(|_| ())
            .or(dev.map(|_| ()))
            .or(self.holding.contains(&[self.head.as_str(), ":", &path.to_string_lossy()].concat()).then_some(()))
    }
}

impl Choice {
    pub(super) fn named(selected: Option<&[Text]>, survivors: usize, commits: &Commits) -> Choice {
        selected.map_or(Choice::Full(Reason::Unreadable), |names| Choice::Chosen {
            selected: names.len(),
            survivors,
            head: commits.head().as_ref().to_owned(),
        })
    }

    pub(super) fn unfetched(tree: &Tree) -> Choice {
        Choice::Full(Reason::Unfetched(tree.as_ref().to_owned()))
    }

    pub(in super::super) fn shards(&self) -> Option<Shards> {
        match self {
            Choice::Full(_) => None,
            Choice::Chosen { selected, .. } => Some(Shards::for_mutants(*selected)),
        }
    }
}

impl fmt::Display for Choice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Choice::Full(reason) => writeln!(formatter, "mutants: every changed line, {reason}"),
            Choice::Chosen { selected, survivors, head } => writeln!(
                formatter,
                "mutants: {selected} selected since {head}: its {survivors} survivors and the lines changed since"
            ),
        }
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reason::NoPrior => formatter.write_str("no earlier run's result to start from"),
            Reason::TestsEdited(path) => {
                write!(formatter, "since test code changed other than by additions: {}", path.display())
            }
            Reason::Unfetched(tree) => {
                write!(formatter, "since the tree the earlier run tested ({tree}) cannot be fetched")
            }
            Reason::Unreadable => formatter.write_str("since git printed a patch this cannot read"),
        }
    }
}
