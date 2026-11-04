use lint_core::SourceFile;
use transition_facts::Transitions;
use type_facts::Facts;

use crate::containment::Containment;
use crate::edge::Edge;
use crate::workspace_types::WorkspaceTypes;

#[derive(Debug)]
pub struct Extraction<'file> {
    facts: Facts<'file>,
    transitions: Transitions<'file>,
}

impl<'file> From<&'file [SourceFile]> for Extraction<'file> {
    fn from(files: &'file [SourceFile]) -> Extraction<'file> {
        Extraction { facts: Facts::of_files(files), transitions: Transitions::of_files(files) }
    }
}

impl<'file> Extraction<'file> {
    #[must_use]
    pub fn facts(&self) -> &Facts<'file> {
        &self.facts
    }

    #[must_use]
    pub fn transitions(&self) -> &Transitions<'file> {
        &self.transitions
    }
}

#[derive(Debug)]
pub struct TypeSystem<'found, 'file> {
    types: WorkspaceTypes<'found>,
    edges: Vec<Edge<'found, 'file>>,
    containment: Containment<'found, 'file>,
    transitions: &'found Transitions<'file>,
}

impl<'found, 'file> TypeSystem<'found, 'file> {
    #[must_use]
    pub fn built_from(files: &'found [SourceFile], extraction: &'found Extraction<'file>) -> TypeSystem<'found, 'file> {
        let types = WorkspaceTypes::declared(extraction.facts(), extraction.transitions());
        let edges = extraction.transitions().all().map(|found| types.edge(found)).collect();
        TypeSystem {
            types,
            edges,
            containment: Containment::clustered(files, extraction.facts(), extraction.transitions()),
            transitions: extraction.transitions(),
        }
    }

    #[must_use]
    pub fn containment(&self) -> &Containment<'found, 'file> {
        &self.containment
    }

    pub(crate) fn types(&self) -> &WorkspaceTypes<'found> {
        &self.types
    }

    pub(crate) fn edges(&self) -> &[Edge<'found, 'file>] {
        &self.edges
    }

    pub(crate) fn transitions(&self) -> &'found Transitions<'file> {
        self.transitions
    }
}
