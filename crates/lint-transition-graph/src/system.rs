pub(crate) mod fillers;

use std::cell::OnceCell;

use std::collections::BTreeMap;
use std::path::Path;

use law_facts::{Laws, Package};
use lint_core::{Parsed, SourceFile};
use lint_item_graph::FileNames;
use transition_facts::Transitions;
use type_facts::Facts;

use crate::algebra::Algebras;
use crate::containment::Containment;
use crate::declarations::Declarations;
use crate::edge::Edge;
use crate::parallel::ParallelStructures;
use crate::workspace_types::WorkspaceTypes;
use fillers::Fillers;

#[derive(Debug, Default)]
pub struct Extraction<'file> {
    facts: Facts<'file>,
    transitions: Transitions<'file>,
    names: FileNames<'file>,
    fillers: Fillers<'file>,
    packages: BTreeMap<&'file Path, Package<'file>>,
}

impl<'file> From<&'file [SourceFile]> for Extraction<'file> {
    fn from(files: &'file [SourceFile]) -> Extraction<'file> {
        files.iter().map(Parsed::from).map(Extraction::from).collect()
    }
}

impl<'head> From<Parsed<'_, 'head>> for Extraction<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> Extraction<'head> {
        Extraction {
            facts: Facts::from(parsed),
            transitions: Transitions::from(parsed),
            names: FileNames::from(parsed),
            fillers: Fillers::read(parsed, ParallelStructures::EQUIVALENCE),
            packages: BTreeMap::from([(parsed.head().path(), Package::from(parsed.head()))]),
        }
    }
}

#[derive(Debug, Default)]
struct Parts<'file> {
    facts: Vec<Facts<'file>>,
    transitions: Vec<Transitions<'file>>,
    names: Vec<FileNames<'file>>,
    fillers: Vec<Fillers<'file>>,
    packages: BTreeMap<&'file Path, Package<'file>>,
}

impl<'file> FromIterator<Extraction<'file>> for Extraction<'file> {
    fn from_iter<Extractions: IntoIterator<Item = Extraction<'file>>>(parts: Extractions) -> Extraction<'file> {
        let split = parts.into_iter().fold(Parts::default(), |mut split, part| {
            split.facts.push(part.facts);
            split.transitions.push(part.transitions);
            split.names.push(part.names);
            split.fillers.push(part.fillers);
            split.packages.extend(part.packages);
            split
        });
        Extraction {
            facts: split.facts.into_iter().collect(),
            transitions: split.transitions.into_iter().collect(),
            names: split.names.into_iter().collect(),
            fillers: split.fillers.into_iter().collect(),
            packages: split.packages,
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

    pub(crate) fn fillers(&self) -> &Fillers<'file> {
        &self.fillers
    }
}

#[derive(Debug)]
pub struct TypeSystem<'found, 'file> {
    types: WorkspaceTypes<'found>,
    edges: Vec<Edge<'found, 'file>>,
    laws: &'found Laws<'file>,
    extraction: &'found Extraction<'file>,
    containment: OnceCell<Containment<'found, 'file>>,
    algebras: OnceCell<Algebras<'found, 'file>>,
    declarations: OnceCell<Declarations<'found, 'file>>,
    transitions: &'found Transitions<'file>,
    facts: &'found Facts<'file>,
}

impl<'found, 'file> TypeSystem<'found, 'file> {
    #[must_use]
    pub fn built_from(extraction: &'found Extraction<'file>, laws: &'found Laws<'file>) -> TypeSystem<'found, 'file> {
        let types = WorkspaceTypes::declared(extraction.facts(), extraction.transitions());
        let edges = extraction.transitions().all().map(|found| types.edge(found)).collect();
        TypeSystem {
            types,
            edges,
            laws,
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
        self.containment.get_or_init(|| Containment::clustered(self.extraction))
    }

    #[must_use]
    pub fn extraction(&self) -> &'found Extraction<'file> {
        self.extraction
    }

    pub(crate) fn algebras(&self) -> &Algebras<'found, 'file> {
        self.algebras.get_or_init(|| Algebras::of_edges(self.edges.iter()))
    }

    pub(crate) fn declarations(&self) -> &Declarations<'found, 'file> {
        self.declarations.get_or_init(|| Declarations::new(self.laws, &self.extraction.packages))
    }

    pub(crate) fn types(&self) -> &WorkspaceTypes<'found> {
        &self.types
    }

    pub(crate) fn edges(&self) -> &[Edge<'found, 'file>] {
        &self.edges
    }

    pub(crate) fn facts(&self) -> &'found Facts<'file> {
        self.facts
    }

    pub(crate) fn transitions(&self) -> &'found Transitions<'file> {
        self.transitions
    }
}
