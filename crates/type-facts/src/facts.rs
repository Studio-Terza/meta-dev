mod collector;

use lint_core::SourceFile;

use crate::trait_fact::TraitFact;
use crate::type_fact::TypeFact;
use collector::Collector;

#[derive(Clone, Debug, Default)]
pub struct Facts<'file> {
    types: Vec<TypeFact<'file>>,
    traits: Vec<TraitFact<'file>>,
}

impl<'file> Facts<'file> {
    #[must_use]
    pub fn of_files(files: &'file [SourceFile]) -> Facts<'file> {
        let collected: Vec<Collector<'file>> =
            files.iter().filter_map(SourceFile::shipped).map(Collector::of_file).collect();
        let blocks: Vec<_> = collected.iter().flat_map(|found| found.blocks().iter()).collect();
        Facts {
            types: collected
                .iter()
                .flat_map(|found| found.types().iter().cloned())
                .map(|declared| blocks.iter().fold(declared, |fact, block| block.applied_to(fact)))
                .collect(),
            traits: collected.iter().flat_map(|found| found.traits().iter().cloned()).collect(),
        }
    }

    #[must_use]
    pub fn types(&self) -> &[TypeFact<'file>] {
        &self.types
    }

    #[must_use]
    pub fn declared_once<'found>(&self, found: &'found TypeFact<'file>) -> Option<&'found TypeFact<'file>> {
        (self.types.iter().filter_map(|other| other.named_like(found)).count() == 1).then_some(found)
    }

    #[must_use]
    pub fn traits(&self) -> &[TraitFact<'file>] {
        &self.traits
    }
}
