pub(crate) mod fillers;

use std::cell::OnceCell;

use std::collections::BTreeMap;
use std::path::Path;

use code_model::Model;
use law_facts::{Laws, Package};
use lint_core::{Parsed, SourceFile};

use crate::algebra::Algebras;
use crate::containment::Containment;
use crate::declarations::Declarations;
use crate::edge::Edge;
use crate::parallel::ParallelStructures;
use crate::transitions::Transitions;
use crate::workspace_types::WorkspaceTypes;
use fillers::Fillers;

#[derive(Debug, Default)]
pub struct Extraction<'file> {
    fillers: Fillers<'file>,
    packages: BTreeMap<&'file Path, Package<'file>>,
}

impl<'file> From<&'file [SourceFile]> for Extraction<'file> {
    fn from(files: &'file [SourceFile]) -> Extraction<'file> {
        let extraction: Extraction<'file> = files.iter().map(Parsed::from).map(Extraction::from).collect();
        let fillers =
            files.iter().map(|file| Fillers::read(Parsed::from(file), ParallelStructures::EQUIVALENCE)).collect();
        Extraction { fillers, ..extraction }
    }
}

impl<'head> From<Parsed<'_, 'head>> for Extraction<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> Extraction<'head> {
        Extraction {
            fillers: Fillers::default(),
            packages: BTreeMap::from([(parsed.head().path(), Package::from(parsed.head()))]),
        }
    }
}

#[derive(Debug, Default)]
struct Parts<'file> {
    fillers: Vec<Fillers<'file>>,
    packages: BTreeMap<&'file Path, Package<'file>>,
}

impl<'file> FromIterator<Extraction<'file>> for Extraction<'file> {
    fn from_iter<Extractions: IntoIterator<Item = Extraction<'file>>>(parts: Extractions) -> Extraction<'file> {
        let split = parts.into_iter().fold(Parts::default(), |mut split, part| {
            split.fillers.push(part.fillers);
            split.packages.extend(part.packages);
            split
        });
        Extraction { fillers: split.fillers.into_iter().collect(), packages: split.packages }
    }
}

impl<'file> Extraction<'file> {
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
    model: &'file Model,
    containment: OnceCell<Containment<'found, 'file>>,
    algebras: OnceCell<Algebras<'found, 'file>>,
    declarations: OnceCell<Declarations<'found, 'file>>,
    transitions: &'found Transitions<'file>,
}

impl<'found, 'file> TypeSystem<'found, 'file> {
    #[must_use]
    pub fn built_from(
        extraction: &'found Extraction<'file>,
        laws: &'found Laws<'file>,
        transitions: &'found Transitions<'file>,
    ) -> TypeSystem<'found, 'file> {
        let model = transitions.model();
        let types = WorkspaceTypes::declared(model, transitions);
        let edges = transitions.all().map(|found| types.edge(found)).collect();
        TypeSystem {
            types,
            edges,
            laws,
            extraction,
            model,
            containment: OnceCell::new(),
            algebras: OnceCell::new(),
            declarations: OnceCell::new(),
            transitions,
        }
    }

    #[must_use]
    pub fn containment(&self) -> &Containment<'found, 'file> {
        self.containment.get_or_init(|| Containment::clustered(self.model))
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

    #[must_use]
    pub fn model(&self) -> &'file Model {
        self.model
    }

    #[must_use]
    pub fn transitions(&self) -> &'found Transitions<'file> {
        self.transitions
    }
}
