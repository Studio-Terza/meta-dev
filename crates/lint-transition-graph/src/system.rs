use std::cell::OnceCell;

use lint_core::SourceFile;
use lint_item_graph::FileNames;
use transition_facts::Transitions;
use type_facts::Facts;

use crate::algebra::Algebras;
use crate::containment::Containment;
use crate::declarations::Declarations;
use crate::edge::Edge;
use crate::workspace_types::WorkspaceTypes;

#[derive(Debug)]
pub struct Extraction<'file> {
    facts: Facts<'file>,
    transitions: Transitions<'file>,
    names: FileNames<'file>,
}

impl<'file> From<&'file [SourceFile]> for Extraction<'file> {
    fn from(files: &'file [SourceFile]) -> Extraction<'file> {
        Extraction {
            facts: Facts::of_files(files),
            transitions: Transitions::of_files(files),
            names: FileNames::from(files),
        }
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

    #[must_use]
    pub fn names(&self) -> &FileNames<'file> {
        &self.names
    }
}

#[derive(Debug)]
pub struct TypeSystem<'found, 'file> {
    types: WorkspaceTypes<'found>,
    edges: Vec<Edge<'found, 'file>>,
    files: &'file [SourceFile],
    extraction: &'found Extraction<'file>,
    containment: OnceCell<Containment<'found, 'file>>,
    algebras: OnceCell<Algebras<'found, 'file>>,
    declarations: OnceCell<Declarations<'file>>,
    transitions: &'found Transitions<'file>,
    facts: &'found Facts<'file>,
}

impl<'found, 'file> TypeSystem<'found, 'file> {
    #[must_use]
    pub fn built_from(files: &'file [SourceFile], extraction: &'found Extraction<'file>) -> TypeSystem<'found, 'file> {
        let types = WorkspaceTypes::declared(extraction.facts(), extraction.transitions());
        let edges = extraction.transitions().all().map(|found| types.edge(found)).collect();
        TypeSystem {
            types,
            edges,
            files,
            extraction,
            containment: OnceCell::new(),
            algebras: OnceCell::new(),
            declarations: OnceCell::new(),
            transitions: extraction.transitions(),
            facts: extraction.facts(),
        }
    }

    #[must_use]
    pub fn containment(&self) -> &Containment<'found, 'file> {
        self.containment.get_or_init(|| Containment::clustered(self.files, self.extraction))
    }

    #[must_use]
    pub fn extraction(&self) -> &'found Extraction<'file> {
        self.extraction
    }

    pub(crate) fn algebras(&self) -> &Algebras<'found, 'file> {
        self.algebras.get_or_init(|| Algebras::of_edges(self.edges.iter()))
    }

    pub(crate) fn declarations(&self) -> &Declarations<'file> {
        self.declarations.get_or_init(|| Declarations::from(self.files))
    }

    pub(crate) fn types(&self) -> &WorkspaceTypes<'found> {
        &self.types
    }

    pub(crate) fn edges(&self) -> &[Edge<'found, 'file>] {
        &self.edges
    }

    pub(crate) fn files(&self) -> &'file [SourceFile] {
        self.files
    }

    pub(crate) fn facts(&self) -> &'found Facts<'file> {
        self.facts
    }

    pub(crate) fn transitions(&self) -> &'found Transitions<'file> {
        self.transitions
    }
}
