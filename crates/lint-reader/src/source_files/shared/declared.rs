use std::mem;

use lint_layout::Failure;
use lint_roots::Survey;

use super::boundary::Partition;
use crate::source_files::located::Node;
use crate::source_files::modules::{Child, Modules};

#[derive(Clone, Copy, Debug)]
pub(super) enum Kind {
    Module,
    Test,
}

#[derive(Debug)]
pub(super) struct Declared {
    children: Vec<Child>,
    tests: Vec<Child>,
    failures: Vec<Failure>,
}

pub(super) type Explored = Result<Declared, Failure>;

impl Declared {
    pub(super) fn read(survey: &Survey, node: &Node<'_>, items: &[syn::Item]) -> Declared {
        let mut declared = Modules::declared_in(survey, node.located().file(), node.children(), items);
        let tests = declared.tests().to_vec();
        let failures = declared.take_failures();
        Declared { children: declared.take_children(survey), tests, failures }
    }

    pub(super) fn nested<Out>(&self, each: impl Fn(&Child, Kind) -> Out) -> Vec<Out> {
        let modules = self.children.iter().map(|child| each(child, Kind::Module));
        modules.chain(self.tests.iter().map(|child| each(child, Kind::Test))).collect()
    }

    pub(super) fn failures(&mut self) -> Vec<Failure> {
        mem::take(&mut self.failures)
    }

    pub(super) fn into_children(self) -> Vec<Child> {
        self.children
    }
}

impl Kind {
    pub(super) fn owner(self, owned: Option<Partition>, share: Partition) -> Option<Partition> {
        match self {
            Kind::Module => owned,
            Kind::Test => Some(share),
        }
    }
}

impl Kind {
    pub(super) fn tested(self, child: &Child) -> Option<Child> {
        match self {
            Kind::Module => None,
            Kind::Test => Some(child.clone()),
        }
    }
}
