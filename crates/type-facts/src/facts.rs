mod collector;

use lint_core::SourceFile;

use crate::trait_fact::TraitFact;
use crate::type_fact::TypeFact;
use collector::Collector;

#[derive(Clone, Debug, Default)]
pub struct Facts {
    types: Vec<TypeFact>,
    traits: Vec<TraitFact>,
}

impl Facts {
    #[must_use]
    pub fn of_files(files: &[SourceFile]) -> Facts {
        let collected: Vec<Collector<'_>> =
            files.iter().filter(|file| file.is_shipped()).map(Collector::of_file).collect();
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
    pub fn types(&self) -> &[TypeFact] {
        &self.types
    }

    #[must_use]
    pub fn traits(&self) -> &[TraitFact] {
        &self.traits
    }
}
