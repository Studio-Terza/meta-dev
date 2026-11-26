use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_core::Joins;
use lint_core::{Readout, SourceFile};
use lint_layout::Failure;
use lint_placement::{CrateFile, Packages};
use lint_roots::{Fact, Survey};

use super::boundary::Partition;
use crate::source_files::located::{Located, Node};
use crate::source_files::modules::Sought;

pub trait Reads {
    fn read(self, files: &impl Files) -> Readout;
}

#[derive(Debug)]
pub(super) struct Due<'tree> {
    node: Node<'tree>,
    owner: Option<Partition>,
}

impl<'tree> Due<'tree> {
    pub(super) fn new(node: Node<'tree>, owner: Option<Partition>) -> Due<'tree> {
        Due { node, owner }
    }

    pub(super) fn owner(&self) -> Option<Partition> {
        self.owner
    }

    pub(super) fn assigned(self, hand: Partition) -> Assigned<'tree> {
        Assigned { node: self.node, hand }
    }
}

impl<'tree> AsRef<Located<'tree>> for Due<'tree> {
    fn as_ref(&self) -> &Located<'tree> {
        self.node.located()
    }
}

#[derive(Debug)]
pub(super) struct Assigned<'tree> {
    node: Node<'tree>,
    hand: Partition,
}

impl<'tree> Assigned<'tree> {
    pub(super) fn node(&self) -> &Node<'tree> {
        &self.node
    }

    pub(super) fn hand(&self) -> Partition {
        self.hand
    }
}

#[derive(Debug)]
pub(super) struct Main<'tree> {
    located: Located<'tree>,
    crate_file: CrateFile,
    tests: Vec<Located<'tree>>,
}

impl<'tree> Main<'tree> {
    pub(super) fn new(located: Located<'tree>, crate_file: CrateFile, tests: Vec<Located<'tree>>) -> Main<'tree> {
        Main { located, crate_file, tests }
    }
}

impl<'tree> AsRef<Located<'tree>> for Main<'tree> {
    fn as_ref(&self) -> &Located<'tree> {
        &self.located
    }
}

#[derive(Debug)]
pub(super) struct Assignment<'tree> {
    mains: Vec<Main<'tree>>,
    failures: Vec<Failure>,
}

impl<'tree> Assignment<'tree> {
    pub(super) fn new(mains: Vec<Main<'tree>>, failures: Vec<Failure>) -> Assignment<'tree> {
        Assignment { mains, failures }
    }

    pub(super) fn readout(self, kept: BTreeMap<Located<'tree>, SourceFile>) -> Readout {
        let mut held = kept;
        let files: Vec<SourceFile> = self
            .mains
            .into_iter()
            .filter_map(|main| {
                let file = held.remove(&main.located)?;
                let tests = main.tests.iter().filter_map(|test| held.remove(test)).collect();
                Some(file.in_crate_file(main.crate_file).with_test_modules(tests))
            })
            .collect();
        let mut ordered = files;
        ordered.sort_by(|left, right| AsRef::<Path>::as_ref(left.head()).cmp(AsRef::<Path>::as_ref(right.head())));
        let failed: Readout = self.failures.into_iter().collect();
        ordered.into_iter().collect::<Readout>().combine(failed)
    }
}

#[derive(Debug)]
pub(super) struct Sighting<'tree> {
    located: Located<'tree>,
    sought: Result<Vec<Sought>, Failure>,
}

impl Sighting<'_> {
    pub(super) fn sought(self) -> Result<Vec<Sought>, Failure> {
        self.sought
    }

    pub(super) fn candidates(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.sought.iter().flatten().flat_map(Sought::candidates)
    }
}

impl<'tree> AsRef<Located<'tree>> for Sighting<'tree> {
    fn as_ref(&self) -> &Located<'tree> {
        &self.located
    }
}

#[derive(Debug, Default)]
pub(super) struct Report<'tree> {
    sightings: Vec<Sighting<'tree>>,
    survey: Survey,
}

impl Joins for Report<'_> {
    fn combine(self, other: Self) -> Self {
        Report { sightings: self.sightings.combine(other.sightings), survey: self.survey.combine(other.survey) }
    }
}

impl<'tree> Report<'tree> {
    pub(super) fn concluded<Out>(self, into: impl FnOnce(Vec<Sighting<'tree>>, Survey) -> Out) -> Out {
        into(self.sightings, self.survey)
    }
}

#[derive(Debug, Default)]
pub(super) struct Parsed<'tree> {
    sightings: Vec<Sighting<'tree>>,
    kept: BTreeMap<Located<'tree>, SourceFile>,
}

impl Joins for Parsed<'_> {
    fn combine(self, other: Self) -> Self {
        let mut kept = self.kept;
        kept.extend(other.kept);
        Parsed { sightings: self.sightings.combine(other.sightings), kept }
    }
}

impl<'tree> Parsed<'tree> {
    pub(super) fn read<Out>(
        files: &impl Files,
        packages: &Packages,
        nodes: &[Node<'tree>],
        into: impl FnOnce(Report<'tree>, BTreeMap<Located<'tree>, SourceFile>) -> Out,
    ) -> Out {
        let located: Vec<Located<'tree>> = nodes.iter().map(|node| node.located().clone()).collect();
        let sources = Located::parsed_many(files, packages, &located);
        let parsed: Parsed<'tree> =
            Joins::concat(nodes.iter().zip(sources).map(|(node, source)| Parsed::sighted(node, source)));
        let candidates: Vec<PathBuf> = parsed.sightings.iter().flat_map(Sighting::candidates).collect();
        let survey = Survey::learned(files, Fact::Followed, &candidates);
        into(Report { sightings: parsed.sightings, survey }, parsed.kept)
    }

    fn sighted(node: &Node<'tree>, source: Result<SourceFile, Failure>) -> Parsed<'tree> {
        match source {
            Ok(file) => Parsed {
                sightings: vec![Sighting {
                    located: node.located().clone(),
                    sought: Ok(Sought::within(
                        AsRef::<Path>::as_ref(node.located()),
                        node.children(),
                        &file.syntax().items,
                    )),
                }],
                kept: BTreeMap::from([(node.located().clone(), file)]),
            },
            Err(failure) => Parsed {
                sightings: vec![Sighting { located: node.located().clone(), sought: Err(failure) }],
                kept: BTreeMap::new(),
            },
        }
    }
}
