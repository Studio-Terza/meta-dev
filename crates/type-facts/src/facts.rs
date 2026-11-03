mod collector;
mod members;

use std::collections::BTreeMap;
use std::mem;

use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::implementation::Implementation;
use crate::method::Method;
use crate::trait_fact::TraitFact;
use crate::type_fact::TypeFact;
use collector::{Collector, ImplBlock};
pub(crate) use members::Members;

#[derive(Clone, Debug, Default)]
pub struct Facts<'file> {
    types: Vec<TypeFact<'file>>,
    traits: Vec<TraitFact<'file>>,
    members: BTreeMap<&'file CrateName, BTreeMap<Name, Members>>,
}

impl<'file> Facts<'file> {
    #[must_use]
    pub fn of_files(files: &'file [SourceFile]) -> Facts<'file> {
        files.iter().filter_map(SourceFile::shipped).fold(Facts::default(), Collector::collected)
    }

    pub(crate) fn add_type(&mut self, found: Option<TypeFact<'file>>) {
        self.types.extend(found);
    }

    pub(crate) fn add_trait(&mut self, found: Option<TraitFact<'file>>) {
        self.traits.extend(found);
    }

    pub(crate) fn add_block(&mut self, found: Option<ImplBlock<'file>>) {
        self.members = found.into_iter().fold(mem::take(&mut self.members), |members, block| block.added_to(members));
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

    #[must_use]
    pub fn methods(&self, found: &TypeFact<'file>) -> &[Method] {
        self.members_of(found).map_or(&[], Members::methods)
    }

    #[must_use]
    pub fn implementations(&self, found: &TypeFact<'file>) -> &[Implementation] {
        self.members_of(found).map_or(&[], Members::implementations)
    }

    fn members_of(&self, found: &TypeFact<'file>) -> Option<&Members> {
        let krate: &CrateName = found.as_ref();
        let name: &Name = found.as_ref();
        self.members.get(krate)?.get(name)
    }
}
